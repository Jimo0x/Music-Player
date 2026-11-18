# État du projet

Mis à jour le 18 novembre 2026, à la publication de la **0.9.0**. À tenir à jour à chaque
version : ce qui vient de sortir, ce qui reste, ce qui n'a pas été vérifié.

## Versions récentes

| Version | Date | Contenu principal |
| --- | --- | --- |
| 0.9.0 | 18/11/2026 | Sons Dolby (AC-3/E-AC-3 5.1) lus partout en Rust pur ; Opus en .m4a lu sans ffmpeg (Audiolab) ; commandes de l'écran verrouillé et touches multimédias sur Windows et Linux (MPRIS) ; page Télécharger (lien direct, lien Spotify, recherche) ; boutons de l'import Spotify remis ; propositions en fin de file ; premier scan qui montre son avancement et ne bloque plus ; visualiseur allégé en mode léger ; tri dans Titres aimés ; Mo et temps restant pendant un envoi dans la discussion |
| 0.8.0 | 18/11/2026 | Cartes de paroles « Arcane » (tarot) ; bouton retour Android = MENU en mode baladeur ; noms d'artistes tirés des titres de vidéos nettoyés ; avancement des téléchargements ; CSP activée ; partage Wi-Fi chiffré ; relais sans routes de compte ; actions CI v5 ; styles en variables du thème ; lot playlists (sélection multiple, ajout à plusieurs playlists, tri dans une playlist, ajout depuis l'écran de lecture) ; guide du contributeur |
| 0.7.1 | 12/11/2026 | Play Protect : l'appli n'installe plus d'APK (mise à jour par le navigateur) ; page de playlist virtualisée ; envoi par code par parties ; salle du relais gardée pendant un transfert |
| 0.7.0 | 12/11/2026 | Mode baladeur (téléphone) : boîtier, molette tactile, 5 finitions, paroles sur petit écran |
| 0.6.1 | 12/11/2026 | Moteur de téléchargement testé pour de vrai (outils hors PATH, pièges de spotDL) |
| 0.6.0 | 12/11/2026 | Moteur de téléchargement (file spotDL puis yt-dlp), import Spotify par lien, données du compte ou fichier |

## Ce qui reste à faire

### Avis des utilisateurs encore ouverts

- **Pochettes des playlists de Méryl** (Android 10, Chrome 114) : correction envoyée, jamais
  vérifiée sur son téléphone. Les deux autres points de son avis (saisie de la discussion
  sous le clavier, champ d'avis en pilule) sont corrigés depuis le 11/11.
- Corrigés en 0.9.0, pas encore essayés sur téléphone (avis à marquer « Traité » une fois
  confirmés) :
  - **Audiolab** : WAV (16/24/32 bits, ADPCM) et AAC/.m4a étaient déjà lus sans ffmpeg ; le
    trou était l'Opus rangé dans un .m4a, illisible sur Android. Désormais décodé par
    `opus.rs` (paquets séparés par symphonia, en-tête « dOps » en gros-boutiste).
  - **Tri dans Favoris** : mêmes tris que les playlists, gardés (`mp:tri-favoris`).
  - **Avancement d'un envoi dans la discussion** : %, Mo envoyés sur le total, temps restant
    et débit (calcul commun avec la section Partage : `src/lib/pace.ts`).

### Retours d'un testeur (Redmi 13C, version 0.7.1), traités le 18/11

- **Premier scan bloqué** à l'installation (contourné en installant une ancienne version,
  puis en mettant à jour). Cause exacte non reproduite (pas d'appareil). Corrigé ce qui
  pouvait l'expliquer : l'avancement s'affiche dès le premier scan (« Bienvenue » restait à
  l'écran sans signe de vie) ; un fichier qui fait paniquer la lecture des étiquettes est
  marqué abîmé au lieu de faire tomber le scan ; le drapeau « scan en cours » est toujours
  remis à zéro ; accès à la musique accordé après coup → bibliothèque relue. **À vérifier**
  avec une installation neuve sur un téléphone.
- **Saccades** : le mode léger (automatique sur Mali-G52) ne touchait pas au visualiseur ;
  en mode léger il dessine maintenant 20 images/s, sans halo, à densité 1.
- **« Plus de morceaux » en fin de file** : propositions (réécouter, dans la même veine,
  plus écoutés, titres aimés, playlists, à redécouvrir) dans « À l'écoute » et dans « À
  suivre » (là, ajout à la suite du dernier morceau). Pas dans le mode baladeur.

### Retours du 18/11 au soir

- **Téléchargements spotDL / yt-dlp invisibles** : la page « Importer de Spotify » (qui
  contient la file de téléchargements depuis la 0.6.0) n'avait plus aucun bouton depuis la
  mise en suspens de Spotify le 28/10. Boutons remis (Playlists → « Depuis Spotify »,
  Réglages → « Importer de Spotify… »). Le travail fait avec ChatGPT (étapes 1 à 9 du
  moteur) était bien intégré. Sur téléphone, pas de téléchargement (outils absents).
- **Sons de The Weeknd illisibles sur le Moov Smart Pro** : 73 fichiers (Trilogy, Starboy,
  After Hours, Kiss Land) en Dolby E-AC-3 5.1 dans des .m4a. Désormais décodés partout en Rust
  pur (`dolby.rs`, filmcraft-ac3), au fil de la lecture, ramenés en stéréo au niveau de
  ffmpeg, avec saut. Vérifié sur les 73 : durées exactes, son identique à ffmpeg
  (corrélation 1,0000). **Pas encore essayé sur téléphone.**
- **Écran verrouillé du PC** : commandes multimédias de Windows et de Linux (MPRIS : écran
  verrouillé de KDE Plasma / CachyOS, GNOME) avec souvlaki, `media_controls.rs`, même pont que
  la session média d'Android. Linux vérifié en conteneur (compilation, mises à jour, pas de bus
  → rien tenté). **Pas encore essayé dans la vraie appli** (Windows ni Linux).
- **Page « Télécharger »** (barre latérale, Réglages) : lien direct (yt-dlp, sans recherche),
  lien Spotify (spotDL) ou « Artiste - Titre », playlist de destination, file partagée avec
  l'import Spotify (`FetchJobs.vue`). Lien direct essayé pour de vrai (vidéo YouTube importée).
- **En attente d'un environnement Android** (pas de SDK sur ce PC, rien d'essayable ici) :
  lecteur plein écran sur l'écran verrouillé (option B choisie), téléchargement sur téléphone
  (réadapter yt-dlp dans l'appli, recherche spotDL refaite par nos soins).
- **Ensuite (demande du propriétaire)** : revue de sécurité, puis optimisation poussée, une
  fois toutes les fonctions confirmées sur appareils.

### « Quoi de neuf ? » (après la 0.9.0, pas encore publié)

- Notes de chaque version dans `src/nouveautes.json` : fenêtre au premier lancement après une
  mise à jour (versions sautées comprises), bouton dans Réglages → Mises à jour, notes de la
  version proposée dans la fenêtre / le bandeau de mise à jour. La CI en fait le texte de la
  version GitHub (`scripts/release-notes.mjs`) et refuse une version sans notes.

### Vérifications sur de vrais appareils (rien de tout cela n'a été essayé sur téléphone)

- CSP sur Android (vérifiée sur PC dans la vraie appli : écrans, pochettes, polices, relais).
- Partage Wi-Fi chiffré entre deux téléphones.
- Noms d'artistes nettoyés sur la vraie bibliothèque.
- Mode baladeur : sensation de la molette, clic, vibration, bouton retour, finitions Graphite
  et Pistache (jamais affichées).
- Envoi d'une grosse playlist par code (parties).
- Import Spotify complet dans l'appli, avec la barre d'avancement.
- Mise à jour par le navigateur, et absence d'alerte Play Protect à partir de la 0.8.0.
- Clavier qui ne cache plus le champ de la discussion avec un proche.
- Copie par lots de « Mes appareils » avec beaucoup de sons.
- Appui long (sélection de morceaux) sur un vrai téléphone.

### Play Protect, si l'alerte revient

- Le choix de l'icône de l'appli désactive l'icône d'origine (ressemble à une appli qui se
  cache) : second suspect, laissé en place.
- Démarche que seul le propriétaire peut faire : vérification du développeur auprès de Google,
  ou publication sur le Play Store.

### Mis de côté par le propriétaire

- Spotify (accès retirés, code gardé ; l'import par lien et les téléchargements restent).
- **Plus tard** : remettre (ou proposer en option) l'ancienne mise à jour Android, installée
  par l'appli elle-même (code retiré dans cf04e2a : `AndroidUpdate.download`, `__mpUpdate`,
  service de téléchargement). Décision du 18/11 : on garde le navigateur pour l'instant,
  l'ancienne méthode faisait réagir Play Protect à chaque mise à jour.
- Le lecteur sur l'écran verrouillé n'est plus mis de côté : option B choisie le 18/11.

## Repères techniques récents

- Moteur de téléchargement : `src-tauri/src/fetch/` (file persistante, `drain_queue` testée,
  adaptateurs spotDL et yt-dlp, import vérifié par correspondance, lien Spotify par
  `spotdl save`).
- Partage : `remote.rs` (par code, SPAKE2 + ChaCha20-Poly1305, parties), `nearby.rs` +
  `sealed.rs` (Wi-Fi, chiffré depuis 0.8.0), `sharing.rs` (archives `.mpshare`).
- Relais Cloudflare : `relay/src/index.js` (Durable Objects Salle, Profil, Zone, Avis, Colis).
- Mode baladeur : `src/components/Baladeur.vue`, `src/lib/baladeur.ts`, `src/stores/baladeur.ts`.
- Sélection multiple et playlists : `src/components/SelectionBar.vue`,
  `src/components/AddToPlaylists.vue`, `src/lib/selection.ts`, `src/lib/trackSort.ts`.
