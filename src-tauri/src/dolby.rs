//! Sons Dolby Digital (AC-3) et Dolby Digital Plus (E-AC-3, souvent en 5.1) : rangés dans des
//! `.m4a` (albums « Dolby Atmos » du commerce) ou bruts (`.ac3`, `.eac3`). Le décodeur intégré
//! ne les lit pas et ffmpeg n'existe pas sur Android : ils sont décodés ici, en Rust pur
//! (filmcraft-ac3, MIT ou Apache-2.0), au fil de la lecture, et ramenés en stéréo.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::num::NonZero;
use std::path::Path;
use std::time::Duration;

use filmcraft_ac3::{parse_header, Decoder};
use rodio::source::SeekError;
use rodio::{Sample, Source};

/// Trames lues : données du flux et position de chaque trame.
struct Frames {
    data: Vec<u8>,
    starts: Vec<usize>,
    sample_rate: u32,
    /// Échantillons (par voie) d'une trame.
    samples_per_frame: usize,
}

/// Zone des données d'un MP4 (boîte « mdat » de premier niveau), ou tout le fichier s'il ne
/// s'agit pas d'un MP4 (flux brut).
fn payload(path: &Path) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let mut at = 0u64;
    let mut header = [0u8; 16];
    file.read_exact(&mut header[..8])?;
    if &header[4..8] != b"ftyp" {
        file.seek(SeekFrom::Start(0))?;
        let mut all = Vec::new();
        file.read_to_end(&mut all)?;
        return Ok(all);
    }
    while at + 8 <= len {
        file.seek(SeekFrom::Start(at))?;
        file.read_exact(&mut header[..8])?;
        let mut size = u64::from(u32::from_be_bytes([header[0], header[1], header[2], header[3]]));
        let mut head = 8u64;
        if size == 1 {
            file.read_exact(&mut header[8..16])?;
            size = u64::from_be_bytes([header[8], header[9], header[10], header[11], header[12], header[13], header[14], header[15]]);
            head = 16;
        } else if size == 0 {
            size = len - at;
        }
        if size < head {
            break;
        }
        if &header[4..8] == b"mdat" {
            let body = (size - head).min(len - at - head);
            let mut data = vec![0u8; usize::try_from(body).map_err(|_| std::io::ErrorKind::InvalidData)?];
            file.read_exact(&mut data)?;
            return Ok(data);
        }
        at += size;
    }
    Err(std::io::ErrorKind::InvalidData.into())
}

/// Trames qui se suivent : une synchro n'est retenue que si la trame suivante commence bien là
/// où la première finit (les octets 0x0B77 du hasard sont écartés).
fn find_frames(data: Vec<u8>) -> Option<Frames> {
    let mut starts = Vec::new();
    let mut pos = 0usize;
    let mut first = None;
    while pos + 8 <= data.len() {
        let chained = parse_header(&data[pos..]).ok().filter(|h| {
            let next = pos + h.frame_bytes;
            h.frame_bytes > 0 && (next >= data.len() || parse_header(&data[next..]).is_ok() || !starts.is_empty())
        });
        match chained {
            Some(h) => {
                if h.is_primary() {
                    first.get_or_insert(h);
                    starts.push(pos);
                }
                pos += h.frame_bytes;
            }
            None if starts.is_empty() => pos += 1,
            // Fin abîmée : on garde ce qui précède.
            None => break,
        }
    }
    let h = first?;
    Some(Frames { data, starts, sample_rate: h.sample_rate, samples_per_frame: h.samples() })
}

/// Le fichier contient-il un son Dolby lisible ici ?
pub fn is_dolby(path: &Path) -> bool {
    crate::ffmpeg::is_dolby_mp4(path) || matches!(crate::ffmpeg::extension_of(path).as_str(), "ac3" | "eac3" | "ec3")
}

/// Ramène les voies d'une trame (ordre WAV : G D C LFE Gs Ds) à la stéréo, coefficients
/// habituels (-3 dB pour le centre et l'arrière), au niveau de ffmpeg, sans le caisson.
fn to_stereo(acmod: u8, lfe: bool, ch: &[Vec<f32>], out: &mut Vec<f32>) {
    const K: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let n = ch.first().map_or(0, Vec::len);
    // Position de chaque voie dans `ch` selon la configuration (Table 5.8 de l'A/52).
    let at = |i: usize| ch.get(i).map(Vec::as_slice);
    let lfe_n = usize::from(lfe);
    type Voice<'a> = Option<&'a [f32]>;
    let (l, r, c, ls, rs): (Voice, Voice, Voice, Voice, Voice) = match acmod {
        1 => (at(0), at(0), None, None, None),
        0 | 2 => (at(0), at(1), None, None, None),
        3 => (at(0), at(1), at(2), None, None),
        4 => (at(0), at(1), None, at(2 + lfe_n), at(2 + lfe_n)),
        5 => (at(0), at(1), at(2), at(3 + lfe_n), at(3 + lfe_n)),
        6 => (at(0), at(1), None, at(2 + lfe_n), at(3 + lfe_n)),
        _ => (at(0), at(1), at(2), at(3 + lfe_n), at(4 + lfe_n)),
    };
    let get = |s: Option<&[f32]>, i: usize| s.and_then(|s| s.get(i)).copied().unwrap_or(0.0);
    out.reserve(n * 2);
    for i in 0..n {
        let centre = K * get(c, i);
        let left = get(l, i) + centre + K * get(ls, i);
        let right = get(r, i) + centre + K * get(rs, i);
        // Même niveau que ffmpeg (pas de baisse générale) ; les rares crêtes sont écrêtées.
        out.push(left.clamp(-1.0, 1.0));
        out.push(right.clamp(-1.0, 1.0));
    }
}

/// Source de lecture : décode une trame quand la précédente est épuisée.
pub struct DolbySource {
    frames: Frames,
    decoder: Decoder,
    next: usize,
    buffer: Vec<f32>,
    read: usize,
}

impl DolbySource {
    pub fn open(path: &Path) -> Result<Self, String> {
        let data = payload(path).map_err(|e| format!("son Dolby illisible ({e})"))?;
        let frames = find_frames(data).ok_or("aucune trame Dolby lisible dans ce fichier")?;
        let mut source = Self { frames, decoder: Decoder::new(), next: 0, buffer: Vec::new(), read: 0 };
        // Première trame tout de suite : un fichier que le décodeur refuse échoue ici, pas en
        // silence pendant la lecture.
        if !source.refill() {
            return Err("ce son Dolby utilise une variante que le décodeur ne connaît pas".into());
        }
        Ok(source)
    }

    /// Décode la trame suivante ; faux en fin de fichier.
    fn refill(&mut self) -> bool {
        while self.next < self.frames.starts.len() {
            let start = self.frames.starts[self.next];
            self.next += 1;
            self.buffer.clear();
            self.read = 0;
            if let Ok(d) = self.decoder.decode(&self.frames.data[start..]) {
                if d.channels.is_empty() {
                    continue;
                }
                to_stereo(d.header.acmod, d.header.lfeon, &d.channels, &mut self.buffer);
                if !self.buffer.is_empty() {
                    return true;
                }
            }
            // Trame abîmée : un court silence plutôt qu'un arrêt.
        }
        false
    }

    fn total_samples(&self) -> u64 {
        self.frames.starts.len() as u64 * self.frames.samples_per_frame as u64
    }
}

impl Iterator for DolbySource {
    type Item = Sample;

    fn next(&mut self) -> Option<Sample> {
        if self.read >= self.buffer.len() && !self.refill() {
            return None;
        }
        let s = self.buffer[self.read];
        self.read += 1;
        Some(Sample::from(s))
    }
}

impl Source for DolbySource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> rodio::ChannelCount {
        NonZero::new(2).unwrap_or(NonZero::<u16>::MIN)
    }

    fn sample_rate(&self) -> rodio::SampleRate {
        NonZero::new(self.frames.sample_rate).unwrap_or(NonZero::<u32>::MIN)
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(Duration::from_secs_f64(self.total_samples() as f64 / f64::from(self.frames.sample_rate.max(1))))
    }

    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let sample = (pos.as_secs_f64() * f64::from(self.frames.sample_rate)) as usize;
        let frame = (sample / self.frames.samples_per_frame.max(1)).min(self.frames.starts.len());
        self.decoder.reset();
        self.next = frame;
        self.buffer.clear();
        self.read = 0;
        if self.refill() {
            // Reste de la trame avant la position demandée.
            let skip = (sample % self.frames.samples_per_frame.max(1)) * 2;
            self.read = skip.min(self.buffer.len());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surround_is_folded_to_stereo_without_clipping() {
        // 5.1 à pleine échelle partout : la stéréo est écrêtée à 1, jamais au-delà.
        let full = vec![vec![1.0f32; 4]; 6];
        let mut out = Vec::new();
        to_stereo(7, true, &full, &mut out);
        assert_eq!(out.len(), 8);
        assert!(out.iter().all(|s| (*s - 1.0).abs() < 1e-5));
        // Le caisson (LFE) n'entre pas dans le mélange.
        let mut only_lfe = vec![vec![0.0f32; 2]; 6];
        only_lfe[3] = vec![1.0; 2];
        let mut out = Vec::new();
        to_stereo(7, true, &only_lfe, &mut out);
        assert!(out.iter().all(|s| *s == 0.0));
        // Mono : la même chose des deux côtés.
        let mut out = Vec::new();
        to_stereo(1, false, &[vec![0.5, -0.5]], &mut out);
        assert_eq!(out, vec![0.5, 0.5, -0.5, -0.5]);
    }

    #[test]
    fn plays_a_dolby_5_1_m4a_without_ffmpeg() {
        // Fichier créé avec ffmpeg : E-AC-3 5.1 dans un .m4a, 440 Hz sur la voie gauche seule.
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dolby-5.1.m4a");
        assert!(is_dolby(&path));
        let source = DolbySource::open(&path).unwrap();
        let secs = source.total_duration().unwrap().as_secs_f64();
        assert!((secs - 1.0).abs() < 0.1, "{secs} s");
        let samples: Vec<f32> = source.collect();
        let energy = |ch: usize| samples.iter().skip(ch).step_by(2).map(|s| s * s).sum::<f32>();
        assert!(energy(0) > 100.0 * (energy(1) + 1e-6), "gauche {} droite {}", energy(0), energy(1));
    }

    #[test]
    fn random_bytes_are_not_taken_for_frames() {
        let mut data = vec![0u8; 4096];
        data[100] = 0x0B;
        data[101] = 0x77;
        assert!(find_frames(data).is_none());
    }
}
