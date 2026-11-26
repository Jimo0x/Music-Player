import { describe, expect, it } from "vitest";
import { BAND_FREQS, EQ_PRESETS, normalizeGains } from "./eq";

describe("égaliseur 10 bandes", () => {
  it("convertit un ancien réglage à 3 bandes", () => {
    expect(normalizeGains([6, -2, 3])).toEqual([6, 6, 6, 6, -2, -2, -2, 3, 3, 3]);
  });

  it("borne les valeurs et refuse un réglage illisible", () => {
    expect(normalizeGains([40, 0, 0, 0, 0, 0, 0, 0, 0, Number.NaN])).toEqual([12, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    expect(normalizeGains("x")).toEqual(Array(10).fill(0));
    expect(normalizeGains([1, 2])).toEqual(Array(10).fill(0));
  });

  it("chaque préréglage a une valeur par bande, dans les limites", () => {
    for (const p of EQ_PRESETS) {
      expect(p.gains, p.name).toHaveLength(BAND_FREQS.length);
      expect(p.gains.every((g) => g >= -12 && g <= 12), p.name).toBe(true);
    }
    expect(new Set(EQ_PRESETS.map((p) => p.gains.join())).size).toBe(EQ_PRESETS.length);
  });
});
