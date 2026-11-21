import { describe, expect, it } from "vitest";
import { shareSummary } from "./shareSummary";

describe("résumé d'un partage reçu (#11)", () => {
  it("dit ce qui est arrivé, ce qui était déjà là et les doublons", () => {
    expect(shareSummary({ name: "Soirée", kind: "playlist", added_tracks: 103, skipped_tracks: 0, merged_tracks: 5, duplicate_tracks: 2 })).toBe(
      "Playlist « Soirée » : 103 morceaux reçus, 5 déjà présents (fusionnés), 2 doublons ignorés.",
    );
  });

  it("reste court quand tout est nouveau (et lit les anciennes versions sans ces chiffres)", () => {
    expect(shareSummary({ name: "Lemon", kind: "track", added_tracks: 1, skipped_tracks: 0 })).toBe("« Lemon » : 1 morceau reçu.");
    expect(shareSummary({ name: "X", kind: "playlist", added_tracks: 3, skipped_tracks: 1, merged_tracks: 1, duplicate_tracks: 0 })).toBe(
      "Playlist « X » : 3 morceaux reçus, 1 déjà présent (fusionné), 1 piste illisible ignorée.",
    );
  });
});
