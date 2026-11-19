//! yt-dlp sur téléphone, mis à jour sans republier l'appli : la version officielle (un seul
//! fichier Python « zipapp », publié par le projet yt-dlp sur GitHub) est téléchargée dans les
//! données de l'appli, vérifiée par son empreinte SHA-256 (fichier SHA2-256SUMS de la même
//! version), puis remplacée quand une nouvelle version sort (vérification au plus une fois
//! par jour). Python (Chaquopy) l'importe directement depuis ce fichier.

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use sha2::{Digest, Sha256};

use super::JobError;

const LATEST: &str = "https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest";
const ASSET: &str = "yt-dlp";
const SUMS: &str = "SHA2-256SUMS";
const MAX_BYTES: u64 = 40 * 1024 * 1024;
const CHECK_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// Dossier de yt-dlp dans les données de l'appli.
pub fn dir(data_dir: &Path) -> PathBuf {
    data_dir.join("yt-dlp")
}

/// Fichier importé par Python.
pub fn zip_path(data_dir: &Path) -> PathBuf {
    dir(data_dir).join("yt-dlp.zip")
}

fn version_path(data_dir: &Path) -> PathBuf {
    dir(data_dir).join("version.txt")
}

/// Version installée, s'il y en a une.
pub fn installed_version(data_dir: &Path) -> Option<String> {
    zip_path(data_dir).is_file().then(|| fs::read_to_string(version_path(data_dir)).ok()).flatten().map(|v| v.trim().to_owned())
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(120)))
        .http_status_as_error(false)
        .build()
        .into()
}

fn get(agent: &ureq::Agent, url: &str, limit: u64) -> Result<Vec<u8>, JobError> {
    let mut response = agent
        .get(url)
        .header("User-Agent", "MusicPlayer (https://github.com/LoickAmg/Music-Player)")
        .call()
        .map_err(|error| JobError::Network(format!("Téléchargement de yt-dlp impossible : {error}")))?;
    if response.status().as_u16() != 200 {
        return Err(JobError::Network(format!("GitHub a répondu {} pour yt-dlp.", response.status().as_u16())));
    }
    let mut bytes = Vec::new();
    response
        .body_mut()
        .with_config()
        .limit(limit)
        .reader()
        .read_to_end(&mut bytes)
        .map_err(|error| JobError::Network(format!("Téléchargement de yt-dlp interrompu : {error}")))?;
    Ok(bytes)
}

/// Empreinte attendue du fichier `name` dans un SHA2-256SUMS (« <hex>  <nom> »).
pub fn expected_sha256(sums: &str, name: &str) -> Option<String> {
    sums.lines().find_map(|line| {
        let (hash, file) = line.split_once(char::is_whitespace)?;
        (file.trim().trim_start_matches('*') == name && hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
            .then(|| hash.to_ascii_lowercase())
    })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// yt-dlp prêt à servir : téléchargé s'il manque, remplacé si une version plus récente est
/// publiée (vérifiée au plus une fois par jour). Sans réseau, la version déjà là sert.
pub fn ensure(data_dir: &Path) -> Result<PathBuf, JobError> {
    let zip = zip_path(data_dir);
    let fresh = fs::metadata(version_path(data_dir))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < CHECK_EVERY);
    if zip.is_file() && fresh {
        return Ok(zip);
    }
    match update(data_dir) {
        Ok(()) => Ok(zip),
        Err(_) if zip.is_file() => Ok(zip),
        Err(error) => Err(error),
    }
}

fn update(data_dir: &Path) -> Result<(), JobError> {
    let agent = agent();
    let release: serde_json::Value = serde_json::from_slice(&get(&agent, LATEST, 2 * 1024 * 1024)?)
        .map_err(|_| JobError::Failed("Réponse de GitHub illisible pour yt-dlp.".to_owned()))?;
    let tag = release.get("tag_name").and_then(|v| v.as_str()).unwrap_or_default().to_owned();
    if tag.is_empty() || !tag.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'.') {
        return Err(JobError::Failed("Version de yt-dlp inattendue sur GitHub.".to_owned()));
    }
    // Déjà la dernière : on note seulement la vérification du jour.
    if installed_version(data_dir).as_deref() == Some(tag.as_str()) {
        let _ = fs::write(version_path(data_dir), &tag);
        return Ok(());
    }
    let base = format!("https://github.com/yt-dlp/yt-dlp/releases/download/{tag}");
    let sums = String::from_utf8(get(&agent, &format!("{base}/{SUMS}"), 64 * 1024)?).unwrap_or_default();
    let expected = expected_sha256(&sums, ASSET).ok_or_else(|| JobError::Failed("Empreinte de yt-dlp introuvable sur GitHub.".to_owned()))?;
    let bytes = get(&agent, &format!("{base}/{ASSET}"), MAX_BYTES)?;
    if sha256_hex(&bytes) != expected {
        return Err(JobError::Failed("Le fichier yt-dlp téléchargé ne correspond pas à son empreinte : il n'a pas été installé.".to_owned()));
    }
    let dir = dir(data_dir);
    fs::create_dir_all(&dir).map_err(|e| JobError::Failed(format!("Dossier de yt-dlp impossible à créer : {e}")))?;
    // Remplacement d'un bloc : jamais de fichier à moitié écrit à la place de l'ancien.
    let tmp = dir.join("yt-dlp.zip.part");
    fs::write(&tmp, &bytes).map_err(|e| JobError::Failed(format!("yt-dlp impossible à enregistrer : {e}")))?;
    fs::rename(&tmp, zip_path(data_dir)).map_err(|e| JobError::Failed(format!("yt-dlp impossible à installer : {e}")))?;
    fs::write(version_path(data_dir), &tag).map_err(|e| JobError::Failed(format!("Version de yt-dlp non notée : {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_expected_fingerprint_of_the_right_file() {
        let a = "a".repeat(64);
        let b = "B".repeat(64);
        let sums = format!("{a}  yt-dlp.exe\n{b}  yt-dlp\n{a}  yt-dlp_linux\n");
        assert_eq!(expected_sha256(&sums, "yt-dlp"), Some("b".repeat(64)));
        assert_eq!(expected_sha256(&sums, "absent"), None);
        assert_eq!(expected_sha256("zz  yt-dlp", "yt-dlp"), None);
    }

    #[test]
    fn fingerprint_matches_the_standard() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    /// Essai réel (réseau) : `cargo test --lib -- --ignored --nocapture installe_le_vrai_yt_dlp`.
    #[test]
    #[ignore]
    fn installe_le_vrai_yt_dlp() {
        let dir = tempfile::tempdir().unwrap();
        let zip = ensure(dir.path()).unwrap();
        println!("yt-dlp {} : {} octets", installed_version(dir.path()).unwrap(), fs::metadata(&zip).unwrap().len());
        // Deuxième appel le même jour : rien n'est retéléchargé.
        let modified = fs::metadata(&zip).unwrap().modified().unwrap();
        ensure(dir.path()).unwrap();
        assert_eq!(fs::metadata(&zip).unwrap().modified().unwrap(), modified);
    }
}
