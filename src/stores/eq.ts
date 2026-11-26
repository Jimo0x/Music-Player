import { defineStore } from "pinia";
import { api } from "@/lib/api";

// Égaliseur 10 bandes d'octave (lot 0.13.0). Le moteur convertit les réglages d'une version à
// 3 bandes ; ici, un tableau de 3 valeurs reçu (sauvegarde ancienne) est converti de même.

export const BAND_FREQS = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000] as const;
export const BAND_LABELS = ["32", "64", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"] as const;
export const EQ_MIN = -12;
export const EQ_MAX = 12;

export type Gains = number[];

const flat = (): Gains => Array(BAND_FREQS.length).fill(0);

export const EQ_PRESETS: { name: string; gains: Gains }[] = [
  { name: "Neutre", gains: flat() },
  { name: "Basses +", gains: [6, 6, 5, 3, 1, 0, 0, 0, 0, 0] },
  { name: "Voix", gains: [-2, -2, -1, 0, 2, 4, 4, 3, 1, 0] },
  { name: "Aigus +", gains: [0, 0, 0, 0, 0, 0, 1, 3, 5, 6] },
  { name: "Loudness", gains: [5, 4, 3, 1, 0, -1, 0, 2, 4, 5] },
  { name: "Acoustique", gains: [2, 2, 2, 2, 2, 2, 2, 3, 3, 3] },
  { name: "Rock", gains: [4, 3, 2, 0, -1, -1, 1, 2, 3, 4] },
  { name: "Électro", gains: [5, 4, 1, 0, -2, 0, 1, 3, 4, 5] },
  { name: "Hip-hop", gains: [6, 5, 3, 1, -1, -1, 1, 1, 2, 3] },
  { name: "Jazz", gains: [3, 2, 1, 2, -1, -1, 0, 1, 2, 3] },
  { name: "Classique", gains: [0, 0, 0, 0, 0, 0, -1, -1, 1, 2] },
  { name: "Podcast", gains: [-4, -3, -1, 1, 3, 4, 4, 2, 0, -2] },
  { name: "Nuit douce", gains: [-1, 0, 0, 0, 0, -1, -2, -3, -4, -5] },
];

/** Réglage reçu (moteur, sauvegarde) : 10 valeurs, ou 3 d'une ancienne version, bornées. */
export function normalizeGains(raw: unknown): Gains {
  const values = Array.isArray(raw) ? raw.map((v) => (typeof v === "number" && Number.isFinite(v) ? Math.max(EQ_MIN, Math.min(EQ_MAX, v)) : 0)) : [];
  if (values.length === BAND_FREQS.length) return values;
  if (values.length === 3) {
    const [b, m, t] = values;
    return [b, b, b, b, m, m, m, t, t, t];
  }
  return flat();
}

export const useEqStore = defineStore("eq", {
  state: () => ({
    gains: flat(),
    /** « Comparer » maintenu : le son passe sans égaliseur, les réglages restent. */
    bypass: false,
  }),
  getters: {
    activePreset: (state) => EQ_PRESETS.find((p) => p.gains.every((g, i) => g === state.gains[i]))?.name ?? null,
  },
  actions: {
    setFromInitialState(gains: unknown) {
      this.gains = normalizeGains(gains);
    },
    /** Change une bande ; l'envoi au moteur suit tout de suite (curseur glissé en direct). */
    async setGain(index: number, value: number) {
      if (index < 0 || index >= BAND_FREQS.length) return;
      const next = [...this.gains];
      next[index] = Math.round(Math.max(EQ_MIN, Math.min(EQ_MAX, value)) * 2) / 2;
      if (next[index] === this.gains[index]) return;
      this.gains = next;
      if (!this.bypass) await api.setEqGains(next);
    },
    async applyPreset(gains: Gains) {
      this.gains = normalizeGains(gains);
      if (!this.bypass) await api.setEqGains(this.gains);
    },
    async reset() {
      await this.applyPreset(flat());
    },
    async setBypass(on: boolean) {
      if (this.bypass === on) return;
      this.bypass = on;
      await api.setEqGains(on ? flat() : this.gains);
    },
  },
});
