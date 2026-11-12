import type { Palette, Rgb } from "./palette";
import { FONT, fit, loadImage, rgb, roundRect } from "./recapCard";

/** Carte d'un extrait de paroles à partager : plusieurs styles et formats (réseaux, stories). */
export const LYRICS_W = 1080;
/** Lignes qu'une carte peut montrer sans devenir illisible. */
export const MAX_LINES = 6;

export type CardStyle = "halo" | "bulles" | "pochette" | "papier" | "arcane";
export type CardFormat = "carre" | "portrait" | "story";

export const CARD_STYLES: { id: CardStyle; label: string }[] = [
  { id: "halo", label: "Halo" },
  { id: "bulles", label: "Bulles" },
  { id: "pochette", label: "Pochette" },
  { id: "papier", label: "Papier" },
  { id: "arcane", label: "Arcane" },
];
export const CARD_FORMATS: { id: CardFormat; label: string; height: number }[] = [
  { id: "carre", label: "Carré", height: 1080 },
  { id: "portrait", label: "Portrait", height: 1350 },
  { id: "story", label: "Story", height: 1920 },
];

/** Découpe un texte en lignes qui tiennent dans `max` pixels. */
export function wrap(measure: (text: string) => number, text: string, max: number): string[] {
  const out: string[] = [];
  let line = "";
  for (const word of text.split(/\s+/).filter(Boolean)) {
    const next = line ? `${line} ${word}` : word;
    if (line && measure(next) > max) {
      out.push(line);
      line = word;
    } else {
      line = next;
    }
  }
  if (line) out.push(line);
  return out;
}

/** Lumière perçue (0 à 1) : pour choisir un texte sombre ou clair sur une couleur. */
export function luminance([r, g, b]: Rgb): number {
  return (0.2126 * r + 0.7152 * g + 0.0722 * b) / 255;
}

/** Couleur vive tirée de la pochette, assez claire pour servir de fond aux bulles. */
function bright(c: Rgb): Rgb {
  const lift = luminance(c) < 0.45 ? 0.45 - luminance(c) : 0;
  return c.map((v) => Math.round(v + (255 - v) * lift * 1.6)) as Rgb;
}

interface Input {
  lines: string[];
  title: string;
  artist: string;
  coverUrl: string | null;
  palette: Palette | null;
  style?: CardStyle;
  format?: CardFormat;
}

export async function drawLyricsCard(input: Input): Promise<HTMLCanvasElement> {
  const style = input.style ?? "halo";
  const height = CARD_FORMATS.find((f) => f.id === (input.format ?? "portrait"))!.height;
  const canvas = document.createElement("canvas");
  canvas.width = LYRICS_W;
  canvas.height = height;
  const ctx = canvas.getContext("2d")!;
  const c1: Rgb = input.palette?.glow ?? [120, 90, 255];
  const c2: Rgb = input.palette?.secondary ?? [255, 80, 160];
  const base: Rgb = input.palette?.base ?? [16, 12, 30];
  const cover = await loadImage(input.coverUrl);
  if (style === "arcane") {
    drawArcane(ctx, input, cover, height, c1, c2, base);
    return canvas;
  }

  // ---------------------------------------------------------------- fond
  let ink = "#fff";
  let sub = "rgba(255,255,255,0.78)";
  if (style === "halo") {
    ctx.fillStyle = rgb(base);
    ctx.fillRect(0, 0, LYRICS_W, height);
    for (const [x, y, r, c, a] of [
      [120, height * 0.12, 820, c1, 0.75],
      [980, height * 0.88, 860, c2, 0.6],
    ] as Array<[number, number, number, Rgb, number]>) {
      const g = ctx.createRadialGradient(x, y, 0, x, y, r);
      g.addColorStop(0, rgb(c, a));
      g.addColorStop(1, rgb(c, 0));
      ctx.fillStyle = g;
      ctx.fillRect(0, 0, LYRICS_W, height);
    }
  } else if (style === "bulles") {
    const bg = bright(c1);
    ctx.fillStyle = rgb(bg);
    ctx.fillRect(0, 0, LYRICS_W, height);
    ink = luminance(bg) > 0.6 ? "#0b0b10" : "#fff";
    sub = luminance(bg) > 0.6 ? "rgba(10,10,16,0.7)" : "rgba(255,255,255,0.8)";
  } else if (style === "pochette") {
    ctx.fillStyle = rgb(base);
    ctx.fillRect(0, 0, LYRICS_W, height);
    if (cover) {
      // Pochette en plein cadre, floutée et assombrie : les paroles restent lisibles.
      const side = Math.max(LYRICS_W, height) * 1.15;
      ctx.save();
      ctx.filter = "blur(36px) saturate(1.25)";
      ctx.drawImage(cover, (LYRICS_W - side) / 2, (height - side) / 2, side, side);
      ctx.restore();
    }
    const shade = ctx.createLinearGradient(0, 0, 0, height);
    shade.addColorStop(0, "rgba(0,0,0,0.35)");
    shade.addColorStop(1, "rgba(0,0,0,0.6)");
    ctx.fillStyle = shade;
    ctx.fillRect(0, 0, LYRICS_W, height);
  } else {
    ctx.fillStyle = "#f4efe6";
    ctx.fillRect(0, 0, LYRICS_W, height);
    ink = "#1b1712";
    sub = "rgba(27,23,18,0.66)";
  }

  // ---------------------------------------------------------------- en-tête
  const left = 90;
  const coverSize = height >= 1900 ? 170 : 150;
  const headTop = height >= 1900 ? 170 : 90;
  if (cover) {
    ctx.save();
    roundRect(ctx, left, headTop, coverSize, coverSize, 18);
    ctx.clip();
    ctx.drawImage(cover, left, headTop, coverSize, coverSize);
    ctx.restore();
  }
  const textLeft = cover ? left + coverSize + 32 : left;
  const textWidth = LYRICS_W - textLeft - left;
  ctx.textBaseline = "alphabetic";
  ctx.fillStyle = ink;
  ctx.font = `800 46px ${FONT}`;
  ctx.fillText(fit(ctx, input.title, textWidth), textLeft, headTop + 60);
  ctx.fillStyle = sub;
  ctx.font = `600 36px ${FONT}`;
  ctx.fillText(fit(ctx, input.artist, textWidth), textLeft, headTop + 114);

  // ---------------------------------------------------------------- paroles
  const pad = style === "bulles" ? 34 : 0;
  const width = LYRICS_W - left * 2 - pad * 2;
  const top = headTop + coverSize + 70;
  const bottom = height - (height >= 1900 ? 240 : 150);
  const gapRatio = 1.28;
  /** Bulles : interligne serré dans la bulle, marge haute et basse autour du texte. */
  const bubbleLine = 1.18;
  const bubblePad = 0.34;
  let size = style === "papier" ? 76 : 84;
  let rows: string[][] = [];
  const weight = style === "papier" ? "700" : "800";
  const family = style === "papier" ? `Georgia, "Times New Roman", serif` : FONT;
  for (; size >= 34; size -= 4) {
    ctx.font = `${weight} ${size}px ${family}`;
    rows = input.lines.map((l) => wrap((t) => ctx.measureText(t).width, l, width));
    const count = rows.reduce((s, r) => s + r.length, 0);
    const needed =
      style === "bulles"
        ? count * size * bubbleLine + rows.length * size * bubblePad * 2 + (rows.length - 1) * size * 0.3
        : count * size * gapRatio;
    if (needed <= bottom - top) break;
  }
  ctx.font = `${weight} ${size}px ${family}`;
  const count = rows.reduce((s, r) => s + r.length, 0);
  const blockGap = style === "bulles" ? size * 0.3 : 0;
  const total =
    style === "bulles"
      ? count * size * bubbleLine + rows.length * size * bubblePad * 2 + (rows.length - 1) * blockGap
      : count * size * gapRatio;
  let y = top + Math.max(0, (bottom - top - total) / 2);

  if (style === "papier") {
    // Guillemet d'ouverture, en couleur de la pochette.
    ctx.fillStyle = rgb(c1);
    ctx.font = `700 ${size * 2.2}px ${family}`;
    ctx.fillText("«", left - 10, y + size * 0.9);
    ctx.font = `${weight} ${size}px ${family}`;
    y += size * 0.6;
  }

  for (const block of rows) {
    if (style === "bulles") {
      // Chaque ligne choisie dans sa bulle sombre, le texte de la couleur du fond.
      const lineH = size * bubbleLine;
      const padY = size * bubblePad;
      const h = block.length * lineH + padY * 2;
      const w = Math.min(LYRICS_W - left * 2, Math.max(...block.map((r) => ctx.measureText(r).width)) + pad * 2);
      if (y + h > bottom + size) break;
      ctx.fillStyle = ink === "#fff" ? "rgba(0,0,0,0.55)" : "#0b0b10";
      roundRect(ctx, left, y, w, h, Math.min(44, size * 0.6));
      ctx.fill();
      ctx.fillStyle = ink === "#fff" ? "#fff" : rgb(bright(c1));
      // Ligne de base : la hauteur visible des lettres (≈ 0,72 × taille) centrée dans sa ligne.
      let ry = y + padY + lineH / 2 + size * 0.36;
      for (const row of block) {
        ctx.fillText(fit(ctx, row, width), left + pad, ry);
        ry += lineH;
      }
      y += h + blockGap;
      continue;
    }
    ctx.fillStyle = ink;
    for (const row of block) {
      y += size * gapRatio;
      if (y > bottom + size) break;
      ctx.fillText(fit(ctx, row, width), left, y - size * (gapRatio - 1));
    }
  }

  // ---------------------------------------------------------------- signature
  ctx.fillStyle = sub;
  ctx.font = `800 34px ${FONT}`;
  ctx.fillText("Music Player", left, height - (height >= 1900 ? 150 : 70));
  return canvas;
}

// ---------------------------------------------------------------- style « Arcane »
// Carte inspirée des tarots, dessinée pour l'appli : double cadre doré, numéro d'arcane en
// chiffres romains, pochette dans une arche rayonnante, paroles en italique, ruban du titre.

const ROMAN = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII", "XIII", "XIV", "XV", "XVI", "XVII", "XVIII", "XIX", "XX", "XXI"];
const SERIF = `"Cormorant Garamond", Georgia, "Times New Roman", serif`;

/** Empreinte stable d'un texte : même morceau, même arcane et mêmes étoiles. */
export function textSeed(text: string): number {
  let h = 2166136261;
  for (let i = 0; i < text.length; i++) h = Math.imul(h ^ text.charCodeAt(i), 16777619);
  return h >>> 0;
}

/** Numéro d'arcane (I à XXI) d'un morceau. */
export function arcaneNumber(title: string, artist: string): string {
  return ROMAN[textSeed(`${artist}|${title}`) % ROMAN.length];
}

function seeded(seed: number): () => number {
  let x = seed || 1;
  return () => {
    x ^= x << 13;
    x ^= x >>> 17;
    x ^= x << 5;
    return ((x >>> 0) % 10000) / 10000;
  };
}

function mix(a: Rgb, b: Rgb, t: number): Rgb {
  return a.map((v, i) => Math.round(v + (b[i] - v) * t)) as Rgb;
}

type Spaced = CanvasRenderingContext2D & { letterSpacing?: string };

function drawArcane(ctx: CanvasRenderingContext2D, input: Input, cover: HTMLImageElement | null, height: number, c1: Rgb, c2: Rgb, base: Rgb) {
  const W = LYRICS_W;
  const random = seeded(textSeed(`${input.artist}|${input.title}`));
  const night = mix(base, [6, 5, 14], 0.55);
  const gold = ctx.createLinearGradient(0, 0, W, height);
  gold.addColorStop(0, "#f6e3a8");
  gold.addColorStop(0.45, "#c9a05a");
  gold.addColorStop(0.7, "#f1d690");
  gold.addColorStop(1, "#a97b3a");
  const cream = "#f5ead2";

  // Fond : nuit teintée par la pochette, lueurs et poussière d'étoiles.
  ctx.fillStyle = rgb(night);
  ctx.fillRect(0, 0, W, height);
  for (const [x, y, r, c, a] of [
    [W * 0.5, height * 0.34, W * 0.75, c1, 0.42],
    [W * 0.15, height * 0.95, W * 0.7, c2, 0.3],
  ] as Array<[number, number, number, Rgb, number]>) {
    const g = ctx.createRadialGradient(x, y, 0, x, y, r);
    g.addColorStop(0, rgb(c, a));
    g.addColorStop(1, rgb(c, 0));
    ctx.fillStyle = g;
    ctx.fillRect(0, 0, W, height);
  }
  for (let i = 0; i < 140; i++) {
    const x = random() * W;
    const y = random() * height;
    const r = random() < 0.9 ? 1 + random() * 1.6 : 2.6 + random() * 1.8;
    ctx.fillStyle = `rgba(255, 244, 214, ${0.25 + random() * 0.55})`;
    ctx.beginPath();
    ctx.arc(x, y, r, 0, Math.PI * 2);
    ctx.fill();
  }

  // Double cadre doré et ornements d'angle.
  const m = 38;
  ctx.strokeStyle = gold;
  ctx.lineWidth = 6;
  roundRect(ctx, m, m, W - m * 2, height - m * 2, 26);
  ctx.stroke();
  ctx.lineWidth = 2;
  roundRect(ctx, m + 16, m + 16, W - (m + 16) * 2, height - (m + 16) * 2, 16);
  ctx.stroke();
  ctx.fillStyle = gold;
  for (const [x, y] of [
    [m + 16, m + 16],
    [W - m - 16, m + 16],
    [m + 16, height - m - 16],
    [W - m - 16, height - m - 16],
  ]) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(Math.PI / 4);
    ctx.fillRect(-11, -11, 22, 22);
    ctx.restore();
  }

  const compact = height <= 1100;
  const scale = compact ? 0.82 : 1;

  // Numéro d'arcane dans son cartouche, lune et étoile de part et d'autre.
  const numeral = arcaneNumber(input.title, input.artist);
  const topY = m + (compact ? 70 : 96);
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  ctx.font = `600 ${Math.round(52 * scale)}px ${SERIF}`;
  const nw = Math.max(110, ctx.measureText(numeral).width + 70);
  ctx.strokeStyle = gold;
  ctx.lineWidth = 2;
  roundRect(ctx, W / 2 - nw / 2, topY - 38 * scale, nw, 76 * scale, 38 * scale);
  ctx.stroke();
  ctx.fillStyle = cream;
  ctx.fillText(numeral, W / 2, topY + 2);
  // Croissant de lune.
  ctx.fillStyle = gold;
  ctx.beginPath();
  ctx.arc(W / 2 - nw / 2 - 70, topY, 22 * scale, 0, Math.PI * 2);
  ctx.fill();
  ctx.fillStyle = rgb(night);
  ctx.beginPath();
  ctx.arc(W / 2 - nw / 2 - 60, topY - 6, 20 * scale, 0, Math.PI * 2);
  ctx.fill();
  // Étoile à huit branches.
  ctx.fillStyle = gold;
  ctx.save();
  ctx.translate(W / 2 + nw / 2 + 70, topY);
  ctx.beginPath();
  for (let i = 0; i < 16; i++) {
    const r = i % 2 === 0 ? 26 * scale : 9 * scale;
    const a = (Math.PI / 8) * i;
    ctx.lineTo(Math.sin(a) * r, -Math.cos(a) * r);
  }
  ctx.closePath();
  ctx.fill();
  ctx.restore();

  // Arche : la pochette, sous des rayons dorés.
  const archW = Math.round((compact ? 400 : 560) * (height >= 1900 ? 1.1 : 1));
  const archH = Math.round(archW * 1.18);
  const archX = (W - archW) / 2;
  const archY = topY + (compact ? 60 : 84);
  const cx = W / 2;
  ctx.save();
  // Rayons gardés à l'intérieur du cadre.
  roundRect(ctx, m + 18, m + 18, W - (m + 18) * 2, height - (m + 18) * 2, 14);
  ctx.clip();
  ctx.strokeStyle = "rgba(246, 227, 168, 0.22)";
  ctx.lineWidth = 3;
  for (let i = 0; i < 36; i++) {
    const a = (Math.PI * 2 * i) / 36;
    const inner = archW * 0.55;
    const outer = archW * (0.82 + (i % 2) * 0.14);
    ctx.beginPath();
    ctx.moveTo(cx + Math.cos(a) * inner, archY + archW / 2 + Math.sin(a) * inner);
    ctx.lineTo(cx + Math.cos(a) * outer, archY + archW / 2 + Math.sin(a) * outer);
    ctx.stroke();
  }
  ctx.restore();
  const archPath = () => {
    ctx.beginPath();
    ctx.moveTo(archX, archY + archH);
    ctx.lineTo(archX, archY + archW / 2);
    ctx.arc(cx, archY + archW / 2, archW / 2, Math.PI, 0);
    ctx.lineTo(archX + archW, archY + archH);
    ctx.closePath();
  };
  ctx.save();
  archPath();
  ctx.clip();
  if (cover) {
    const side = Math.max(archW, archH);
    ctx.drawImage(cover, cx - side / 2, archY + archH / 2 - side / 2, side, side);
  } else {
    const g = ctx.createLinearGradient(0, archY, 0, archY + archH);
    g.addColorStop(0, rgb(c1));
    g.addColorStop(1, rgb(c2));
    ctx.fillStyle = g;
    ctx.fillRect(archX, archY, archW, archH);
  }
  // Voile bas : l'arche se fond dans la carte.
  const veil = ctx.createLinearGradient(0, archY + archH * 0.6, 0, archY + archH);
  veil.addColorStop(0, "rgba(0,0,0,0)");
  veil.addColorStop(1, rgb(night, 0.55));
  ctx.fillStyle = veil;
  ctx.fillRect(archX, archY, archW, archH);
  ctx.restore();
  ctx.strokeStyle = gold;
  ctx.lineWidth = 5;
  archPath();
  ctx.stroke();

  // Ruban du titre, en bas.
  const ribbonH = compact ? 110 : 130;
  const ribbonY = height - m - 40 - ribbonH;
  const rx = m + 70;
  const ribbonW = W - 2 * rx;
  ctx.fillStyle = rgb(mix(night, [0, 0, 0], 0.35), 0.92);
  ctx.beginPath();
  ctx.moveTo(rx, ribbonY);
  ctx.lineTo(rx + ribbonW, ribbonY);
  ctx.lineTo(rx + ribbonW - 26, ribbonY + ribbonH / 2);
  ctx.lineTo(rx + ribbonW, ribbonY + ribbonH);
  ctx.lineTo(rx, ribbonY + ribbonH);
  ctx.lineTo(rx + 26, ribbonY + ribbonH / 2);
  ctx.closePath();
  ctx.fill();
  ctx.strokeStyle = gold;
  ctx.lineWidth = 3;
  ctx.stroke();
  ctx.fillStyle = cream;
  ctx.font = `700 ${Math.round(46 * scale)}px ${SERIF}`;
  (ctx as Spaced).letterSpacing = "6px";
  ctx.fillText(fit(ctx, input.title.toUpperCase(), ribbonW - 120), W / 2, ribbonY + ribbonH * 0.4);
  (ctx as Spaced).letterSpacing = "0px";
  ctx.fillStyle = "rgba(245, 234, 210, 0.75)";
  ctx.font = `italic 500 ${Math.round(30 * scale)}px ${SERIF}`;
  ctx.fillText(fit(ctx, input.artist, ribbonW - 140), W / 2, ribbonY + ribbonH * 0.76);

  // Paroles : italique avec empattements, centrées entre l'arche et le ruban.
  const top = archY + archH + 44;
  const bottom = ribbonY - 30;
  const width = W - 2 * (m + 90);
  let size = 64;
  let rows: string[] = [];
  for (; size >= 26; size -= 3) {
    ctx.font = `italic 600 ${size}px ${SERIF}`;
    rows = input.lines.flatMap((l) => wrap((t) => ctx.measureText(t).width, l, width));
    if (rows.length * size * 1.3 <= bottom - top) break;
  }
  ctx.font = `italic 600 ${size}px ${SERIF}`;
  const lineH = size * 1.3;
  const shown = rows.slice(0, Math.max(1, Math.floor((bottom - top) / lineH)));
  let y = top + (bottom - top - shown.length * lineH) / 2 + lineH / 2;
  ctx.fillStyle = cream;
  ctx.shadowColor = "rgba(0,0,0,0.5)";
  ctx.shadowBlur = 12;
  for (const row of shown) {
    ctx.fillText(fit(ctx, row, width), W / 2, y);
    y += lineH;
  }
  ctx.shadowBlur = 0;

  // Signature discrète sous le ruban.
  ctx.fillStyle = "rgba(246, 227, 168, 0.6)";
  ctx.font = `600 22px ${FONT}`;
  ctx.fillText("MUSIC PLAYER", W / 2, height - m - 22);
  ctx.textAlign = "left";
  ctx.textBaseline = "alphabetic";
}
