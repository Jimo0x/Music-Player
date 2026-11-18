//! Liens Spotify lus sans spotDL ni compte : la page « embed » publique (le petit lecteur que
//! les sites intègrent) contient le titre, les artistes et la durée des morceaux. Marche
//! partout, téléphones compris. Limite de Spotify : 100 morceaux au plus pour une playlist ;
//! au-delà, spotDL (ordinateur) ou l'export des données du compte prennent le relais.

use std::time::Duration;

use serde_json::Value;

use super::link::{LinkPlaylist, LinkTrack};
use super::JobError;

/// Nombre de morceaux au-delà duquel la page embed coupe une playlist.
pub const EMBED_LIMIT: usize = 100;
const MAX_PAGE_BYTES: u64 = 8 * 1024 * 1024;
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36";

/// `kind` : « playlist », « album » ou « track » ; `id` : identifiant Spotify déjà validé.
pub fn read(kind: &str, id: &str) -> Result<LinkPlaylist, JobError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(20)))
        .http_status_as_error(false)
        .build()
        .into();
    let url = format!("https://open.spotify.com/embed/{kind}/{id}");
    let mut response = agent
        .get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept-Language", "fr,en;q=0.8")
        .call()
        .map_err(|error| JobError::Network(format!("Spotify ne répond pas : {error}")))?;
    match response.status().as_u16() {
        200 => {}
        404 => return Err(JobError::NotFound),
        code => return Err(JobError::Network(format!("Spotify a répondu {code}."))),
    }
    let page = response
        .body_mut()
        .with_config()
        .limit(MAX_PAGE_BYTES)
        .read_to_string()
        .map_err(|error| JobError::Network(format!("Page Spotify illisible : {error}")))?;
    parse_page(&page)
}

/// Lit la liste dans le JSON `__NEXT_DATA__` de la page.
pub fn parse_page(page: &str) -> Result<LinkPlaylist, JobError> {
    let unreadable = || JobError::Failed("Spotify a changé sa page : la liste des morceaux est illisible.".to_owned());
    let start_tag = page.find("<script id=\"__NEXT_DATA__\"").ok_or_else(unreadable)?;
    let start = page[start_tag..].find('>').map(|i| start_tag + i + 1).ok_or_else(unreadable)?;
    let end = page[start..].find("</script>").map(|i| start + i).ok_or_else(unreadable)?;
    let data: Value = serde_json::from_str(&page[start..end]).map_err(|_| unreadable())?;
    let entity = data.pointer("/props/pageProps/state/data/entity").ok_or_else(unreadable)?;

    let text = |v: &Value, key: &str| v.get(key).and_then(Value::as_str).map(str::trim).unwrap_or_default().to_owned();
    let secs = |v: &Value| v.get("duration").and_then(Value::as_f64).filter(|ms| *ms > 0.0).map(|ms| (ms / 1000.0).round());
    let name = {
        let n = text(entity, "name");
        if n.is_empty() { text(entity, "title") } else { n }
    };

    let tracks: Vec<LinkTrack> = match entity.get("type").and_then(Value::as_str) {
        Some("track") => {
            let artists: Vec<String> = entity
                .get("artists")
                .and_then(Value::as_array)
                .map(|list| list.iter().filter_map(|a| a.get("name").and_then(Value::as_str)).map(str::to_owned).collect())
                .unwrap_or_default();
            vec![LinkTrack { title: name.clone(), artist: artists.join(", "), album: String::new(), duration_secs: secs(entity) }]
        }
        Some("playlist" | "album") => {
            let album = if entity.get("type").and_then(Value::as_str) == Some("album") { name.clone() } else { String::new() };
            entity
                .get("trackList")
                .and_then(Value::as_array)
                .ok_or_else(unreadable)?
                .iter()
                .map(|t| LinkTrack {
                    title: text(t, "title"),
                    // « Artiste 1,\u{a0}Artiste 2 » : espaces insécables remis en espaces simples.
                    artist: text(t, "subtitle").replace('\u{a0}', " "),
                    album: album.clone(),
                    duration_secs: secs(t),
                })
                .filter(|t| !t.title.is_empty())
                .collect()
        }
        _ => return Err(unreadable()),
    };
    if tracks.is_empty() {
        return Err(JobError::NotFound);
    }
    Ok(LinkPlaylist { name, tracks, partial: false })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(entity: &str) -> String {
        format!(r#"<html><script id="__NEXT_DATA__" type="application/json">{{"props":{{"pageProps":{{"state":{{"data":{{"entity":{entity}}}}}}}}}}}</script></html>"#)
    }

    #[test]
    fn reads_a_playlist_with_its_artists_and_durations() {
        let list = parse_page(&page(
            r#"{"type":"playlist","name":"Soirée","trackList":[{"title":"Blinding Lights","subtitle":"The Weeknd","duration":200040},{"title":"BbY WOW","subtitle":"KAROL G, Judeline","duration":225834},{"title":"","subtitle":"x"}]}"#,
        ))
        .unwrap();
        assert_eq!(list.name, "Soirée");
        assert_eq!(list.tracks.len(), 2);
        assert_eq!(list.tracks[1].artist, "KAROL G, Judeline");
        assert_eq!(list.tracks[0].duration_secs, Some(200.0));
        assert_eq!(list.tracks[0].album, "");
    }

    #[test]
    fn reads_an_album_and_a_single_track() {
        let album = parse_page(&page(r#"{"type":"album","name":"After Hours","trackList":[{"title":"Alone Again","subtitle":"The Weeknd","duration":250000}]}"#)).unwrap();
        assert_eq!(album.tracks[0].album, "After Hours");
        let track = parse_page(&page(r#"{"type":"track","name":"Blinding Lights","artists":[{"name":"The Weeknd"}],"duration":200040}"#)).unwrap();
        assert_eq!(track.tracks[0].artist, "The Weeknd");
        assert_eq!(track.tracks[0].title, "Blinding Lights");
    }

    #[test]
    fn a_changed_page_gives_a_clear_error() {
        assert!(parse_page("<html>rien</html>").is_err());
        assert!(parse_page(&page(r#"{"type":"show"}"#)).is_err());
    }

    /// Essai réel (réseau) : `cargo test --lib -- --ignored --nocapture lit_une_vraie_page_spotify`.
    #[test]
    #[ignore]
    fn lit_une_vraie_page_spotify() {
        let list = read("album", "4yP0hdKOZPNshxUOjY0cZj").unwrap();
        println!("{} : {} morceaux, premier {:?}", list.name, list.tracks.len(), list.tracks.first());
        assert_eq!(list.tracks.len(), 14);
    }
}
