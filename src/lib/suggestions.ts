// Propositions affichées quand la file est finie (« Il n'y a plus de morceaux ») : réécouter,
// dans la même veine, les plus écoutés, les titres aimés, une playlist, ou au hasard.

import type { Listen } from "./recap";
import type { Playlist, Track } from "./types";

export interface Suggestion {
  id: string;
  title: string;
  sub: string;
  icon: "repeat" | "sparkle" | "stats" | "heart" | "playlist" | "shuffle";
  trackIds: string[];
  /** Lu dans le désordre (sinon dans l'ordre donné). */
  shuffled: boolean;
}

export interface SuggestionInput {
  /** Dernier morceau écouté (celui qui vient de finir), s'il est connu. */
  last: Track | null;
  /** File qui vient de se terminer. */
  queueIds: string[];
  /** Bibliothèque de musique (sans les vocaux). */
  tracks: Track[];
  favoriteIds: string[];
  listens: Listen[];
  playlists: Playlist[];
  /** Tirage au hasard (remplaçable dans les tests). */
  random?: () => number;
}

const MAX = 50;

const count = (n: number) => `${n} morceau${n > 1 ? "x" : ""}`;

function sample<T>(items: T[], n: number, random: () => number): T[] {
  const copy = [...items];
  for (let i = copy.length - 1; i > 0; i--) {
    const j = Math.floor(random() * (i + 1));
    [copy[i], copy[j]] = [copy[j], copy[i]];
  }
  return copy.slice(0, n);
}

export function endOfQueueSuggestions(input: SuggestionInput): Suggestion[] {
  const random = input.random ?? Math.random;
  const known = new Set(input.tracks.map((t) => t.id));
  const out: Suggestion[] = [];
  const add = (s: Omit<Suggestion, "sub"> & { sub?: string }) => {
    const ids = s.trackIds.filter((id) => known.has(id));
    if (ids.length) out.push({ ...s, trackIds: ids, sub: s.sub ?? count(ids.length) });
  };

  add({ id: "replay", title: "Réécouter", icon: "repeat", trackIds: input.queueIds, shuffled: false });

  // Dans la même veine : même artiste, puis même album, puis même genre.
  const last = input.last;
  if (last) {
    const same = (a: string, b: string) => a.trim().toLowerCase() === b.trim().toLowerCase();
    const others = input.tracks.filter((t) => t.id !== last.id);
    const byArtist = others.filter((t) => same(t.artist, last.artist) || same(t.album_artist, last.album_artist));
    const byAlbum = others.filter((t) => same(t.album, last.album));
    const byGenre = last.genre ? others.filter((t) => t.genre && same(t.genre, last.genre!)) : [];
    const ids = [...new Set([...byArtist, ...byAlbum, ...byGenre].map((t) => t.id))].slice(0, MAX);
    add({ id: "similar", title: "Dans la même veine", sub: `Autour de ${last.artist}`, icon: "sparkle", trackIds: ids, shuffled: true });
  }

  // Les plus écoutés : nombre d'écoutes, du plus au moins écouté.
  const plays = new Map<string, number>();
  for (const l of input.listens) plays.set(l.track_id, (plays.get(l.track_id) ?? 0) + 1);
  const top = [...plays.entries()]
    .filter(([id]) => known.has(id))
    .sort((a, b) => b[1] - a[1])
    .slice(0, MAX)
    .map(([id]) => id);
  add({ id: "top", title: "Vos plus écoutés", icon: "stats", trackIds: top, shuffled: false });

  add({ id: "favorites", title: "Titres aimés", icon: "heart", trackIds: input.favoriteIds, shuffled: true });

  // Trois playlists non vides, en commençant par celles qui ne sont pas la file qui vient de finir.
  const finished = input.queueIds.join("|");
  input.playlists
    .filter((p) => p.track_ids.some((id) => known.has(id)) && p.track_ids.join("|") !== finished)
    .slice(0, 3)
    .forEach((p) => add({ id: `playlist:${p.id}`, title: p.name, icon: "playlist", trackIds: p.track_ids, shuffled: false }));

  // Au hasard, de préférence parmi les morceaux jamais écoutés.
  const unheard = input.tracks.filter((t) => !plays.has(t.id));
  const pool = unheard.length >= 10 ? unheard : input.tracks;
  add({
    id: "random",
    title: unheard.length >= 10 ? "À redécouvrir" : "Au hasard",
    sub: unheard.length >= 10 ? "Des morceaux jamais écoutés" : undefined,
    icon: "shuffle",
    trackIds: sample(pool, MAX, random).map((t) => t.id),
    shuffled: false,
  });

  return out;
}
