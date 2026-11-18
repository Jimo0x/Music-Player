// « Quoi de neuf ? » : notes de chaque version, écrites pour les utilisateurs dans
// src/nouveautes.json (seule source : la CI en tire aussi le texte de la version GitHub,
// scripts/release-notes.mjs, et refuse de publier une version sans notes).

import releases from "@/nouveautes.json";

export interface Release {
  version: string;
  date: string;
  items: string[];
}

export const RELEASES: Release[] = releases;

/** Dernière version dont les nouveautés ont été montrées. */
export const SEEN_KEY = "mp:nouveautes-vues";

/** « 0.10.2 » → [0, 10, 2]. */
function key(version: string): number[] {
  return version
    .replace(/^app-v|^v/, "")
    .split(".")
    .map((n) => Number.parseInt(n, 10) || 0);
}

export function compareVersions(a: string, b: string): number {
  const ka = key(a);
  const kb = key(b);
  for (let i = 0; i < Math.max(ka.length, kb.length); i++) {
    const d = (ka[i] ?? 0) - (kb[i] ?? 0);
    if (d) return d;
  }
  return 0;
}

/**
 * Nouveautés à montrer au lancement : celles des versions arrivées depuis la dernière vue
 * (plusieurs si des mises à jour ont été sautées). Jamais vues (première version avec cette
 * fenêtre, ou installation neuve) : celles de la version installée seulement.
 */
export function unseenReleases(current: string, lastSeen: string | null, all: Release[] = RELEASES): Release[] {
  if (!lastSeen) return all.filter((r) => compareVersions(r.version, current) === 0);
  return all.filter((r) => compareVersions(r.version, lastSeen) > 0 && compareVersions(r.version, current) <= 0);
}

/** Points de la section « Quoi de neuf ? » du texte d'une version GitHub (mise à jour proposée). */
export function notesFromReleaseBody(body: string | null | undefined): string[] {
  if (!body) return [];
  const start = body.search(/^##\s*Quoi de neuf/im);
  if (start < 0) return [];
  const items: string[] = [];
  for (const line of body.slice(start).split(/\r?\n/).slice(1)) {
    if (/^##\s/.test(line)) break;
    const m = line.match(/^\s*[-*]\s+(.+)$/);
    if (m) items.push(m[1].trim());
  }
  return items;
}
