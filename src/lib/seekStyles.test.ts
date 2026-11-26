import { describe, expect, it } from "vitest";
import { SEEK_STYLES, normalizeSeekStyle, seekMask, waveformMask } from "./seekStyles";

describe("styles de la barre de progression", () => {
  it("les styles dessinés ont un masque SVG en ligne, les pleins n'en ont pas", () => {
    for (const s of ["wave", "dots", "segments"] as const) {
      const mask = seekMask(s)!;
      expect(mask.image.startsWith('url("data:image/svg+xml,')).toBe(true);
      expect(mask.size).toMatch(/px 100%$/);
    }
    expect(seekMask("classic")).toBeNull();
    expect(seekMask("neon")).toBeNull();
    expect(seekMask("waveform")).toBeNull();
  });

  it("la forme d'onde dessine une barre par valeur, à sa hauteur", () => {
    const svg = decodeURIComponent(waveformMask([0, 255, 128]));
    expect(svg.match(/<rect /g)).toHaveLength(3);
    expect(svg).toContain('height="100.0"');
    expect(svg).toContain('height="6.0"'); // hauteur minimale
    expect(svg).toContain('preserveAspectRatio="none"');
    // Sans données (analyse en cours) : 160 barres régulières.
    expect(decodeURIComponent(waveformMask(null)).match(/<rect /g)).toHaveLength(160);
  });

  it("un style inconnu revient au classique", () => {
    expect(normalizeSeekStyle("arc-en-ciel")).toBe("classic");
    expect(normalizeSeekStyle("wave")).toBe("wave");
    expect(SEEK_STYLES).toHaveLength(6);
  });
});
