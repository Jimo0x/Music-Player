<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api } from "@/lib/api";
import type { FetchWanted } from "@/lib/types";
import { useFetchStore } from "@/stores/fetch";
import { usePlaylistsStore } from "@/stores/playlists";
import { useUiStore } from "@/stores/ui";
import FetchJobs from "@/components/FetchJobs.vue";
import Icon from "@/components/Icon.vue";

// Télécharger sans passer par l'import Spotify : un lien (YouTube, SoundCloud… lu par
// yt-dlp ; Spotify, morceau ou playlist, lu par spotDL) ou une recherche « Artiste - Titre ».
const fetchStore = useFetchStore();
const playlists = usePlaylistsStore();
const ui = useUiStore();
const isAndroid = /Android/i.test(navigator.userAgent);

const input = ref("");
const playlistId = ref("");
const busy = ref(false);
const error = ref<string | null>(null);

onMounted(() => {
  if (isAndroid) return;
  void fetchStore.initialize();
  void fetchStore.refreshAvailability();
});

const kind = computed<"spotify" | "link" | "search" | null>(() => {
  const text = input.value.trim();
  if (!text) return null;
  if (/^https?:\/\/(open|play)\.spotify\.com\//i.test(text) || /^spotify:/i.test(text)) return "spotify";
  if (/^https?:\/\//i.test(text)) return "link";
  return "search";
});

const hint = computed(() => {
  switch (kind.value) {
    case "spotify":
      return fetchStore.availability.spotdl ? "Lien Spotify : chaque morceau sera cherché puis téléchargé." : "Les liens Spotify demandent spotDL.";
    case "link":
      return fetchStore.availability.ytdlp ? "Lien direct : le son sera téléchargé tel quel." : "Les liens directs demandent yt-dlp.";
    case "search":
      return "Recherche : le meilleur résultat qui correspond au titre (et à l'artiste) est gardé.";
    default:
      return "";
  }
});

const canSubmit = computed(() => {
  if (busy.value || !kind.value) return false;
  if (kind.value === "spotify") return fetchStore.availability.spotdl;
  if (kind.value === "link") return fetchStore.availability.ytdlp;
  return fetchStore.anyTool;
});

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
  try {
    let wanted: FetchWanted[];
    if (kind.value === "spotify") {
      const list = await api.fetchReadLink(text);
      wanted = list.tracks.map((t) => ({ title: t.title, artist: t.artist, album: t.album, duration_secs: t.duration_secs, playlist_id: target }));
    } else if (kind.value === "link") {
      wanted = [{ title: "", artist: "", url: text, playlist_id: target }];
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

    <section v-if="isAndroid" class="card">
      <p>Le téléchargement sur téléphone arrive avec le moteur intégré (en préparation). En attendant, télécharge depuis le PC : les morceaux arrivent sur le téléphone par « Mes appareils ».</p>
    </section>

    <template v-else>
      <form class="card form" @submit.prevent="submit">
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
          <button type="submit" class="pill pill-accent" :disabled="!canSubmit">
            <Icon name="import" :size="15" /> {{ busy ? "Lecture…" : "Télécharger" }}
          </button>
        </div>
        <p v-if="hint" class="muted small">{{ hint }}</p>
        <p v-if="error" class="error" role="alert">{{ error }}</p>
      </form>

      <section class="card tools">
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
    </template>
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
.error {
  margin: 0;
  color: var(--danger-text);
}
code {
  font-size: 12px;
}
</style>
