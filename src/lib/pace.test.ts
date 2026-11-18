import { describe, expect, it } from "vitest";
import { pace, remaining, type RateSample, updateRate } from "./pace";

describe("débit et temps restant", () => {
  it("mesure le débit après 700 ms puis le lisse", () => {
    const s: RateSample = { rate: null, sampleAt: 0, sampleDone: 0 };
    updateRate(s, 0, true, 1000);
    updateRate(s, 500_000, true, 1500); // trop tôt : pas de mesure
    expect(s.rate).toBeNull();
    updateRate(s, 1_000_000, true, 2000);
    expect(s.rate).toBe(1_000_000);
    updateRate(s, 1_500_000, true, 3000);
    expect(s.rate).toBe(1_000_000 * 0.75 + 500_000 * 0.25);
  });

  it("repart à zéro après une attente", () => {
    const s: RateSample = { rate: 1000, sampleAt: 5, sampleDone: 10 };
    updateRate(s, 10, false, 100);
    expect(s.sampleAt).toBe(0);
  });

  it("écrit le temps restant et le débit, rien tant que le débit est inconnu", () => {
    expect(pace(0, 10, null)).toBe("");
    expect(remaining(3)).toBe("presque fini");
    expect(remaining(42)).toBe("≈ 45 s restantes");
    expect(remaining(3900)).toBe("≈ 1 h 05 restantes");
    expect(pace(0, 120_000_000, 1_000_000)).toMatch(/^≈ 2 min restantes · .+\/s$/);
  });
});
