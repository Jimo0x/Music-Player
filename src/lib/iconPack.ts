// Jeu d'icônes importé (lot 0.12.0) : des fichiers SVG nommés comme les icônes de l'appli
// (« play.svg », « playlist.svg »…) remplacent les icônes d'origine ; les autres restent.
// Un SVG venu d'ailleurs peut contenir du script, des styles ou des liens : il est donc
// reconstruit à partir d'une liste fermée d'éléments et d'attributs, couleurs ramenées à
// celle du texte (le jeu suit le thème), avant d'être enregistré et affiché.
import { shallowRef } from "vue";
import { PATHS } from "./iconPaths";

export interface CustomIcon {
  viewBox: string;
  /** Contenu de l'icône (éléments déjà nettoyés), sans la balise <svg>. */
  body: string;
  fill: "currentColor" | "none";
  stroke: "currentColor" | "none";
  strokeWidth: string | undefined;
}

/** Noms des icônes de l'appli (ceux qu'un fichier importé peut remplacer). */
export const ICON_NAMES: readonly string[] = Object.keys(PATHS).sort();

const ELEMENTS = new Set(["g", "path", "circle", "ellipse", "rect", "line", "polyline", "polygon"]);
const GEOMETRY = new Set(["d", "cx", "cy", "r", "rx", "ry", "x", "y", "width", "height", "x1", "y1", "x2", "y2", "points"]);
const PAINT = new Set(["fill", "stroke"]);
const STYLE_ATTRS = new Set(["stroke-width", "stroke-linecap", "stroke-linejoin", "fill-rule", "clip-rule", "opacity", "fill-opacity", "stroke-opacity", "transform"]);
const MAX_FILE_CHARS = 64 * 1024;
const MAX_BODY_CHARS = 32 * 1024;
const MAX_ELEMENTS = 400;

/** Valeur sans rien qui puisse charger, exécuter ou sortir de l'attribut. */
function safeValue(name: string, value: string): string | null {
  const v = value.trim();
  if (!v || v.length > 8000) return null;
  if (/url\s*\(|javascript:|expression|[<>"'`\\]|&#/i.test(v)) return null;
  if (name === "d") return /^[MmLlHhVvCcSsQqTtAaZz0-9eE.,\s+-]+$/.test(v) ? v : null;
  if (name === "points") return /^[0-9eE.,\s+-]+$/.test(v) ? v : null;
  if (name === "transform") return /^((matrix|translate|scale|rotate|skewX|skewY)\([0-9eE.,\s+-]*\)\s*)+$/.test(v) ? v : null;
  if (name === "stroke-linecap") return ["butt", "round", "square"].includes(v) ? v : null;
  if (name === "stroke-linejoin") return ["miter", "round", "bevel"].includes(v) ? v : null;
  if (name === "fill-rule" || name === "clip-rule") return ["nonzero", "evenodd"].includes(v) ? v : null;
  return /^-?[0-9.]+(e-?[0-9]+)?(%|px)?$/i.test(v) ? v : null;
}

/** fill / stroke : « none » reste, toute autre couleur devient celle du texte (thème). */
function paint(value: string | null): "currentColor" | "none" | null {
  if (value === null) return null;
  return value.trim().toLowerCase() === "none" ? "none" : "currentColor";
}

function cleanElement(el: Element, budget: { left: number }): string {
  const tag = el.localName;
  if (!ELEMENTS.has(tag) || budget.left-- <= 0) return "";
  const attrs: string[] = [];
  for (const attr of Array.from(el.attributes)) {
    const name = attr.localName;
    if (PAINT.has(name)) {
      const p = paint(attr.value);
      if (p) attrs.push(`${name}="${p}"`);
    } else if (GEOMETRY.has(name) || STYLE_ATTRS.has(name)) {
      const v = safeValue(name, attr.value);
      if (v !== null) attrs.push(`${name}="${v}"`);
    }
  }
  const children = tag === "g" ? Array.from(el.children).map((c) => cleanElement(c, budget)).join("") : "";
  return tag === "g" ? `<g ${attrs.join(" ")}>${children}</g>` : `<${tag} ${attrs.join(" ")}/>`;
}

/** viewBox de l'icône : celui du fichier, sinon sa largeur et sa hauteur, sinon 24×24. */
function viewBoxOf(svg: Element): string {
  const vb = (svg.getAttribute("viewBox") ?? "").trim().split(/[\s,]+/).map(Number);
  if (vb.length === 4 && vb.every(Number.isFinite) && vb[2] > 0 && vb[3] > 0) return vb.join(" ");
  const w = parseFloat(svg.getAttribute("width") ?? "");
  const h = parseFloat(svg.getAttribute("height") ?? "");
  return w > 0 && h > 0 ? `0 0 ${w} ${h}` : "0 0 24 24";
}

/** Icône nettoyée à partir du texte d'un fichier SVG, ou null s'il n'en contient pas. */
export function sanitizeSvg(source: string): CustomIcon | null {
  if (!source || source.length > MAX_FILE_CHARS) return null;
  let doc: Document;
  try {
    doc = new DOMParser().parseFromString(source, "image/svg+xml");
  } catch {
    return null;
  }
  const svg = doc.documentElement;
  if (!svg || svg.localName !== "svg" || doc.getElementsByTagName("parsererror").length) return null;
  const budget = { left: MAX_ELEMENTS };
  const body = Array.from(svg.children).map((c) => cleanElement(c, budget)).join("");
  if (!body || body.length > MAX_BODY_CHARS) return null;
  const strokeWidth = svg.hasAttribute("stroke-width") ? safeValue("stroke-width", svg.getAttribute("stroke-width")!) ?? undefined : undefined;
  return {
    viewBox: viewBoxOf(svg),
    body,
    // Sans indication, un SVG est rempli (comme les jeux d'icônes « pleins » courants).
    fill: paint(svg.getAttribute("fill")) ?? "currentColor",
    stroke: paint(svg.getAttribute("stroke")) ?? "none",
    strokeWidth,
  };
}

/** Nom d'icône d'après un nom de fichier (« Play.svg » → « play »), si l'appli le connaît. */
export function iconNameOf(fileName: string): string | null {
  const base = fileName.replace(/^.*[\\/]/, "").replace(/\.svg$/i, "").trim().toLowerCase();
  return ICON_NAMES.includes(base) ? base : null;
}

/** Jeu enregistré dans les réglages : chaque icône est de nouveau nettoyée (fichier de
 *  réglages ou thème partagé : rien n'est cru sur parole). */
export function normalizeIcons(raw: unknown): Record<string, CustomIcon> {
  const out: Record<string, CustomIcon> = {};
  if (!raw || typeof raw !== "object") return out;
  for (const [name, value] of Object.entries(raw as Record<string, unknown>)) {
    if (!ICON_NAMES.includes(name) || !value || typeof value !== "object") continue;
    const v = value as Partial<CustomIcon>;
    if (typeof v.body !== "string" || typeof v.viewBox !== "string") continue;
    const root = [
      `viewBox="${v.viewBox.replace(/[^0-9.\s-]/g, "")}"`,
      v.fill === "none" ? 'fill="none"' : "",
      v.stroke === "currentColor" ? 'stroke="currentColor"' : "",
      typeof v.strokeWidth === "string" ? `stroke-width="${v.strokeWidth.replace(/[^0-9.]/g, "")}"` : "",
    ].join(" ");
    const icon = sanitizeSvg(`<svg xmlns="http://www.w3.org/2000/svg" ${root}>${v.body}</svg>`);
    if (icon) out[name] = icon;
  }
  return out;
}

// Jeu actif, lu par chaque <Icon> (posé par la personnalisation).
const active = shallowRef<Record<string, CustomIcon>>({});

export function setCustomIcons(icons: Record<string, CustomIcon>) {
  if (active.value !== icons) active.value = icons;
}

export function customIcon(name: string): CustomIcon | null {
  return active.value[name] ?? null;
}
