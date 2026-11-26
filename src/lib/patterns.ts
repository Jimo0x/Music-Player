// Motifs posés sur le fond de l'application (lot 0.12.0) : des tuiles SVG répétées, utilisées
// comme masque d'un aplat de la couleur choisie (texte ou accent du thème). Rien d'animé, rien
// à télécharger : léger même sur un petit téléphone.

export type PatternId = "none" | "dots" | "grid" | "lines" | "waves" | "diamonds" | "hexagons" | "stars" | "grain";
export type PatternColor = "text" | "accent";

export interface PatternSettings {
  id: PatternId;
  /** Intensité (opacité de l'aplat), 0,02 à 0,4. */
  opacity: number;
  /** Taille des tuiles, 0,5 à 3. */
  scale: number;
  color: PatternColor;
}

interface PatternDef {
  id: Exclude<PatternId, "none">;
  label: string;
  /** Taille de la tuile à l'échelle 1, en pixels CSS. */
  w: number;
  h: number;
  /** Contenu SVG de la tuile, dessiné en noir opaque (le masque ne garde que l'alpha). */
  body: string;
}

const STROKE = 'fill="none" stroke="#000" stroke-width="1.2" stroke-linecap="round"';

export const PATTERNS: PatternDef[] = [
  { id: "dots", label: "Points", w: 18, h: 18, body: '<circle cx="9" cy="9" r="1.6" fill="#000"/>' },
  { id: "grid", label: "Grille", w: 24, h: 24, body: `<path d="M24 0V24M0 24H24" ${STROKE}/>` },
  { id: "lines", label: "Lignes", w: 14, h: 14, body: `<path d="M-2 16L16 -2M12 16L16 12M-2 2L2 -2" ${STROKE}/>` },
  { id: "waves", label: "Vagues", w: 32, h: 16, body: `<path d="M0 8 Q8 2 16 8 T32 8" ${STROKE}/>` },
  { id: "diamonds", label: "Losanges", w: 20, h: 20, body: `<path d="M10 2L18 10L10 18L2 10Z" ${STROKE}/>` },
  {
    id: "hexagons",
    label: "Hexagones",
    w: 28,
    h: 48,
    body: `<path d="M14 0L28 8V24L14 32L0 24V8Z M14 32V48 M0 48V40 M28 48V40" ${STROKE}/>`,
  },
  {
    id: "stars",
    label: "Étoiles",
    w: 96,
    h: 96,
    body:
      '<circle cx="12" cy="18" r="1.3" fill="#000"/><circle cx="58" cy="9" r="0.8" fill="#000"/><circle cx="83" cy="40" r="1.6" fill="#000"/>' +
      '<circle cx="34" cy="52" r="0.9" fill="#000"/><circle cx="70" cy="76" r="1.1" fill="#000"/><circle cx="18" cy="84" r="0.7" fill="#000"/>' +
      '<path d="M46 30l1.2 3.2 3.2 1.2-3.2 1.2-1.2 3.2-1.2-3.2-3.2-1.2 3.2-1.2z" fill="#000"/>',
  },
  {
    id: "grain",
    label: "Grain",
    w: 160,
    h: 160,
    // Bruit en niveaux d'alpha : la couleur reste unie, seule sa présence varie.
    body:
      '<filter id="g"><feTurbulence type="fractalNoise" baseFrequency="0.85" numOctaves="2" stitchTiles="stitch"/>' +
      '<feColorMatrix values="0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  0 0 0 1.6 -0.55"/></filter><rect width="100%" height="100%" filter="url(#g)"/>',
  },
];

export const PATTERN_IDS: readonly PatternId[] = ["none", ...PATTERNS.map((p) => p.id)];

export function defaultPattern(): PatternSettings {
  return { id: "none", opacity: 0.08, scale: 1, color: "text" };
}

const clamp = (v: unknown, lo: number, hi: number, d: number) => (typeof v === "number" && Number.isFinite(v) ? Math.max(lo, Math.min(hi, v)) : d);

export function normalizePattern(raw: unknown): PatternSettings {
  const base = defaultPattern();
  if (!raw || typeof raw !== "object") return base;
  const o = raw as Partial<PatternSettings>;
  return {
    id: PATTERN_IDS.includes(o.id as PatternId) ? (o.id as PatternId) : base.id,
    opacity: clamp(o.opacity, 0.02, 0.4, base.opacity),
    scale: clamp(o.scale, 0.5, 3, base.scale),
    color: o.color === "accent" ? "accent" : "text",
  };
}

/** Masque CSS d'un motif (image et taille de tuile), ou null pour « aucun ». */
export function patternMask(id: PatternId, scale = 1): { image: string; size: string } | null {
  const def = PATTERNS.find((p) => p.id === id);
  if (!def) return null;
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${def.w}" height="${def.h}" viewBox="0 0 ${def.w} ${def.h}">${def.body}</svg>`;
  return {
    image: `url("data:image/svg+xml,${encodeURIComponent(svg)}")`,
    size: `${Math.round(def.w * scale)}px ${Math.round(def.h * scale)}px`,
  };
}
