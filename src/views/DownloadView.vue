<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api } from "@/lib/api";
import { formatDuration } from "@/lib/format";
import type { FetchWanted, SearchResult } from "@/lib/types";
import { useFetchStore } from "@/stores/fetch";
import { usePlaylistsStore } from "@/stores/playlists";
import { useUiStore } from "@/stores/ui";
import FetchJobs from "@/components/FetchJobs.vue";
import Icon from "@/components/Icon.vue";

// Télécharger sans passer par l'import Spotify : un lien (YouTube, SoundCloud… lu par
// yt-dlp), un lien Spotify (morceau, album ou playlist, lu sans spotDL puis chaque morceau
// cherché) ou une recherche « Artiste - Titre ». Sur téléphone, yt-dlp est intégré à l'appli.
const fetchStore = useFetchStore();
const playlists = usePlaylistsStore();
const ui = useUiStore();
const isAndroid = /Android/i.test(navigator.userAgent);

const input = ref("");
const playlistId = ref("");
const busy = ref(false);
const error = ref<string | null>(null);
/** Playlist coupée par Spotify (100 premiers morceaux). */
const notice = ref<string | null>(null);

// Recherche à la façon d'Audiomack : une liste de résultats, chacun téléchargeable d'un geste.
const results = ref<SearchResult[] | null>(null);
/** Texte des résultats affichés (la liste reste tant qu'on ne relance pas). */
const searchedFor = ref("");
const searching = ref(false);
/** Liens déjà envoyés à la file depuis cette liste (bouton remplacé par « Ajouté »). */
const sent = ref(new Set<string>());

onMounted(() => {
  void fetchStore.initialize();
  void fetchStore.refreshAvailability();
});

 main
/** Lien court du bouton « Partager » de Spotify : il ne mène au morceau qu'après un passage
 *  par le navigateur, que l'appli ne fait pas. */

/** Lien court du bouton « Partager » de Spotify : l'appli le suit jusqu'au vrai lien. */
 main
const SHORT_SPOTIFY = /^https?:\/\/(spotify\.link|spoti\.fi)\//i;

/** Ancienne adresse `play.spotify.com` : même lien sous `open.spotify.com` (seule acceptée). */
function spotifyLink(text: string): string {
  return text.replace(/^https?:\/\/play\.spotify\.com\//i, "https://open.spotify.com/");
}

const kind = computed<"spotify" | "short" | "link" | "search" | null>(() => {
  const text = input.value.trim();
  if (!text) return null;
  if (SHORT_SPOTIFY.test(text)) return "short";
  if (/^https?:\/\/(open|play)\.spotify\.com\//i.test(text) || /^spotify:/i.test(text)) return "spotify";
  if (/^https?:\/\//i.test(text)) return "link";
  return "search";
});

const hint = computed(() => {
  switch (kind.value) {
    case "spotify":
      return fetchStore.anyTool ? "Lien Spotify : chaque morceau sera cherché puis téléchargé." : "Les liens Spotify demandent yt-dlp ou spotDL.";
    case "short":
 main
      return "Lien court de Spotify : ouvre-le dans le navigateur, puis colle ici l'adresse open.spotify.com qui s'affiche.";

      return fetchStore.anyTool ? "Lien Spotify (lien court du bouton Partager) : chaque morceau sera cherché puis téléchargé." : "Les liens Spotify demandent yt-dlp ou spotDL.";
 main
    case "link":
      return fetchStore.availability.ytdlp ? "Lien direct : le son est téléchargé tel quel (une playlist donne un morceau par tâche)." : "Les liens directs demandent yt-dlp.";
    case "search":
      return fetchStore.availability.ytdlp
        ? "Recherche : choisis le bon résultat dans la liste, ou laisse l'appli prendre celui qui correspond le mieux."
        : "Recherche : le meilleur résultat qui correspond au titre (et à l'artiste) est gardé.";
    default:
      return "";
  }
});

const canSubmit = computed(() => {
 main
  if (busy.value || !kind.value || kind.value === "short") return false;
  if (kind.value === "spotify") return fetchStore.anyTool;

  if (busy.value || !kind.value) return false;
  if (kind.value === "spotify" || kind.value === "short") return fetchStore.anyTool;
 main
  if (kind.value === "link") return fetchStore.availability.ytdlp;
  return fetchStore.anyTool;
});

const canSearch = computed(() => !searching.value && kind.value === "search" && fetchStore.availability.ytdlp);

async function runSearch() {
  if (!canSearch.value) return;
  const text = input.value.trim();
  searching.value = true;
  error.value = null;
  notice.value = null;
  try {
    results.value = await api.fetchSearch(text);
    searchedFor.value = text;
    sent.value = new Set();
  } catch (e) {
    results.value = null;
    error.value = String(e);
  } finally {
    searching.value = false;
  }
}

/** Un résultat choisi : téléchargé tel quel par son lien (le titre de la vidéo est nettoyé à
 *  l'import, comme pour un lien collé). */
async function downloadResult(result: SearchResult) {
  const ids = await fetchStore.enqueue([
    { title: result.title, artist: result.artist, duration_secs: result.duration_secs, url: result.url, playlist_id: playlistId.value || null },
  ]);
  if (ids.length) {
    sent.value = new Set([...sent.value, result.url]);
    ui.notify("Ajouté à la file de téléchargements");
  } else if (fetchStore.lastError) {
    error.value = fetchStore.lastError;
  }
}

/** Entrée dans le champ : une recherche montre ses résultats, un lien se télécharge. */
function onEnter() {
  if (kind.value === "search" && canSearch.value) void runSearch();
  else void submit();
}

/** « Artiste - Titre » (tiret entouré d'espaces), sinon tout est le titre. */
function parseSearch(text: string): { title: string; artist: string } {
  const at = text.search(/\s[-–—]\s/);
  if (at < 0) return { title: text, artist: "" };
  return { artist: text.slice(0, at).trim(), title: text.slice(at + 3).trim() };
}

async function submit() {
  if (!canSubmit.value) return;
  const text = input.value.trim();
  const target = playlistId.value || null;
  busy.value = true;
  error.value = null;
  notice.value = null;
  try {
    let wanted: FetchWanted[];
 main
    if (kind.value === "spotify") {

    if (kind.value === "spotify" || kind.value === "short") {
 main
      const list = await api.fetchReadLink(spotifyLink(text));
      if (list.partial) notice.value = "Spotify n'a donné que les 100 premiers morceaux de cette playlist. Pour la liste complète : importe tes données Spotify (page « Importer de Spotify »), ou installe spotDL sur ordinateur.";
      wanted = list.tracks.map((t) => ({ title: t.title, artist: t.artist, album: t.album, duration_secs: t.duration_secs, playlist_id: target }));
    } else if (kind.value === "link") {
      // Playlist ou album : un morceau par tâche (sinon une seule tâche recevait tous les
      // fichiers et échouait). Lecture impossible : le lien part tel quel, comme avant.
      const media = await api.fetchReadMediaLink(text).catch(() => null);
      wanted = media?.tracks.length
        ? media.tracks.map((t) => ({ title: t.title, artist: t.artist, duration_secs: t.duration_secs, url: t.url, playlist_id: target }))
        : [{ title: "", artist: "", url: text, playlist_id: target }];
    } else {
      wanted = [{ ...parseSearch(text), playlist_id: target }];
    }
    const ids = await fetchStore.enqueue(wanted);
    if (ids.length) {
      ui.notify(ids.length > 1 ? `${ids.length} morceaux ajoutés à la file` : "Ajouté à la file de téléchargements");
      input.value = "";
    } else if (fetchStore.lastError) {
      error.value = fetchStore.lastError;
    }
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="page">
    <h1 class="page-title">Télécharger</h1>
    <p class="page-sub">Un lien ou une recherche : le morceau arrive dans la bibliothèque.</p>

      <form class="card form" @submit.prevent="onEnter">
        <label class="field">
          <span class="muted small">Lien YouTube, SoundCloud, Spotify… ou « Artiste - Titre »</span>
          <input v-model="input" class="fieldbox" type="text" inputmode="url" autocomplete="off" spellcheck="false" placeholder="https://… ou The Weeknd - Blinding Lights" aria-label="Lien ou recherche" />
        </label>
        <div class="row">
          <label class="field playlist">
            <span class="muted small">Ranger dans</span>
            <select v-model="playlistId" class="fieldbox" aria-label="Playlist de destination">
              <option value="">La bibliothèque seulement</option>
              <option v-for="p in playlists.items" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
          </label>
          <template v-if="kind === 'search' && fetchStore.availability.ytdlp">
            <button type="submit" class="pill pill-accent" :disabled="!canSearch">
              <Icon name="search" :size="15" /> {{ searching ? "Recherche…" : "Rechercher" }}
            </button>
            <button type="button" class="pill pill-ghost" :disabled="!canSubmit" @click="submit">
              <Icon name="import" :size="15" /> {{ busy ? "Lecture…" : "Meilleur résultat" }}
            </button>
          </template>
          <button v-else type="submit" class="pill pill-accent" :disabled="!canSubmit">
            <Icon name="import" :size="15" /> {{ busy ? "Lecture…" : "Télécharger" }}
          </button>
        </div>
        <p v-if="hint" class="muted small">{{ hint }}</p>
        <p v-if="error" class="error" role="alert">{{ error }}</p>
        <p v-if="notice" class="notice" role="status">{{ notice }}</p>
      </form>

      <section v-if="results" class="card" aria-live="polite">
        <div class="queue-head">
          <h2>Résultats pour « {{ searchedFor }} »</h2>
          <button type="button" class="pill pill-ghost" @click="results = null">Fermer</button>
        </div>
        <p v-if="!results.length" class="muted small">Aucun résultat : essaie avec l'artiste et le titre.</p>
        <ul v-else class="results">
          <li v-for="r in results" :key="r.url" class="result">
            <img v-if="r.thumbnail" class="thumb" :src="r.thumbnail" alt="" loading="lazy" referrerpolicy="no-referrer" />
            <span v-else class="thumb thumb-empty" aria-hidden="true"><Icon name="play" :size="16" /></span>
            <div class="result-text">
              <strong class="result-title">{{ r.title }}</strong>
              <span class="muted small">{{ r.artist || "Artiste inconnu" }}<template v-if="r.duration_secs"> · {{ formatDuration(r.duration_secs) }}</template></span>
            </div>
            <span v-if="sent.has(r.url)" class="added small"><Icon name="check" :size="14" /> Ajouté</span>
            <button v-else type="button" class="pill pill-accent" :aria-label="`Télécharger ${r.title}`" @click="downloadResult(r)">
              <Icon name="import" :size="15" /><span class="result-action">Télécharger</span>
            </button>
          </li>
        </ul>
      </section>

      <section v-if="isAndroid" class="card tools">
        <p class="small muted">yt-dlp est intégré à l'appli : il se télécharge à la première utilisation (3 Mo) et se met à jour tout seul. Les morceaux arrivent en .m4a, lus directement par l'appli.</p>
      </section>
      <section v-else class="card tools">
        <p class="small">
          <span :class="fetchStore.availability.ytdlp ? 'ok' : 'missing'">yt-dlp {{ fetchStore.availability.ytdlp ? "trouvé" : "absent" }}</span>
          ·
          <span :class="fetchStore.availability.spotdl ? 'ok' : 'missing'">spotDL {{ fetchStore.availability.spotdl ? "trouvé" : "absent" }}</span>
          <button type="button" class="link" @click="fetchStore.refreshAvailability()">Vérifier à nouveau</button>
        </p>
        <p v-if="fetchStore.initialized && !fetchStore.anyTool" class="muted small">
          Installe yt-dlp (<code>winget install yt-dlp</code>) ou spotDL (<code>py -m pip install -U spotdl</code>), puis vérifie à nouveau.
        </p>
      </section>

      <section v-if="fetchStore.jobs.length" class="card">
        <div class="queue-head">
          <h2>File de téléchargements</h2>
          <button v-if="fetchStore.finished.length" type="button" class="pill pill-ghost" @click="fetchStore.clearFinished()">Vider les terminées</button>
        </div>
        <p v-if="fetchStore.active.length" class="muted small">{{ fetchStore.active.length }} en cours ou en attente · la file continue si tu quittes cette page.</p>
        <FetchJobs />
      </section>
  </div>
</template>

<style scoped>
.card {
  margin-top: var(--space-3);
  padding: var(--space-4);
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.04);
}
.form {
  display: grid;
  gap: var(--space-3);
}
.field {
  display: grid;
  gap: 4px;
}
.field input,
.field select {
  padding: 9px 11px;
}
.row {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-end;
  gap: var(--space-3);
}
.playlist {
  flex: 1;
  min-width: 200px;
}
.small {
  margin: 0;
  font-size: 12px;
}
.ok {
  color: var(--success);
}
.missing {
  color: var(--text-3);
}
.tools .link {
  margin-left: var(--space-2);
  padding: 0;
  border: 0;
  background: none;
  color: var(--accent);
  font-size: inherit;
  cursor: pointer;
}
.queue-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
}
.queue-head h2 {
  margin: 0;
  font-size: 16px;
}
.notice {
  margin: 0;
  color: var(--warning);
  font-size: 13px;
}
.error {
  margin: 0;
  color: var(--danger-text);
}
code {
  font-size: 12px;
}
.results {
  display: grid;
  gap: 6px;
  margin: var(--space-3) 0 0;
  padding: 0;
  list-style: none;
}
.result {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 6px 8px;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.04);
}
.thumb {
  flex: none;
  width: 64px;
  height: 36px;
  border-radius: calc(var(--radius) / 2);
  object-fit: cover;
  background: rgba(var(--fg-rgb), 0.08);
}
.thumb-empty {
  display: grid;
  place-items: center;
  color: var(--text-3);
}
.result-text {
  display: grid;
  flex: 1;
  min-width: 0;
}
.result-title {
  overflow: hidden;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.added {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--success);
}
/* Téléphone : bouton réduit à son icône pour laisser la place au titre. */
@media (max-width: 480px) {
  .result-action {
    display: none;
  }
}
</style>
