import { describe, expect, it } from "vitest";
import { defaultBackdrop } from "./backdrop";
import { defaultPattern } from "./patterns";
import { BUILTIN_THEMES } from "./themes";
import { buildThemePack, parseThemePack, type PackSource } from "./themePack";

const ocean = BUILTIN_THEMES.find((t) => t.id === "ocean")!;
const bytes = (s: string) => new TextEncoder().encode(s);

function source(): PackSource {
  return {
    theme: { ...ocean, id: "perso-1", name: "Mon océan", builtin: undefined, colors: { ...ocean.colors, sidebar: "#102030" } },
    fonts: { ui: "system", display: "custom:font-a.ttf", lyrics: "lora" },
    customFonts: [{ id: "custom:font-a.ttf", label: "Ma Police", file: "font-a.ttf" }],
    buttons: { shape: "pill", style: "glass" },
    fields: { shape: "square", style: "underline" },
    icons: "duo",
    pattern: { ...defaultPattern(), id: "waves", opacity: 0.12 },
    backdrop: { ...defaultBackdrop(), source: "image", image: "image-b.jpg", blur: 12 },
    customIcons: {},
  };
}

describe("paquet de thème complet", () => {
  it("fait l'aller-retour : couleurs, polices jointes, fond, motif et styles", async () => {
    const files: Record<string, string> = { "font-a.ttf": "POLICE", "image-b.jpg": "IMAGE" };
    const pack = await buildThemePack(source(), async (f) => bytes(files[f]));
    expect(pack.fonts).toEqual({ ui: { system: true }, display: { embedded: 0 }, lyrics: { catalog: "lora" } });
    const back = parseThemePack(JSON.stringify(pack))!;
    expect(back.name).toBe("Mon océan");
    expect(back.theme.colors.sidebar).toBe("#102030");
    expect(new TextDecoder().decode(back.embeddedFonts[0]!.bytes)).toBe("POLICE");
    expect(back.embeddedFonts[0]!.label).toBe("Ma Police");
    expect(new TextDecoder().decode(back.look.backdropImage!.bytes)).toBe("IMAGE");
    expect(back.look.backdrop).toMatchObject({ source: "image", blur: 12 });
    expect(back.look.pattern).toMatchObject({ id: "waves", opacity: 0.12 });
    expect(back.look.buttons).toEqual({ shape: "pill", style: "glass" });
    expect(back.look.icons).toBe("duo");
  });

  it("sans image jointe, un fond « image » n'est pas gardé", async () => {
    const src = { ...source(), backdrop: { ...defaultBackdrop(), source: "artwork" as const } };
    const pack = await buildThemePack(src, async () => bytes("x"));
    expect(pack.look.backdrop.image).toBeNull();
    expect(parseThemePack(JSON.stringify(pack))!.look.backdrop.source).toBe("artwork");
  });

  it("refuse un fichier qui n'est pas un paquet, et assainit un paquet trafiqué", async () => {
    expect(parseThemePack("pas du json")).toBeNull();
    expect(parseThemePack(JSON.stringify({ format: "autre", version: 1 }))).toBeNull();
    const pack = JSON.parse(JSON.stringify(await buildThemePack(source(), async () => bytes("x"))));
    pack.embeddedFonts[0].ext = "exe";
    pack.look.backdrop.image.data = "!!! pas du base64 !!!";
    pack.look.buttons.style = "<script>";
    pack.fonts.lyrics = { catalog: "../../etc" };
    const back = parseThemePack(JSON.stringify(pack))!;
    expect(back.embeddedFonts[0]).toBeNull();
    expect(back.fonts.display).toEqual({ system: true });
    expect(back.fonts.lyrics).toEqual({ system: true });
    expect(back.look.backdropImage).toBeNull();
    expect(back.look.backdrop.source).toBe("none");
    expect(back.look.buttons.style).toBe("gradient");
  });
});
