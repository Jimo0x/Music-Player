<script setup lang="ts">
import { useSelectionPool } from "@/lib/selection";
import { useUiStore } from "@/stores/ui";
import { computed } from "vue";
import { formatCollection } from "@/lib/format";
import { useFavoritesStore } from "@/stores/favorites";
import { useLibraryStore } from "@/stores/library";
import { usePlayerStore } from "@/stores/player";
import FastScroll from "@/components/FastScroll.vue";
import Icon from "@/components/Icon.vue";
import TrackRow from "@/components/TrackRow.vue";
import VirtualList from "@/components/VirtualList.vue";
import { remembered } from "@/lib/remember";
import { SORTS, sortTracks } from "@/lib/trackSort";
import type { Track } from "@/lib/types";
import { t } from "@/lib/i18n";

const library = useLibraryStore();
const favorites = useFavoritesStore();
const player = usePlayerStore();

// Les plus récemment aimés en premier : l'ordre n'est pas gardé côté Rust (un simple
// ensemble), donc on retombe sur l'ordre de la bibliothèque, le plus stable disponible.
const liked = computed(() => library.tracks.filter((t) => favorites.isFavorite(t.id)));
// Mêmes tris que les playlists, gardés d'une visite à l'autre.
const SORT_IDS = ["perso", ...SORTS.map((s) => s.id)];
const sortId = remembered<string>("mp:tri-favoris", "perso", SORT_IDS);
const sort = computed(() => SORTS.find((x) => x.id === sortId.value));
const tracks = computed(() => (sort.value ? sortTracks(liked.value, sort.value.key, sort.value.ascending) : liked.value));
// Lettre de la barre de défilement rapide : celle du champ trié.
function initial(t: Track | undefined): string {
  const key = sort.value?.key;
  const text = key === "artist" ? t?.artist : key === "album" ? t?.album : t?.title;
  return text?.trim().charAt(0).toUpperCase() ?? "";
}
const ui = useUiStore();
useSelectionPool(() => tracks.value.map((t) => t.id));
const totalDuration = computed(() => tracks.value.reduce((s, t) => s + t.duration_secs, 0));

function play(index: number) {
  const ids = tracks.value.map((t) => t.id);
  void player.playQueue(ids, ids[index]);
}
</script>

<template>
  <div class="page">
    <div class="top">
      <div>
        <h1 class="page-title">{{ t("Titres aimés") }}</h1>
        <p class="page-sub">{{ formatCollection(tracks.length, totalDuration) }}</p>
      </div>
      <div v-if="tracks.length" class="top-actions">
        <button type="button" class="pill pill-ghost" :class="{ on: ui.selecting }" @click="ui.selecting ? ui.clearSelection() : ui.startSelection()">
          <Icon name="check" :size="15" /> {{ ui.selecting ? t("Terminer") : t("Sélectionner") }}
        </button>
        <button type="button" class="pill pill-ghost" @click="player.playShuffled(tracks.map((t) => t.id))">
          <Icon name="shuffle" :size="15" /> {{ t("Tout en aléatoire") }}
        </button>
      </div>
    </div>
    <label v-if="tracks.length > 1" class="sort">
      <span class="muted">{{ t("Trier") }}</span>
      <select v-model="sortId" class="fieldbox" :aria-label="t('Trier les titres aimés')">
        <option value="perso">{{ t("Ordre de la bibliothèque") }}</option>
        <option v-for="s in SORTS" :key="s.id" :value="s.id">{{ s.label }}</option>
      </select>
    </label>
    <p v-if="!tracks.length" class="empty">
      {{ t("Aucun morceau aimé pour l'instant : touchez le cœur dans le menu d'un morceau (clic droit, ou « ⋯ »).") }}
    </p>
    <VirtualList v-else :items="tracks" :row-height="52">
      <template #default="{ item, index }">
        <TrackRow :track="item" :index="index" striped @play="play(index)" />
      </template>
    </VirtualList>
    <FastScroll :count="tracks.length" :label="(i) => initial(tracks[i])" />
  </div>
</template>

<style scoped>
.top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
}
.top-actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  gap: var(--space-2);
}
.sort {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  margin-bottom: var(--space-3);
  font-size: 13px;
}
.empty {
  margin-top: 8px;
  color: var(--text-2);
}
</style>
