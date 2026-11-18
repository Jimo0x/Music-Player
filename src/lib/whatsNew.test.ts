import { describe, expect, it } from "vitest";
import { compareVersions, notesFromReleaseBody, RELEASES, unseenReleases, type Release } from "./whatsNew";
import pkg from "../../package.json";

const all: Release[] = [
  { version: "0.10.0", date: "", items: ["c"] },
  { version: "0.9.1", date: "", items: ["b"] },
  { version: "0.9.0", date: "", items: ["a"] },
];

describe("Quoi de neuf ?", () => {
  it("compare les numéros comme des nombres", () => {
    expect(compareVersions("0.10.0", "0.9.9")).toBeGreaterThan(0);
    expect(compareVersions("app-v0.9.0", "0.9.0")).toBe(0);
  });

  it("montre les versions arrivées depuis la dernière vue, sautées comprises", () => {
    expect(unseenReleases("0.10.0", "0.9.0", all).map((r) => r.version)).toEqual(["0.10.0", "0.9.1"]);
    expect(unseenReleases("0.10.0", "0.10.0", all)).toEqual([]);
  });

  it("jamais vues : seulement la version installée", () => {
    expect(unseenReleases("0.9.1", null, all).map((r) => r.version)).toEqual(["0.9.1"]);
  });

  it("lit la section « Quoi de neuf ? » du texte d'une version GitHub", () => {
    const body = "## Quoi de neuf ?\n\n- Premier point\n- Second point\n\n## Installation\n- pas ça";
    expect(notesFromReleaseBody(body)).toEqual(["Premier point", "Second point"]);
    expect(notesFromReleaseBody("Installeurs générés automatiquement")).toEqual([]);
  });

  it("la version en cours du projet a ses nouveautés (sinon la CI refuse de publier)", () => {
    const current = RELEASES.find((r) => r.version === pkg.version);
    expect(current?.items.length).toBeGreaterThan(0);
    // Plus récentes d'abord.
    const sorted = [...RELEASES].sort((a, b) => compareVersions(b.version, a.version));
    expect(RELEASES.map((r) => r.version)).toEqual(sorted.map((r) => r.version));
  });
});
