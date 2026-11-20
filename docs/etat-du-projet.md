# État du projet

Mis à jour le **20 novembre 2026** : **0.10.1** publiée, un travail en cours sur une branche
locale (voir ci-dessous). À tenir à jour à chaque version : ce qui vient de sortir, ce qui
reste, ce qui n'a pas été vérifié.

## En cours : à reprendre en premier

### Retour à l'ancienne mise à jour Android (décision du propriétaire, 20/11)

Pourquoi : la mise à jour par le navigateur bloque parfois et peut abîmer le fichier. Le 19/11,
la 0.10.0 refusait de s'installer sur le Pixel 6 Pro : le journal d'Android disait « SHA-256
digest of contents did not verify » ; l'APK téléchargé par le navigateur avait la bonne taille
mais un contenu différent de l'APK publié (empreintes `2addc7b3…` contre `1b6f1cf0…`). L'APK
publié était sain (installé ensuite par câble, `adb install -r`, données gardées).

Fait (branche **locale** `wip/mise-a-jour-dans-l-appli`, commit `d5b810c`, **non poussée**) :
annulation exacte du retrait `cf04e2a` (`git apply -R`) sur `scripts/android-patch.mjs`,
`src/stores/updater.ts` et `src/lib/androidMedia.ts`. Le bloc `UpdateBridge.kt` n'avait pas
bougé depuis le retrait (vérifié). Revient : permission `REQUEST_INSTALL_PACKAGES`,
`UpdateFileProvider` + `res/xml/update_paths.xml`, `UpdateNowReceiver`,
`UpdateDownloadedReceiver`, téléchargement par le service d'Android avec nouveaux essais,
notification « Prête à installer », `AndroidUpdate.download(url, version)` et `__mpUpdate`.
Le bouton « Télécharger avec le navigateur » reste en secours. Lint, types et tests verts.

Reste à faire, dans l'ordre :
1. **Vérifier l'empreinte SHA-256 de l'APK avant l'installation.** L'API des versions de
   GitHub donne `"digest": "sha256:…"` pour chaque fichier (vérifié sur la 0.10.1). Ajouter un
   champ `sha256` à `UpdateInfo` (`src-tauri/src/updater.rs`, `newest_apk_for` lit
   `asset["digest"]`), le passer à `AndroidUpdate.download(url, version, sha256)`, et dans
   `UpdateBridge.watch` (après `STATUS_SUCCESSFUL`) comme dans `UpdateDownloadedReceiver`,
   comparer avant `launchInstaller` / `notifyReady` ; si l'empreinte diffère : supprimer le
   fichier, un nouvel essai, puis un message clair. Test Rust sur la lecture du champ.
2. **Essai réel sur l'émulateur** : compiler en local avec une version plus basse que la
   dernière publiée, signée avec la **clé de version** (`%USERPROFILE%\.music-player-android\`
   : `release.p12` et ses secrets en fichiers texte, voir le LISEZ-MOI du dossier), l'installer,
   laisser l'appli proposer la version GitHub, la télécharger et l'installer d'elle-même.
3. Nouveautés (`src/nouveautes.json`) puis publication (correctif 0.10.2).
4. Risque connu et accepté : Play Protect réagissait à cette méthode (0.5.16 à 0.7.0). Si
   l'alerte revient : voir « Play Protect » plus bas.

## Versions récentes

| Version | Date | Contenu principal |
| --- | --- | --- |
| 0.10.1 | 20/11/2026 | Trois corrections du contributeur : liens de playlist ou d'album = un morceau par tâche, un son partagé depuis un Mix YouTube reste un seul son (`--no-playlist`, `noplaylist`) ; recherche avec le titre seul ; liens Spotify courts (explication) et anciens `play.spotify.com` ; synchronisation des appareils liés qui dit pourquoi elle attend et reprend au retour dans l'appli |
| 0.10.0 | 19/11/2026 | Téléchargement sur téléphone (yt-dlp intégré : Python 3.11 par Chaquopy 17.2.0) ; liens Spotify lus sans spotDL (page « embed », 100 morceaux au plus) ; recherche à la façon de spotDL (5 candidats, notre correspondance choisit) ; « Quoi de neuf ? » ; listes déroulantes lisibles. APK plus gros : arm64 58 Mo, armv7 43 Mo, complet 91 Mo |
| 0.9.0 | 18/11/2026 | Sons Dolby (AC-3/E-AC-3 5.1) lus partout en Rust pur ; Opus en .m4a lu sans ffmpeg (Audiolab) ; écran verrouillé et touches multimédias sur Windows et Linux (MPRIS) ; page Télécharger ; boutons de l'import Spotify remis ; propositions en fin de file ; premier scan qui montre son avancement et ne bloque plus ; visualiseur allégé en mode léger ; tri dans Titres aimés ; Mo et temps restant pendant un envoi dans la discussion |
| 0.8.0 | 18/11/2026 | Cartes de paroles « Arcane » (tarot) ; bouton retour Android = MENU en mode baladeur ; noms d'artistes tirés des titres de vidéos nettoyés ; avancement des téléchargements ; CSP activée ; partage Wi-Fi chiffré ; relais sans routes de compte ; actions CI v5 ; styles en variables du thème ; lot playlists ; guide du contributeur |
| 0.7.1 | 12/11/2026 | Play Protect : l'appli n'installe plus d'APK (mise à jour par le navigateur) ; page de playlist virtualisée ; envoi par code par parties ; salle du relais gardée pendant un transfert |
| 0.7.0 | 12/11/2026 | Mode baladeur (téléphone) : boîtier, molette tactile, 5 finitions, paroles sur petit écran |
| 0.6.1 | 12/11/2026 | Moteur de téléchargement testé pour de vrai (outils hors PATH, pièges de spotDL) |
| 0.6.0 | 12/11/2026 | Moteur de téléchargement (file spotDL puis yt-dlp), import Spotify par lien, données du compte ou fichier |

## Avis des utilisateurs (8 sur le relais au 20/11)

Lus avec le code administrateur (`%USERPROFILE%\.music-player-avis\code-admin.txt`) :
`POST https://music-player-relais.loickamg.workers.dev/v1/avis/liste` avec `{"admin": code}` ;
`/v1/avis/supprimer` avec `{"admin", "id"}` ; `/v1/avis/image` avec `{"admin", "id", "n"}`.
Ne supprimer qu'avec l'accord du propriétaire.

| Avis (début de l'id) | Sujet | État |
| --- | --- | --- |
| `f49992b3` (Méryl, 18/11) | « Tout est réglé » | Confirme `bf3c947a` et `5422dd57` : **ces 3 avis peuvent être supprimés** (accord du propriétaire demandé le 20/11, pas encore donné) |
| `bf3c947a`, `5422dd57` (Méryl) | Pochettes des playlists, saisie de la discussion, champ d'avis | Réglés (confirmés par `f49992b3`) |
| `0f6f48d3` (19/11, 0.9.0) | Envoyer plusieurs sons d'un coup ; dans la discussion, « + » doit permettre de choisir plusieurs sons et plusieurs playlists | **À faire** |
| `8a2b18bc` | Tri dans Favoris et les playlists | Fait (0.8.0 playlists, 0.9.0 Favoris), confirmation du testeur attendue |
| `a6243c4d` | %, Mo et temps restant pendant un envoi | Fait en 0.9.0, confirmation attendue |
| `86f96b44`, `23757f22` | Enregistrements Audiolab .wav/.m4a illisibles | Fait en 0.9.0 (le trou était l'Opus dans un .m4a), confirmation attendue |

## À faire ensuite (ordre proposé au propriétaire le 20/11, pas encore tranché)

1. **Accès rapides sur téléphone** : Télécharger n'est accessible que par Réglages →
   « Télécharger… » (sur PC : onglet de la barre latérale) ; le mode baladeur seulement par
   Réglages → Personnalisation. Proposé : cartes d'accès rapide sur l'Accueil (Télécharger,
   Mode baladeur, Importer de Spotify, Vocaux) et un bouton baladeur dans « À l'écoute ».
2. **Notifications comme WhatsApp, appli fermée** (messages, sons, demandes des proches).
   Aujourd'hui : tâche de fond `ProchesCheckJob` toutes les 15 min au mieux (Android la
   repousse souvent de plusieurs heures) et option « Notifications instantanées » (service
   `ProchesService`, vérification chaque minute, désactivée par défaut, souvent arrêtée par
   les économiseurs de batterie). Proposé : notifications poussées par le relais via Firebase
   Cloud Messaging (projet Firebase gratuit à créer par le propriétaire, clé de service en
   secret du relais, `google-services.json` dans la construction), et la vérification actuelle
   gardée en secours pour les téléphones sans services Google (Huawei de Méryl).
3. **Envoi groupé** de sons et de playlists (avis `0f6f48d3`).
4. **Lecteur plein écran sur l'écran verrouillé Android, façon Apple Music** (option B
   choisie le 18/11) : activité affichée par-dessus l'écran verrouillé ; pour qu'elle s'ouvre
   seule à l'allumage de l'écran, il faut l'autorisation « afficher par-dessus les autres
   applis » (risque Play Protect à surveiller). Pas commencé.
5. **Vérifications sur de vrais appareils** (liste plus bas).
6. **Traduction** Français / English / langue du système, choisie par l'utilisateur (décision
   du 19/11 : plus tard) : environ 1 200 textes (870 dans l'interface, 340 messages du moteur
   Rust), 75 écrans.
7. **Revue de sécurité, puis optimisation poussée**, une fois les fonctions confirmées.

### Points en attente d'une décision du propriétaire

- Suppression des 3 avis de Méryl (voir le tableau des avis).
- **Fichiers d'outillage publiés dans le dépôt**, contraire à sa règle globale (fichiers
  d'outillage exclus par `.git/info/exclude`, aucun outil nommé dans un fichier versionné) :
  le fichier de consignes à la racine (suivi depuis `6fa2185`, 18/11, bien que listé dans
  `.git/info/exclude`), le lien du README vers lui (ligne « Pour reprendre le travail ») et le
  document de prompt dans `docs/`. Proposé : retirer le fichier du suivi
  (`git rm --cached`, il reste sur le disque), retirer le lien, retirer le document. Rien fait
  sans son accord (suppression de fichiers du dépôt).

## Vérifications sur de vrais appareils

Rien de cette liste n'a été essayé sur un vrai téléphone ou une vraie machine, sauf mention.

- **Pixel 6 Pro (Android 17)** : 0.10.0 installée par câble le 19/11 ; téléchargement sur
  téléphone pas encore essayé par le propriétaire.
- **Moov Smart Pro** (0.5.15 installée le 18/11) : sons Dolby de The Weeknd (Trilogy,
  Starboy, After Hours, Kiss Land : 73 fichiers en E-AC-3 5.1) après mise à jour.
- **Écran verrouillé du PC** dans la vraie appli (Windows) ; **Linux / CachyOS** (AppImage,
  MPRIS vérifié seulement en conteneur).
- Premier scan après une installation neuve ; propositions de fin de file ; visualiseur allégé.
- CSP sur Android (vérifiée sur PC dans la vraie appli).
- Partage Wi-Fi chiffré entre deux téléphones ; envoi d'une grosse playlist par code.
- Noms d'artistes nettoyés sur la vraie bibliothèque.
- Mode baladeur : molette, clic, vibration, bouton retour, finitions Graphite et Pistache.
- Import Spotify complet avec la barre d'avancement ; copie par lots de « Mes appareils ».
- Clavier qui ne cache plus le champ de la discussion ; appui long (sélection de morceaux).
- Synchronisation des appareils liés (0.10.1) avec deux vrais appareils.

## Play Protect, si l'alerte revient

- Le choix de l'icône de l'appli désactive l'icône d'origine (ressemble à une appli qui se
  cache) : second suspect, laissé en place.
- Le retour de l'installation par l'appli (`REQUEST_INSTALL_PACKAGES`) est le premier suspect.
- Démarche que seul le propriétaire peut faire : vérification du développeur auprès de Google,
  ou publication sur le Play Store.
- Essai du 19/11 sur l'émulateur Android 17 avec Play Store : analyse « This app looks safe »
  pour la 0.10.0 (installation depuis Fichiers, mise à jour réussie).

## Contributeur

**Jimo0x** (fork `Jimo0x/Music-Player`, branches `correctif/…`). Demandes #1, #2 et #3
relues, essayées, approuvées et fusionnées le 20/11 (merge commits `c127aa4`, `41e89cb`,
`aaf7075`) ; #2 d'abord renvoyée pour correction (son d'un Mix YouTube → 500 téléchargements),
corrigée en `09cfaaa` et `92d0536`. #4 et #5 : demandes de test de la CI, fermées par lui sans
fusion. Le guide (`CONTRIBUTING.md`) explique où lire les relectures et comment régler ses
notifications GitHub. Toute relecture commence par **@Jimo0x** (sinon il n'est pas toujours
prévenu).

## Fait les 18-20/11 (détail utile)

- **Premier scan bloqué** (testeur, Redmi 13C, 0.7.1) : cause exacte non reproduite ; corrigé
  ce qui pouvait l'expliquer (avancement affiché dès le premier scan, fichier piégé marqué
  abîmé au lieu de faire tomber le scan, drapeau « scan en cours » toujours remis à zéro,
  bibliothèque relue quand l'accès à la musique est accordé après coup).
- **Saccades** : en mode léger, le visualiseur dessine 20 images/s, sans halo, à densité 1.
- **Fin de file** : propositions (réécouter, dans la même veine, plus écoutés, titres aimés,
  playlists, à redécouvrir) dans « À l'écoute » et « À suivre ». Pas dans le mode baladeur.
- **Téléchargements invisibles** : la page « Importer de Spotify » n'avait plus de bouton
  depuis la mise en suspens de Spotify (28/10) ; boutons remis. Le moteur de téléchargement
  (étapes 1 à 9 écrites avec un assistant extérieur puis relues) était bien intégré.
- **Dolby** : `dolby.rs` (filmcraft-ac3), décodage au fil de la lecture, stéréo au niveau de
  ffmpeg, saut ; vérifié sur les 73 fichiers (durées exactes, corrélation 1,0000 avec ffmpeg).
- **Téléchargement sur téléphone** : essayé de bout en bout sur l'émulateur Android 14
  (« The Weeknd - Blinding Lights » trouvé, téléchargé en 15 s, rangé dans
  « Musique/Music Player/Téléchargés » en .m4a, lu).

## Mis de côté par le propriétaire

- Spotify (accès retirés, code gardé ; l'import par lien et les téléchargements restent).

## Repères techniques récents

- Moteur de téléchargement : `src-tauri/src/fetch/` : file persistante (`queue.rs`,
  `drain_queue` testée), adaptateurs spotDL et yt-dlp (`spotdl.rs`, `ytdlp.rs` : 5 candidats,
  `pick_candidate`, `read_media_link` pour les liens de playlist), correspondance
  (`matching.rs`), import (`import.rs` : extension réelle gardée, dossier de travail à part,
  copie entre espaces de stockage), liens Spotify (`link.rs`, `spotify_embed.rs`).
- Téléphone : `fetch/android.rs` (pont JNI vers `YtDlpBridge.kt`), `fetch/ytdlp_zip.rs`
  (yt-dlp téléchargé, SHA-256 vérifié, contrôle quotidien),
  `src-tauri/android/python/mp_ytdlp.py` (copié par `android-patch.mjs`, règle R8
  `proguard-ytdlp.pro`), morceaux rangés par `sharing::downloads_dir`.
- Sons : `dolby.rs`, `opus.rs` (Ogg et MP4), `audio.rs::open_decoder`.
- Écran verrouillé et touches multimédias (PC) : `media_controls.rs` (souvlaki : Windows SMTC,
  Linux MPRIS par zbus), pont dans `src/lib/androidMedia.ts`.
- « Quoi de neuf ? » : `src/nouveautes.json`, `src/lib/whatsNew.ts`,
  `src/components/WhatsNew.vue`, `scripts/release-notes.mjs` (texte des versions GitHub).
- Interface : `DownloadView.vue`, `FetchJobs.vue`, `EndOfQueue.vue` + `src/lib/suggestions.ts`,
  `src/lib/pace.ts` (débit et temps restant).
- Partage : `remote.rs` (par code, SPAKE2 + ChaCha20-Poly1305, parties), `nearby.rs` +
  `sealed.rs` (Wi-Fi, chiffré), `sharing.rs` (archives `.mpshare`).
- Relais Cloudflare : `relay/src/index.js` (Durable Objects Salle, Profil, Zone, Avis, Colis).
- Mode baladeur : `src/components/Baladeur.vue`, `src/lib/baladeur.ts`, `src/stores/baladeur.ts`.
- Sélection multiple et playlists : `SelectionBar.vue`, `AddToPlaylists.vue`,
  `src/lib/selection.ts`, `src/lib/trackSort.ts`.
