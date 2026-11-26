// Styles de la barre de progression de « À l'écoute » (lot 0.13.0). Hors « classique », la
// barre est un fond discret et une copie colorée découpée jusqu'à la position (clip-path),
// toutes deux masquées par le même dessin : vague, vraie forme d'onde du morceau, perles,
// segments. Les masques sont des SVG en ligne : rien à charger, autorisés par la CSP (data:).

export type SeekStyle = "classic" | "wave" | "waveform" | "dots" | "neon" | "segments";

export const SEEK_STYLES: { id: SeekStyle; label: string }[] = [
  { id: "classic", label: "Classique" },
  { id: "wave", label: "Vague" },
  { id: "waveform", label: "Forme d'onde" },
  { id: "dots", label: "Perles" },
  { id: "neon", label: "Néon" },
  { id: "segments", label: "Segments" },
];

export const SEEK_STYLE_IDS: readonly SeekStyle[] = SEEK_STYLES.map((s) => s.id);

export function normalizeSeekStyle(value: unknown): SeekStyle {
  return SEEK_STYLE_IDS.includes(value as SeekStyle) ? (value as SeekStyle) : "classic";
}

const svg = (w: number, h: number, body: string, ratio = "xMidYMid meet") =>
  `url("data:image/svg+xml,${encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" preserveAspectRatio="${ratio}">${body}</svg>`)}")`;

/** Masque répété d'un style (taille de la tuile comprise), ou null pour la forme d'onde et les
 *  styles pleins. */
export function seekMask(style: SeekStyle): { image: string; size: string } | null {
  switch (style) {
    case "wave":
      // Une période de sinus par tuile de 24 px, trait de 3 px, sur 16 px de haut.
      return { image: svg(24, 16, '<path d="M0 8 Q6 2 12 8 T24 8" fill="none" stroke="#000" stroke-width="3" stroke-linecap="round"/>'), size: "24px 100%" };
    case "dots":
      return { image: svg(10, 10, '<circle cx="5" cy="5" r="3" fill="#000"/>'), size: "10px 100%" };
    case "segments":
      return { image: svg(7, 12, '<rect x="0" y="0" width="5" height="12" rx="1" fill="#000"/>'), size: "7px 100%" };
    default:
      return null;
  }
}

/** Masque de la vraie forme d'onde : une barre arrondie par valeur (0 à 255), centrée en
 *  hauteur, étirée sur toute la largeur. Sans données : des barres basses, régulières. */
export function waveformMask(bars: number[] | null): string {
  const values = bars?.length ? bars : Array.from({ length: 160 }, () => 30);
  const n = values.length;
  const body = values
    .map((v, i) => {
      const h = Math.max(6, Math.min(100, (v / 255) * 100));
      return `<rect x="${i * 2 + 0.25}" y="${((100 - h) / 2).toFixed(1)}" width="1.5" height="${h.toFixed(1)}" rx="0.75" fill="#000"/>`;
    })
    .join("");
  return svg(n * 2, 100, body, "none");
}
