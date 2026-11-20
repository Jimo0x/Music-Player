# Contribuer à Music Player

Bienvenue ! Ce guide explique comment travailler sur le projet sans casser ce qui marche ni
la façon dont il avance. Lis-le en entier avant ta première contribution. En cas de doute,
**demande avant de coder** : c'est toujours plus rapide qu'une demande de fusion refusée.

À lire aussi :

- [README.md](README.md) : ce que fait l'appli, sa structure, comment la lancer.
- [DESIGN.md](DESIGN.md) : les couleurs, espacements, arrondis et composants de l'interface.

## Comment le projet avance

- Le propriétaire du projet, Mahouna, décide des fonctionnalités, de leur ordre et de leur
  forme. Une idée se propose d'abord dans une issue GitHub ; on code une fois qu'elle est
  validée.
- **Toute contribution est relue et vérifiée avant d'être intégrée**, puis fusionnée par le
  propriétaire. Personne ne pousse directement sur `main`.
- Les versions, les publications et le relais en ligne sont réservés au propriétaire (voir
  « Ce que tu ne fais pas »).

## Les règles d'or

1. **Ne jamais réécrire un code qui marche.** Avant d'écrire quoi que ce soit, cherche si le
   projet sait déjà le faire (un composant, une fonction, un style) et pars de là. On réutilise
   et on adapte, on ne refait pas « en mieux » à côté.
2. **Rien de nouveau tant que l'existant n'est pas fonctionnel.** Un bogue connu passe avant
   une nouvelle fonctionnalité.
3. **Une demande de fusion = un sujet.** Petite, lisible, qui ne mélange pas correctif,
   refonte et nouveauté.
4. **Tout doit passer les vérifications** (ci-dessous) avant d'ouvrir la demande de fusion.
5. **Pas de donnée personnelle** dans le code, les tests, les journaux ou les captures :
   pas de vrais noms, e-mails, numéros, chemins de ton ordinateur, ni de captures montrant
   autre chose que l'appli.
6. **Pas de secret** : aucune clé, aucun mot de passe, aucun jeton, jamais, même « pour
   tester ».
7. **Aucune attribution à un outil dans l'historique.** Tu peux t'aider de l'outil de ton
   choix, mais tes commits n'en portent aucune trace : pas de ligne `Co-Authored-By:`, pas de
   « Generated with … », pas de lien vers une session (`…-Session: https://…`). GitHub ajoute
   chaque co-auteur nommé à la liste des contributeurs du dépôt. Une personne qui a contribué
   ouvre sa propre demande de fusion. La CI refuse un commit qui porte une de ces lignes
   (tâche « commits »). Si ton outil les ajoute tout seul, coupe ce réglage, ou retire-les avant
   de pousser : `git rebase -i main`, puis `reword` sur chaque commit concerné.

## Mise en place

Prérequis : Node 20 ou plus, Rust stable (`rustup`), et les dépendances de Tauri pour ton
système ([doc officielle](https://tauri.app/start/prerequisites/)).

```bash
git clone https://github.com/LoickAmg/Music-Player.git
cd Music-Player
npm install
npm run tauri dev   # l'appli complète (Rust + interface)
npm run dev         # l'interface seule dans un navigateur, en mode démo (rien ne joue)
```

Le mode démo (`src/lib/tauriMock.ts`) simule le moteur Rust : pratique pour travailler
l'interface, mais il ne remplace pas un essai dans la vraie appli.

## Le flux de travail

1. **Une issue validée** décrit ce qu'on fait et pourquoi.
2. **Une branche par sujet**, partie de `main` à jour :
   `correctif/pochettes-chrome-114`, `fonction/tri-playlist`, `doc/guide-relais`…
3. **Des commits petits**, en français, qui disent ce qui change et pourquoi
   (« Playlists : tri par date d'ajout, gardé d'une visite à l'autre »).
4. **Les vérifications en local** (section suivante), toutes vertes.
5. **La demande de fusion** vers `main`, en remplissant le modèle proposé : ce qui change,
   comment c'est testé, captures avant/après pour l'interface (PC **et** téléphone).
6. **La relecture** : on te demandera peut-être des changements. C'est normal et ça fait
   partie du travail ; réponds dans la discussion et pousse les corrections sur la même
   branche. Chaque relecture te **mentionne (@ton-pseudo)** et se lit dans l'onglet
   **« Conversation »** de la demande (pas dans « Files changed »). Pour ne rien manquer :
   - sur la page du dépôt, **Watch → Participating and @mentions** (ou « All Activity ») ;
   - dans GitHub, **Settings → Notifications** : coche **Email** (et/ou **Web and Mobile**)
     pour « Participating, @mentions and custom », et vérifie l'adresse e-mail choisie ;
   - l'appli **GitHub Mobile** prévient aussi sur le téléphone.
7. **La fusion** est faite par le propriétaire, une fois la relecture terminée et la CI verte.

## Les vérifications (obligatoires)

Depuis la racine :

```bash
npm run lint                 # ESLint
npx vue-tsc --noEmit         # types TypeScript
npm test                     # tests de l'interface (Vitest)
```

Dans `src-tauri/` :

```bash
cargo clippy --all-targets -- -D warnings   # aucun avertissement accepté
cargo test -- --test-threads=1              # tests Rust, un à la fois comme la CI
```

La CI (GitHub Actions) relance tout cela sur chaque demande de fusion. Une CI rouge bloque la
fusion.

## Conventions

### En général

- **Le français partout** : interface, commentaires, messages d'erreur, commits. Les messages
  affichés disent ce qui se passe et quoi faire, sans jargon. Suis le ton de l'écran voisin.
- Les commentaires expliquent **pourquoi**, pas ce que le code fait déjà.
- Toute logique nouvelle a **son test** (Vitest ou `cargo test`). Un test qui a besoin du
  réseau ou d'un outil installé est marqué `#[ignore]` (Rust) et ne tourne pas dans la CI.

### Interface (Vue 3 + TypeScript + Pinia)

- Composants en `<script setup lang="ts">`, état partagé dans les stores Pinia (`src/stores`).
- **Tout appel au moteur Rust passe par `src/lib/api.ts`** ; les types vont dans
  `src/lib/types.ts`. Une nouvelle commande Rust a aussi sa réponse factice dans
  `src/lib/tauriMock.ts`, sinon le mode démo casse.
- **Styles : les jetons de [DESIGN.md](DESIGN.md)** (`var(--radius)`, `var(--space-3)`,
  `var(--accent)`…), jamais de couleur ou de valeur en dur quand un jeton existe. L'appli a
  des thèmes clairs et sombres : vérifie les deux.
- **Téléphone et ordinateur** : chaque écran doit marcher sur les deux (`ui.isMobile`). Teste
  au moins en largeur téléphone (375 px) et ordinateur.
- **Listes longues** (morceaux, playlists, artistes) : `VirtualList`, jamais un `v-for` sur
  des milliers d'éléments.
- Accessibilité : un bouton fait d'une icône a un `aria-label`, on peut tout faire au clavier
  sur ordinateur.
- La CSP de l'appli interdit les scripts extérieurs et `eval` : pas de bibliothèque chargée
  depuis un CDN, pas de `v-html` avec du contenu venu de l'extérieur.

### Moteur (Rust + Tauri)

- **Jamais deux verrous tenus en même temps.** Prends un verrou, copie ce qu'il te faut,
  relâche-le, puis prends le suivant. Utilise `.locked()` (trait `crate::state::Locked`).
- **Pas de `unwrap()` ni d'`expect()` hors des tests** : une erreur remonte à l'interface sous
  forme de message en français.
- Une commande longue (réseau, disque, programme externe) s'écrit
  `#[tauri::command(async)]` pour ne pas figer l'appli.
- Un programme externe se lance avec `crate::bounded::run` (délai maximal, arrêt possible),
  et ses arguments se passent un par un, jamais par un shell.
- Un chemin venu de l'extérieur (nom de fichier reçu, titre de morceau) est nettoyé avant
  d'écrire sur le disque (voir `fetch/source.rs`).

## Ce que tu ne fais pas (sans accord explicite)

Ces zones touchent la sécurité, la publication ou les appareils des utilisateurs : on en
parle d'abord dans une issue.

- **Versions, tags `app-v*`, publications** : réservés au propriétaire.
- **Le relais en ligne** (`relay/`) : il est en production, utilisé par tous les appareils.
- **`scripts/android-patch.mjs`** : permissions Android, services, installation. Une
  permission de trop et Play Protect signale l'appli comme dangereuse.
- **`src-tauri/tauri.conf.json`** (sécurité, CSP, protocole des fichiers), la signature, la
  mise à jour automatique, **`.github/workflows/`**.
- **Les formats de fichiers enregistrés** (playlists, session, file de téléchargements,
  sauvegardes) : un changement doit rester lisible par les versions déjà installées.
- **Les dépendances** : pas de nouvelle bibliothèque sans en parler.

## Ce qui fait refuser une demande de fusion

- La CI ne passe pas, ou des vérifications ont été sautées.
- Du code existant réécrit au lieu d'être réutilisé, ou des changements sans rapport avec le
  sujet.
- Une fonctionnalité non validée, ou différente de ce qui était convenu.
- Un écran cassé sur téléphone, en thème clair ou en thème sombre.
- Une donnée personnelle, un secret, ou une zone protégée modifiée sans accord.
- Un commit avec une ligne `Co-Authored-By:`, « Generated with … » ou un lien de session.
- Pas de test pour la logique ajoutée.

Merci de ton aide, et bienvenue dans le projet !
