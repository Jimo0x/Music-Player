//! Commandes Tauri et traitement en arrière-plan de la file de téléchargements.
//!
//! Un seul fil de travail à la fois. Règle du projet : jamais deux verrous tenus en même
//! temps (file, annulations, bibliothèque, racine, playlists) ; les événements partent une
//! fois les verrous relâchés.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::fetch::import::download_and_import;
use crate::fetch::queue::{QueueItem, QueueStore};
use crate::fetch::source::Source;
use crate::fetch::spotdl::SpotDlSource;
use crate::fetch::ytdlp::YtDlpSource;
use crate::fetch::{JobError, JobState, Wanted};
use crate::library::Track;
use crate::state::{AppState, Locked};

const QUEUE_EVENT: &str = "fetch-queue-updated";
const JOB_EVENT: &str = "fetch-job-updated";
const QUEUE_FILE: &str = "telechargements.json";
const PROGRESS_EVENT: &str = "fetch-job-progress";

#[derive(Clone, Serialize)]
struct JobProgress {
    id: String,
    /// Part téléchargée (0 à 1) ; absente si l'outil ne la donne pas (spotDL).
    part: Option<f32>,
}

/// Avancement d'une tâche vers l'interface, au plus un envoi par pourcent ou par quart de seconde.
fn progress_events(app: &tauri::AppHandle, id: &str) -> crate::fetch::common::ProgressSink {
    let app = app.clone();
    let id = id.to_owned();
    let last = Mutex::new((f32::MIN, std::time::Instant::now()));
    Arc::new(move |part: Option<f32>| {
        let value = part.unwrap_or(-1.0);
        {
            let mut last = last.locked();
            let step = (value - last.0).abs() >= 0.01 || value >= 1.0;
            if !step || (last.1.elapsed() < std::time::Duration::from_millis(250) && value < 1.0) {
                return;
            }
            *last = (value, std::time::Instant::now());
        }
        let _ = app.emit(PROGRESS_EVENT, JobProgress { id: id.clone(), part });
    })
}

/// État partagé de la file, enregistré dans Tauri sous `Arc<FetchRuntime>`.
pub struct FetchRuntime {
    queue: Mutex<QueueStore>,
    cancellations: Mutex<HashMap<String, Arc<AtomicBool>>>,
    worker_running: AtomicBool,
}

impl FetchRuntime {
    /// Ouvre la file enregistrée. Fichier illisible pour une raison système : file vide (et le
    /// message), pour que l'appli démarre quand même.
    pub fn open(data_dir: &Path) -> (Self, Option<String>) {
        let path = data_dir.join(QUEUE_FILE);
        let (queue, problem) = match QueueStore::open(&path) {
            Ok(queue) => (queue, None),
            Err(error) => (QueueStore::empty(&path), Some(format!("File de téléchargements illisible, repartie à vide : {error}"))),
        };
        let runtime = Self { queue: Mutex::new(queue), cancellations: Mutex::new(HashMap::new()), worker_running: AtomicBool::new(false) };
        (runtime, problem)
    }
}

#[derive(Debug, Serialize)]
pub struct FetchAvailability {
    pub spotdl: bool,
    pub ytdlp: bool,
}

#[derive(Clone, Debug, Serialize)]
struct JobUpdate<'a> {
    id: &'a str,
    state: JobState,
    error: &'a Option<String>,
}

#[tauri::command(async)]
pub fn fetch_availability() -> FetchAvailability {
    FetchAvailability { spotdl: SpotDlSource.available(), ytdlp: YtDlpSource.available() }
}

#[tauri::command]
pub fn fetch_jobs(runtime: tauri::State<'_, Arc<FetchRuntime>>) -> Vec<QueueItem> {
    runtime.queue.locked().items().to_vec()
}

#[tauri::command]
pub fn fetch_enqueue(app: tauri::AppHandle, runtime: tauri::State<'_, Arc<FetchRuntime>>, wanted: Vec<Wanted>) -> Result<Vec<String>, String> {
    if wanted.is_empty() {
        return Ok(Vec::new());
    }
    let ids = runtime.queue.locked().enqueue_many(wanted).map_err(|error| error.to_string())?;
    emit_queue(&app, runtime.inner());
    start_worker(app, Arc::clone(runtime.inner()));
    Ok(ids)
}

#[tauri::command]
pub fn fetch_cancel(app: tauri::AppHandle, runtime: tauri::State<'_, Arc<FetchRuntime>>, id: String) -> Result<(), String> {
    let active = runtime.cancellations.locked().get(&id).cloned();
    if let Some(cancel) = active {
        cancel.store(true, Ordering::Relaxed);
        return Ok(());
    }
    {
        let mut queue = runtime.queue.locked();
        let item = queue.items().iter().find(|item| item.id == id).ok_or_else(|| "Tâche de téléchargement introuvable.".to_owned())?;
        if item.state != JobState::Queued {
            return Err("Cette tâche ne peut plus être annulée.".to_owned());
        }
        queue.set_state(&id, JobState::Cancelled, Some("Arrêt demandé.".to_owned())).map_err(|error| error.to_string())?;
    }
    emit_job(&app, runtime.inner(), &id);
    emit_queue(&app, runtime.inner());
    Ok(())
}

#[tauri::command]
pub fn fetch_clear_finished(app: tauri::AppHandle, runtime: tauri::State<'_, Arc<FetchRuntime>>) -> Result<usize, String> {
    let removed = runtime.queue.locked().remove_finished().map_err(|error| error.to_string())?;
    emit_queue(&app, runtime.inner());
    Ok(removed)
}

/// Lance le fil de travail s'il ne tourne pas déjà.
pub fn start_worker(app: tauri::AppHandle, runtime: Arc<FetchRuntime>) {
    if runtime.worker_running.swap(true, Ordering::AcqRel) {
        return;
    }
    thread::spawn(move || worker_loop(app, runtime));
}

fn worker_loop(app: tauri::AppHandle, runtime: Arc<FetchRuntime>) {
    drain_queue(
        &runtime,
        |id, wanted, cancel| crate::fetch::common::with_progress(progress_events(&app, id), || run_job(&app, wanted, cancel)),
        |wanted, track| add_imported_track(&app, wanted, track),
        |notice| match notice {
            Notice::Job(id) => emit_job(&app, &runtime, id),
            Notice::Queue => emit_queue(&app, &runtime),
            Notice::Error(message) => {
                let _ = app.emit("fetch-error", message);
            }
        },
    );
}

/// Ce que la boucle annonce à l'interface.
enum Notice<'a> {
    Job(&'a str),
    Queue,
    Error(String),
}

/// Traite la file jusqu'à ce qu'elle soit vide : prend la tâche suivante, la lance (`run`),
/// range le morceau obtenu (`import`), enregistre l'issue et prévient (`notify`). Séparée de
/// Tauri pour être testée avec de faux outils. Jamais deux verrous tenus en même temps.
fn drain_queue(
    runtime: &FetchRuntime,
    run: impl Fn(&str, &Wanted, &AtomicBool) -> Result<Track, JobError>,
    import: impl Fn(&Wanted, Track) -> Result<(), String>,
    notify: impl Fn(Notice<'_>),
) {
    loop {
        let next = {
            let mut queue = runtime.queue.locked();
            let Some(item) = queue.next_queued() else {
                // Remis à faux file verrouillée : un ajout concurrent relancera un fil.
                runtime.worker_running.store(false, Ordering::Release);
                return;
            };
            let (id, wanted) = (item.id.clone(), item.wanted.clone());
            if let Err(error) = queue.set_state(&id, JobState::Running, None) {
                runtime.worker_running.store(false, Ordering::Release);
                drop(queue);
                notify(Notice::Error(format!("Impossible de démarrer la tâche : {error}")));
                return;
            }
            (id, wanted)
        };
        let (id, wanted) = next;
        let cancel = Arc::new(AtomicBool::new(false));
        runtime.cancellations.locked().insert(id.clone(), Arc::clone(&cancel));
        notify(Notice::Job(&id));
        notify(Notice::Queue);

        let (state, error) = match run(&id, &wanted, &cancel) {
            Ok(track) => match import(&wanted, track) {
                Ok(()) => (JobState::Done, None),
                Err(detail) => (JobState::Failed, Some(detail)),
            },
            Err(JobError::Cancelled) => (JobState::Cancelled, Some("Arrêt demandé.".to_owned())),
            Err(error) => (JobState::Failed, Some(error.to_string())),
        };

        runtime.cancellations.locked().remove(&id);
        let saved = runtime.queue.locked().set_state(&id, state, error);
        if let Err(save_error) = saved {
            notify(Notice::Error(format!("Impossible d'enregistrer l'état de la tâche : {save_error}")));
        }
        notify(Notice::Job(&id));
        notify(Notice::Queue);
    }
}

/// spotDL d'abord ; yt-dlp s'il est absent, ne trouve rien ou se trompe de morceau.
fn run_job(app: &tauri::AppHandle, wanted: &Wanted, cancel: &AtomicBool) -> Result<Track, JobError> {
    let root = app
        .state::<AppState>()
        .library_root
        .locked()
        .clone()
        .ok_or_else(|| JobError::Failed("Choisissez d'abord le dossier de votre bibliothèque.".to_owned()))?;
    let root = PathBuf::from(root);
    // Téléphone : yt-dlp intégré seul ; travail dans les données privées, morceau rangé dans
    // « Musique/Music Player/Téléchargés » (la racine du stockage est interdite d'écriture).
    if cfg!(target_os = "android") {
        let state = app.state::<AppState>();
        let destination = crate::sharing::downloads_dir(&state);
        let staging = state.data_dir.join(super::import::STAGING_DIR);
        return super::import::download_and_import_into(&YtDlpSource, wanted, &destination, &staging, cancel);
    }
    // Lien direct : yt-dlp seul (spotDL ne lit que des liens Spotify, résolus avant la file).
    if wanted.url.is_some() {
        return download_and_import(&YtDlpSource, wanted, &root, cancel);
    }
    match download_and_import(&SpotDlSource, wanted, &root, cancel) {
        Err(JobError::NotFound | JobError::Failed(_) | JobError::ToolMissing(_)) if !cancel.load(Ordering::Relaxed) => {
            download_and_import(&YtDlpSource, wanted, &root, cancel)
        }
        other => other,
    }
}

/// Morceau importé : bibliothèque, playlist demandée, cache, puis l'interface prévenue.
/// Un verrou à la fois (la racine est lue avant de toucher à la bibliothèque).
fn add_imported_track(app: &tauri::AppHandle, wanted: &Wanted, track: Track) -> Result<(), String> {
    let state = app.state::<AppState>();
    let track_id = track.id.clone();
    {
        let mut library = state.library.locked();
        if !library.iter().any(|existing| existing.id == track.id) {
            library.push(track);
        }
    }
    if let Some(playlist_id) = &wanted.playlist_id {
        let mut playlists = state.playlists.locked();
        playlists
            .add_tracks(playlist_id, vec![track_id])
            .map_err(|_| "Playlist introuvable ; morceau conservé dans la bibliothèque.".to_owned())?;
        playlists.save(&state.playlists_path()).map_err(|error| format!("Morceau importé, mais playlist non enregistrée : {error}"))?;
    }
    let root = state.library_root.locked().clone();
    if let Some(root) = root {
        let library = state.library.locked();
        let _ = crate::library::save_cache(&state.library_cache_path(), &root, &library);
    }
    let _ = app.emit("library-updated", &*state.library.locked());
    Ok(())
}

fn emit_job(app: &tauri::AppHandle, runtime: &FetchRuntime, id: &str) {
    let queue = runtime.queue.locked();
    if let Some(item) = queue.items().iter().find(|item| item.id == id) {
        let _ = app.emit(JOB_EVENT, JobUpdate { id: &item.id, state: item.state, error: &item.last_error });
    }
}

fn emit_queue(app: &tauri::AppHandle, runtime: &FetchRuntime) {
    let _ = app.emit(QUEUE_EVENT, runtime.queue.locked().items());
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Mutex;

    use super::{drain_queue, FetchRuntime, Notice};
    use crate::fetch::{JobError, JobState, Wanted};
    use crate::library::Track;
    use crate::state::Locked;

    fn wanted(title: &str) -> Wanted {
        Wanted { title: title.into(), artist: "Artiste".into(), album: String::new(), duration_secs: None, playlist_id: None, url: None }
    }

    fn track(title: &str) -> Track {
        Track {
            id: format!("id-{title}"),
            path: format!("/x/{title}.mp3"),
            title: title.into(),
            artist: "Artiste".into(),
            album: String::new(),
            album_artist: "Artiste".into(),
            track_no: None,
            disc_no: None,
            year: None,
            genre: None,
            duration_secs: 1.0,
            has_cover: false,
            added_secs: 0,
            damage: None,
        }
    }

    #[test]
    fn the_queue_is_worked_through_and_each_outcome_is_saved() {
        let dir = tempfile::tempdir().unwrap();
        let (runtime, problem) = FetchRuntime::open(dir.path());
        assert!(problem.is_none());
        let titles = ["Trouvé", "Introuvable", "Arrêté", "Rangement raté"];
        runtime.queue.locked().enqueue_many(titles.iter().map(|t| wanted(t)).collect()).unwrap();
        runtime.worker_running.store(true, Ordering::Release);

        let imported = Mutex::new(Vec::new());
        let notices = Mutex::new(Vec::new());
        let during_run_cancellable = AtomicBool::new(false);
        drain_queue(
            &runtime,
            |id, wanted, _| {
                // Pendant le travail, la tâche peut être arrêtée (drapeau enregistré).
                if runtime.cancellations.locked().contains_key(id) {
                    during_run_cancellable.store(true, Ordering::Relaxed);
                }
                match wanted.title.as_str() {
                    "Introuvable" => Err(JobError::NotFound),
                    "Arrêté" => Err(JobError::Cancelled),
                    other => Ok(track(other)),
                }
            },
            |wanted, track| {
                if wanted.title == "Rangement raté" {
                    return Err("Playlist introuvable.".into());
                }
                imported.lock().unwrap().push(track.title);
                Ok(())
            },
            |notice| {
                notices.lock().unwrap().push(match notice {
                    Notice::Job(_) => "tâche",
                    Notice::Queue => "file",
                    Notice::Error(_) => "erreur",
                })
            },
        );

        let queue = runtime.queue.locked();
        let state = |title: &str| queue.items().iter().find(|i| i.wanted.title == title).map(|i| (i.state, i.last_error.clone()));
        assert_eq!(state("Trouvé"), Some((JobState::Done, None)));
        assert!(matches!(state("Introuvable"), Some((JobState::Failed, Some(_)))));
        assert!(matches!(state("Arrêté"), Some((JobState::Cancelled, Some(_)))));
        assert_eq!(state("Rangement raté"), Some((JobState::Failed, Some("Playlist introuvable.".into()))));
        assert_eq!(*imported.lock().unwrap(), vec!["Trouvé".to_string()]);
        assert!(during_run_cancellable.load(Ordering::Relaxed));
        assert!(runtime.cancellations.locked().is_empty(), "drapeaux d'arrêt retirés");
        assert!(!runtime.worker_running.load(Ordering::Acquire), "fil arrêté, file vide");
        // Chaque tâche : départ puis fin, chacun annoncé à l'interface (tâche + file).
        assert_eq!(notices.lock().unwrap().len(), titles.len() * 4);
    }

    #[test]
    fn an_empty_queue_stops_at_once() {
        let dir = tempfile::tempdir().unwrap();
        let (runtime, _) = FetchRuntime::open(dir.path());
        runtime.worker_running.store(true, Ordering::Release);
        drain_queue(&runtime, |_, _, _| unreachable!("rien à lancer"), |_, _| Ok(()), |_| {});
        assert!(!runtime.worker_running.load(Ordering::Acquire));
    }
}
