<script lang="ts">
import type { SearchResult as Found } from "@/lib/types";

/** Recherches déjà faites, partagées entre les listes (onglets) : revenir ne relance rien. Les
 *  plus anciennes partent au-delà de 30. */
const cache = new Map<string, Found[]>();
function remember(text: string, found: Found[]) {
  cache.set(text, found);
  if (cache.size > 30) cache.delete(cache.keys().next().value as string);
}
</script>

<script setup lang="ts">
// Résultats « Sur Internet » d'une recherche : le morceau cherché, les morceaux d'un artiste ou
// ce qui s'en rapproche le plus, trouvés par la même recherche que la page Télécharger
// (yt-dlp, PC et téléphone). Un geste les télécharge dans la bibliothèque ; un morceau déjà
// là se lit directement.
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { api } from "@/lib/api";
import { formatDuration } from "@/lib/format";
import { LibraryMatcher } from "@/lib/spotifyImport";
import type { SearchResult, Track } from "@/lib/types";
import { useFetchStore } from "@/stores/fetch";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player";
import { useUiStore } from "@/stores/ui";
import Icon from "./Icon.vue";

const props = withDefaults(defineProps<{ query: string; limit?: number }>(), { limit: Infinity });
const emit = defineEmits<{ more: [] }>();

const fetchStore = useFetchStore();
const library = useLibraryStore();
const player = usePlayerStore();
const ui = useUiStore();

const results = ref<SearchResult[] | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);
let timer: ReturnType<typeof setTimeout> | undefined;
let asked = "";

/** Recherche lancée un instant après la dernière lettre (pas une par lettre tapée). */
const WAIT_MS = 700;

const available = computed(() => fetchStore.availability.ytdlp);
const shown = computed(() => (results.value ?? []).slice(0, props.limit));

async function run(text: string) {
  asked = text;
  const cached = cache.get(text);
  if (cached) {
    results.value = cached;
    return;
  }
  loading.value = true;
  error.value = null;
  try {
    const found = await api.fetchSearch(text);
    remember(text, found);
    // Une recherche plus récente a pu partir entre-temps : seule la dernière s'affiche.
    if (asked !== text) return;
    results.value = found;
  } catch (e) {
    if (asked !== text) return;
    results.value = null;
    error.value = String(e);
  } finally {
    if (asked === text) loading.value = false;
  }
}

function schedule() {
  clearTimeout(timer);
  const text = props.query.trim();
  if (text.length < 2 || !available.value) {
    results.value = null;
    asked = "";
    return;
  }
  timer = setTimeout(() => void run(text), cache.has(text) ? 0 : WAIT_MS);
}
watch(() => [props.query, available.value], schedule);
onMounted(() => {
  void fetchStore.initialize().then(schedule);
});
onBeforeUnmount(() => clearTimeout(timer));

// ---- Où en est chaque résultat : dans la bibliothèque, en téléchargement, ou à télécharger ----
const matcher = computed(() => new LibraryMatcher(library.tracks));
function inLibrary(r: SearchResult): Track | null {
  return matcher.value.find({ title: r.title, artist: r.artist, album: "" });
}
function job(r: SearchResult) {
  // Le dernier envoi de ce lien à la file (un nouvel essai remplace un échec).
  for (let i = fetchStore.jobs.length - 1; i >= 0; i--) if (fetchStore.jobs[i].wanted.url === r.url) return fetchStore.jobs[i];
  return null;
}
function jobLabel(r: SearchResult): string | null {
  const j = job(r);
  if (!j) return null;
  if (j.state === "queued") return "En attente…";
  if (j.state === "running") {
    const p = fetchStore.progress[j.id];
    return p != null ? `Téléchargement… ${Math.round(p * 100)} %` : "Téléchargement…";
  }
  if (j.state === "done") return "Ajouté à la bibliothèque";
  return null;
}

async function download(r: SearchResult) {
  const ids = await fetchStore.enqueue([{ title: r.title, artist: r.artist, duration_secs: r.duration_secs, url: r.url, playlist_id: null }]);
  if (ids.length) ui.notify(`« ${r.title} » : téléchargement lancé`);
  else if (fetchStore.lastError) ui.notify(fetchStore.lastError);
}
function play(t: Track) {
  void player.playQueue([t.id], t.id);
}
</script>

<template>
  <section class="online" aria-live="polite">
    <div class="head">
      <h2><Icon name="import" :size="17" /> Sur Internet</h2>
      <span v-if="loading" class="muted small">Recherche…</span>
      <button v-else-if="(results?.length ?? 0) > shown.length" type="button" class="see-all" @click="emit('more')">Tout voir ({{ results!.length }})</button>
    </div>

    <p v-if="fetchStore.initialized && !available" class="muted small">
      Pour chercher et télécharger des sons qui ne sont pas encore dans la bibliothèque, installez yt-dlp (page Télécharger).
    </p>
    <p v-else-if="error" class="muted small">{{ error }}</p>
    <p v-else-if="results && !results.length" class="muted small">Rien trouvé sur Internet pour « {{ query }} ».</p>

    <ul v-if="shown.length" class="list">
      <li v-for="r in shown" :key="r.url" class="result">
        <img v-if="r.thumbnail" class="thumb" :src="r.thumbnail" alt="" loading="lazy" referrerpolicy="no-referrer" />
        <span v-else class="thumb thumb-empty" aria-hidden="true"><Icon name="play" :size="16" /></span>
        <div class="text">
          <strong class="title" :title="r.title">{{ r.title }}</strong>
          <span class="muted small">{{ r.artist || "Artiste inconnu" }}<template v-if="r.duration_secs"> · {{ formatDuration(r.duration_secs) }}</template></span>
        </div>
        <button v-if="inLibrary(r)" type="button" class="pill pill-ghost" :aria-label="`Lire ${r.title} (déjà dans la bibliothèque)`" @click="play(inLibrary(r)!)">
          <Icon name="play" :size="15" /><span class="action">Dans la bibliothèque</span>
        </button>
        <span v-else-if="jobLabel(r)" class="state small">{{ jobLabel(r) }}</span>
        <button v-else type="button" class="pill pill-accent" :aria-label="`Télécharger ${r.title}`" @click="download(r)">
          <Icon name="import" :size="15" /><span class="action">Télécharger</span>
        </button>
      </li>
    </ul>
  </section>
</template>

<style scoped>
.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-2);
}
.see-all {
  border: 0;
  background: none;
  color: var(--text-2);
  font: inherit;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}
.head h2 {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}
.list {
  display: grid;
  gap: 6px;
  margin: 0;
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
.text {
  display: grid;
  flex: 1;
  min-width: 0;
}
.title {
  overflow: hidden;
  font-size: 13px;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.state {
  flex: none;
  color: var(--text-2);
  white-space: nowrap;
}
.pill {
  flex: none;
}
/* Téléphone : boutons réduits à leur icône, titre sur deux lignes. */
@media (max-width: 480px) {
  .action {
    display: none;
  }
  .title {
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    white-space: normal;
    overflow-wrap: anywhere;
  }
}
</style>
