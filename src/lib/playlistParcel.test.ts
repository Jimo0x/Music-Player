import { describe, expect, it } from "vitest";
import { toBase64 } from "./sync";
import { PIECE_MAX_TRACKS, decodePlaylist, encodePlaylist, pieceTrackCount, playlistIncomplete } from "./playlistParcel";

const tracks = (n: number) => Array.from({ length: n }, (_, i) => ({ titre: `Titre ${i} é`, artiste: "Damso", album: "QALF" }));

describe("liste complète d'une playlist jointe (#10)", () => {
  it("fait l'aller-retour, découpée en morceaux comme sur le relais", () => {
    const raw = encodePlaylist(tracks(1200));
    const chunks = [toBase64(raw.subarray(0, 1000)), toBase64(raw.subarray(1000))];
    expect(decodePlaylist(chunks)).toEqual(tracks(1200));
  });

  it("vérifie une liste venue d'ailleurs", () => {
    const hostile = new TextEncoder().encode(JSON.stringify({ morceaux: [{ titre: "x".repeat(500), artiste: 3 }, null, { titre: "" }, "texte"] }));
    expect(decodePlaylist([toBase64(hostile)])).toEqual([{ titre: "x".repeat(200), artiste: "", album: "" }]);
    expect(() => decodePlaylist([toBase64(new TextEncoder().encode("pas du json"))])).toThrow(/illisible/);
    expect(() => decodePlaylist([toBase64(new TextEncoder().encode("{}"))])).toThrow(/illisible/);
    expect(decodePlaylist([toBase64(encodePlaylist(tracks(PIECE_MAX_TRACKS + 5)))])).toHaveLength(PIECE_MAX_TRACKS);
  });

  it("annonce le total tant que la liste complète n'est pas reprise", () => {
    const colis = { id: "0".repeat(32), cle: "k", taille: 10 };
    const piece = { type: "playlist" as const, nom: "Longue", morceaux: tracks(300), total: 450, colis };
    expect(pieceTrackCount(piece)).toBe(450);
    expect(playlistIncomplete(piece)).toBe(true);
    // Sans colis (ancienne version, ou liste déjà reprise) : ce qui est là.
    expect(pieceTrackCount({ ...piece, colis: null })).toBe(300);
    expect(playlistIncomplete({ ...piece, morceaux: tracks(450), colis: null })).toBe(false);
    expect(pieceTrackCount({ ...piece, total: 1e9 })).toBe(PIECE_MAX_TRACKS);
  });
});
