//! Commandes multimédias du système, sous Windows : morceau en cours (titre, artiste,
//! pochette), lecture/pause, suivant, précédent et position, sur l'écran verrouillé, dans le
//! volet du volume et avec les touches multimédias du clavier.
//!
//! Même principe que la session média d'Android : l'interface publie le morceau
//! (`media_update`, `media_clear`) et les commandes reviennent par l'événement
//! `media-command`, traité par le même code que celles du téléphone (`androidMedia.ts`).
//! Ailleurs (macOS, Linux, Android), les deux commandes ne font rien.

use serde::Deserialize;
#[cfg(windows)]
use serde::Serialize;
use tauri::State;

/// Morceau publié par l'interface.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(windows), allow(dead_code))]
pub struct NowPlaying {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub position_ms: u64,
    pub playing: bool,
    /// Fichier de la pochette (cache de l'appli), ou vide.
    pub cover: String,
}

#[cfg(windows)]
#[derive(Clone, Serialize)]
struct MediaCommand {
    command: &'static str,
    /// Millisecondes (position visée, ou décalage pour `seekBy`).
    arg: f64,
}

#[derive(Default)]
pub struct SystemMedia {
    #[cfg(windows)]
    controls: std::sync::Mutex<Option<souvlaki::MediaControls>>,
}

#[cfg(windows)]
fn command_of(event: souvlaki::MediaControlEvent) -> Option<MediaCommand> {
    use souvlaki::{MediaControlEvent as E, SeekDirection};
    const STEP_MS: f64 = 10_000.0;
    let signed = |d: SeekDirection, ms: f64| if matches!(d, SeekDirection::Forward) { ms } else { -ms };
    let (command, arg) = match event {
        E::Play => ("play", 0.0),
        E::Pause | E::Stop => ("pause", 0.0),
        E::Toggle => ("toggle", 0.0),
        E::Next => ("next", 0.0),
        E::Previous => ("previous", 0.0),
        E::Seek(d) => ("seekBy", signed(d, STEP_MS)),
        E::SeekBy(d, by) => ("seekBy", signed(d, by.as_millis() as f64)),
        E::SetPosition(souvlaki::MediaPosition(at)) => ("seek", at.as_millis() as f64),
        _ => return None,
    };
    Some(MediaCommand { command, arg })
}

impl SystemMedia {
    /// Branche les commandes du système sur la fenêtre principale. Un échec (vieux Windows,
    /// service absent) laisse simplement l'appli sans ces commandes.
    #[cfg(windows)]
    pub fn attach(&self, app: &tauri::AppHandle, window: &tauri::WebviewWindow) {
        use crate::state::Locked;
        use tauri::Emitter;
        let Ok(hwnd) = window.hwnd() else { return };
        let config = souvlaki::PlatformConfig {
            display_name: "Music Player",
            dbus_name: "music_player",
            hwnd: Some(hwnd.0),
        };
        let Ok(mut controls) = souvlaki::MediaControls::new(config) else { return };
        let app = app.clone();
        let attached = controls.attach(move |event| {
            if let Some(command) = command_of(event) {
                let _ = app.emit("media-command", command);
            }
        });
        if attached.is_ok() {
            *self.controls.locked() = Some(controls);
        }
    }

    #[cfg(not(windows))]
    pub fn attach(&self, _app: &tauri::AppHandle, _window: &tauri::WebviewWindow) {}
}

#[tauri::command]
pub fn media_update(media: State<SystemMedia>, info: NowPlaying) {
    #[cfg(windows)]
    {
        use crate::state::Locked;
        use std::time::Duration;
        let mut guard = media.controls.locked();
        let Some(controls) = guard.as_mut() else { return };
        let cover = (!info.cover.is_empty()).then(|| format!("file://{}", info.cover));
        let _ = controls.set_metadata(souvlaki::MediaMetadata {
            title: Some(&info.title),
            artist: Some(&info.artist),
            album: Some(&info.album),
            cover_url: cover.as_deref(),
            duration: (info.duration_ms > 0).then(|| Duration::from_millis(info.duration_ms)),
        });
        let progress = Some(souvlaki::MediaPosition(Duration::from_millis(info.position_ms)));
        let _ = controls.set_playback(if info.playing {
            souvlaki::MediaPlayback::Playing { progress }
        } else {
            souvlaki::MediaPlayback::Paused { progress }
        });
    }
    #[cfg(not(windows))]
    let _ = (media, info);
}

#[tauri::command]
pub fn media_clear(media: State<SystemMedia>) {
    #[cfg(windows)]
    {
        use crate::state::Locked;
        if let Some(controls) = media.controls.locked().as_mut() {
            let _ = controls.set_playback(souvlaki::MediaPlayback::Stopped);
        }
    }
    #[cfg(not(windows))]
    let _ = media;
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use souvlaki::{MediaControlEvent as E, MediaPosition, SeekDirection};
    use std::time::Duration;

    #[test]
    fn system_buttons_become_the_same_commands_as_on_android() {
        let c = |e| command_of(e).map(|m| (m.command, m.arg));
        assert_eq!(c(E::Toggle), Some(("toggle", 0.0)));
        assert_eq!(c(E::Stop), Some(("pause", 0.0)));
        assert_eq!(c(E::Next), Some(("next", 0.0)));
        assert_eq!(c(E::Seek(SeekDirection::Backward)), Some(("seekBy", -10_000.0)));
        assert_eq!(c(E::SeekBy(SeekDirection::Forward, Duration::from_secs(5))), Some(("seekBy", 5_000.0)));
        assert_eq!(c(E::SetPosition(MediaPosition(Duration::from_millis(61_500)))), Some(("seek", 61_500.0)));
        assert_eq!(c(E::SetVolume(0.5)), None);
    }
}
