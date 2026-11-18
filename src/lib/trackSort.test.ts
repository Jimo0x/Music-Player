import { describe, expect, it } from "vitest";
import { SORTS, sortTracks } from "./trackSort";
import type { Track } from "./types";

const t = (title: string, artist: string, added: number, duration = 100): Track =>
  ({ id: title, path: `/${title}`, title, artist, album: "A", album_artist: artist, track_no: null, disc_no: null, year: null, genre: null, duration_secs: duration, has_cover: false, added_secs: added }) as Track;

describe("tri des morceaux", () => {
  const list = [t("Bravo", "Zed", 2, 300), t("alpha", "Ana", 3, 100), t("Écho", "Ana", 1, 200)];

  it("trie sans tenir compte des majuscules ni des accents, sans toucher à la liste d'origine", () => {
    expect(sortTracks(list, "title", true).map((x) => x.title)).toEqual(["alpha", "Bravo", "Écho"]);
    expect(list[0].title).toBe("Bravo");
  });

  it("propose les plus récents, l'artiste et la durée", () => {
    expect(sortTracks(list, "added", false).map((x) => x.title)).toEqual(["alpha", "Bravo", "Écho"]);
    expect(sortTracks(list, "artist", true).map((x) => x.title)).toEqual(["alpha", "Écho", "Bravo"]);
    expect(sortTracks(list, "duration", true).map((x) => x.duration_secs)).toEqual([100, 200, 300]);
    expect(SORTS.map((s) => s.id)).toContain("recent");
  });
});
