//! Import d'une playlist, d'un album ou d'un morceau Spotify par son lien. D'abord la page
//! « embed » publique de Spotify (`spotify_embed.rs` : partout, téléphones compris, 100
//! morceaux au plus) ; sur ordinateur, spotDL (`spotdl save`) pour une playlist plus longue ou
//! si la page ne se lit pas. Jamais de clé Spotify.
//!
//! Le texte collé n'est jamais transmis tel quel : le lien est validé, puis reconstruit à partir
//! du type et de l'identifiant seuls.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::Manager;

use super::common::{cleanup_attempt, create_attempt_directory, join_logs, network_failure, truncate_log};
use super::import::STAGING_DIR;
use super::spotdl::{resolve_runner, Runner};
use super::JobError;
use crate::bounded::{self, Bounded};
use crate::ffmpeg;
use crate::state::{AppState, Locked};

const TOOL: &str = "spotDL";
const MAX_FILE_BYTES: u64 = 20 * 1024 * 1024;
const MAX_TRACKS: usize = 10_000;
/// spotDL lit les métadonnées morceau par morceau (83 s mesurées pour un seul titre, 12/11) :
/// une playlist demande bien plus que 2 min.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10 * 60);
const INSTALL_MESSAGE: &str = "Spotify n'a pas pu être lu ; pour une longue playlist, utilisez l'export de vos données Spotify (ou spotDL sur ordinateur).";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkTrack {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_secs: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkPlaylist {
    pub name: String,
    pub tracks: Vec<LinkTrack>,
    /// Liste coupée par Spotify (100 premiers morceaux d'une playlist plus longue).
    #[serde(default)]
    pub partial: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Playlist,
    Album,
    Track,
}

#[derive(Debug, PartialEq, Eq)]
struct ValidatedLink {
    kind: Kind,
    url: String,
}

/// Morceau tel que l'écrit `spotdl save` (seuls les champs utiles).
#[derive(Debug, Deserialize)]
struct SpotdlTrack {
    #[serde(default)]
    name: String,
    #[serde(default)]
    artists: serde_json::Value,
    #[serde(default)]
    album_name: Option<String>,
    #[serde(default)]
    duration: Option<f64>,
    #[serde(default)]
    list_name: Option<String>,
}

#[tauri::command(async)]
pub fn fetch_read_link(app: tauri::AppHandle, url: String) -> Result<LinkPlaylist, String> {
    // Lien court du bouton « Partager » de l'appli Spotify : suivi jusqu'au vrai lien.
    let url = if is_short_link(&url) { resolve_short_link(&url)? } else { url };
    let link = validate_spotify_link(&url)?;
    let embed = super::spotify_embed::read(link.kind.segment(), link.id());
    // Playlist peut-être coupée à 100 : spotDL (ordinateur) lit la liste complète, s'il est là.
    let maybe_cut = matches!(&embed, Ok(list) if link.kind == Kind::Playlist && list.tracks.len() >= super::spotify_embed::EMBED_LIMIT);
    if embed.is_ok() && !maybe_cut {
        return embed.map_err(|error| error.to_string());
    }
    let full = if cfg!(target_os = "android") { Err(INSTALL_MESSAGE.to_owned()) } else { read_with_spotdl(&app, &link) };
    match (embed, full) {
        (_, Ok(list)) => Ok(list),
        (Ok(mut list), Err(_)) => {
            list.partial = true;
            Ok(list)
        }
        (Err(error), Err(_)) => Err(error.to_string()),
    }
}

/// Contenu d'un lien direct (playlist, album… lu par yt-dlp) : un morceau par tâche. Liste vide
/// pour un lien vers un seul son.
#[derive(Debug, Clone, Serialize)]
pub struct MediaLink {
    pub name: String,
    pub tracks: Vec<super::ytdlp::LinkEntry>,
}

#[tauri::command(async)]
pub fn fetch_read_media_link(url: String) -> Result<MediaLink, String> {
    super::ytdlp::read_media_link(&url).map(|(name, tracks)| MediaLink { name, tracks }).map_err(|error| error.to_string())
}

/// Lecture complète par spotDL (ordinateur).
fn read_with_spotdl(app: &tauri::AppHandle, link: &ValidatedLink) -> Result<LinkPlaylist, String> {
    let root = app
        .state::<AppState>()
        .library_root
        .locked()
        .clone()
        .ok_or_else(|| "Choisissez d'abord le dossier de votre bibliothèque.".to_owned())?;
    let cancel = AtomicBool::new(false);
    let runner = resolve_runner(&cancel).map_err(|error| match error {
        JobError::ToolMissing(_) => INSTALL_MESSAGE.to_owned(),
        other => other.to_string(),
    })?;
    read_link_with_runner(runner, link, &Path::new(&root).join(STAGING_DIR), &cancel).map_err(|error| error.to_string())
}

impl Kind {
    fn segment(self) -> &'static str {
        match self {
            Kind::Playlist => "playlist",
            Kind::Album => "album",
            Kind::Track => "track",
        }
    }
}

impl ValidatedLink {
    /// Identifiant Spotify (dernier segment du lien reconstruit).
    fn id(&self) -> &str {
        self.url.rsplit('/').next().unwrap_or_default()
    }
}

/// Accepte `https://open.spotify.com/[intl-xx/]{playlist|album|track}/<22 caractères>` (les
/// paramètres `?si=…` et autres sont ignorés) et `spotify:{playlist|album|track}:<id>`.
fn validate_spotify_link(input: &str) -> Result<ValidatedLink, String> {
    let input = input.trim();
    let (kind, id) = if let Some(uri) = input.strip_prefix("spotify:") {
        uri.split_once(':').ok_or_else(|| "Lien Spotify incomplet.".to_owned())?
    } else {
        let rest = input
            .strip_prefix("https://open.spotify.com/")
            .ok_or_else(|| "Le lien doit commencer par https://open.spotify.com/.".to_owned())?;
        // On reconstruit le lien : paramètres et fragment sont simplement abandonnés.
        let path = rest.split(['?', '#']).next().unwrap_or_default();
        let mut parts = path.split('/').filter(|part| !part.is_empty()).peekable();
        if parts.peek().is_some_and(|first| is_locale(first)) {
            parts.next();
        }
        let kind = parts.next().unwrap_or_default();
        let id = parts.next().unwrap_or_default();
        if parts.next().is_some() {
            return Err("Le lien Spotify contient un chemin supplémentaire inattendu.".to_owned());
        }
        (kind, id)
    };
    let kind = match kind {
        "playlist" => Kind::Playlist,
        "album" => Kind::Album,
        "track" => Kind::Track,
        _ => return Err("Le lien doit désigner une playlist, un album ou un morceau Spotify.".to_owned()),
    };
    if id.len() != 22 || !id.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return Err("Ce lien Spotify n'a pas un identifiant valide (22 lettres ou chiffres).".to_owned());
    }
    Ok(ValidatedLink { kind, url: format!("https://open.spotify.com/{}/{id}", kind.segment()) })
}

/// Hôtes des liens courts de Spotify (bouton « Partager » de l'appli, anciens liens).
const SHORT_HOSTS: [&str; 3] = ["spotify.link", "spoti.fi", "spotify.app.link"];
const SHORT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36";
const SHORT_MAX_HOPS: usize = 5;
const SHORT_MAX_PAGE_BYTES: u64 = 2 * 1024 * 1024;

/// Hôte d'une adresse https (sans port ni chemin), en minuscules.
fn https_host(url: &str) -> Option<String> {
    let rest = url.trim().strip_prefix("https://").or_else(|| url.trim().strip_prefix("http://"))?;
    let host = rest.split(['/', '?', '#']).next()?;
    (!host.is_empty() && !host.contains(['@', ':'])).then(|| host.to_ascii_lowercase())
}

/// `https://spotify.link/…` ou `https://spoti.fi/…`.
fn is_short_link(url: &str) -> bool {
    https_host(url).is_some_and(|host| SHORT_HOSTS.contains(&host.as_str()))
}

/// Premier lien open.spotify.com valide (playlist, album, morceau) dans une page ou une
/// redirection ; les `\/` du JSON sont d'abord remis en `/`.
fn find_spotify_url(text: &str) -> Option<String> {
    let text = text.replace("\\/", "/");
    text.match_indices("https://open.spotify.com/").find_map(|(at, _)| {
        let candidate: String = text[at..].chars().take_while(|c| !c.is_whitespace() && !matches!(c, '"' | '\'' | '<' | '>' | '\\')).collect();
        validate_spotify_link(&candidate).ok().map(|link| link.url)
    })
}

/// Suit un lien court (redirections une à une, hôtes de Spotify seulement) jusqu'au lien
/// open.spotify.com, sans jamais ouvrir d'autre site.
fn resolve_short_link(input: &str) -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .max_redirects(0)
        .http_status_as_error(false)
        .build()
        .into();
    let unreadable = "Ce lien court de Spotify ne mène à aucun morceau, album ou playlist. Ouvre-le dans le navigateur, puis colle ici l'adresse open.spotify.com qui s'affiche.";
    let mut url = input.trim().to_owned();
    for _ in 0..SHORT_MAX_HOPS {
        if !is_short_link(&url) {
            return Err(unreadable.to_owned());
        }
        let mut response = agent
            .get(&url)
            .header("User-Agent", SHORT_USER_AGENT)
            .call()
            .map_err(|error| format!("Spotify ne répond pas : {error}"))?;
        let status = response.status().as_u16();
        if (300..400).contains(&status) {
            let location = response.headers().get("location").and_then(|v| v.to_str().ok()).unwrap_or_default().to_owned();
            if let Some(found) = find_spotify_url(&location) {
                return Ok(found);
            }
            url = location;
            continue;
        }
        if status != 200 {
            return Err(format!("Spotify a répondu {status} pour ce lien court."));
        }
        let page = response.body_mut().with_config().limit(SHORT_MAX_PAGE_BYTES).read_to_string().unwrap_or_default();
        return find_spotify_url(&page).ok_or_else(|| unreadable.to_owned());
    }
    Err(unreadable.to_owned())
}

/// `intl-fr`, `intl-pt`, `intl-pt-br`…
fn is_locale(part: &str) -> bool {
    part.strip_prefix("intl-").is_some_and(|locale| {
        (2..=5).contains(&locale.len()) && locale.bytes().all(|byte| byte.is_ascii_alphabetic() || byte == b'-')
    })
}

fn read_link_with_runner(runner: Runner, link: &ValidatedLink, staging: &Path, cancel: &AtomicBool) -> Result<LinkPlaylist, JobError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(JobError::Cancelled);
    }
    let attempt_dir = create_attempt_directory(staging)?;
    let save_file = attempt_dir.join("lien.spotdl");
    let result = run_save(runner, &link.url, &save_file, cancel).and_then(|()| read_spotdl_file(&save_file, link.kind));
    cleanup_attempt(&attempt_dir);
    result
}

fn run_save(runner: Runner, link: &str, save_file: &Path, cancel: &AtomicBool) -> Result<(), JobError> {
    let mut command = runner.command();
    command
        // Le lien suit directement l'opération (après les options, spotDL le refuse) et sans
        // « -- » (refusé aussi) ; reconstruit, il commence toujours par « https:// ».
        .arg("save")
        .arg(link)
        .arg("--save-file")
        .arg(save_file);
    ffmpeg::quiet(&mut command);
    let outcome = bounded::run(&mut command, COMMAND_TIMEOUT, cancel).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            JobError::ToolMissing(INSTALL_MESSAGE.to_owned())
        } else {
            JobError::Failed(format!("Impossible de lancer spotDL : {error}"))
        }
    })?;
    match outcome {
        Bounded::Finished { status, .. } if status.success() => Ok(()),
        Bounded::Finished { stdout, stderr, .. } => {
            let detail = join_logs(&stdout, &stderr, TOOL);
            if network_failure(&detail.to_lowercase()) {
                Err(JobError::Network(truncate_log(&detail)))
            } else {
                Err(JobError::Failed(truncate_log(&detail)))
            }
        }
        Bounded::TimedOut { .. } => Err(JobError::Timeout),
        Bounded::Cancelled { .. } => Err(JobError::Cancelled),
    }
}

fn read_spotdl_file(path: &Path, kind: Kind) -> Result<LinkPlaylist, JobError> {
    let file = File::open(path).map_err(|error| JobError::Failed(format!("spotDL n'a rien écrit pour ce lien : {error}")))?;
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| JobError::Failed(format!("Impossible de lire le résultat de spotDL : {error}")))?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err(JobError::Failed("Le résultat de spotDL dépasse 20 Mo.".to_owned()));
    }
    let entries: Vec<SpotdlTrack> =
        serde_json::from_slice(&bytes).map_err(|error| JobError::Failed(format!("Résultat de spotDL illisible : {error}")))?;
    if entries.len() > MAX_TRACKS {
        return Err(JobError::Failed(format!("Le lien contient plus de {MAX_TRACKS} morceaux.")));
    }

    let named = |value: Option<&str>| value.map(str::trim).filter(|name| !name.is_empty()).map(str::to_owned);
    let list_name = named(entries.iter().find_map(|entry| entry.list_name.as_deref()));
    let first_album = named(entries.first().and_then(|entry| entry.album_name.as_deref()));
    let first_title = named(entries.first().map(|entry| entry.name.as_str()));

    let tracks: Vec<LinkTrack> = entries
        .into_iter()
        .filter_map(|entry| {
            let title = entry.name.trim().to_owned();
            if title.is_empty() {
                return None;
            }
            let artist = match entry.artists {
                serde_json::Value::String(value) => value.trim().to_owned(),
                serde_json::Value::Array(values) => values
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|artist| !artist.is_empty())
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => String::new(),
            };
            Some(LinkTrack {
                title,
                artist,
                album: entry.album_name.map(|album| album.trim().to_owned()).unwrap_or_default(),
                duration_secs: entry.duration.filter(|duration| duration.is_finite() && *duration > 0.0),
            })
        })
        .collect();
    if tracks.is_empty() {
        return Err(JobError::Failed("Aucun morceau lisible derrière ce lien (playlist privée ou vide ?).".to_owned()));
    }

    // Nom donné par spotDL ; à défaut, l'album ou le titre selon le type de lien.
    let fallback = match kind {
        Kind::Album => first_album,
        Kind::Track => first_title,
        Kind::Playlist => None,
    };
    let name = list_name.or(fallback).unwrap_or_else(|| "Playlist Spotify".to_owned());
    Ok(LinkPlaylist { name, tracks, partial: false })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{find_spotify_url, is_short_link, read_spotdl_file, resolve_short_link, validate_spotify_link, Kind, LinkTrack};

    #[test]
    fn short_links_are_recognised_and_followed_only_on_spotify_hosts() {
        assert!(is_short_link("https://spotify.link/AbCdEf123"));
        assert!(is_short_link("https://spoti.fi/3xYz"));
        assert!(!is_short_link("https://open.spotify.com/track/4PTG3Z6ehGkBFwjybzWkR8"));
        assert!(!is_short_link("https://spotify.link.evil.com/x"));
        assert!(!is_short_link("https://user@spotify.link/x"));
        assert!(resolve_short_link("https://example.com/x").is_err());
    }

    #[test]
    fn finds_the_spotify_link_in_a_redirect_or_a_page() {
        let id = "4PTG3Z6ehGkBFwjybzWkR8";
        let expected = Some(format!("https://open.spotify.com/track/{id}"));
        assert_eq!(find_spotify_url(&format!("https://open.spotify.com/track/{id}?si=abc&utm_source=copy-link")), expected);
        assert_eq!(find_spotify_url(&format!(r#"<a href="https://open.spotify.com/intl-fr/track/{id}?si=x">"#)), expected);
        assert_eq!(find_spotify_url(&format!(r#"{{"url":"https:\/\/open.spotify.com\/track\/{id}"}}"#)), expected);
        assert_eq!(find_spotify_url("https://open.spotify.com/ puis https://open.spotify.com/artist/x"), None);
    }

    const ID: &str = "0123456789ABCDEFGHIJKL";

    #[test]
    fn accepte_les_liens_spotify_et_les_reconstruit() {
        for (input, expected) in [
            (format!("https://open.spotify.com/playlist/{ID}"), format!("https://open.spotify.com/playlist/{ID}")),
            (format!("  https://open.spotify.com/album/{ID}?si=abc-123_X&pt=x  "), format!("https://open.spotify.com/album/{ID}")),
            (format!("https://open.spotify.com/intl-fr/track/{ID}#x"), format!("https://open.spotify.com/track/{ID}")),
            (format!("https://open.spotify.com/intl-pt-br/playlist/{ID}/"), format!("https://open.spotify.com/playlist/{ID}")),
            (format!("spotify:playlist:{ID}"), format!("https://open.spotify.com/playlist/{ID}")),
        ] {
            assert_eq!(validate_spotify_link(&input).map(|link| link.url), Ok(expected), "{input}");
        }
    }

    #[test]
    fn refuse_les_autres_liens_et_les_injections() {
        for input in [
            format!("https://example.com/playlist/{ID}"),
            format!("http://open.spotify.com/playlist/{ID}"),
            format!("https://open.spotify.com.evil.com/playlist/{ID}"),
            "https://open.spotify.com/playlist/short".to_owned(),
            "https://open.spotify.com/playlist/--help".to_owned(),
            format!("https://open.spotify.com/artist/{ID}"),
            format!("https://open.spotify.com/playlist/{ID}/extra"),
            format!("spotify:playlist:{ID} --help"),
            String::new(),
        ] {
            assert!(validate_spotify_link(&input).is_err(), "{input}");
        }
    }

    #[test]
    fn lit_un_fichier_spotdl() {
        let directory = tempfile::tempdir().expect("dossier temporaire");
        let path = directory.path().join("test.spotdl");
        fs::write(
            &path,
            r#"[{"name":"Titre","artists":["Artiste"," Invité "],"album_name":"Album","duration":180,"list_name":"Ma playlist"},
                {"name":"  ","artists":[]}]"#,
        )
        .expect("fichier de test");
        let playlist = read_spotdl_file(&path, Kind::Playlist).expect("lecture");
        assert_eq!(playlist.name, "Ma playlist");
        assert_eq!(
            playlist.tracks,
            vec![LinkTrack { title: "Titre".into(), artist: "Artiste, Invité".into(), album: "Album".into(), duration_secs: Some(180.0) }]
        );
    }

    #[test]
    fn nomme_un_album_sans_nom_de_liste_d_apres_l_album() {
        let directory = tempfile::tempdir().expect("dossier temporaire");
        let path = directory.path().join("test.spotdl");
        fs::write(&path, r#"[{"name":"Titre","artists":"Artiste","album_name":"Mon album"}]"#).expect("fichier de test");
        assert_eq!(read_spotdl_file(&path, Kind::Album).expect("lecture").name, "Mon album");
    }

    #[test]
    fn refuse_un_resultat_vide() {
        let directory = tempfile::tempdir().expect("dossier temporaire");
        let path = directory.path().join("test.spotdl");
        fs::write(&path, "[]").expect("fichier de test");
        assert!(read_spotdl_file(&path, Kind::Playlist).is_err());
    }

    /// Vrai lien, vrai spotDL (réseau) : `cargo test -- --ignored lit_un_vrai_lien`.
    #[test]
    #[ignore = "réseau et spotDL installé"]
    fn lit_un_vrai_lien_spotify() {
        use std::sync::atomic::AtomicBool;
        let cancel = AtomicBool::new(false);
        let runner = super::resolve_runner(&cancel).expect("spotDL trouvé");
        let link = validate_spotify_link("https://open.spotify.com/track/4PTG3Z6ehGkBFwjybzWkR8?si=x").expect("lien");
        let staging = tempfile::tempdir().expect("dossier");
        let playlist = super::read_link_with_runner(runner, &link, staging.path(), &cancel).expect("lecture du lien");
        println!("{playlist:?}");
        assert_eq!(playlist.tracks.len(), 1);
        let track = &playlist.tracks[0];
        assert!(track.title.to_lowercase().contains("never gonna give you up"), "{track:?}");
        assert!(track.artist.contains("Rick Astley"), "{track:?}");
        assert!(track.duration_secs.is_some_and(|d| d > 150.0), "{track:?}");
        assert_eq!(fs::read_dir(staging.path()).expect("dossier").count(), 0, "dossier de tâche effacé");
    }
}
