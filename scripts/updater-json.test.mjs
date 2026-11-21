import { describe, expect, it } from "vitest";
import { EXPECTED_PLATFORMS, buildUpdaterJson, platformsOf } from "./updater-json.mjs";

// Noms réels des fichiers de la 0.11.0 (publiée avec ses 9 plateformes).
const base = "https://github.com/LoickAmg/Music-Player/releases/download/app-v0.11.0/";
const names = [
  "Music.Player_0.11.0_amd64.AppImage",
  "Music.Player_0.11.0_amd64.deb",
  "Music.Player-0.11.0-1.x86_64.rpm",
  "Music.Player_aarch64.app.tar.gz",
  "Music.Player_0.11.0_x64_en-US.msi",
  "Music.Player_0.11.0_x64-setup.exe",
  "Music.Player_0.11.0_aarch64.dmg",
  "Music-Player-android-arm64.apk",
];
const signed = names.filter((n) => !n.endsWith(".dmg") && !n.endsWith(".apk"));

describe("latest.json écrit une seule fois (#12)", () => {
  it("retrouve les 9 plateformes de la 0.11.0, avec le bon fichier par défaut", () => {
    const signatures = Object.fromEntries(signed.map((n) => [n, `sig de ${n}\n`]));
    const { json, missing } = buildUpdaterJson({ version: "0.11.0", notes: "Notes", pubDate: "2026-11-21T20:00:00Z", assets: names.map((name) => ({ name, url: base + name })), signatures });
    expect(missing).toEqual([]);
    expect(Object.keys(json.platforms)).toEqual([...EXPECTED_PLATFORMS].sort());
    expect(json.platforms["windows-x86_64"].url).toBe(base + "Music.Player_0.11.0_x64_en-US.msi");
    expect(json.platforms["windows-x86_64-nsis"].url).toBe(base + "Music.Player_0.11.0_x64-setup.exe");
    expect(json.platforms["linux-x86_64"].url).toBe(base + "Music.Player_0.11.0_amd64.AppImage");
    expect(json.platforms["darwin-aarch64"].signature).toBe("sig de Music.Player_aarch64.app.tar.gz");
    expect(json).toMatchObject({ version: "0.11.0", notes: "Notes", pub_date: "2026-11-21T20:00:00Z" });
  });

  it("dit ce qui manque quand un système n'a pas publié", () => {
    const signatures = Object.fromEntries(signed.filter((n) => !n.endsWith(".app.tar.gz")).map((n) => [n, "sig"]));
    const { missing } = buildUpdaterJson({ version: "0.11.0", notes: "", pubDate: "", assets: names.map((name) => ({ name, url: base + name })), signatures });
    expect(missing).toEqual(["darwin-aarch64", "darwin-aarch64-app"]);
  });

  it("ignore ce qui n'est pas un installateur de mise à jour", () => {
    expect(platformsOf("Music.Player_0.11.0_aarch64.dmg")).toEqual([]);
    expect(platformsOf("Music-Player-android.apk")).toEqual([]);
    expect(platformsOf("latest.json")).toEqual([]);
  });
});
