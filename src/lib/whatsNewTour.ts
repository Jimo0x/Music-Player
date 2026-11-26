// « Quoi de neuf ? » interactif (lot 0.13.0) : chaque nouveauté devient une carte, avec un
// pictogramme, un titre court (ce qui précède « : »), le texte, et « Essayer » quand elle
// mène quelque part dans l'appli. Le sujet est reconnu à des mots du texte : rien à ajouter
// dans nouveautes.json, qui reste la seule source.
import type { Route } from "@/stores/ui";
import type { Release } from "./whatsNew";

export interface TourCard {
  version: string;
  title: string;
  text: string;
  icon: string;
  route: Route | null;
}

const TOPICS: { test: RegExp; icon: string; route: Route | null }[] = [
  { test: /immersi|8D|acoustique|ambiance/i, icon: "orbit", route: { name: "settings", section: "mode-immersif" } },
  { test: /égaliseur/i, icon: "eq", route: { name: "settings" } },
  { test: /sortie audio/i, icon: "volume", route: { name: "settings" } },
  { test: /barre de progression|forme d'onde|vague/i, icon: "play", route: { name: "appearance" } },
  { test: /arrière-plan|motif|thème|icône|police|jaquette|personnalis|apparence/i, icon: "brush", route: { name: "appearance" } },
  { test: /discussion|proche|envoy/i, icon: "share", route: { name: "friends" } },
  { test: /partage/i, icon: "share", route: { name: "shared" } },
  { test: /recherche|télécharg|spotify|lien/i, icon: "import", route: { name: "download" } },
  { test: /playlist/i, icon: "playlist", route: { name: "playlists" } },
  { test: /mise à jour|installation/i, icon: "refresh", route: null },
];

/** Carte d'une nouveauté : « Titre : texte » coupé au premier deux-points (s'il est court). */
export function toCard(version: string, item: string): TourCard {
  const cut = item.indexOf(" : ");
  const hasTitle = cut > 0 && cut <= 40;
  const title = hasTitle ? item.slice(0, cut) : "Nouveauté";
  const text = hasTitle ? item.slice(cut + 3) : item;
  const topic = TOPICS.find((t) => t.test.test(item));
  return {
    version,
    title: title.charAt(0).toUpperCase() + title.slice(1),
    text: text.charAt(0).toUpperCase() + text.slice(1),
    icon: topic?.icon ?? "sparkle",
    route: topic?.route ?? null,
  };
}

/** Cartes de toutes les nouveautés montrées, de la plus récente version à la plus ancienne. */
export function tourCards(releases: Release[]): TourCard[] {
  return releases.flatMap((r) => r.items.map((item) => toCard(r.version, item)));
}

/** Au-delà, la liste complète est plus pratique que les cartes une à une (historique). */
export const TOUR_MAX_CARDS = 12;
