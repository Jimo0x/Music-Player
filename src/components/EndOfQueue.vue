<script setup lang="ts">
import { computed, onMounted } from "vue";
import { endOfQueueSuggestions, type Suggestion } from "@/lib/suggestions";
import { useFavoritesStore } from "@/stores/favorites";
import { useLibraryStore } from "@/stores/library";
import { useListeningStore } from "@/stores/listening";
import { usePlayerStore } from "@/stores/player";
import { usePlaylistsStore } from "@/stores/playlists";
import { useUiStore } from "@/stores/ui";
import Artwork from "./Artwork.vue";
import Icon from "./Icon.vue";

// Quand la file est finie (ou sur le point de l'être) : que mettre ensuite ?
// « play » remplace la file (plus rien ne joue) ; « enqueue » ajoute à la suite du morceau en cours.
const props = withDefaults(defineProps<{ mode?: "play" | "enqueue"; compact?: boolean }>(), { mode: "play", compact: false });

const player = usePlayerStore();
const library = useLibraryStore();
const favorites = useFavoritesStore();
const listening = useListeningStore();
const playlists = usePlaylistsStore();
const ui = useUiStore();

onMounted(() => {
  if (!listening.loaded) void listening.load();
});

const suggestions = computed(() =>
  endOfQueueSuggestions({
    last: player.ended?.last ?? player.currentTrack,
    queueIds: player.ended?.queueIds ?? player.queueIds,
    tracks: library.tracks,
    favoriteIds: [...favorites.ids],
    listens: listening.listens,
    playlists: playlists.items,
  }).slice(0, props.compact ? 4 : 8),
);

function shuffle(ids: string[]): string[] {
  const copy = [...ids];
  for (let i = copy.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [copy[i], copy[j]] = [copy[j], copy[i]];
  }
  return copy;
}

async function choose(s: Suggestion) {
  const ids = s.shuffled ? shuffle(s.trackIds) : s.trackIds;
  if (props.mode === "enqueue") {
    await player.enqueue(ids);
    ui.notify(`« ${s.title} » ajouté à la suite`);
  } else {
    await player.playQueue(ids, ids[0]);
  }
}
</script>

<template>
  <div v-if="suggestions.length" class="end" :class="{ compact }">
    <template v-if="!compact">
      <h2>Fin de la file</h2>
      <p class="lead">Que voulez-vous écouter ensuite ?</p>
    </template>
    <p v-else class="lead">Ensuite, ajouter :</p>
    <div class="grid">
      <button v-for="s in suggestions" :key="s.id" type="button" class="card" @click="choose(s)">
        <span class="art">
          <Artwork :track="library.byId(s.trackIds[0])" :radius="6" />
          <span class="badge"><Icon :name="s.icon" :size="13" /></span>
        </span>
        <span class="text">
          <strong>{{ s.title }}</strong>
          <small>{{ s.sub }}</small>
        </span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.end {
  width: 100%;
  max-width: 640px;
  margin: 0 auto;
  padding: var(--space-4);
  text-align: left;
}
.end h2 {
  margin: 0;
  color: var(--text);
  font-size: 22px;
}
.lead {
  margin: 4px 0 var(--space-3);
  color: var(--text-2);
  font-size: 14px;
}
.compact {
  padding: 0 8px;
}
.compact .lead {
  margin: 0 0 8px;
  font-size: 12px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: var(--space-2);
}
.compact .grid {
  grid-template-columns: 1fr;
}
.card {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  min-width: 0;
  padding: 8px;
  border: 0;
  border-radius: var(--radius);
  background: var(--bg-hover);
  color: var(--text);
  text-align: left;
  cursor: pointer;
}
.card:hover,
.card:focus-visible {
  background: var(--bg-active);
}
.art {
  position: relative;
  flex: none;
  width: 48px;
  height: 48px;
}
.compact .art {
  width: 36px;
  height: 36px;
}
.badge {
  position: absolute;
  right: -4px;
  bottom: -4px;
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--accent);
  color: #fff;
}
.text {
  display: grid;
  min-width: 0;
}
.text strong,
.text small {
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
}
.text strong {
  font-size: 14px;
}
.text small {
  color: var(--text-2);
  font-size: 12px;
}
</style>
