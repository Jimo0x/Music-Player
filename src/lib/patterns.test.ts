import { describe, expect, it } from "vitest";
import { PATTERNS, defaultPattern, normalizePattern, patternMask } from "./patterns";

describe("motifs du fond", () => {
  it("chaque motif donne un masque SVG en ligne, à la taille demandée", () => {
    for (const p of PATTERNS) {
      const mask = patternMask(p.id, 2);
      expect(mask, p.id).not.toBeNull();
      expect(mask!.image.startsWith('url("data:image/svg+xml,')).toBe(true);
      expect(mask!.size).toBe(`${p.w * 2}px ${p.h * 2}px`);
      // Rien d'extérieur : ni script ni adresse chargée.
      expect(decodeURIComponent(mask!.image)).not.toMatch(/<script|https?:\/\/(?!www\.w3\.org)/);
    }
  });

  it("« aucun » ne donne pas de masque", () => {
    expect(patternMask("none")).toBeNull();
  });

  it("borne les réglages et refuse un motif inconnu", () => {
    expect(normalizePattern(null)).toEqual(defaultPattern());
    const p = normalizePattern({ id: "zebre", opacity: 2, scale: 0.1, color: "rouge" });
    expect(p).toEqual({ id: "none", opacity: 0.4, scale: 0.5, color: "text" });
    expect(normalizePattern({ id: "waves", color: "accent" })).toMatchObject({ id: "waves", color: "accent" });
  });
});
