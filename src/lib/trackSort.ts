// Tris des listes de morceaux, partagés par la page Morceaux et la page d'une playlist (le
// code vient de SongsView, déplacé tel quel).

import { collator } from "./format";
import type { Track } from "./types";

export type SortKey = "title" | "artist" | "album" | "duration" | "added";
export const SORT_KEYS: readonly SortKey[] = ["title", "artist", "album", "duration", "added"];

export const comparators: Record<SortKey, (a: Track, b: Track) => number> = {
  title: (a, b) => collator.compare(a.title, b.title),
  artist: (a, b) => collator.compare(a.artist, b.artist) || collator.compare(a.title, b.title),
  album: (a, b) => collator.compare(a.album, b.album) || (a.track_no ?? 0) - (b.track_no ?? 0),
  duration: (a, b) => a.duration_secs - b.duration_secs,
  added: (a, b) => a.added_secs - b.added_secs || collator.compare(a.title, b.title),
};

// Tris proposés dans le menu (téléphone compris, où les en-têtes de colonnes sont masqués).
export const SORTS: { id: string; label: string; key: SortKey; ascending: boolean }[] = [
  { id: "title", label: "Titre (A-Z)", key: "title", ascending: true },
  { id: "title-desc", label: "Titre (Z-A)", key: "title", ascending: false },
  { id: "recent", label: "Plus récents", key: "added", ascending: false },
  { id: "oldest", label: "Plus anciens", key: "added", ascending: true },
  { id: "artist", label: "Artiste (A-Z)", key: "artist", ascending: true },
  { id: "album", label: "Album (A-Z)", key: "album", ascending: true },
  { id: "duration", label: "Durée", key: "duration", ascending: true },
];

/** Liste triée (copie), dans le sens demandé. */
export function sortTracks(tracks: Track[], key: SortKey, ascending: boolean): Track[] {
  const sorted = [...tracks].sort(comparators[key]);
  return ascending ? sorted : sorted.reverse();
}
