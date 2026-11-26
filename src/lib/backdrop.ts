// Arrière-plan de l'application (lot 0.12.0) : une image choisie par la personne ou la
// pochette du morceau en cours, posée sous toute l'interface, avec flou, assombrissement et
// cadrage. Le mode léger retire le flou (coûteux sur les téléphones modestes).

export type BackdropSource = "none" | "image" | "artwork";

export interface BackdropSettings {
  source: BackdropSource;
  /** Image importée (nom dans le dossier des fichiers personnalisés). */
  image: string | null;
  /** Flou, en pixels (0 à 30). */
  blur: number;
  /** Assombrissement par la couleur de fond du thème (0 à 0,9) : garde le texte lisible. */
  dim: number;
  /** Point de l'image gardé au centre du cadre (pourcentages, 0 à 100). */
  x: number;
  y: number;
}

export const BACKDROP_SOURCES: readonly BackdropSource[] = ["none", "image", "artwork"];

export function defaultBackdrop(): BackdropSettings {
  return { source: "none", image: null, blur: 8, dim: 0.65, x: 50, y: 50 };
}

const clamp = (v: unknown, lo: number, hi: number, d: number) => (typeof v === "number" && Number.isFinite(v) ? Math.max(lo, Math.min(hi, v)) : d);

/** Réglages lus d'un fichier (éventuellement ancien ou modifié à la main), complétés et bornés. */
export function normalizeBackdrop(raw: unknown): BackdropSettings {
  const base = defaultBackdrop();
  if (!raw || typeof raw !== "object") return base;
  const o = raw as Partial<BackdropSettings>;
  const image = typeof o.image === "string" && /^[A-Za-z0-9.-]{1,120}$/.test(o.image) && !o.image.startsWith(".") ? o.image : null;
  let source = BACKDROP_SOURCES.includes(o.source as BackdropSource) ? (o.source as BackdropSource) : base.source;
  // Une image demandée mais absente : pas de fond plutôt qu'un fond vide.
  if (source === "image" && !image) source = "none";
  return {
    source,
    image,
    blur: clamp(o.blur, 0, 30, base.blur),
    dim: clamp(o.dim, 0, 0.9, base.dim),
    x: clamp(o.x, 0, 100, base.x),
    y: clamp(o.y, 0, 100, base.y),
  };
}

/** Taille d'une image ramenée à `max` pixels sur son plus grand côté (jamais agrandie). */
export function fitWithin(width: number, height: number, max: number): { width: number; height: number } {
  if (width <= 0 || height <= 0) return { width: 0, height: 0 };
  const scale = Math.min(1, max / Math.max(width, height));
  return { width: Math.round(width * scale), height: Math.round(height * scale) };
}

/** Plus grand côté gardé pour une image de fond : net sur un écran 1440p, léger à afficher. */
export const BACKDROP_MAX_SIDE = 2560;

/** Image choisie, réduite au besoin et réencodée en JPEG (une photo de 25 Mo ne doit pas
 *  ralentir chaque affichage). */
export async function prepareBackdropImage(file: Blob): Promise<Blob> {
  const url = URL.createObjectURL(file);
  try {
    const img = await new Promise<HTMLImageElement>((resolve, reject) => {
      const el = new Image();
      el.onload = () => resolve(el);
      el.onerror = () => reject(new Error("Image illisible"));
      el.src = url;
    });
    const size = fitWithin(img.naturalWidth, img.naturalHeight, BACKDROP_MAX_SIDE);
    if (!size.width) throw new Error("Image illisible");
    const canvas = document.createElement("canvas");
    canvas.width = size.width;
    canvas.height = size.height;
    const ctx = canvas.getContext("2d");
    if (!ctx) throw new Error("Dessin impossible");
    ctx.drawImage(img, 0, 0, size.width, size.height);
    return await new Promise<Blob>((resolve, reject) => canvas.toBlob((b) => (b ? resolve(b) : reject(new Error("Encodage impossible"))), "image/jpeg", 0.88));
  } finally {
    URL.revokeObjectURL(url);
  }
}
