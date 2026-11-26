// Miroir TypeScript des structures Rust (serde) de src-tauri/src/*.rs, à garder
// synchronisé à la main avec les `#[derive(Serialize)]` côté Rust.

export interface Track {
  id: string;
  path: string;
  title: string;
  artist: string;
  album: string;
  album_artist: string;
  track_no: number | null;
  disc_no: number | null;
  year: number | null;
  genre: string | null;
  duration_secs: number;
  has_cover: boolean;
  added_secs: number;
  /** Fichier illisible (abîmé, téléchargement inachevé) : la raison. */
  damage?: string | null;
}

export type RepeatMode = "off" | "one" | "all";

export interface QueueView {
  track_ids: string[];
  position: number | null;
  shuffle: boolean;
  repeat: RepeatMode;
}

export interface PlaybackStatus {
  current_track: Track | null;
  position_secs: number;
  is_paused: boolean;
  volume: number;
}

export interface Playlist {
  id: string;
  name: string;
  track_ids: string[];
  /** Thème de la jaquette générée (voir lib/playlistThemes.ts). */
  theme: string;
  /** Personnalisation de la jaquette et de la page (image, police…). */
  style?: PlaylistStyle | null;
}

/** Jaquette et page d'une playlist personnalisées (tout est facultatif). */
export interface PlaylistStyle {
  /** Image choisie (fichier du dossier de personnalisation), à la place du dégradé. */
  image?: string | null;
  /** Cadrage de l'image : centre horizontal et vertical (0 à 100 %), zoom (1 à 3). */
  imageX?: number;
  imageY?: number;
  imageZoom?: number;
  /** Assombrissement de l'image sous le nom (0 à 0,7), pour la lisibilité. */
  dim?: number;
  /** Afficher le nom sur la jaquette. */
  showName?: boolean;
  /** Police, couleur, taille (multiplicateur) et position du nom. */
  font?: string | null;
  ink?: string | null;
  size?: number;
  align?: "start" | "center" | "end";
  place?: "top" | "middle" | "bottom";
  uppercase?: boolean;
  /** Motif décoratif du dégradé. */
  pattern?: "rings" | "none" | "dots" | "stripes" | "grid" | "waves" | "stars" | "diamonds" | "hexagons" | "grain";
  /** Jaquette faite des pochettes de la playlist (lot 0.12.0) : une seule, ou une mosaïque de
   *  quatre albums différents. Une image choisie reste prioritaire. */
  artwork?: "single" | "mosaic" | null;
  /** Police du titre sur la page de la playlist. */
  titleFont?: string | null;
}

export interface LyricWord {
  time_ms: number;
  text: string;
}

export interface LyricLine {
  time_ms: number;
  text: string;
  /** Minutage mot par mot (LRC enrichi), quand la source le donne. */
  words?: LyricWord[] | null;
}

export interface Lyrics {
  synced: LyricLine[] | null;
  plain: string | null;
  instrumental: boolean;
  source: string;
}

/** Passe d'amélioration des paroles (version synchronisée cherchée en arrière-plan). */
export interface LyricsUpgrade {
  lyrics: Lyrics | null;
  /** Service de paroles surchargé : une nouvelle passe plus tard peut réussir. */
  retry: boolean;
}

/** Modèle de reconnaissance vocale de la synchronisation automatique des paroles. */
export interface AlignModelStatus {
  installed: boolean;
  /** Taille du téléchargement (octets). */
  bytes: number;
}

/** Proposition de la recherche manuelle de paroles (LRCLIB). */
export interface LyricsCandidate {
  id: number;
  title: string;
  artist: string;
  album: string | null;
  duration: number | null;
  synced: boolean;
  instrumental: boolean;
  preview: string | null;
}

export interface ScanProgress {
  done: number;
  total: number;
}

export interface InitialState {
  library_root: string | null;
  library: Track[];
  queue: QueueView;
  current_track: Track | null;
  position_secs: number;
  volume: number;
  eq_gains: [number, number, number];
  playlists: Playlist[];
  /** Identifiants des morceaux aimés. */
  favorites: string[];
  scanning: boolean;
  /** Réglages d'apparence enregistrés (null : apparence d'origine). */
  appearance: unknown;
  /** Dossier des fichiers importés (polices, images). */
  custom_dir: string;
}

/** Album reconstitué côté interface à partir des pistes. */
export interface Album {
  key: string;
  title: string;
  artist: string;
  year: number | null;
  tracks: Track[];
  /** Piste dont on affiche la pochette (la première qui en a une). */
  coverTrack: Track | null;
  addedSecs: number;
  durationSecs: number;
}

export interface Artist {
  name: string;
  albums: Album[];
  tracks: Track[];
}

/** Ce que l'on partage avec un autre appareil. */
export interface ShareItem {
  /** « backup » : contenu JSON (id) envoyé par code, pour relier un autre appareil. */
  /** « copy » : « jeton|id1,id2,… », copie vers un autre de mes appareils. */
  kind: "track" | "playlist" | "backup" | "copy";
  id: string;
}

/** Offre d'envoi : montrée à la personne qui reçoit avant qu'elle accepte. */
export interface ShareOffer {
  from: string;
  /** « copie » : envoi d'un autre de mes appareils (name = jeton de la demande). */
  kind: "track" | "playlist" | "sauvegarde" | "copie";
  name: string;
  tracks: number;
  size: number;
}

/** Appareil avec Music Player ouvert sur le même réseau. */
export interface NearbyPeer {
  id: string;
  name: string;
  addresses: string[];
  /** Code d'ami (Proches), quand cet appareil l'a activé. */
  code?: string | null;
}

/** Avancement d'un envoi ou d'une réception. */
export interface TransferProgress {
  id: string;
  /** « reprise » : connexion perdue, nouvel essai automatique (done = essai, total = essais permis). */
  stage: "attente" | "envoi" | "reception" | "reprise" | "fini" | "refuse" | "erreur";
  done: number;
  total: number;
  message: string | null;
  /** Envoi par cycles (#11) : partie en cours (à partir de 1), nombre de parties, morceaux
   *  des parties terminées et de tout l'envoi. Absents pour un envoi en une fois. */
  part?: number;
  parts?: number;
  tracks_done?: number;
  tracks_total?: number;
}

/** Doublons retirés, et fichiers que l'appli n'a pas pu effacer elle-même (chemins). */
export interface DuplicateOutcome {
  removed: number;
  failed: string[];
}

/** Résultat de l'import d'une archive de partage (`.mpshare`). */
export interface ShareImportResult {
  name: string;
  kind: "track" | "playlist";
  added_tracks: number;
  skipped_tracks: number;
  playlist_id: string | null;
  track_ids: string[];
  /** Déjà dans la bibliothèque : fusionnés avec le morceau présent (#11). */
  merged_tracks?: number;
  /** Déjà dans la playlist : ignorés (#11). */
  duplicate_tracks?: number;
}

/** Nouvelle version proposée par la mise à jour automatique (ordinateur). */
export interface UpdateInfo {
  version: string;
  current: string;
  notes: string | null;
  /** Android : adresse de l'APK à télécharger. */
  url: string | null;
  /** Android : taille de l'APK (octets). */
  size: number | null;
  /** Android : empreinte SHA-256 publiée par GitHub, vérifiée avant l'installation. */
  sha256: string | null;
}

/** File de téléchargements (spotDL puis yt-dlp), miroir de `fetch::queue::QueueItem`. */
export type FetchJobState = "queued" | "running" | "done" | "failed" | "cancelled";

export interface FetchWanted {
  title: string;
  artist: string;
  album?: string;
  duration_secs?: number | null;
  /** Playlist où ranger le morceau une fois importé. */
  playlist_id?: string | null;
  /** Lien direct (YouTube, SoundCloud…), téléchargé tel quel par yt-dlp. */
  url?: string | null;
}

export interface FetchJob {
  id: string;
  wanted: FetchWanted;
  state: FetchJobState;
  attempts: number;
  last_error: string | null;
}

export interface FetchAvailability {
  spotdl: boolean;
  ytdlp: boolean;
}

/** Résultat de la recherche de la page Télécharger (`fetch::ytdlp::SearchResult`). */
export interface SearchResult {
  title: string;
  artist: string;
  duration_secs: number | null;
  url: string;
  thumbnail: string | null;
}

export interface FetchJobUpdate {
  id: string;
  state: FetchJobState;
  error: string | null;
}

/** Contenu d'un lien direct lu par yt-dlp (`fetch::link::MediaLink`) : vide pour un seul son. */
export interface MediaLink {
  name: string;
  tracks: { title: string; artist: string; duration_secs: number | null; url: string }[];
}

/** Playlist lue depuis un lien Spotify (`fetch::link::LinkPlaylist`). */
export interface LinkPlaylist {
  name: string;
  tracks: { title: string; artist: string; album: string; duration_secs: number | null }[];
  /** Liste coupée par Spotify (100 premiers morceaux d'une longue playlist). */
  partial?: boolean;
}
