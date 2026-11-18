import { beforeEach, describe, expect, it } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import { useUiStore } from "./ui";

beforeEach(() => setActivePinia(createPinia()));

describe("recherche", () => {
  it("revient aux résultats depuis une autre section sans retaper (champ gardé)", () => {
    const ui = useUiStore();
    ui.setSearch("the weeknd");
    expect(ui.route).toEqual({ name: "search", query: "the weeknd" });
    ui.go({ name: "artists", artist: "The Weeknd" });
    expect(ui.search).toBe("the weeknd");
    ui.reopenSearch();
    expect(ui.route).toEqual({ name: "search", query: "the weeknd" });
  });

  it("ne fait rien si le champ est vide ou si les résultats sont déjà là", () => {
    const ui = useUiStore();
    ui.go({ name: "albums" });
    ui.reopenSearch();
    expect(ui.route.name).toBe("albums");
    ui.setSearch("damso");
    const depth = ui.history.length;
    ui.reopenSearch();
    expect(ui.history.length).toBe(depth);
  });
});

describe("sélection de plusieurs morceaux", () => {
  it("coche, décoche, tout sélectionner puis rien, et se termine", () => {
    const ui = useUiStore();
    ui.selectionPool = ["a", "b", "c"];
    ui.startSelection("b");
    expect(ui.selecting).toBe(true);
    expect(ui.selected).toEqual(["b"]);
    ui.toggleSelected("c");
    ui.toggleSelected("b");
    expect(ui.selected).toEqual(["c"]);
    ui.selectAll();
    expect(ui.selected).toEqual(["a", "b", "c"]);
    ui.selectAll();
    expect(ui.selected).toEqual([]);
    ui.clearSelection();
    expect(ui.selecting).toBe(false);
  });

  it("le bouton retour ferme d'abord la fenêtre, puis la sélection, sans changer de page", () => {
    const ui = useUiStore();
    ui.go({ name: "songs" });
    ui.startSelection("a");
    ui.addToPlaylists = ["a"];
    ui.onPopState();
    expect(ui.addToPlaylists).toBeNull();
    expect(ui.selecting).toBe(true);
    ui.onPopState();
    expect(ui.selecting).toBe(false);
    expect(ui.route).toEqual({ name: "songs" });
  });
});
