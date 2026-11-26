// Paquet de thème complet (.mptheme, lot 0.12.0) : tout ce qui fait l'apparence, dans un seul
// fichier à partager. Couleurs (avancées comprises), arrondi et halos ; polices par rôle, les
// polices importées étant jointes ; boutons, champs, style et jeu d'icônes ; motif ; fond, avec
// son image jointe. Un paquet reçu vient d'ailleurs : chaque champ est revérifié ici, les
// fichiers joints le sont encore côté Rust (format, taille) à leur enregistrement.
import { normalizeBackdrop, type BackdropSettings } from "./backdrop";
import { SYSTEM_FONT } from "./fonts";
import { normalizeIcons, type CustomIcon } from "./iconPack";
import { normalizePattern, type PatternSettings } from "./patterns";
import { parseThemeFile, type FontRole, type Theme } from "./themes";

export const THEME_PACK_FORMAT = "music-player-theme-pack";
const ROLES: FontRole[] = ["ui", "display", "lyrics"];
const FONT_EXTENSIONS = ["ttf", "otf", "woff", "woff2"];
const IMAGE_EXTENSIONS = ["png", "jpg", "jpeg", "webp", "gif"];
/** Taille d'un fichier joint, une fois décodé (même limite que l'import d'un fichier). */
const MAX_ASSET_BYTES = 25 * 1024 * 1024;

/** Police d'un rôle : celle du système, une du catalogue, ou une police jointe au paquet. */
export type PackFont = { system: true } | { catalog: string } | { embedded: number };

export interface EmbeddedFile {
  ext: string;
  /** Contenu en base64. */
  data: string;
  /** Nom affiché (polices). */
  label?: string;
}

export interface ThemePackLook {
  buttons: { shape: string; style: string };
  fields: { shape: string; style: string };
  icons: string;
  pattern: PatternSettings;
  backdrop: Omit<BackdropSettings, "image"> & { image: EmbeddedFile | null };
  customIcons: Record<string, CustomIcon>;
}

export interface ThemePack {
  format: typeof THEME_PACK_FORMAT;
  version: 1;
  name: string;
  theme: Theme;
  fonts: Record<FontRole, PackFont>;
  embeddedFonts: EmbeddedFile[];
  look: ThemePackLook;
}

/** Ce qu'il faut de l'apparence actuelle pour en faire un paquet. */
export interface PackSource {
  theme: Theme;
  fonts: Record<FontRole, string>;
  customFonts: { id: string; label: string; file: string }[];
  buttons: { shape: string; style: string };
  fields: { shape: string; style: string };
  icons: string;
  pattern: PatternSettings;
  backdrop: BackdropSettings;
  customIcons: Record<string, CustomIcon>;
}

export function toBase64(bytes: Uint8Array): string {
  let binary = "";
  for (let i = 0; i < bytes.length; i += 0x8000) binary += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  return btoa(binary);
}

function fromBase64(data: string): Uint8Array | null {
  try {
    const binary = atob(data);
    const out = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) out[i] = binary.charCodeAt(i);
    return out;
  } catch {
    return null;
  }
}

const extOf = (file: string) => (file.split(".").pop() ?? "").toLowerCase();

/** Paquet de l'apparence actuelle. `read` lit un fichier importé (police, image). */
export async function buildThemePack(src: PackSource, read: (file: string) => Promise<Uint8Array>): Promise<ThemePack> {
  const embeddedFonts: EmbeddedFile[] = [];
  const fonts = {} as Record<FontRole, PackFont>;
  for (const role of ROLES) {
    const id = src.fonts[role];
    const custom = src.customFonts.find((f) => f.id === id);
    if (custom) {
      let index = embeddedFonts.findIndex((f) => f.label === custom.label && f.ext === extOf(custom.file));
      if (index < 0) {
        embeddedFonts.push({ ext: extOf(custom.file), label: custom.label, data: toBase64(await read(custom.file)) });
        index = embeddedFonts.length - 1;
      }
      fonts[role] = { embedded: index };
    } else if (!id || id === SYSTEM_FONT || id.startsWith("custom:")) {
      fonts[role] = { system: true };
    } else {
      fonts[role] = { catalog: id };
    }
  }
  const { image, ...backdrop } = src.backdrop;
  const embeddedImage = image && backdrop.source === "image" ? { ext: extOf(image), data: toBase64(await read(image)) } : null;
  const theme: Theme = { ...src.theme, colors: { ...src.theme.colors }, fonts: undefined, builtin: undefined };
  return {
    format: THEME_PACK_FORMAT,
    version: 1,
    name: src.theme.name,
    theme,
    fonts,
    embeddedFonts,
    look: {
      buttons: { ...src.buttons },
      fields: { ...src.fields },
      icons: src.icons,
      pattern: { ...src.pattern },
      backdrop: { ...backdrop, source: embeddedImage || backdrop.source !== "image" ? backdrop.source : "none", image: embeddedImage },
      customIcons: src.customIcons,
    },
  };
}

function file(raw: unknown, allowed: string[]): (EmbeddedFile & { bytes: Uint8Array }) | null {
  if (!raw || typeof raw !== "object") return null;
  const o = raw as Partial<EmbeddedFile>;
  const ext = typeof o.ext === "string" ? o.ext.toLowerCase() : "";
  if (!allowed.includes(ext) || typeof o.data !== "string" || o.data.length > MAX_ASSET_BYTES * 1.4) return null;
  const bytes = fromBase64(o.data);
  if (!bytes || !bytes.length || bytes.length > MAX_ASSET_BYTES) return null;
  const label = typeof o.label === "string" ? o.label.replace(/[^\p{L}\p{N} '_-]/gu, "").trim().slice(0, 60) : undefined;
  return { ext, data: o.data, label, bytes };
}

const oneOf = (value: unknown, allowed: readonly string[], fallback: string) => (typeof value === "string" && allowed.includes(value) ? value : fallback);

/** Paquet lu d'un fichier reçu, vérifié champ par champ ; null s'il n'en est pas un. */
export function parseThemePack(text: string) {
  let data: Record<string, unknown>;
  try {
    data = JSON.parse(text);
  } catch {
    return null;
  }
  if (!data || data.format !== THEME_PACK_FORMAT || data.version !== 1) return null;
  const theme = parseThemeFile(JSON.stringify({ theme: data.theme }));
  if (!theme) return null;
  const embeddedFonts = (Array.isArray(data.embeddedFonts) ? data.embeddedFonts.slice(0, 3) : []).map((f) => file(f, FONT_EXTENSIONS));
  const rawFonts = (data.fonts ?? {}) as Record<string, unknown>;
  const fonts = {} as Record<FontRole, PackFont>;
  for (const role of ROLES) {
    const f = rawFonts[role] as Record<string, unknown> | undefined;
    if (f && typeof f.catalog === "string" && /^[a-z0-9-]{1,40}$/.test(f.catalog)) fonts[role] = { catalog: f.catalog };
    else if (f && Number.isInteger(f.embedded) && embeddedFonts[f.embedded as number]) fonts[role] = { embedded: f.embedded as number };
    else fonts[role] = { system: true };
  }
  const look = (data.look ?? {}) as Record<string, Record<string, unknown> | unknown>;
  const b = (look.buttons ?? {}) as Record<string, unknown>;
  const fl = (look.fields ?? {}) as Record<string, unknown>;
  const rawBackdrop = (look.backdrop ?? {}) as Record<string, unknown>;
  const image = file(rawBackdrop.image, IMAGE_EXTENSIONS);
  const backdrop = normalizeBackdrop({ ...rawBackdrop, image: image ? "image.jpg" : null });
  return {
    name: theme.name,
    theme,
    fonts,
    embeddedFonts,
    look: {
      buttons: { shape: oneOf(b.shape, ["pill", "round", "square"], "round"), style: oneOf(b.style, ["gradient", "flat", "outline", "glass"], "gradient") },
      fields: { shape: oneOf(fl.shape, ["round", "pill", "square"], "round"), style: oneOf(fl.style, ["filled", "outline", "underline"], "filled") },
      icons: oneOf(look.icons, ["classic", "thin", "bold", "square", "duo"], "classic"),
      pattern: normalizePattern(look.pattern),
      backdrop,
      backdropImage: image,
      customIcons: normalizeIcons(look.customIcons),
    },
  };
}

export type ParsedThemePack = NonNullable<ReturnType<typeof parseThemePack>>;
