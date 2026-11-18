// Débit d'un transfert et temps restant, partagés par la section Partage et la discussion
// avec un proche (code venu de stores/share.ts et components/TransferRow.vue, déplacé tel quel).

import { formatBytes } from "./format";

/** Mesure du débit : valeur lissée (octets/s), dernier instant (ms) et octets transférés. */
export interface RateSample {
  rate: number | null;
  sampleAt: number;
  sampleDone: number;
}

/** Met à jour le débit lissé. `moving` faux (attente, reprise) : la mesure repart au prochain octet. */
export function updateRate(s: RateSample, done: number, moving: boolean, now = performance.now()): void {
  if (!moving) {
    s.sampleAt = 0;
  } else if (s.sampleAt === 0 || done < s.sampleDone) {
    s.sampleAt = now;
    s.sampleDone = done;
  } else if (now - s.sampleAt >= 700) {
    const instant = ((done - s.sampleDone) * 1000) / (now - s.sampleAt);
    s.rate = s.rate === null ? instant : s.rate * 0.75 + instant * 0.25;
    s.sampleAt = now;
    s.sampleDone = done;
  }
}

export function remaining(secs: number): string {
  if (secs < 5) return "presque fini";
  if (secs < 60) return `≈ ${Math.ceil(secs / 5) * 5} s restantes`;
  if (secs < 3600) return `≈ ${Math.round(secs / 60)} min restantes`;
  const h = Math.floor(secs / 3600);
  const m = Math.round((secs % 3600) / 60);
  return `≈ ${h} h ${m.toString().padStart(2, "0")} restantes`;
}

/** « ≈ 2 min restantes · 1,2 Mo/s », d'après le débit mesuré ; vide tant qu'il n'est pas fiable. */
export function pace(done: number, total: number, rate: number | null): string {
  if (rate === null || rate < 1) return "";
  return `${remaining(Math.max(0, total - done) / rate)} · ${formatBytes(rate)}/s`;
}
