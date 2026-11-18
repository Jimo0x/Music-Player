// Texte d'une version GitHub : « Quoi de neuf ? » (src/nouveautes.json), puis l'installation.
// Utilisé par la CI à chaque publication ; échoue si la version n'a pas de nouveautés, pour
// qu'aucune version ne sorte sans elles.
//
//   node scripts/release-notes.mjs 0.9.0        (ou app-v0.9.0)

import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const version = (process.argv[2] ?? "").replace(/^app-v|^v/, "").trim();
if (!version) {
  console.error("Usage : node scripts/release-notes.mjs <version>");
  process.exit(2);
}

const releases = JSON.parse(readFileSync(join(root, "src", "nouveautes.json"), "utf8"));
const release = releases.find((r) => r.version === version);
if (!release || !Array.isArray(release.items) || release.items.length === 0) {
  console.error(`Pas de nouveautés pour la version ${version} dans src/nouveautes.json : ajoute-les avant de publier.`);
  process.exit(1);
}

const lines = [
  "## Quoi de neuf ?",
  "",
  ...release.items.map((item) => `- ${item}`),
  "",
  "## Installation",
  "",
  "Installeurs générés automatiquement par la CI pour Windows, macOS, Linux et Android.",
  "Choisis le fichier correspondant à ton système ci-dessous.",
  "Les applications déjà installées (à partir de la 0.4.3 sur ordinateur, 0.4.4 sur Android) proposent la mise à jour d'elles-mêmes.",
  "",
  "Configuration requise :",
  "- Android 8.0 ou plus, Android System WebView à jour (111+) : Music-Player-android-arm64.apk (la plupart des téléphones), Music-Player-android-armv7.apk (anciens téléphones 32 bits) ou Music-Player-android.apk (complet, pour tous)",
  "- Windows 10/11 64 bits : Music.Player_*_x64-setup.exe",
  "- macOS 10.15 ou plus, Mac Apple Silicon : Music.Player_*_aarch64.dmg",
  "- Linux x86_64 (Ubuntu 22.04, Debian 12, Fedora 38 ou plus récents, Arch / CachyOS) : .deb, .rpm ou .AppImage",
];
process.stdout.write(`${lines.join("\n")}\n`);
