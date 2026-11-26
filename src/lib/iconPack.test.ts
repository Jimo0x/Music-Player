import { describe, expect, it } from "vitest";
import { ICON_NAMES, iconNameOf, normalizeIcons, sanitizeSvg } from "./iconPack";

describe("jeu d'icônes importé", () => {
  it("garde le dessin et ramène les couleurs à celle du texte", () => {
    const icon = sanitizeSvg('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" fill="#ff0000"><path d="M4 4h24v24H4z" fill="#00ff00"/><circle cx="16" cy="16" r="4" stroke="blue" fill="none"/></svg>')!;
    expect(icon.viewBox).toBe("0 0 32 32");
    expect(icon.fill).toBe("currentColor");
    expect(icon.body).toBe('<path d="M4 4h24v24H4z" fill="currentColor"/><circle cx="16" cy="16" r="4" stroke="currentColor" fill="none"/>');
  });

  it("retire scripts, styles, liens, images et gestionnaires d'événements", () => {
    const evil =
      '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 24 24">' +
      '<script>alert(1)</script><style>*{display:none}</style><image href="https://x.test/a.png"/>' +
      '<a href="javascript:alert(1)"><path d="M0 0h1"/></a><use xlink:href="#p"/>' +
      '<path id="p" d="M1 1h10" onclick="alert(1)" style="fill:red" fill="url(#g)" transform="translate(1 1)"/>' +
      '<foreignObject><div>hello</div></foreignObject></svg>';
    const icon = sanitizeSvg(evil)!;
    expect(icon.body).toBe('<path d="M1 1h10" fill="currentColor" transform="translate(1 1)"/>');
    expect(icon.body).not.toMatch(/script|style|href|onclick|image|foreignObject|url\(/i);
  });

  it("refuse ce qui n'est pas un SVG ou un SVG vide", () => {
    expect(sanitizeSvg("<html><body>non</body></html>")).toBeNull();
    expect(sanitizeSvg("pas du xml")).toBeNull();
    expect(sanitizeSvg('<svg xmlns="http://www.w3.org/2000/svg"><script>x</script></svg>')).toBeNull();
  });

  it("refuse un tracé ou une transformation piégés", () => {
    const icon = sanitizeSvg('<svg xmlns="http://www.w3.org/2000/svg"><path d="M0 0 expression(alert(1))"/><rect width="4" height="4" transform="url(x)"/></svg>')!;
    expect(icon.body).toBe('<path /><rect width="4" height="4"/>');
    expect(icon.viewBox).toBe("0 0 24 24");
  });

  it("reconnaît le nom de l'icône d'après le fichier", () => {
    expect(ICON_NAMES).toContain("play");
    expect(iconNameOf("Play.svg")).toBe("play");
    expect(iconNameOf("C:\\icones\\playlist.SVG")).toBe("playlist");
    expect(iconNameOf("inconnue.svg")).toBeNull();
  });

  it("renettoie un jeu lu dans les réglages et ignore les noms inconnus", () => {
    const icons = normalizeIcons({
      play: { viewBox: "0 0 24 24", body: '<path d="M8 5v14l11-7z"/><script>x</script>', fill: "currentColor", stroke: "none" },
      truc: { viewBox: "0 0 24 24", body: '<path d="M0 0"/>' },
      pause: "pas un objet",
    });
    expect(Object.keys(icons)).toEqual(["play"]);
    expect(icons.play.body).toBe('<path d="M8 5v14l11-7z"/>');
  });
});
