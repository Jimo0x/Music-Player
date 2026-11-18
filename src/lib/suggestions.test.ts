import { describe, expect, it } from "vitest";
import type { Listen } from "./recap";
import { endOfQueueSuggestions } from "./suggestions";
import type { Track } from "./types";

function track(id: string, artist: string, extra: Partial<Track> = {}): Track {
  return {
    id,
    path: `/m/${id}.mp3`,
    title: id,
    artist,
    album: `Album ${artist}`,
    album_artist: artist,
    track_no: null,
    disc_no: null,
    year: null,
    genre: null,
    duration_secs: 180,
    has_cover: false,
    added_secs: 0,
    damage: null,
    ...extra,
  };
}

const listen = (track_id: string): Listen => ({ track_id, title: track_id, artist: "", album: "", started: 0, secs: 180, duration_secs: 180 });

describe("propositions en fin de file", () => {
  const tracks = [track("a1", "Damso"), track("a2", "Damso"), track("b1", "Aya"), track("b2", "Aya"), track("c1", "Tiakola", { genre: "Rap" })];

  it("propose de réécouter, la même veine, les plus écoutés, les aimés et les playlists", () => {
    const s = endOfQueueSuggestions({
      last: tracks[0],
      queueIds: ["a1", "b1"],
      tracks,
      favoriteIds: ["b2", "inconnu"],
      listens: [listen("b1"), listen("b1"), listen("a2"), listen("disparu")],
      playlists: [
        { id: "p1", name: "Soirée", track_ids: ["c1"], theme: "x" },
        { id: "p2", name: "Vide", track_ids: [], theme: "x" },
        { id: "p3", name: "La file", track_ids: ["a1", "b1"], theme: "x" },
      ],
      random: () => 0,
    });
    const byId = Object.fromEntries(s.map((x) => [x.id, x]));
    expect(byId.replay.trackIds).toEqual(["a1", "b1"]);
    expect(byId.similar.trackIds).toEqual(["a2"]);
    expect(byId.top.trackIds).toEqual(["b1", "a2"]);
    expect(byId.favorites.trackIds).toEqual(["b2"]);
    expect(s.filter((x) => x.icon === "playlist").map((x) => x.title)).toEqual(["Soirée"]);
    expect(byId.random.trackIds).toHaveLength(5);
  });

  it("ne propose rien de vide (bibliothèque sans écoute ni favori)", () => {
    const s = endOfQueueSuggestions({ last: null, queueIds: [], tracks: [track("x", "Solo")], favoriteIds: [], listens: [], playlists: [] });
    expect(s.map((x) => x.id)).toEqual(["random"]);
  });

  it("préfère les morceaux jamais écoutés quand il y en a assez", () => {
    const many = Array.from({ length: 12 }, (_, i) => track(`t${i}`, "X"));
    const s = endOfQueueSuggestions({ last: null, queueIds: [], tracks: [...many, track("vu", "Y")], favoriteIds: [], listens: [listen("vu")], playlists: [] });
    const random = s.find((x) => x.id === "random")!;
    expect(random.title).toBe("À redécouvrir");
    expect(random.trackIds).not.toContain("vu");
  });
});
