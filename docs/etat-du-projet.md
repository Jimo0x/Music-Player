# État du projet

Mis à jour le **20 novembre 2026** (soir) : **0.10.4** publiée, rien en cours. À tenir à jour à
chaque version : ce qui vient de sortir, ce qui reste, ce qui n'a pas été vérifié.

## En cours : à reprendre en premier

Rien en cours. Prochain chantier proposé : le lot **0.11.0 « Envoyer »** (voir « À faire
ensuite »).

### Fait le 20/11 : retour de la mise à jour Android par l'appli (0.10.2)

Décision du propriétaire : la mise à jour par le navigateur bloquait et pouvait abîmer le
fichier (0.10.0 refusée sur le Pixel 6 Pro : « SHA-256 digest of contents did not verify »).
L'appli télécharge de nouveau l'APK par le service de téléchargement d'Android, puis ouvre
l'installation ; le navigateur reste en secours. Ajouté : l'APK reçu est comparé à l'empreinte
SHA-256 publiée par GitHub (champ `digest` de la version) avant l'installation ; s'il diffère,
il est effacé, retéléchargé une fois, puis un message clair s'affiche. Seules les adresses de
version du projet sont acceptées. **Essai réel réussi** sur l'émulateur Android 14 : une 0.10.0
signée avec la clé de version a proposé la 0.10.1, l'a téléchargée, vérifiée puis installée.
Pas essayé en réel : le cas d'un fichier abîmé (écrit et testé en logique seulement).
Risque Play Protect connu et accepté (voir plus bas).

## Versions récentes

| Version | Date | Contenu principal |
| --- | --- | --- |
| 0.10.4 | 20/11/2026 | Correspondance des téléchargements : l'artiste est aussi reconnu dans le titre de la vidéo (chaîne de paroles, chaîne absente des résultats, plusieurs artistes), mentions « (feat.) », « [From …] » ignorées ; essai réel sur les Top 50 France et Monde lus sur Spotify : 20 échecs sur 50 avant, 1 après, vidéos choisies vérifiées une à une. Liens courts du bouton « Partager » de Spotify (`spotify.link`, `spoti.fi`) suivis jusqu'au vrai lien |
| 0.10.3 | 20/11/2026 | Contributeur : recherche avec liste de résultats dans Télécharger (15 résultats, miniature, chaîne, durée, « Meilleur résultat » garde l'ancien choix automatique) ; recherche tapée à l'envers (« Titre - Artiste ») |
| 0.10.2 | 20/11/2026 | Mise à jour Android de nouveau faite par l'appli, avec vérification SHA-256 de l'APK |
| 0.10.1 | 20/11/2026 | Trois corrections du contributeur : liens de playlist ou d'album = un morceau par tâche, un son partagé depuis un Mix YouTube reste un seul son ; recherche avec le titre seul ; liens Spotify courts (explication) et anciens `play.spotify.com` ; synchronisation des appareils liés qui dit pourquoi elle attend |
| 0.10.0 | 19/11/2026 | Téléchargement sur téléphone (yt-dlp intégré : Python 3.11 par Chaquopy 17.2.0) ; liens Spotify lus sans spotDL (page « embed », 100 morceaux au plus) ; recherche à la façon de spotDL ; « Quoi de neuf ? » ; listes déroulantes lisibles |
| 0.9.0 | 18/11/2026 | Sons Dolby (AC-3/E-AC-3 5.1) lus partout en Rust pur ; Opus en .m4a ; écran verrouillé et touches multimédias sur Windows et Linux (MPRIS) ; page Télécharger ; propositions en fin de file ; tri dans Titres aimés |
| 0.8.0 | 18/11/2026 | Cartes de paroles « Arcane » ; bouton retour Android = MENU en mode baladeur ; CSP activée ; partage Wi-Fi chiffré ; lot playlists ; guide du contributeur |
| 0.7.0 / 0.7.1 | 12/11/2026 | Mode baladeur ; envoi par code par parties ; page de playlist virtualisée |

Publication : la tâche macOS de la 0.10.4 a échoué une fois au moment de mettre à jour
`latest.json` (« Not Found » de GitHub, sans doute plusieurs tâches qui modifient ce fichier en
même temps) ; relancée seule, elle est passée (9 plateformes). Si ça revient : relancer la tâche
en échec, ou faire mettre à jour `latest.json` par une seule tâche à la fin.

## Avis des utilisateurs (5 sur le relais au 20/11)

Lus avec le code administrateur (`%USERPROFILE%\.music-player-avis\code-admin.txt`) :
`POST https://music-player-relais.loickamg.workers.dev/v1/avis/liste` avec `{"admin": code}` ;
`/v1/avis/supprimer` avec `{"admin", "id"}` ; `/v1/avis/image` avec `{"admin", "id", "n"}`.
Ne supprimer qu'avec l'accord du propriétaire.

| Avis (début de l'id) | Sujet | État |
| --- | --- | --- |
| `0f6f48d3` (19/11, 0.9.0) | Envoyer plusieurs sons d'un coup ; dans la discussion, « + » doit permettre de choisir plusieurs sons et plusieurs playlists | **À faire** (lot 0.11.0) |
| `8a2b18bc` | Tri dans Favoris et les playlists | Fait (0.8.0 playlists, 0.9.0 Favoris), confirmé par le propriétaire le 20/11 (« les filtres, c'est fait ») |
| `a6243c4d` | %, Mo et temps restant pendant un envoi | Fait en 0.9.0, confirmation attendue |
| `86f96b44`, `23757f22` | Enregistrements Audiolab .wav/.m4a illisibles | Fait en 0.9.0 (le trou était l'Opus dans un .m4a), confirmation attendue |

Les 3 avis de Méryl (« Tout est réglé », pochettes des playlists, saisie de la discussion)
ont été supprimés le 20/11 avec l'accord du propriétaire.

## À faire ensuite

Retours du propriétaire (vocaux du 20/11) et demandes du même jour, regroupés en lots.

**0.11.0 « Envoyer »** (prochain, **confié à Jimo0x le 20/11** : tickets #8 à #11 ; petites choses : #12) :
1. Envoyer **plusieurs sons d'un coup** (sélection, ou sons d'une playlist) avec « Envoyer à
   quelqu'un » ; avis `0f6f48d3`.
2. Envoyer **plusieurs playlists d'un coup**, dans la discussion ou ailleurs.
3. **Plus de limite de 300 sons** pour une playlist jointe à un message : aujourd'hui, une
   playlist jointe est décrite dans le message lui-même, plafonné par le relais (`MAX_BODY`,
   96 Ko, « 300 morceaux décrits au plus »).
4. **Partage par code fiable** : une playlist part en parties de 40 morceaux ou 150 Mo
   (`sharing::PART_TRACKS`, `PART_BYTES`) ; si une partie échoue (coupure chez le
   destinataire), la suite est abandonnée sans reprise. À faire : reprise après coupure,
   avancement clair, explication du premier essai (103 morceaux envoyés, 97 ou 98 reçus :
   probablement des doublons fusionnés à l'arrivée, à vérifier), et avis sur le découpage et
   les limites du relais gratuit.

**0.12.0 Personnalisation totale** : import de thèmes, d'icônes et de polices ; jaquettes de
playlists ; couleurs, motifs, arrière-plans.

**0.13.0 Ludique** : plusieurs barres de progression (vagues, style inspiré du lecteur de
Huawei…) ; égaliseur et réglages de sortie audio plus amusants ; « Quoi de neuf ? »
interactif ; accès rapides sur téléphone (Télécharger, mode baladeur hors des Réglages).

**0.14.0 Mode immersif** : spatialisation du son (effet 8D, acoustique de salle), en Rust.

**Ensuite** :
- Notifications comme WhatsApp, appli fermée : notifications poussées par le relais via
  Firebase Cloud Messaging (projet Firebase gratuit à créer par le propriétaire), la
  vérification actuelle gardée en secours pour les téléphones sans services Google.
- Lecteur plein écran sur l'écran verrouillé Android, façon Apple Music (option B, 18/11).
- Vérifications sur de vrais appareils (liste plus bas).
- Traduction Français / English / langue du système (décision du 19/11 : plus tard ;
  environ 1 200 textes).
- Revue de sécurité, puis optimisation poussée.

**Petites choses** : sur téléphone, les titres de la liste de résultats sont coupés court (deux
lignes aideraient) ; dans `src/lib/api.ts`, le commentaire de `fetchReadLink` est au-dessus de
`fetchSearch`.

### Points en attente d'une décision du propriétaire

- **Fichiers d'outillage publiés dans le dépôt**, contraire à sa règle globale : le fichier de
  consignes à la racine (suivi depuis `6fa2185`, bien que listé dans `.git/info/exclude`), le
  lien du README vers lui et le document de prompt dans `docs/`. Proposé : retirer le fichier
  du suivi (`git rm --cached`, il reste sur le disque), retirer le lien et le document.
- Supprimer `C:\Android\sdk` et `C:\Android\dl` (≈ 3,6 Go inutiles ; disque C: presque plein).
- Supprimer la branche locale `sauvegarde-main-avant-nettoyage` (ancien `main`, voir
  « Contributeur »).
- Rendre la tâche CI « commits » obligatoire dans la protection de `main` (réglage GitHub,
  à faire par le propriétaire).

## Vérifications sur de vrais appareils

Rien de cette liste n'a été essayé sur un vrai téléphone ou une vraie machine, sauf mention.

- **Pixel 6 Pro (Android 17)** : 0.10.1 puis 0.10.3 installées par câble le 20/11
  (`adb install -r`, données gardées). À essayer : la mise à jour 0.10.4 par l'appli,
  la recherche avec liste de résultats, « Télécharger les manquants », un vrai lien
  `spotify.link`.
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

**Jimo0x** (fork `Jimo0x/Music-Player`, branches `correctif/…` et `fonction/…`). Demandes #1,
#2 et #3 fusionnées le 20/11 (merge commits) ; #6 (liste de résultats) et #7 (recherche
inversée) relues, essayées (recherche réelle avec yt-dlp, écran en mode démo), approuvées et
fusionnées **en un seul commit chacune, au message réécrit**. Toute relecture commence par
**@Jimo0x** (sinon il n'est pas toujours prévenu).

**Lignes d'attribution retirées (20/11)** : ses commits portaient `Co-Authored-By:` (un outil)
et un lien de session ; GitHub listait l'outil parmi les contributeurs. Avec l'accord du
propriétaire, `main` a été réécrit (messages seulement, contenu identique, auteurs et dates
gardés) puis poussé en force ; les étiquettes `app-v0.10.1` et `app-v0.10.2` ont été déplacées
sur les nouveaux commits (versions et fichiers intacts). L'ancien `main` reste dans la branche
locale `sauvegarde-main-avant-nettoyage`. Jimo0x doit remettre son fork à jour sur le nouveau
`main`. Depuis : règle d'or n° 7 de `CONTRIBUTING.md`, case du modèle de demande de fusion, et
tâche CI « commits » qui refuse un commit portant `Co-Authored-By:`, « Generated with … » ou un
lien de session. Fusionner ses demandes en un seul commit au message relu.

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

- Mise à jour Android : `src-tauri/src/updater.rs` (`sha256_of` lit le `digest` GitHub),
  `scripts/android-patch.mjs` (`Updates.digestOf`, `sha256Of`, `verifyApk`, `UpdateBridge.watch`,
  `UpdateDownloadedReceiver`), `src/stores/updater.ts`.
- Correspondance : `src-tauri/src/fetch/matching.rs` (`artist_evidence` : chaîne d'abord, sinon
  titre de la vidéo ; `core_title` retire « (feat.) », « [From …] », « (with …) »). Recherche avec
  résultats : `fetch::commands::fetch_search`, `ytdlp::search_songs`, `DownloadView.vue`.
- Liens Spotify : `fetch/link.rs` (`resolve_short_link` suit `spotify.link`/`spoti.fi` sur les
  hôtes de Spotify seulement, `find_spotify_url`), `spotify_embed.rs`.
- Moteur de téléchargement : `src-tauri/src/fetch/` : file persistante (`queue.rs`), adaptateurs
  spotDL et yt-dlp (`spotdl.rs`, `ytdlp.rs` : 5 candidats, `pick_candidate`), import (`import.rs`).
- Téléphone : `fetch/android.rs` (pont JNI vers `YtDlpBridge.kt`), `fetch/ytdlp_zip.rs`
  (yt-dlp téléchargé, SHA-256 vérifié), `src-tauri/android/python/mp_ytdlp.py`.
- Sons : `dolby.rs`, `opus.rs`, `audio.rs::open_decoder`.
- Écran verrouillé et touches multimédias (PC) : `media_controls.rs` ; pont `src/lib/androidMedia.ts`.
- « Quoi de neuf ? » : `src/nouveautes.json` (obligatoire à chaque version, sinon le test
  `whatsNew.test.ts` et la CI échouent), `src/lib/whatsNew.ts`, `src/components/WhatsNew.vue`.
- Partage : `remote.rs` (par code, SPAKE2 + ChaCha20-Poly1305, parties), `nearby.rs` + `sealed.rs`
  (Wi-Fi, chiffré), `sharing.rs` (archives `.mpshare`, `playlist_parts`).
- Relais Cloudflare : `relay/src/index.js` (Durable Objects Salle, Profil, Zone, Avis, Colis).
- Mode baladeur : `src/components/Baladeur.vue`, `src/lib/baladeur.ts`, `src/stores/baladeur.ts`.
- Environnement Android local : Android Studio, SDK et NDK dans `%LOCALAPPDATA%\Android\Sdk`,
  Java 17 `C:\Android\jdk-17`, émulateurs `MusicPlayer_API34` et `Medium_Phone_API_37.0` ;
  `bash scripts/android-local.sh x86_64`. APK de version signé avec
  `%USERPROFILE%\.music-player-android\release.p12` (`apksigner --ks-type PKCS12`).
