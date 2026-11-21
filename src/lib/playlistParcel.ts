// Playlist jointe à un message, au-delà de 300 morceaux (#10). Le message garde les 300 premiers
// décrits (ce que lisent les versions déjà installées et ce que le relais accepte en clair) ; la
// liste complète part à côté dans un colis chiffré, comme le fichier d'un son, et n'est reprise
// que par les versions qui savent la lire.
import type { Piece } from "./friendsApi";

/** Morceaux décrits dans le message lui-même (limite du relais pour un message en clair). */
export const PIECE_INLINE_TRACKS = 300;
/** Morceaux d'une playlist jointe au plus (liste complète du colis). */
export const PIECE_MAX_TRACKS = 10_000;

export type PieceTrack = { titre: string; artiste: string; album: string };

const text = (value: unknown, max: number) => (typeof value === "string" ? value.slice(0, max) : "");

/** Liste complète, en octets prêts pour `sendParcel`. */
export function encodePlaylist(morceaux: PieceTrack[]): Uint8Array {
  const list = morceaux.slice(0, PIECE_MAX_TRACKS).map((m) => ({ titre: m.titre, artiste: m.artiste, album: m.album }));
  return new TextEncoder().encode(JSON.stringify({ morceaux: list }));
}

/** Liste reçue d'un colis : venue d'ailleurs, chaque champ est vérifié (texte, longueur, nombre). */
export function decodePlaylist(base64Chunks: string[]): PieceTrack[] {
  const bytes = base64Chunks.flatMap((chunk) => [...atob(chunk)].map((c) => c.charCodeAt(0)));
  let value: unknown;
  try {
    value = JSON.parse(new TextDecoder().decode(new Uint8Array(bytes)));
  } catch {
    throw new Error("Liste de la playlist illisible.");
  }
  const morceaux = (value as { morceaux?: unknown })?.morceaux;
  if (!Array.isArray(morceaux)) throw new Error("Liste de la playlist illisible.");
  return morceaux
    .slice(0, PIECE_MAX_TRACKS)
    .map((m: unknown) => {
      const o = (m && typeof m === "object" ? m : {}) as Record<string, unknown>;
      return { titre: text(o.titre, 200), artiste: text(o.artiste, 200), album: text(o.album, 200) };
    })
    .filter((m) => m.titre);
}

/** Nombre de morceaux annoncé (liste complète), même tant qu'elle n'est pas reprise. */
export function pieceTrackCount(piece: Piece): number {
  if (piece.type !== "playlist") return 0;
  return Math.max(piece.morceaux.length, piece.colis && Number.isFinite(piece.total) ? Math.min(piece.total ?? 0, PIECE_MAX_TRACKS) : 0);
}

/** La liste complète reste à reprendre (colis pas encore lu). */
export function playlistIncomplete(piece: Piece): boolean {
  return piece.type === "playlist" && !!piece.colis && pieceTrackCount(piece) > piece.morceaux.length;
}
