# Music Player — consignes pour reprendre le travail

À lire au début de chaque session. L'état du projet, les versions publiées et ce qui reste à
faire sont dans [docs/etat-du-projet.md](docs/etat-du-projet.md) ; les règles pour les
contributeurs humains dans [CONTRIBUTING.md](CONTRIBUTING.md) ; le système visuel dans
[DESIGN.md](DESIGN.md).

## Avec le propriétaire (Mahouna)

- **Toujours répondre en français.** Interface, commentaires, commits : en français aussi.
- **Ne jamais réécrire un code qui marche** : chercher l'existant, le réutiliser, l'adapter.
  Rien de nouveau tant que l'existant n'est pas entièrement fonctionnel.
- **Commits en local ; pousser, publier une version ou déployer le relais seulement à sa
  demande** (chaque fois, pas d'accord permanent).
- **Versions** : un lot de fonctionnalités = version mineure (0.8.0 → 0.9.0) ; correctifs ou
  petites additions = correctif (0.8.1). Choisir soi-même le numéro et le dire.
- **Contributeur externe** : toute demande de fusion est relue et vérifiée avant intégration
  (récupérer la branche, relancer toutes les vérifications, essayer dans l'appli, rendre un
  avis). `main` est protégée (règle « Protection de main » : demande de fusion, relecture des
  Code Owners, CI `test` verte ; l'administrateur peut contourner).
- **Captures** : seulement la fenêtre de l'appli (PrintWindow), jamais l'écran entier, sans lui
  prendre le focus. Ne jamais installer Microsoft Edge sur ce PC (WebView2 seul).
- Usage surtout personnel ; le propriétaire n'est pas en France.

## Vérifications avant chaque commit

```bash
npm run lint && npx vue-tsc --noEmit && npx vitest run
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test -- --test-threads=1
```

La CI (`.github/workflows/ci.yml`) lance les tests sur **windows-latest** ; les machines sans
carte son plantaient si un test ouvrait la sortie audio : en test, `AppState` utilise
`AudioHandle::silent()`. En cas d'échec, `.github/scripts/diagnostic-tests.sh` publie la fin
du journal en annotations lisibles sans connexion (API publique `check-runs/<id>/annotations`).

Tests réels désactivés par défaut (réseau, outils installés) :
`cargo test --lib -- --ignored --nocapture lit_un_vrai_lien` (lien Spotify),
`vrai_telechargement` (spotDL, yt-dlp), `archive_crosses_the_live_relay` (relais en ligne).

## Règles du code

- **Rust** : jamais deux verrous tenus en même temps ; `.locked()` (trait
  `crate::state::Locked`) ; pas d'`unwrap`/`expect` hors tests ; erreurs en français pour
  l'interface ; commandes longues en `#[tauri::command(async)]` ; programmes externes par
  `crate::bounded::run` / `run_watching` (délai, arrêt de l'arbre de processus).
- **Interface** : appels Rust uniquement par `src/lib/api.ts`, types dans `src/lib/types.ts`,
  réponse factice de toute nouvelle commande dans `src/lib/tauriMock.ts` ; jetons de
  `DESIGN.md` (les thèmes redéfinissent `--radius`) ; listes longues en `VirtualList` ;
  chaque écran vérifié sur téléphone (`ui.isMobile`) et ordinateur.
- **Zones sensibles** : `scripts/android-patch.mjs` (aucune permission d'installation : Play
  Protect), `src-tauri/tauri.conf.json` (CSP active), `relay/` (production), formats des
  fichiers enregistrés (rester lisibles par les versions installées).

## Publier une version

1. Monter la version dans **5 fichiers** : `package.json`, `src-tauri/tauri.conf.json`,
   `src-tauri/Cargo.toml` (1re occurrence), `src-tauri/Cargo.lock` (entrée du paquet
   `music-player` seulement), `package-lock.json` (**les 2 entrées racine seulement** : un
   remplacement global a déjà modifié une dépendance). Le commit doit faire 6 lignes.
2. `git commit -m "chore: version X.Y.Z [android]"`, `git tag app-vX.Y.Z`, pousser `main`
   puis le tag. La CI construit Windows, macOS, Linux et Android et crée la version GitHub
   (17 fichiers attendus, `latest.json` compris).
3. Suivre l'exécution : API publique
   `https://api.github.com/repos/LoickAmg/Music-Player/actions/runs?branch=app-vX.Y.Z`.

## Déployer le relais

Depuis `relay/` : `npx wrangler@4.160.0 deploy` (compte Cloudflare déjà connecté sur ce PC).
Vérifier ensuite : `curl https://music-player-relais.loickamg.workers.dev/` puis
`cargo test --lib -- --ignored archive_crosses_the_live_relay`.

## Outils de vérification utiles

- **Mode démo navigateur** : `npm run dev` (port 1420), IPC simulé par `tauriMock.ts`.
  Quand le panneau du navigateur est caché, les transitions de page se figent : une capture
  d'écran les débloque.
- **Vraie appli pilotée sans toucher à la souris** : `npx tauri build --debug --no-bundle`, puis
  lancer `src-tauri/target/debug/music-player.exe` avec
  `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9333` et lire la console ou
  évaluer du JavaScript par le protocole DevTools (l'appli installée doit être fermée : instance
  unique).
- **Avis des utilisateurs** : dans l'appli de bureau, « Avis reçus » (administrateur).
- **Outils de téléchargement de ce PC** : spotDL et yt-dlp dans Anaconda
  (`C:\ProgramData\anaconda3\Scripts`), FFmpeg et Deno dans `~/.spotdl` ; l'appli les trouve
  hors du PATH (`fetch::common::tool_locations`).

## Pièges connus

- spotDL 4.6 : refuse `--` ; la recherche doit suivre l'opération (`download <recherche>
  --options`) ; il retire le point des dossiers de `--output` (le lancer dans le dossier de la
  tâche avec un modèle relatif) ; YouTube Music peut bloquer → `--audio youtube-music youtube`.
- yt-dlp : `--print` le rend silencieux, `--progress` remet l'avancement.
- Relais : une salle d'envoi par code reste ouverte tant que les deux appareils y sont (12 h au
  plus) ; une grosse playlist part par parties de 150 Mo / 40 morceaux.
