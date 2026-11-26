import { describe, expect, it } from "vitest";
import { RELEASES } from "./whatsNew";
import { toCard, tourCards } from "./whatsNewTour";

describe("« Quoi de neuf ? » en cartes", () => {
  it("coupe le titre au premier « : » et reconnaît le sujet", () => {
    const c = toCard("0.13.0", "Égaliseur : 10 bandes, une courbe à façonner du doigt.");
    expect(c).toMatchObject({ title: "Égaliseur", text: "10 bandes, une courbe à façonner du doigt.", icon: "eq", route: { name: "settings" } });
    const d = toCard("0.12.0", "Arrière-plan : mettez votre image en fond.");
    expect(d.route).toEqual({ name: "appearance" });
  });

  it("les nouveautés du mode immersif mènent à sa section des Réglages", () => {
    const c = toCard("0.14.0", "Acoustique de salle : écoutez comme dans une cathédrale.");
    expect(c).toMatchObject({ icon: "orbit", route: { name: "settings", section: "mode-immersif" } });
    expect(toCard("0.14.0", "Ambiance par playlist : sa propre ambiance.").route).toEqual({ name: "settings", section: "mode-immersif" });
  });

  it("une nouveauté sans titre ni sujet reconnu reste une carte simple", () => {
    const c = toCard("0.9.0", "Les sons Dolby se lisent maintenant partout.");
    expect(c.title).toBe("Nouveauté");
    expect(c.icon).toBe("sparkle");
    expect(c.route).toBeNull();
  });

  it("chaque nouveauté publiée donne une carte avec un texte", () => {
    const cards = tourCards(RELEASES.slice(0, 3));
    expect(cards.length).toBe(RELEASES.slice(0, 3).reduce((n, r) => n + r.items.length, 0));
    expect(cards.every((c) => c.text.length > 3 && c.title.length > 0)).toBe(true);
  });
});
