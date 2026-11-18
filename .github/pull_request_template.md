## Ce qui change

<!-- En quelques phrases : quoi, et pourquoi. Lien vers l'issue validée : « Issue #… » -->

## Comment c'est testé

- [ ] `npm run lint`, `npx vue-tsc --noEmit`, `npm test` : verts
- [ ] `cargo clippy --all-targets -- -D warnings`, `cargo test -- --test-threads=1` (dans `src-tauri/`) : verts
- [ ] Nouveau test pour la logique ajoutée (ou : pourquoi ce n'est pas possible)
- [ ] Essayé dans la vraie appli (`npm run tauri dev`), pas seulement en mode démo

## Interface (si elle change)

- [ ] Captures avant / après, sur ordinateur **et** en largeur téléphone
- [ ] Thème clair et thème sombre vérifiés
- [ ] Jetons de DESIGN.md utilisés, pas de valeurs en dur

## À vérifier avant de demander la relecture

- [ ] Un seul sujet ; aucun code qui marchait n'a été réécrit
- [ ] Aucune donnée personnelle ni aucun secret (code, tests, captures)
- [ ] Aucune zone protégée modifiée sans accord (`relay/`, `scripts/android-patch.mjs`, `tauri.conf.json`, `.github/workflows/`, formats de fichiers, dépendances)
