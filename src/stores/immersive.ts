import { defineStore } from "pinia";
import { api } from "@/lib/api";
import type { ImmersiveProfiles, ImmersiveSettings } from "@/lib/types";

// Mode immersif (lot 0.14.0) : rotation 8D autour de la tête et acoustique de salle,
// calculées en Rust (immersive.rs). Réglages globaux, et ambiance propre à une playlist qui
// s'applique quand la lecture est lancée depuis elle.

export const DEFAULT_IMMERSIVE: ImmersiveSettings = {
  enabled: false,
  rotation: true,
  engine: "head",
  speed: 10,
  clockwise: true,
  intensity: 0.85,
  room: false,
  room_preset: "concert",
  room_size: 0.72,
  room_damping: 0.45,
  room_mix: 0.3,
  room_predelay_ms: 25,
};

export interface RoomPreset {
  id: string;
  name: string;
  hint: string;
  size: number;
  damping: number;
  mix: number;
  predelay: number;
}

/** Salles proposées : taille, amortissement des aigus, dosage, pré-délai (ms). */
export const ROOM_PRESETS: RoomPreset[] = [
  { id: "studio", name: "Studio", hint: "Petite pièce traitée, juste un peu d'air", size: 0.15, damping: 0.7, mix: 0.15, predelay: 5 },
  { id: "chambre", name: "Chambre", hint: "Une pièce de maison, intime", size: 0.35, damping: 0.55, mix: 0.22, predelay: 10 },
  { id: "club", name: "Club", hint: "Salle de concert serrée, chaude", size: 0.5, damping: 0.35, mix: 0.28, predelay: 15 },
  { id: "concert", name: "Salle de concert", hint: "Grande salle, son ample et clair", size: 0.72, damping: 0.45, mix: 0.3, predelay: 25 },
  { id: "cathedrale", name: "Cathédrale", hint: "Voûtes de pierre, longue résonance", size: 0.95, damping: 0.25, mix: 0.38, predelay: 45 },
  { id: "arene", name: "Arène", hint: "Stade en plein air, écho lointain", size: 0.88, damping: 0.6, mix: 0.34, predelay: 70 },
];

/** Bornes des réglages (identiques au moteur). */
export const SPEED_MIN = 2;
export const SPEED_MAX = 40;
export const PREDELAY_MAX = 120;

const clamp = (v: unknown, min: number, max: number, fallback: number) =>
  typeof v === "number" && Number.isFinite(v) ? Math.max(min, Math.min(max, v)) : fallback;

/** Réglages reçus (moteur, ancienne version) complétés et bornés. */
export function normalizeImmersive(raw: unknown): ImmersiveSettings {
  const o = (raw && typeof raw === "object" ? raw : {}) as Partial<ImmersiveSettings>;
  const d = DEFAULT_IMMERSIVE;
  return {
    enabled: typeof o.enabled === "boolean" ? o.enabled : d.enabled,
    rotation: typeof o.rotation === "boolean" ? o.rotation : d.rotation,
    engine: o.engine === "hrtf" ? "hrtf" : "head",
    speed: clamp(o.speed, SPEED_MIN, SPEED_MAX, d.speed),
    clockwise: typeof o.clockwise === "boolean" ? o.clockwise : d.clockwise,
    intensity: clamp(o.intensity, 0, 1, d.intensity),
    room: typeof o.room === "boolean" ? o.room : d.room,
    room_preset: typeof o.room_preset === "string" ? o.room_preset : d.room_preset,
    room_size: clamp(o.room_size, 0, 1, d.room_size),
    room_damping: clamp(o.room_damping, 0, 1, d.room_damping),
    room_mix: clamp(o.room_mix, 0, 1, d.room_mix),
    room_predelay_ms: clamp(o.room_predelay_ms, 0, PREDELAY_MAX, d.room_predelay_ms),
  };
}

/** Préréglage de salle dont les valeurs correspondent exactement, sinon null (« sur mesure »). */
export function matchingRoom(s: ImmersiveSettings): RoomPreset | null {
  const near = (a: number, b: number) => Math.abs(a - b) < 0.005;
  return ROOM_PRESETS.find((p) => near(p.size, s.room_size) && near(p.damping, s.room_damping) && near(p.mix, s.room_mix) && Math.abs(p.predelay - s.room_predelay_ms) < 0.5) ?? null;
}

/** Résumé court d'une ambiance (« 8D · Cathédrale »), pour un bouton ou une étiquette. */
export function describeImmersive(s: ImmersiveSettings): string {
  if (!s.enabled || (!s.rotation && !s.room)) return "Désactivé";
  const parts: string[] = [];
  if (s.rotation) parts.push("8D");
  if (s.room) parts.push(matchingRoom(s)?.name ?? "Salle sur mesure");
  return parts.join(" · ");
}

function normalizeProfiles(raw: ImmersiveProfiles | null | undefined): ImmersiveProfiles {
  const playlists: Record<string, ImmersiveSettings> = {};
  for (const [id, s] of Object.entries(raw?.playlists ?? {})) playlists[id] = normalizeImmersive(s);
  return { global: normalizeImmersive(raw?.global), playlists, context: raw?.context ?? null };
}

export const useImmersiveStore = defineStore("immersive", {
  state: () => ({
    profiles: { global: { ...DEFAULT_IMMERSIVE }, playlists: {}, context: null } as ImmersiveProfiles,
    /** « Comparer » maintenu : le son passe sans effet. */
    bypass: false,
  }),
  getters: {
    /** Ambiance qui joue maintenant (celle de la playlist en cours si elle en a une). */
    effective(state): ImmersiveSettings {
      const id = state.profiles.context;
      return (id && state.profiles.playlists[id]) || state.profiles.global;
    },
    /** La lecture en cours suit l'ambiance de sa playlist. */
    fromPlaylist(state): boolean {
      const id = state.profiles.context;
      return !!id && !!state.profiles.playlists[id];
    },
  },
  actions: {
    async load() {
      try {
        this.profiles = normalizeProfiles(await api.getImmersive());
      } catch {
        // moteur indisponible : réglages par défaut
      }
    },
    /** Réglages modifiés par le panneau : globaux, ou ceux d'une playlist. */
    settingsFor(playlistId: string | null): ImmersiveSettings {
      return (playlistId && this.profiles.playlists[playlistId]) || this.profiles.global;
    },
    hasOwn(playlistId: string): boolean {
      return !!this.profiles.playlists[playlistId];
    },
    /** Change des réglages ; l'envoi au moteur suit tout de suite (curseur glissé en direct). */
    async update(playlistId: string | null, patch: Partial<ImmersiveSettings>) {
      const next = normalizeImmersive({ ...this.settingsFor(playlistId), ...patch });
      if (playlistId) {
        this.profiles.playlists = { ...this.profiles.playlists, [playlistId]: next };
        this.profiles = normalizeProfiles(await api.setPlaylistImmersive(playlistId, next));
      } else {
        this.profiles.global = next;
        this.profiles = normalizeProfiles(await api.setImmersive(next));
      }
    },
    applyRoom(playlistId: string | null, preset: RoomPreset) {
      return this.update(playlistId, {
        room: true,
        room_preset: preset.id,
        room_size: preset.size,
        room_damping: preset.damping,
        room_mix: preset.mix,
        room_predelay_ms: preset.predelay,
      });
    },
    /** Donne à une playlist sa propre ambiance (copie des réglages globaux, activés) ou la lui retire. */
    async setPlaylistOwn(playlistId: string, own: boolean) {
      const settings = own ? { ...this.profiles.global, enabled: true } : null;
      this.profiles = normalizeProfiles(await api.setPlaylistImmersive(playlistId, settings));
    },
    /** Lecture lancée depuis une playlist (son identifiant) ou d'ailleurs (null). */
    async setContext(playlistId: string | null) {
      if (this.profiles.context === playlistId) return;
      this.profiles.context = playlistId;
      try {
        this.profiles = normalizeProfiles(await api.setImmersiveContext(playlistId));
      } catch {
        // sans moteur (démo) : l'état local suffit
      }
    },
    /**
     * Interrupteur en un geste (bouton de « À l'écoute ») : allume ou éteint l'ambiance qui
     * joue maintenant (celle de la playlist en cours si elle en a une), avec ses réglages
     * gardés. Allumé sans aucun effet coché : la rotation 8D est cochée.
     */
    async toggle(): Promise<boolean> {
      const target = this.fromPlaylist ? this.profiles.context : null;
      const s = this.settingsFor(target);
      const on = !s.enabled;
      await this.update(target, on && !s.rotation && !s.room ? { enabled: true, rotation: true } : { enabled: on });
      return on;
    },
    async setBypass(on: boolean) {
      if (this.bypass === on) return;
      this.bypass = on;
      await api.setImmersiveBypass(on);
    },
  },
});
