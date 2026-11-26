import { describe, expect, it } from "vitest";
import { DEFAULT_IMMERSIVE, describeImmersive, matchingRoom, normalizeImmersive, ROOM_PRESETS } from "./immersive";

describe("mode immersif", () => {
  it("complète et borne des réglages reçus", () => {
    const s = normalizeImmersive({ enabled: true, engine: "autre", speed: 999, intensity: -1, room_mix: Number.NaN });
    expect(s.enabled).toBe(true);
    expect(s.engine).toBe("head");
    expect(s.speed).toBe(40);
    expect(s.intensity).toBe(0);
    expect(s.room_mix).toBe(DEFAULT_IMMERSIVE.room_mix);
    expect(normalizeImmersive(null)).toEqual(DEFAULT_IMMERSIVE);
  });

  it("reconnaît chaque salle proposée, et une salle retouchée comme sur mesure", () => {
    for (const p of ROOM_PRESETS) {
      const s = { ...DEFAULT_IMMERSIVE, room_size: p.size, room_damping: p.damping, room_mix: p.mix, room_predelay_ms: p.predelay };
      expect(matchingRoom(s)?.id).toBe(p.id);
    }
    expect(matchingRoom({ ...DEFAULT_IMMERSIVE, room_size: 0.33 })).toBeNull();
  });

  it("résume une ambiance en quelques mots", () => {
    expect(describeImmersive(DEFAULT_IMMERSIVE)).toBe("Désactivé");
    const cathedrale = ROOM_PRESETS.find((p) => p.id === "cathedrale")!;
    const s = { ...DEFAULT_IMMERSIVE, enabled: true, room: true, room_size: cathedrale.size, room_damping: cathedrale.damping, room_mix: cathedrale.mix, room_predelay_ms: cathedrale.predelay };
    expect(describeImmersive(s)).toBe("8D · Cathédrale");
    expect(describeImmersive({ ...s, rotation: false, room_size: 0.1 })).toBe("Salle sur mesure");
  });

  it("le préréglage par défaut du moteur est la salle de concert", () => {
    expect(matchingRoom(DEFAULT_IMMERSIVE)?.id).toBe("concert");
  });
});
