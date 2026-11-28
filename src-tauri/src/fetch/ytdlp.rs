//! Adaptateur yt-dlp. Une recherche se fait à la façon de spotDL : yt-dlp liste 5 candidats
//! sans rien télécharger (`ytsearch5:`), notre correspondance (titre, artiste, durée, version)
//! choisit le meilleur, puis yt-dlp télécharge celui-là par son lien. Un lien direct est
//! téléchargé tel quel. Les fichiers sont d'abord créés dans un dossier isolé, puis repris et
//! renommés avec un nom sûr.
//!
//! Le dossier est donné à part (`-P`) et le modèle de nom reste relatif (`-o`) : yt-dlp lit tout
//! `%` du modèle comme une variable, et un dossier de bibliothèque peut en contenir un.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use crate::bounded::{self, Bounded};
use crate::ffmpeg;

use super::common::{cleanup_attempt, create_attempt_directory, ffmpeg_missing, join_logs, network_failure, progress_sink, spotdl_ffmpeg, take_output, tool_locations, truncate_log, ytdlp_percent};
use super::source::Source;
use super::spotdl::requested_file_name;
use super::{JobError, Wanted};

const TOOL: &str = "yt-dlp";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(6 * 60);
const PROBE_TIMEOUT: Duration = Duration::from_secs(10);
/// Modèle de nom, relatif au dossier donné par `-P` (un `-o` absolu ferait ignorer `-P`).
const OUTPUT_TEMPLATE: &str = "%(title)s.%(ext)s";

#[derive(Clone)]
struct Runner {
    executable: PathBuf,
    module: bool,
}

impl Runner {
    fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        if self.module {
            command.arg("-m").arg("yt_dlp");
        }
        command
    }
}

pub struct YtDlpSource;

impl Source for YtDlpSource {
    fn name(&self) -> &'static str {
        TOOL
    }

    fn available(&self) -> bool {
        // Téléphone : yt-dlp intégré (Python), téléchargé à la première utilisation.
        cfg!(target_os = "android") || resolve_runner(&AtomicBool::new(false)).is_ok()
    }

    fn fetch(&self, wanted: &Wanted, dest_dir: &Path, cancel: &AtomicBool) -> Result<PathBuf, JobError> {
        if cancel.load(Ordering::Relaxed) {
            return Err(JobError::Cancelled);
        }
        if cfg!(target_os = "android") {
            return fetch_on_phone(wanted, dest_dir, cancel);
        }
        let runner = resolve_runner(cancel)?;
        let query = if wanted.url.is_some() {
            make_query(wanted)?
        } else {
            let candidates = search_candidates(&runner, &make_query(wanted)?, cancel)?;
            pick_candidate(wanted, &candidates).ok_or(JobError::NotFound)?.url.clone()
        };
        let attempt_dir = create_attempt_directory(dest_dir)?;
        let file_name = requested_file_name(wanted);

        let result = match run_download(runner, &query, &attempt_dir, cancel) {
            Ok(Bounded::Finished { status, .. }) if status.success() => take_output(&attempt_dir, &file_name, TOOL),
            Ok(Bounded::Finished { stdout, stderr, .. }) => Err(classify_failure(&stdout, &stderr)),
            Ok(Bounded::TimedOut { .. }) => Err(JobError::Timeout),
            Ok(Bounded::Cancelled { .. }) => Err(JobError::Cancelled),
            Err(error) => Err(error),
        };
        if result.is_err() {
            cleanup_attempt(&attempt_dir);
        }
        result
    }
}

fn resolve_runner(cancel: &AtomicBool) -> Result<Runner, JobError> {
    // PATH d'abord, puis les installations hors PATH (Anaconda, uv…), puis Python.
    let mut candidates = vec![Runner { executable: "yt-dlp".into(), module: false }];
    candidates.extend(tool_locations("yt-dlp").into_iter().map(|executable| Runner { executable, module: false }));
    #[cfg(windows)]
    candidates.push(Runner { executable: "py".into(), module: true });
    candidates.push(Runner { executable: "python".into(), module: true });

    let mut last_error = None;
    for runner in candidates {
        if cancel.load(Ordering::Relaxed) {
            return Err(JobError::Cancelled);
        }
        let mut command = runner.command();
        command.arg("--version");
        match run_command(&mut command, PROBE_TIMEOUT, cancel) {
            Ok(Bounded::Finished { status, .. }) if status.success() => return Ok(runner),
            Ok(Bounded::Finished { stdout, stderr, .. }) => last_error = Some(join_logs(&stdout, &stderr, TOOL)),
            Ok(Bounded::TimedOut { .. }) => return Err(JobError::Timeout),
            Ok(Bounded::Cancelled { .. }) => return Err(JobError::Cancelled),
            Err(JobError::ToolMissing(_)) => {}
            Err(error) => last_error = Some(error.to_string()),
        }
    }
    let detail = last_error.unwrap_or_else(|| "yt-dlp est introuvable. Installez-le séparément puis redémarrez Music Player.".to_owned());
    Err(JobError::ToolMissing(truncate_log(&detail)))
}

/// Téléphone : mêmes étapes que sur ordinateur (5 candidats, notre choix, téléchargement),
/// par yt-dlp dans Python (fetch/android.rs). Le fichier garde son format (.m4a le plus souvent).
fn fetch_on_phone(wanted: &Wanted, dest_dir: &Path, cancel: &AtomicBool) -> Result<PathBuf, JobError> {
    use super::android;
    let zip = super::ytdlp_zip::ensure(android::data_dir()?)?;
    let (url, chosen) = match wanted.direct_url() {
        Some(url) => (url.to_owned(), None),
        None if wanted.url.is_some() => return Err(JobError::Failed("Ce lien n'est pas une adresse web valable (http ou https).".to_owned())),
        None => {
            let raw = android::search(&zip, &make_query(wanted)?)?;
            let candidates = parse_candidates(&raw);
            let chosen = pick_candidate(wanted, &candidates).ok_or_else(|| no_match(&candidates, &raw))?.clone();
            (chosen.url.clone(), Some(chosen))
        }
    };
    if cancel.load(Ordering::Relaxed) {
        return Err(JobError::Cancelled);
    }
    let attempt_dir = create_attempt_directory(dest_dir)?;
    let result = android::download(&zip, &url, &attempt_dir, cancel);
    // Sans ffmpeg, yt-dlp n'écrit pas d'étiquettes : celles du candidat choisi, pour que la
    // vérification de l'import compare la même chose que sur ordinateur.
    if let (Ok(path), Some(chosen)) = (&result, &chosen) {
        let tags = Wanted { title: chosen.title.clone(), artist: chosen.artist.clone(), album: String::new(), duration_secs: None, playlist_id: None, url: None };
        super::import::retag(path, &tags);
    }
    if result.is_err() {
        cleanup_attempt(&attempt_dir);
    }
    result
}

/// Aucun candidat retenu : lesquels ont été vus (pour comprendre, et pour l'écran « Détails »).
fn no_match(candidates: &[Candidate], raw: &str) -> JobError {
    if candidates.is_empty() {
        let start: String = raw.chars().take(160).collect();
        return JobError::Failed(format!("La recherche n'a renvoyé aucune vidéo YouTube (réponse : {start}…)."));
    }
    let seen: Vec<String> = candidates
        .iter()
        .take(5)
        .map(|c| format!("« {} » ({}, {})", c.title, c.artist, c.duration_secs.map(|d| format!("{d:.0} s")).unwrap_or_else(|| "durée ?".into())))
        .collect();
    JobError::Failed(format!("Aucun des {} résultats ne correspond assez : {}.", candidates.len(), seen.join(" ; ")))
}

/// Résultats montrés par la recherche de la page Télécharger (à la façon d'Audiomack).
pub const SEARCH_RESULTS: usize = 15;

/// Un résultat de recherche, à télécharger par son lien si la personne le choisit.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct SearchResult {
    pub title: String,
    pub artist: String,
    pub duration_secs: Option<f64>,
    pub url: String,
    /// Miniature publique de YouTube (i.ytimg.com), reconstruite depuis l'identifiant.
    pub thumbnail: Option<String>,
}

/// Recherche tapée dans l'appli : les résultats, sans rien télécharger. Mêmes chemins que la
/// recherche du moteur (yt-dlp « à plat » sur ordinateur, yt-dlp intégré sur téléphone).
pub fn search_songs(text: &str) -> Result<Vec<SearchResult>, JobError> {
    let query = search_query(text)?;
    let candidates = if cfg!(target_os = "android") {
        let zip = super::ytdlp_zip::ensure(super::android::data_dir()?)?;
        parse_candidates(&super::android::search(&zip, &query)?)
    } else {
        let cancel = AtomicBool::new(false);
        let runner = resolve_runner(&cancel)?;
        search_candidates(&runner, &query, &cancel)?
    };
    Ok(present_results(candidates))
}

/// Résultats montrés comme des morceaux (à la façon d'Audiomack) : « Artiste - Titre (Clip
/// officiel) » devient titre et artiste, « Artiste - Topic » (son officiel de YouTube Music)
/// devient l'artiste. Les sons officiels d'abord, les longues compilations à la fin ; sinon
/// l'ordre de YouTube est gardé.
fn present_results(candidates: Vec<Candidate>) -> Vec<SearchResult> {
    let mut ranked: Vec<(u8, SearchResult)> = candidates
        .into_iter()
        .map(|c| {
            let thumbnail = video_id(&c.url).map(|id| format!("https://i.ytimg.com/vi/{id}/mqdefault.jpg"));
            let (title, artist, rank) = present(&c.title, &c.artist, c.duration_secs);
            (rank, SearchResult { title, artist, duration_secs: c.duration_secs, url: c.url, thumbnail })
        })
        .collect();
    ranked.sort_by_key(|(rank, _)| *rank);
    ranked.into_iter().map(|(_, r)| r).collect()
}

/// (titre, artiste, rang) d'une vidéo : 0 son officiel, 1 clip ou audio officiel, 2 autre,
/// 3 compilation de plus de 20 minutes.
fn present(video_title: &str, channel: &str, duration: Option<f64>) -> (String, String, u8) {
    let topic = channel.strip_suffix(" - Topic");
    let title = strip_noise(video_title);
    let official = {
        let lower = video_title.to_lowercase();
        ["official", "officiel", "audio", "lyric", "paroles", "visualizer"].iter().any(|w| lower.contains(w))
    };
    let long = duration.is_some_and(|d| d > 20.0 * 60.0);
    if let Some(artist) = topic {
        return (title, artist.trim().to_owned(), if long { 3 } else { 0 });
    }
    // « Artiste - Titre » (tiret entouré d'espaces) : la chaîne n'est souvent qu'un label.
    let split = [" - ", " – ", " — "].iter().find_map(|sep| title.split_once(sep));
    let (title, artist) = match split {
        Some((artist, rest)) if !artist.trim().is_empty() && !rest.trim().is_empty() => (rest.trim().to_owned(), artist.trim().to_owned()),
        _ => (title.clone(), channel.trim().to_owned()),
    };
    let rank = if long { 3 } else if official { 1 } else { 2 };
    (title, artist, rank)
}

/// Retire du titre les mentions entre parenthèses ou crochets qui ne sont pas le titre
/// (« (Official Video) », « [Clip officiel] », « (Lyrics) »…). Garde « (feat. X) », « (Remix) ».
fn strip_noise(title: &str) -> String {
    const NOISE: [&str; 14] = ["official", "officiel", "officielle", "clip", "video", "vidéo", "audio", "lyric", "lyrics", "paroles", "visualizer", "hd", "4k", "music"];
    let mut out = String::with_capacity(title.len());
    let mut rest = title;
    while let Some(start) = rest.find(['(', '[']) {
        let close = if rest[start..].starts_with('(') { ')' } else { ']' };
        let Some(len) = rest[start..].find(close) else { break };
        let inside = rest[start + 1..start + len].to_lowercase();
        let noise = inside.split(|c: char| !c.is_alphanumeric() && c != 'é').filter(|w| !w.is_empty()).all(|w| NOISE.contains(&w) || (w == "version" && inside.contains("official")));
        out.push_str(&rest[..start]);
        if !noise {
            out.push_str(&rest[start..=start + len]);
        }
        rest = &rest[start + len + 1..];
    }
    out.push_str(rest);
    let cleaned = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() { title.trim().to_owned() } else { cleaned }
}

/// `ytsearch15:<texte>` : espaces regroupés, 200 caractères au plus, jamais de « - » en tête
/// (sur ordinateur la recherche suit déjà « -- », sur téléphone elle va à Python telle quelle).
fn search_query(text: &str) -> Result<String, JobError> {
    let joined = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let cleaned: String = joined.trim_start_matches(|c: char| c == '-' || c.is_whitespace()).chars().take(200).collect();
    if cleaned.trim().is_empty() {
        return Err(JobError::Failed("Tapez un titre, un artiste ou les deux.".to_owned()));
    }
    Ok(format!("ytsearch{SEARCH_RESULTS}:{}", cleaned.trim()))
}

/// Identifiant d'une vidéo YouTube (11 caractères) dans un lien `watch?v=` ou `/shorts/`.
fn video_id(url: &str) -> Option<&str> {
    let rest = url.split_once("v=").or_else(|| url.split_once("/shorts/"))?.1;
    let id = rest.split(['&', '?', '#', '/']).next()?;
    (id.len() == 11 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')).then_some(id)
}

/// Résultat de recherche YouTube, avant tout téléchargement.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Candidate {
    pub title: String,
    pub artist: String,
    pub duration_secs: Option<f64>,
    pub url: String,
}

/// Candidats lus dans la réponse de `--flat-playlist --dump-single-json` : seuls les liens
/// YouTube sont gardés (ils sont ensuite passés tels quels à yt-dlp).
pub(super) fn parse_candidates(json: &str) -> Vec<Candidate> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else { return Vec::new() };
    let text = |e: &serde_json::Value, key: &str| e.get(key).and_then(|v| v.as_str()).map(str::trim).unwrap_or_default().to_owned();
    value
        .get("entries")
        .and_then(|v| v.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|e| {
                    let url = text(e, "url");
                    let youtube = url.starts_with("https://www.youtube.com/watch?v=")
                        || url.starts_with("https://music.youtube.com/watch?v=")
                        || url.starts_with("https://www.youtube.com/shorts/");
                    let channel = if text(e, "channel").is_empty() { text(e, "uploader") } else { text(e, "channel") };
                    (youtube && !url.chars().any(char::is_whitespace)).then(|| Candidate {
                        title: text(e, "title"),
                        artist: channel.trim_end_matches(" - Topic").to_owned(),
                        duration_secs: e.get("duration").and_then(|v| v.as_f64()),
                        url,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Le candidat qui correspond le mieux (titre, artiste, durée, même version), s'il y en a un.
pub(super) fn pick_candidate<'a>(wanted: &Wanted, candidates: &'a [Candidate]) -> Option<&'a Candidate> {
    use super::matching::{score_match, TrackInfo};
    let asked = TrackInfo { title: &wanted.title, artist: &wanted.artist, duration_secs: wanted.duration_secs };
    candidates
        .iter()
        .map(|c| (c, score_match(asked, TrackInfo { title: &c.title, artist: &c.artist, duration_secs: c.duration_secs })))
        .filter(|(_, score)| score.accepted)
        .fold(None::<(&Candidate, f64)>, |best, (c, score)| match best {
            Some((_, total)) if total >= score.total => best,
            _ => Some((c, score.total)),
        })
        .map(|(c, _)| c)
}

/// Candidats d'une recherche, sans téléchargement.
fn search_candidates(runner: &Runner, search: &str, cancel: &AtomicBool) -> Result<Vec<Candidate>, JobError> {
    let mut command = runner.command();
    command
        .arg("--ignore-config")
        .arg("--flat-playlist")
        .arg("--dump-single-json")
        .arg("--no-warnings")
        .arg("--")
        .arg(search);
    match run_command(&mut command, PROBE_TIMEOUT * 6, cancel)? {
        Bounded::Finished { status, stdout, .. } if status.success() => Ok(parse_candidates(&stdout)),
        Bounded::Finished { stdout, stderr, .. } => Err(classify_failure(&stdout, &stderr)),
        Bounded::TimedOut { .. } => Err(JobError::Timeout),
        Bounded::Cancelled { .. } => Err(JobError::Cancelled),
    }
}

/// Morceau derrière un lien de playlist ou d'album (YouTube, SoundCloud, Audiomack…).
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct LinkEntry {
    pub title: String,
    pub artist: String,
    pub duration_secs: Option<f64>,
    pub url: String,
}

/// Contenu d'un lien direct, sans rien télécharger : (nom, morceaux). Un lien vers un seul son
/// donne une liste vide : il est téléchargé tel quel. Sans cette lecture, une playlist entière
/// partait dans une seule tâche (« plusieurs fichiers pour un seul morceau » sur ordinateur,
/// seul le premier morceau gardé sur téléphone).
pub fn read_media_link(url: &str) -> Result<(String, Vec<LinkEntry>), JobError> {
    let wanted = Wanted { title: String::new(), artist: String::new(), album: String::new(), duration_secs: None, playlist_id: None, url: Some(url.to_owned()) };
    let url = wanted.direct_url().ok_or_else(|| JobError::Failed("Ce lien n'est pas une adresse web valable (http ou https).".to_owned()))?;
    let cancel = AtomicBool::new(false);
    let json = if cfg!(target_os = "android") {
        let zip = super::ytdlp_zip::ensure(super::android::data_dir()?)?;
        super::android::search(&zip, url)?
    } else {
        let runner = resolve_runner(&cancel)?;
        let mut command = runner.command();
        link_arguments(&mut command, url);
        match run_command(&mut command, PROBE_TIMEOUT * 6, &cancel)? {
            Bounded::Finished { status, stdout, .. } if status.success() => stdout,
            Bounded::Finished { stdout, stderr, .. } => return Err(classify_failure(&stdout, &stderr)),
            Bounded::TimedOut { .. } => return Err(JobError::Timeout),
            Bounded::Cancelled { .. } => return Err(JobError::Cancelled),
        }
    };
    Ok(parse_link_entries(&json))
}

/// Lecture « à plat » d'un lien. `--no-playlist` : un son partagé depuis une playlist ou un Mix
/// YouTube (`watch?v=…&list=RD…`) reste ce son-là ; sans lui, le Mix entier (jusqu'à 500
/// morceaux) partait dans la file. Un vrai lien de playlist (`playlist?list=…`, album) est
/// toujours lu en entier : l'option ne vaut que pour les liens qui désignent les deux.
fn link_arguments(command: &mut Command, url: &str) {
    command.arg("--ignore-config").arg("--flat-playlist").arg("--no-playlist").arg("--dump-single-json").arg("--no-warnings").arg("--").arg(url);
}

/// Nom et morceaux d'une réponse `--flat-playlist --dump-single-json` (même forme côté
/// téléphone). Seuls les liens http(s) sans espace sont gardés ; 500 au plus (taille d'un lot).
pub(super) fn parse_link_entries(json: &str) -> (String, Vec<LinkEntry>) {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else { return (String::new(), Vec::new()) };
    let text = |e: &serde_json::Value, key: &str| e.get(key).and_then(|v| v.as_str()).map(str::trim).unwrap_or_default().to_owned();
    let name = text(&value, "title");
    let entries = value
        .get("entries")
        .and_then(|v| v.as_array())
        .map(|entries| {
            entries
                .iter()
                .filter_map(|e| {
                    let url = text(e, "url");
                    let web = url.starts_with("https://") || url.starts_with("http://");
                    let channel = if text(e, "channel").is_empty() { text(e, "uploader") } else { text(e, "channel") };
                    (web && url.len() <= 2000 && !url.chars().any(|c| c.is_whitespace() || c.is_control())).then(|| LinkEntry {
                        title: text(e, "title"),
                        artist: channel.trim_end_matches(" - Topic").to_owned(),
                        duration_secs: e.get("duration").and_then(|v| v.as_f64()).filter(|d| d.is_finite() && *d > 0.0),
                        url,
                    })
                })
                .take(super::queue::MAX_BATCH_SIZE)
                .collect()
        })
        .unwrap_or_default();
    (name, entries)
}

/// Recherche YouTube de 5 candidats (`ytsearch5:`), ou le lien direct demandé.
fn make_query(wanted: &Wanted) -> Result<String, JobError> {
    // Lien direct : passé tel quel (après « -- », il ne peut pas être pris pour une option).
    if wanted.url.is_some() {
        return wanted
            .direct_url()
            .map(str::to_owned)
            .ok_or_else(|| JobError::Failed("Ce lien n'est pas une adresse web valable (http ou https).".to_owned()));
    }
    super::spotdl::make_query(wanted).map(|search| format!("ytsearch5:{search}"))
}

fn run_download(runner: Runner, query: &str, attempt_dir: &Path, cancel: &AtomicBool) -> Result<Bounded, JobError> {
    let mut command = runner.command();
    command
        .arg("--ignore-config")
        .arg("--no-playlist")
        .arg("--extract-audio")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg("0")
        .arg("--embed-metadata")
        .arg("--embed-thumbnail")
        // webp (format courant des vignettes YouTube) ne s'intègre pas partout : converti pour
        // que la pochette se lise sur toutes les versions installées (#25).
        .arg("--convert-thumbnails")
        .arg("jpg")
        .arg("--no-write-info-json")
        .arg("--restrict-filenames")
        .arg("--retries")
        .arg("3")
        .arg("--fragment-retries")
        .arg("3")
        .arg("--extractor-retries")
        .arg("3")
        .arg("--newline")
        // « --print » rend yt-dlp silencieux : l'avancement est demandé quand même.
        .arg("--progress")
        .arg("--paths")
        .arg(attempt_dir)
        .arg("--output")
        .arg(OUTPUT_TEMPLATE)
        .arg("--print")
        .arg("after_move:filepath");
    // FFmpeg hors du PATH : celui installé par spotDL, s'il existe.
    if ffmpeg::find_tool("ffmpeg").is_none() {
        if let Some(location) = spotdl_ffmpeg() {
            command.arg("--ffmpeg-location").arg(location);
        }
    }
    command
        // Le séparateur empêche la recherche d'être interprétée comme une option.
        .arg("--")
        .arg(query);
    // Avancement : chaque « [download] 45.3% » est transmis (yt-dlp écrit une ligne par étape
    // grâce à --newline).
    let watch: Option<crate::bounded::Watch> = progress_sink().map(|sink| {
        std::sync::Arc::new(move |text: &str| {
            for line in text.split(['\n', '\r']) {
                if let Some(part) = ytdlp_percent(line) {
                    sink(Some(part));
                }
            }
        }) as crate::bounded::Watch
    });
    ffmpeg::quiet(&mut command);
    bounded::run_watching(&mut command, COMMAND_TIMEOUT, cancel, watch).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            JobError::ToolMissing("yt-dlp est introuvable. Installez-le séparément puis redémarrez Music Player.".to_owned())
        } else {
            JobError::Failed(format!("Impossible de lancer yt-dlp : {error}"))
        }
    })
}

fn run_command(command: &mut Command, timeout: Duration, cancel: &AtomicBool) -> Result<Bounded, JobError> {
    ffmpeg::quiet(command);
    bounded::run(command, timeout, cancel).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            JobError::ToolMissing("yt-dlp est introuvable. Installez-le séparément puis redémarrez Music Player.".to_owned())
        } else {
            JobError::Failed(format!("Impossible de lancer yt-dlp : {error}"))
        }
    })
}

fn classify_failure(stdout: &str, stderr: &str) -> JobError {
    let detail = join_logs(stdout, stderr, TOOL);
    let lower = detail.to_lowercase();
    if ffmpeg_missing(&lower) {
        return JobError::ToolMissing("FFmpeg est introuvable ; yt-dlp en a besoin pour extraire l'audio.".to_owned());
    }
    if lower.contains("no video results")
        || lower.contains("no matches found")
        || lower.contains("no results")
        || lower.contains("unable to download webpage: http error 404")
    {
        return JobError::NotFound;
    }
    if network_failure(&lower) {
        return JobError::Network(truncate_log(&detail));
    }
    JobError::Failed(truncate_log(&detail))
}

#[cfg(test)]
mod tests {

    #[test]
    fn a_direct_link_is_downloaded_as_is_and_bad_links_are_refused() {
        let mut w = wanted("", "");
        w.url = Some(" https://www.youtube.com/watch?v=abc123 ".to_owned());
        assert_eq!(make_query(&w).expect("lien"), "https://www.youtube.com/watch?v=abc123");
        for bad in ["--exec calc", "file:///etc/passwd", "https://a b", "javascript:alert(1)"] {
            w.url = Some(bad.to_owned());
            assert!(make_query(&w).is_err(), "{bad}");
        }
    }
    use std::sync::atomic::AtomicBool;

    use super::{classify_failure, make_query, parse_candidates, pick_candidate, YtDlpSource, OUTPUT_TEMPLATE};
    use crate::fetch::source::Source;
    use crate::fetch::{JobError, Wanted};

    fn wanted(title: &str, artist: &str) -> Wanted {
        Wanted { title: title.to_owned(), artist: artist.to_owned(), album: String::new(), duration_secs: Some(180.0), playlist_id: None, url: None }
    }

    #[test]
    fn lit_les_morceaux_d_un_lien_de_playlist() {
        let json = r#"{"_type":"playlist","title":"Ma playlist","entries":[
            {"title":"Un","channel":"Artiste - Topic","duration":200,"url":"https://www.youtube.com/watch?v=aaa"},
            {"title":"Deux","uploader":"Autre","url":"https://soundcloud.com/x/deux"},
            {"title":"Piège","url":"file:///etc/passwd"},
            {"title":"Espace","url":"https://a b"}]}"#;
        let (name, entries) = super::parse_link_entries(json);
        assert_eq!(name, "Ma playlist");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].artist, "Artiste");
        assert_eq!(entries[0].duration_secs, Some(200.0));
        assert_eq!(entries[1].url, "https://soundcloud.com/x/deux");
        // Un seul son (pas d'« entries ») : rien à éclater, le lien est téléchargé tel quel.
        assert!(super::parse_link_entries(r#"{"_type":"video","title":"Seul"}"#).1.is_empty());
        assert!(super::parse_link_entries("pas du json").1.is_empty());
    }

    #[test]
    fn un_son_partage_depuis_un_mix_reste_un_seul_son() {
        let mut command = std::process::Command::new("yt-dlp");
        let url = "https://www.youtube.com/watch?v=abc&list=RDabc&start_radio=1";
        super::link_arguments(&mut command, url);
        let args: Vec<String> = command.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert!(args.contains(&"--no-playlist".to_owned()), "{args:?}");
        // Le lien reste un seul argument, après « -- ».
        assert_eq!(args[args.len() - 2..], ["--".to_owned(), url.to_owned()]);
    }

    #[test]
    fn garde_les_shorts_parmi_les_candidats() {
        let json = r#"{"entries":[{"title":"T","channel":"A","duration":50,"url":"https://www.youtube.com/shorts/abc"}]}"#;
        assert_eq!(parse_candidates(json).len(), 1);
    }

    #[test]
    fn recherche_cinq_candidats_et_garde_la_recherche_en_argument_unique() {
        assert_eq!(make_query(&wanted("Titre --help", "Artiste")).expect("recherche"), "ytsearch5:Artiste - Titre --help");
    }

    #[test]
    fn choisit_le_candidat_qui_correspond_comme_spotdl() {
        // Réponse réelle de yt-dlp (18/11), raccourcie : la vidéo officielle a une intro (263 s),
        // la version audio a la durée de Spotify (200 s).
        let json = r#"{"entries":[
            {"title":"The Weeknd - Blinding Lights (Official Video)","channel":"The Weeknd","duration":263,"url":"https://www.youtube.com/watch?v=4NRXx6U8ABQ"},
            {"title":"The Weeknd - Blinding Lights (Official Audio)","channel":"The Weeknd","duration":204,"url":"https://www.youtube.com/watch?v=fHI8X4OXluQ"},
            {"title":"The Weeknd - Blinding Lights (Lyrics)","channel":"7clouds Indie","duration":201,"url":"https://www.youtube.com/watch?v=BwX8OS4cegU"},
            {"title":"The Weeknd - Blinding Lights (Live On The 2020 MTV VMAs)","channel":"The Weeknd","duration":199,"url":"https://www.youtube.com/watch?v=ewfdRy5jfF8"},
            {"title":"Ailleurs","channel":"X","duration":200,"url":"file:///etc/passwd"}]}"#;
        let candidates = parse_candidates(json);
        assert_eq!(candidates.len(), 4, "le lien qui n'est pas YouTube est écarté");
        let mut w = wanted("Blinding Lights", "The Weeknd");
        w.duration_secs = Some(200.0);
        assert_eq!(pick_candidate(&w, &candidates).map(|c| c.url.as_str()), Some("https://www.youtube.com/watch?v=fHI8X4OXluQ"));
        // Rien qui corresponde : rien n'est choisi (et rien n'est téléchargé).
        let other = wanted("Save Your Tears", "The Weeknd");
        assert!(pick_candidate(&other, &candidates).is_none());
    }

    #[test]
    fn refuse_un_titre_vide() {
        assert!(make_query(&wanted(" ", "Artiste")).is_err());
    }

    #[test]
    fn le_modele_de_sortie_est_relatif_et_sans_dossier() {
        // Le dossier passe par `-P` : un `%` dans son nom ne peut pas être pris pour une variable.
        assert!(OUTPUT_TEMPLATE.contains("%(title)s") && OUTPUT_TEMPLATE.ends_with("%(ext)s"));
        assert!(!OUTPUT_TEMPLATE.contains('/') && !OUTPUT_TEMPLATE.contains('\\'));
    }

    #[test]
    fn expose_un_nom_de_source_stable() {
        assert_eq!(YtDlpSource.name(), "yt-dlp");
    }

    #[test]
    fn une_annulation_preexistante_est_detectee_avant_le_lancement() {
        let cancel = AtomicBool::new(true);
        let result = YtDlpSource.fetch(&wanted("Titre", "Artiste"), std::path::Path::new("."), &cancel);
        assert!(matches!(result, Err(JobError::Cancelled)));
    }

    #[test]
    fn classe_les_echecs() {
        assert!(matches!(classify_failure("", "ERROR: ffmpeg not found. Please install"), JobError::ToolMissing(_)));
        assert!(matches!(classify_failure("", "ERROR: [youtube:search] No video results"), JobError::NotFound));
        assert!(matches!(classify_failure("", "ERROR: Unable to download webpage: HTTP Error 404"), JobError::NotFound));
        assert!(matches!(classify_failure("", "ERROR: HTTP Error 429: Too Many Requests"), JobError::Network(_)));
    }

    #[test]
    fn la_recherche_de_l_appli_demande_quinze_resultats_sans_option_cachee() {
        assert_eq!(super::search_query("  The   Weeknd  ").expect("recherche"), "ytsearch15:The Weeknd");
        assert_eq!(super::search_query("--exec calc").expect("recherche"), "ytsearch15:exec calc");
        assert!(super::search_query(" - ").is_err());
        assert!(super::search_query("").is_err());
        assert_eq!(super::search_query(&"a".repeat(500)).expect("recherche").len(), "ytsearch15:".len() + 200);
    }

    #[test]
    fn presente_les_videos_comme_des_morceaux() {
        // Son officiel (chaîne « - Topic ») : le titre tel quel, l'artiste sans « - Topic ».
        assert_eq!(super::present("Blinding Lights", "The Weeknd - Topic", Some(200.0)), ("Blinding Lights".into(), "The Weeknd".into(), 0));
        // Clip : « Artiste - Titre (Official Video) » → titre et artiste.
        assert_eq!(super::present("The Weeknd - Blinding Lights (Official Video)", "TheWeekndVEVO", Some(263.0)), ("Blinding Lights".into(), "The Weeknd".into(), 1));
        assert_eq!(super::present("Damso - Θ. Macarena [Clip Officiel]", "Damso", Some(190.0)), ("Θ. Macarena".into(), "Damso".into(), 1));
        // Ce qui fait partie du titre reste.
        assert_eq!(super::present("Bruxelles vie (feat. Damso) (Remix)", "Label", None).0, "Bruxelles vie (feat. Damso) (Remix)");
        // Compilation d'une heure : à la fin.
        assert_eq!(super::present("Damso - Best of (Mix)", "Fan", Some(3600.0)).2, 3);
        // Rien à séparer : la chaîne est l'artiste.
        assert_eq!(super::present("Macarena", "Damso", Some(190.0)), ("Macarena".into(), "Damso".into(), 2));
    }

    #[test]
    fn range_les_sons_officiels_d_abord_sans_changer_le_reste() {
        let c = |title: &str, artist: &str, d: f64| super::Candidate { title: title.into(), artist: artist.into(), duration_secs: Some(d), url: format!("https://www.youtube.com/watch?v={:0>11}", title.len()) };
        let out = super::present_results(vec![c("Live à Paris", "Fan", 300.0), c("Mix 1h", "Fan", 4000.0), c("Macarena", "Damso - Topic", 190.0), c("Autre", "Fan", 200.0)]);
        assert_eq!(out.iter().map(|r| r.title.as_str()).collect::<Vec<_>>(), ["Macarena", "Live à Paris", "Autre", "Mix 1h"]);
    }

    #[test]
    fn retrouve_la_miniature_d_une_video() {
        assert_eq!(super::video_id("https://www.youtube.com/watch?v=fHI8X4OXluQ"), Some("fHI8X4OXluQ"));
        assert_eq!(super::video_id("https://music.youtube.com/watch?v=fHI8X4OXluQ&list=x"), Some("fHI8X4OXluQ"));
        assert_eq!(super::video_id("https://www.youtube.com/shorts/fHI8X4OXluQ"), Some("fHI8X4OXluQ"));
        assert_eq!(super::video_id("https://www.youtube.com/watch?v=../../x"), None);
        assert_eq!(super::video_id("https://soundcloud.com/a/b"), None);
    }
}
