// Traduction de l'interface (#23) : français, anglais, ou langue du système quand elle
// correspond à l'une des deux (sinon français, qui reste la langue par défaut de l'appli).
//
// Le texte français écrit dans chaque composant sert lui-même de clé : `t("Rechercher")` rend
// le français tel quel si la langue courante est le français, et va chercher sa traduction
// anglaise sinon (ou le français, si cette chaîne n'a pas encore été traduite — jamais de trou
// dans l'interface pendant que la traduction avance écran par écran).
//
// Ne couvre, pour l'instant, que les textes écrits dans l'interface (Vue). Les messages
// d'erreur renvoyés par le moteur Rust restent en français (CLAUDE.md : « erreurs en français
// pour l'interface ») : les traduire demanderait de leur donner un code plutôt qu'un texte tout
// fait, un chantier à part à valider avec le propriétaire avant de s'y lancer.

import { reactive } from "vue";

export type Locale = "fr" | "en";
/** Préférence retenue : une langue précise, ou la langue du système (déduite à chaque démarrage,
 * puisqu'elle peut changer entre deux lancements). */
export type LocalePref = Locale | "systeme";

const KEY = "mp:langue";

function systemLocale(): Locale {
  const lang = (typeof navigator !== "undefined" && navigator.language ? navigator.language : "fr").toLowerCase();
  return lang.startsWith("en") ? "en" : "fr";
}

function readPref(): LocalePref {
  try {
    const v = localStorage.getItem(KEY);
    return v === "fr" || v === "en" ? v : "systeme";
  } catch {
    return "systeme";
  }
}

const state = reactive({ pref: readPref() as LocalePref, locale: "fr" as Locale });
state.locale = state.pref === "systeme" ? systemLocale() : state.pref;
if (typeof document !== "undefined") document.documentElement.lang = state.locale;

export function localePref(): LocalePref {
  return state.pref;
}

export function currentLocale(): Locale {
  return state.locale;
}

export function setLocalePref(pref: LocalePref) {
  state.pref = pref;
  state.locale = pref === "systeme" ? systemLocale() : pref;
  try {
    if (pref === "systeme") localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, pref);
  } catch {
    // préférence valable pour cette session seulement
  }
  if (typeof document !== "undefined") document.documentElement.lang = state.locale;
}

/** Traductions anglaises, par le texte français source. Une chaîne absente d'ici s'affiche en
 * français : jamais bloquant, à compléter écran par écran. */
const en: Record<string, string> = {
  // Barre latérale
  Rechercher: "Search",
  "Rechercher dans la bibliothèque": "Search the library",
  Bibliothèque: "Library",
  "Ajouts récents": "Recently added",
  Artistes: "Artists",
  Albums: "Albums",
  Morceaux: "Songs",
  "Titres aimés": "Liked songs",
  "Mon écoute": "My listening",
  Télécharger: "Download",
  Vocaux: "Voices",
  Doublons: "Duplicates",
  Proches: "Friends",
  "Playlists partagées": "Shared playlists",
  "Nouvelle playlist": "New playlist",
  Playlists: "Playlists",
  Réglages: "Settings",
  // Réglages : langue
  Langue: "Language",
  "Langue du système": "System language",
  Français: "French",
  Anglais: "English",
  "L'interface s'affiche dans cette langue ; « Langue du système » la reprend sur celle de votre appareil (français si ce n'est pas l'anglais).":
    "The interface is shown in this language; “System language” follows your device's language (French if it isn't English).",
  // Titres aimés
  Terminer: "Done",
  Sélectionner: "Select",
  "Tout en aléatoire": "Shuffle all",
  Trier: "Sort",
  "Trier les titres aimés": "Sort liked songs",
  "Ordre de la bibliothèque": "Library order",
  "Aucun morceau aimé pour l'instant : touchez le cœur dans le menu d'un morceau (clic droit, ou « ⋯ »).":
    "No liked songs yet: tap the heart in a song's menu (right-click, or “⋯”).",
};

export function t(fr: string): string {
  if (state.locale === "fr") return fr;
  return en[fr] ?? fr;
}
