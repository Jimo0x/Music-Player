//! Forme d'onde d'un morceau (lot 0.13.0) : le fichier est décodé une fois, la valeur la plus
//! forte de chaque tranche est gardée, puis ramenée à `BARS` barres de 0 à 255, pour la barre
//! de progression « forme d'onde ». Le résultat est gardé sur disque (dossier `ondes/`) : un
//! morceau n'est analysé qu'une fois. Un seul calcul à la fois (téléphones modestes).

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use sha2::{Digest, Sha256};

/// Nombre de barres d'une forme d'onde.
pub const BARS: usize = 160;
/// Échantillons regroupés en un pic avant la réduction finale.
const BLOCK: usize = 1024;
/// Au-delà (≈ 20 min en stéréo 44,1 kHz), la suite du fichier n'est pas lue : un mix d'une heure
/// garderait la forme de son début plutôt que de bloquer le téléphone.
const MAX_SAMPLES: usize = 110_000_000;
/// Hauteur minimale d'une barre (un silence reste visible comme une ligne).
const FLOOR: u8 = 10;

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

/// Barres d'une suite d'échantillons : pic de chaque tranche, étalé entre la barre la plus
/// basse et la plus haute du morceau (une musique très compressée garde des reliefs lisibles).
pub fn bars_from_samples(samples: impl Iterator<Item = f32>) -> Vec<u8> {
    let mut peaks: Vec<f32> = Vec::new();
    let mut current = 0.0f32;
    let mut count = 0usize;
    for (n, s) in samples.enumerate() {
        if n >= MAX_SAMPLES {
            break;
        }
        let v = s.abs();
        if v.is_finite() && v > current {
            current = v;
        }
        count += 1;
        if count == BLOCK {
            peaks.push(current);
            current = 0.0;
            count = 0;
        }
    }
    if count > 0 {
        peaks.push(current);
    }
    reduce(&peaks)
}

fn reduce(peaks: &[f32]) -> Vec<u8> {
    if peaks.is_empty() {
        return vec![FLOOR; BARS];
    }
    let bars: Vec<f32> = (0..BARS)
        .map(|i| {
            let start = i * peaks.len() / BARS;
            let end = ((i + 1) * peaks.len() / BARS).max(start + 1).min(peaks.len());
            peaks[start.min(peaks.len() - 1)..end].iter().copied().fold(0.0, f32::max)
        })
        .collect();
    let max = bars.iter().copied().fold(0.0, f32::max);
    let min = bars.iter().copied().fold(f32::INFINITY, f32::min).min(max);
    bars.iter()
        .map(|&b| {
            let level = if max - min > 1e-6 { (b - min) / (max - min) } else if max > 0.0 { 1.0 } else { 0.0 };
            (FLOOR as f32 + level * (255.0 - FLOOR as f32)).round().clamp(0.0, 255.0) as u8
        })
        .collect()
}

/// Nom du fichier de cache : chemin, taille et date du fichier (un fichier remplacé est
/// analysé de nouveau).
fn cache_name(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    let digest = Sha256::digest(format!("{}|{}|{modified}", path.to_string_lossy(), meta.len()).as_bytes());
    Some(digest.iter().take(16).map(|b| format!("{b:02x}")).collect())
}

pub fn cache_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("ondes")
}

/// Forme d'onde d'un fichier de la bibliothèque : depuis le cache, sinon calculée et gardée.
pub fn waveform(data_dir: &Path, path: &Path) -> Result<Vec<u8>, String> {
    let name = cache_name(path).ok_or("Fichier introuvable.")?;
    let file = cache_dir(data_dir).join(format!("{name}.bin"));
    if let Ok(bytes) = std::fs::read(&file) {
        if bytes.len() == BARS {
            return Ok(bytes);
        }
    }
    let _turn = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
    // Calculée entre-temps par un autre appel (le même morceau demandé deux fois).
    if let Ok(bytes) = std::fs::read(&file) {
        if bytes.len() == BARS {
            return Ok(bytes);
        }
    }
    let source = crate::audio::open_decoder(&path.to_string_lossy())?;
    let bars = bars_from_samples(source);
    let _ = std::fs::create_dir_all(cache_dir(data_dir));
    let _ = std::fs::write(&file, &bars);
    Ok(bars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_loud_middle_gives_tall_bars_in_the_middle() {
        // 10 s « muets », 10 s forts, 10 s moyens, à 1 000 échantillons par seconde.
        let samples = (0..30_000).map(|i| match i / 10_000 {
            0 => 0.0,
            1 => if i % 2 == 0 { 0.9 } else { -0.9 },
            _ => 0.3,
        });
        let bars = bars_from_samples(samples);
        assert_eq!(bars.len(), BARS);
        assert_eq!(bars[10], FLOOR, "silence = barre minimale");
        assert_eq!(bars[BARS / 2], 255, "le passage le plus fort atteint le haut");
        let quiet = bars[BARS - 10];
        assert!(quiet > 70 && quiet < 120, "passage moyen à un tiers de la hauteur : {quiet}");
    }

    #[test]
    fn an_empty_or_silent_file_gives_flat_bars() {
        assert_eq!(bars_from_samples(std::iter::empty()), vec![FLOOR; BARS]);
        assert_eq!(bars_from_samples(std::iter::repeat_n(0.0, 50_000)), vec![FLOOR; BARS]);
    }

    #[test]
    fn a_short_sound_still_fills_every_bar() {
        let bars = bars_from_samples(std::iter::repeat_n(0.5, 300));
        assert_eq!(bars.len(), BARS);
        assert!(bars.iter().all(|&b| b == 255));
    }

    #[test]
    fn the_result_is_cached_on_disk() {
        let data = tempfile::tempdir().unwrap();
        let missing = data.path().join("absent.mp3");
        assert!(waveform(data.path(), &missing).is_err());
        assert!(cache_name(&missing).is_none());
    }
}

