# État du projet

Mis à jour le 18 novembre 2026, à la publication de la **0.8.0**. À tenir à jour à chaque
version : ce qui vient de sortir, ce qui reste, ce qui n'a pas été vérifié.

## Versions récentes

| Version | Date | Contenu principal |
| --- | --- | --- |
| 0.8.0 | 18/11/2026 | Cartes de paroles « Arcane » (tarot) ; bouton retour Android = MENU en mode baladeur ; noms d'artistes tirés des titres de vidéos nettoyés ; avancement des téléchargements ; CSP activée ; partage Wi-Fi chiffré ; relais sans routes de compte ; actions CI v5 ; styles en variables du thème ; lot playlists (sélection multiple, ajout à plusieurs playlists, tri dans une playlist, ajout depuis l'écran de lecture) ; guide du contributeur |
| 0.7.1 | 12/11/2026 | Play Protect : l'appli n'installe plus d'APK (mise à jour par le navigateur) ; page de playlist virtualisée ; envoi par code par parties ; salle du relais gardée pendant un transfert |
| 0.7.0 | 12/11/2026 | Mode baladeur (téléphone) : boîtier, molette tactile, 5 finitions, paroles sur petit écran |
| 0.6.1 | 12/11/2026 | Moteur de téléchargement testé pour de vrai (outils hors PATH, pièges de spotDL) |
| 0.6.0 | 12/11/2026 | Moteur de téléchargement (file spotDL puis yt-dlp), import Spotify par lien, données du compte ou fichier |

## Ce qui reste à faire

### Avis des utilisateurs encore ouverts

- **Audiolab** : enregistrements .wav / .m4a illisibles (signalé deux fois). Essayé le
  18/11 avec deux vrais fichiers (WAV PCM 16 bits et AAC de WhatsApp) : lus par le décodeur
  intégré, sans ffmpeg, donc aussi sur Android. Variantes essayées : WAV 24 bits, 32 bits
  flottant, ADPCM, 48 kHz stéréo, AAC en .m4a : toutes lues. **Seul trou trouvé** : de l'Opus
  dans un conteneur MP4 (.m4a). Sur PC, ffmpeg prend le relais ; sur Android, ce fichier
  reste illisible (`opus::is_ogg_opus` ne reconnaît que l'Ogg). C'est sans doute le cas
  d'Audiolab (le message parlait d'.opus).
- **Pochettes des playlists de Méryl** (Android 10, Chrome 114) : correction envoyée, jamais
  vérifiée sur son téléphone. Les deux autres points de son avis (saisie de la discussion
  sous le clavier, champ d'avis en pilule) sont corrigés depuis le 11/11.
- **Tri dans Favoris** : fait dans les playlists (0.8.0), absent de Favoris.
- **Avancement d'un envoi dans la discussion** : la section Partage montre déjà le
  pourcentage, les Mo et le temps restant ; la discussion n'affiche que le pourcentage.

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

- Mini-lecteur sur l'écran verrouillé.
- Spotify (accès retirés, code gardé).

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
