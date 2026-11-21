// « latest.json » de la mise à jour automatique, écrit une seule fois à la fin d'une
// publication (#12). Avant, chaque tâche (Windows, Linux, macOS) le relisait, le complétait et le
// renvoyait en même temps que les autres : à la 0.10.4, macOS a échoué (« Not Found -
// get-a-release-asset ») parce qu'une autre tâche venait de le remplacer. Ici, une seule tâche,
// après toutes les autres, le refait à partir des signatures (.sig) jointes à la version.
//
//   GITHUB_TOKEN=… GITHUB_REPOSITORY=LoickAmg/Music-Player node scripts/updater-json.mjs app-v0.11.1

import { pathToFileURL } from "node:url";

/** Plateformes attendues dans chaque publication (celles de la 0.11.0, publiée complète). */
export const EXPECTED_PLATFORMS = [
  "darwin-aarch64",
  "darwin-aarch64-app",
  "linux-x86_64",
  "linux-x86_64-appimage",
  "linux-x86_64-deb",
  "linux-x86_64-rpm",
  "windows-x86_64",
  "windows-x86_64-msi",
  "windows-x86_64-nsis",
];

/** Processeur lu dans le nom d'un fichier de Tauri (« _amd64 », « _x64 », « .x86_64 », « _aarch64 »). */
function arch(name) {
  if (/(^|[_.-])(aarch64|arm64)([_.-]|$)/i.test(name)) return "aarch64";
  if (/(^|[_.-])(x86_64|amd64|x64)([_.-]|$)/i.test(name)) return "x86_64";
  return null;
}

/**
 * Plateformes servies par un installateur, d'après son nom. La première est aussi celle par
 * défaut de son système (même choix que tauri-action : AppImage, .app, .msi).
 */
export function platformsOf(name) {
  const cpu = arch(name);
  if (!cpu) return [];
  if (name.endsWith(".AppImage")) return [`linux-${cpu}`, `linux-${cpu}-appimage`];
  if (name.endsWith(".deb")) return [`linux-${cpu}-deb`];
  if (name.endsWith(".rpm")) return [`linux-${cpu}-rpm`];
  if (name.endsWith(".app.tar.gz")) return [`darwin-${cpu}`, `darwin-${cpu}-app`];
  if (name.endsWith(".msi")) return [`windows-${cpu}`, `windows-${cpu}-msi`];
  if (name.endsWith("-setup.exe")) return [`windows-${cpu}-nsis`];
  return [];
}

/**
 * Contenu de latest.json. `assets` : fichiers de la version ({ name, url }) ; `signatures` :
 * nom de l'installateur → contenu de son .sig. Renvoie aussi les plateformes manquantes.
 */
export function buildUpdaterJson({ version, notes, pubDate, assets, signatures }) {
  const platforms = {};
  for (const asset of assets) {
    const signature = signatures[asset.name];
    if (!signature) continue;
    for (const key of platformsOf(asset.name)) platforms[key] = { signature: signature.trim(), url: asset.url };
  }
  const sorted = Object.fromEntries(Object.entries(platforms).sort(([a], [b]) => a.localeCompare(b)));
  const missing = EXPECTED_PLATFORMS.filter((key) => !sorted[key]);
  return { json: { version, notes, pub_date: pubDate, platforms: sorted }, missing };
}

async function main() {
  const tag = process.argv[2] ?? process.env.GITHUB_REF_NAME ?? "";
  const repo = process.env.GITHUB_REPOSITORY;
  const token = process.env.GITHUB_TOKEN;
  if (!tag || !repo || !token) {
    console.error("Usage : GITHUB_TOKEN=… GITHUB_REPOSITORY=propriétaire/dépôt node scripts/updater-json.mjs <tag>");
    process.exit(2);
  }
  const api = (path, init = {}) =>
    fetch(path.startsWith("http") ? path : `https://api.github.com/repos/${repo}${path}`, {
      ...init,
      headers: { authorization: `Bearer ${token}`, "x-github-api-version": "2022-11-28", ...(init.headers ?? {}) },
    });

  const response = await api(`/releases/tags/${encodeURIComponent(tag)}`);
  if (!response.ok) throw new Error(`Version ${tag} introuvable (${response.status}).`);
  const release = await response.json();

  const assets = release.assets.map((a) => ({ id: a.id, name: a.name, url: a.browser_download_url }));
  const signatures = {};
  for (const sig of assets.filter((a) => a.name.endsWith(".sig"))) {
    const r = await api(`/releases/assets/${sig.id}`, { headers: { accept: "application/octet-stream" } });
    if (!r.ok) throw new Error(`Signature ${sig.name} illisible (${r.status}).`);
    signatures[sig.name.slice(0, -".sig".length)] = await r.text();
  }

  const { json, missing } = buildUpdaterJson({
    version: tag.replace(/^app-v|^v/, ""),
    notes: release.body ?? "",
    pubDate: new Date().toISOString(),
    assets: assets.filter((a) => !a.name.endsWith(".sig")),
    signatures,
  });
  if (missing.length) {
    // Une plateforme absente : les applis de ce système ne verraient pas la mise à jour.
    console.log(`::error::latest.json incomplet, plateformes absentes : ${missing.join(", ")}`);
    process.exit(1);
  }

  // Remplace l'ancien latest.json s'il existe (tauri-action ne l'écrit plus).
  for (const old of assets.filter((a) => a.name === "latest.json")) {
    const r = await api(`/releases/assets/${old.id}`, { method: "DELETE" });
    if (!r.ok && r.status !== 404) throw new Error(`Ancien latest.json non retiré (${r.status}).`);
  }
  const upload = release.upload_url.replace(/\{.*\}$/, "");
  const r = await api(`${upload}?name=latest.json`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(json, null, 2),
  });
  if (!r.ok) throw new Error(`Envoi de latest.json refusé (${r.status}) : ${await r.text()}`);
  console.log(`latest.json : ${Object.keys(json.platforms).length} plateformes (${tag}).`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  main().catch((e) => {
    console.log(`::error::${e instanceof Error ? e.message : String(e)}`);
    process.exit(1);
  });
}
