# État du projet

Mis à jour le **27 novembre 2026** : **0.14.0** publiée (Mode immersif), rien en cours. À tenir à jour à
chaque version : ce qui vient de sortir, ce qui reste, ce qui n'a pas été vérifié.

## En cours : à reprendre en premier

Rien en cours. **0.14.0 publiée le 27/11** (CI verte, 17 fichiers, `latest.json` 9 plateformes).
À écouter au casque par le propriétaire (bureau, Pixel 6 Pro) : non fait avant publication.
Repères du lot (cadré avec lui le 26/11) :
- **Rotation 8D** (le son, ramené au centre, tourne autour de la tête ; vitesse 2 à 40
  tours/min, sens, intensité) avec **deux moteurs au choix** : *tête simplifiée* (modèle
  sphérique de Brown et Duda : décalage et ombre entre les oreilles, assourdi derrière) et
  *tête mesurée* (HRTF de la base **SADIE II**, sujet D1 Neumann KU100, Université d'York,
  licence Apache 2.0, plan horizontal tous les 5°, 256 coefficients, 44,1 et 48 kHz
  d'origine, autres fréquences rééchantillonnées ; `src-tauri/assets/hrir/`, LISEZ-MOI).
- **Acoustique de salle** (Freeverb, 6 salles : Studio, Chambre, Club, Salle de concert,
  Cathédrale, Arène ; taille, dosage, murs mats, distance des murs) ; mode léger = moitié
  moins de filtres.
- **Global et par playlist** : réglages globaux dans `session.json` (`immersive`), ambiance
  propre à une playlist appliquée quand la lecture part d'elle (Lire, Aléatoire, un titre).
- Interface : panneau « Mode immersif » dans les Réglages (tête vue de dessus animée, salle en
  ondes, « Maintenir pour comparer »), bouton dans « À l'écoute » et sur chaque playlist.
- Code : `immersive.rs` (après l'égaliseur, sortie toujours stéréo, plus de 2 canaux :
  inchangé), `stores/immersive.ts`, `Immersive.vue`, `ImmersiveDialog.vue`. Coût mesuré
  (`cargo test --release --lib -- --ignored --nocapture cout_du_mode_immersif`) : 0,2 % d'un
  cœur (tête), 0,8 % (HRTF + salle) sur le i7 de ce PC.
- Essai réel (exe debug piloté) : MP3, M4A, WAV lus avec HRTF + Cathédrale, bascule de moteur,
  « Comparer », mode léger et saut pendant la lecture, sans erreur ; réglages bien enregistrés.
  **Pas encore écouté à l'oreille.** Deux FLAC de Damso (« Vieux Sons ») refusent de se lire :
  fichiers remplis de zéros, sans rapport avec ce lot.
- « Essayer » des nouveautés mène à la section `mode-immersif` des Réglages (`section` de la
  route `settings`).

## Versions récentes

| Version | Date | Contenu principal |
| --- | --- | --- |
| 0.14.0 | 27/11/2026 | Lot Mode immersif : rotation 8D (tête simplifiée Brown-Duda ou tête mesurée SADIE II), acoustique de salle (6 salles, Freeverb), réglages globaux et par playlist, panneau dans les Réglages, boutons dans « À l'écoute » et sur chaque playlist |
| 0.13.0 | 26/11/2026 | Lot Ludique : vraie forme d'onde du morceau (Rust, `waveform.rs`, en cache) ; 6 styles de barre de progression sur « À l'écoute » (classique, vague, forme d'onde, perles, néon, segments) ; égaliseur 10 bandes (32 Hz à 16 kHz, anciens réglages à 3 bandes convertis) avec courbe à façonner, spectre en direct, 13 préréglages et « Comparer » ; choix de la sortie audio sur ordinateur ; « Quoi de neuf ? » en cartes avec « Essayer » ; accès rapides Télécharger et baladeur sur l'accueil du téléphone |
| 0.12.0 | 26/11/2026 | Lot Personnalisation : arrière-plan (image ou pochette en cours, flou, voile, cadrage) ; motifs sur le fond ; couleurs avancées d'un thème ; jeu d'icônes SVG importé et nettoyé ; jaquettes faites des pochettes (une ou mosaïque de 4) ; paquet de thème complet `.mptheme` |
| 0.11.1 | 21/11/2026 | Partage par code fiable (contributeur, #21) : bogue 103 → 97 corrigé (doublons répartis sur deux parties), reprise 30 min puis avec le même code, avancement de tout l'envoi, résumé à l'arrivée ; `latest.json` écrit par une seule tâche CI (#20) |
| 0.11.0 | 21/11/2026 | Contributeur : envoi de plusieurs sons et playlists d'un coup, plus de limite de 300 sons (liste complète en colis chiffré), recherche « Sur Internet » dans la barre de recherche |
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

## Avis des utilisateurs (5 sur le relais au 26/11, aucun nouveau depuis le 20/11)

Lus avec le code administrateur (`%USERPROFILE%\.music-player-avis\code-admin.txt`) :
`POST https://music-player-relais.loickamg.workers.dev/v1/avis/liste` avec `{"admin": code}` ;
`/v1/avis/supprimer` avec `{"admin", "id"}` ; `/v1/avis/image` avec `{"admin", "id", "n"}`.
Ne supprimer qu'avec l'accord du propriétaire.

| Avis (début de l'id) | Sujet | État |
| --- | --- | --- |
| `0f6f48d3` (19/11, 0.9.0) | Envoyer plusieurs sons d'un coup ; dans la discussion, « + » doit permettre de choisir plusieurs sons et plusieurs playlists | Fait en 0.11.0 (envoi groupé de sons et de playlists), confirmation du testeur attendue |
| `8a2b18bc` | Tri dans Favoris et les playlists | Fait (0.8.0 playlists, 0.9.0 Favoris), confirmé par le propriétaire le 20/11 (« les filtres, c'est fait ») |
| `a6243c4d` | %, Mo et temps restant pendant un envoi | Fait en 0.9.0, confirmation attendue |
| `86f96b44`, `23757f22` | Enregistrements Audiolab .wav/.m4a illisibles | Fait en 0.9.0 (le trou était l'Opus dans un .m4a), confirmation attendue |

Les 3 avis de Méryl (« Tout est réglé », pochettes des playlists, saisie de la discussion)
ont été supprimés le 20/11 avec l'accord du propriétaire.

## À faire ensuite

**Ensuite** :
- Notifications comme WhatsApp, appli fermée (#22, Jimo0x).
- Lecteur plein écran sur l'écran verrouillé Android, façon Apple Music (option B, 18/11).
- Vérifications sur de vrais appareils (liste plus bas).
- Traduction Français / English / langue du système (#23, Jimo0x).
- Revue de sécurité, puis optimisation poussée.

### Points en attente d'une décision du propriétaire

- Rendre la tâche CI « commits » obligatoire : vérifié le 27/11, la règle « Protection de
  main » n'exige que `test` (réglage GitHub, par le propriétaire).

Réglé le 27/11 : notes de travail internes retirées du suivi (gardées en local, ignorées par
`.git/info/exclude`) ; `C:\Android\sdk` et `C:\Android\dl` supprimés (`C:\Android\jdk-17` gardé) ;
branche `sauvegarde-main-avant-nettoyage` supprimée (ancienne tête `d42bc2a`).

### Confié au contributeur (Jimo0x, 27/11)

- #22 Notifications poussées appli fermée (Firebase Cloud Messaging, projet Firebase gratuit
  créé par lui, propriétaire ajouté comme Propriétaire ; secrets transmis en privé).
- #23 Traduction Français / English / langue du système.
- #24 Lignes `Co-authored-by:` automatiques dans ses branches et son fork (le `main` du dépôt
  est propre).
- #25 Bogue : sons téléchargés sans pochette (chemin yt-dlp, ordinateur et téléphone ; seul
  spotDL intègre la pochette). Diagnostic et correctif détaillés dans le ticket.
- #26 Reconnaître un son (écoute, fredonnement, bouts de paroles) : évaluation des services
  (aucun gratuit et fiable pour l'écoute et le fredonnement ; LRCLIB ne cherche pas dans les
  paroles), étape 0 d'essais réels avant tout code, puis paroles d'abord. Décision du 28/11 :
  écoute et fredonnement par **notre propre outil** intégré (empreinte à repères, recherche par
  mélodie), alternatives possibles si les essais le justifient.

## Vérifications sur de vrais appareils

Rien de cette liste n'a été essayé sur un vrai téléphone ou une vraie machine, sauf mention.

- **Pixel 6 Pro (Android 17)** : 0.10.3 installée par câble le 20/11. À essayer : la mise
  à jour par l'appli jusqu'à la 0.13.0, la recherche avec liste de résultats et « Sur
  Internet », « Télécharger les manquants », un vrai lien `spotify.link`, l'égaliseur
  10 bandes à l'oreille, les styles de barre, les accès rapides.
- **Vraie appli de bureau (Windows)** : lots 0.12.0 et 0.13.0 essayés le 26/11 (exe de
  débogage piloté par DevTools, données sauvegardées puis remises à l'identique) : fond,
  motif, couleurs avancées, icônes, paquet de thème, mosaïque, forme d'onde, égaliseur
  10 bandes reçu par le moteur, bascule de sortie audio sans coupure. Le PC n'a qu'une
  sortie (« Speakers ») : bascule entre deux sorties différentes pas essayée.
- **Partage par code** d'une grosse playlist avec coupure du Wi-Fi chez le destinataire
  (0.11.1) ; envoi de plusieurs sons et playlists entre deux appareils (0.11.0).
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

**Jimo0x** (fork `Jimo0x/Music-Player`, branches `correctif/…` et `fonction/…`). Le 20-21/11 :
#1 à #3, #6, #7, #14 à #17, #19 à #21 relues, vérifiées et fusionnées (tickets #8 à #12 et #18
fermés). Toute relecture commence par **@Jimo0x** (sinon il n'est pas toujours prévenu).
Fusionner ses demandes **en un seul commit au message relu** (sans ligne d'attribution).
Demandes empilées (#15 ⊂ #16 ⊂ #17 le 21/11) : après chaque fusion, rebaser la suivante sur
`main` en ne gardant que son propre commit (il autorise les modifications du propriétaire).

**Lignes d'attribution retirées (20/11)** : ses commits portaient `Co-Authored-By:` (un outil)
et un lien de session ; GitHub listait l'outil parmi les contributeurs. Avec l'accord du
propriétaire, `main` a été réécrit (messages seulement, contenu identique, auteurs et dates
gardés) puis poussé en force ; les étiquettes `app-v0.10.1` et `app-v0.10.2` ont été déplacées
sur les nouveaux commits (versions et fichiers intacts). L'ancien `main` reste dans la branche
locale `sauvegarde-main-avant-nettoyage`. Depuis : règle d'or n° 7 de `CONTRIBUTING.md`, case
du modèle de demande de fusion, et tâche CI « commits » qui refuse un commit portant
`Co-Authored-By:`, « Generated with … » ou un lien de session.

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

- Lot 0.13.0 : `src-tauri/src/waveform.rs` (forme d'onde, cache `ondes/`, commande
  `track_waveform`) ; `src/lib/seekStyles.ts` + `SeekBar.vue` (props `look`, `preview`) ;
  `src-tauri/src/eq.rs` (`BANDS = 10`, `gains_from_values` convertit 3 → 10,
  `deserialize_gains` pour `session.json`) + `src/stores/eq.ts` + `Equalizer.vue` ;
  sortie audio : `audio.rs` (`OUTPUT_CHOICE`, `SWITCH_OUTPUT` traité comme un flux perdu),
  `session.output_device`, `OutputDevices.vue` ; `src/lib/whatsNewTour.ts` ;
  accès rapides dans `RecentView.vue`.
- Lot 0.12.0 : `src/lib/backdrop.ts` + `AppBackdrop.vue` (calque sous `.app` isolée) ;
  `src/lib/patterns.ts` (masques SVG) ; couleurs avancées dans `themes.ts` ;
  `src/lib/iconPack.ts` (nettoyeur SVG à liste fermée) + `iconPaths.ts` ;
  `PlaylistCover.vue` (`trackIds`, mosaïque) ; `src/lib/themePack.ts` + commandes
  `save_theme_pack`, `read_asset` (la CSP interdit `fetch()` sur les fichiers locaux).
- Publication : insérer les nouveautés avec un contrôle `json.loads` (un premier « ] » dans
  un texte avait cassé `nouveautes.json` le 21/11) et n'enchaîner l'envoi qu'après des
  vérifications vertes.
- Vidéo de présentation (hors dépôt) : projet séparé `~/projects/music-player-video`
  (Remotion, captures du mode démo) ; voir son `LISEZ-MOI.md`.
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
