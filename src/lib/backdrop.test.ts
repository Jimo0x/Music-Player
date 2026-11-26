import { describe, expect, it } from "vitest";
import { defaultBackdrop, fitWithin, normalizeBackdrop } from "./backdrop";

describe("arrière-plan", () => {
  it("revient aux réglages par défaut pour un fichier absent ou illisible", () => {
    expect(normalizeBackdrop(undefined)).toEqual(defaultBackdrop());
    expect(normalizeBackdrop("x")).toEqual(defaultBackdrop());
  });

  it("borne les curseurs et refuse une source inconnue", () => {
    const s = normalizeBackdrop({ source: "video", blur: 99, dim: -1, x: 140, y: Number.NaN });
    expect(s.source).toBe("none");
    expect(s.blur).toBe(30);
    expect(s.dim).toBe(0);
    expect(s.x).toBe(100);
    expect(s.y).toBe(50);
  });

  it("n'accepte qu'un simple nom de fichier pour l'image", () => {
    expect(normalizeBackdrop({ source: "image", image: "image-abc.jpg" }).image).toBe("image-abc.jpg");
    expect(normalizeBackdrop({ source: "image", image: "../appearance.json" }).image).toBeNull();
    expect(normalizeBackdrop({ source: "image", image: "dossier/x.jpg" }).image).toBeNull();
  });

  it("sans image, la source « image » retombe sur « aucun »", () => {
    expect(normalizeBackdrop({ source: "image", image: null }).source).toBe("none");
    expect(normalizeBackdrop({ source: "artwork" }).source).toBe("artwork");
  });

  it("réduit une grande image sans jamais agrandir une petite", () => {
    expect(fitWithin(6000, 4000, 2560)).toEqual({ width: 2560, height: 1707 });
    expect(fitWithin(800, 600, 2560)).toEqual({ width: 800, height: 600 });
    expect(fitWithin(0, 600, 2560)).toEqual({ width: 0, height: 0 });
  });
});
