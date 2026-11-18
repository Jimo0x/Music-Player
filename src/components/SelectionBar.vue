<script setup lang="ts">
// Barre de la sélection de plusieurs morceaux : combien, tout cocher, ajouter à des
// playlists, lire, annuler.
import { usePlayerStore } from "@/stores/player";
import { useUiStore } from "@/stores/ui";
import Icon from "@/components/Icon.vue";

const ui = useUiStore();
const player = usePlayerStore();

function addToPlaylists() {
  if (ui.selected.length) ui.addToPlaylists = [...ui.selected];
}

async function playNext() {
  if (!ui.selected.length) return;
  const started = await player.playNext([...ui.selected]);
  ui.notify(started ? "Lecture lancée" : `${ui.selected.length} morceau${ui.selected.length > 1 ? "x" : ""} lus ensuite`);
  ui.clearSelection();
}
</script>

<template>
  <Transition name="bar">
    <div v-if="ui.selecting" class="selection-bar" role="toolbar" aria-label="Morceaux sélectionnés">
      <button type="button" class="icon-btn" aria-label="Annuler la sélection" @click="ui.clearSelection()"><Icon name="close" :size="18" /></button>
      <span class="count">{{ ui.selected.length }} sélectionné{{ ui.selected.length > 1 ? "s" : "" }}</span>
      <button v-if="ui.selectionPool.length" type="button" class="pill pill-ghost" @click="ui.selectAll()">
        {{ ui.selected.length === ui.selectionPool.length ? "Aucun" : "Tout" }}
      </button>
      <button type="button" class="pill pill-ghost" :disabled="!ui.selected.length" @click="playNext"><Icon name="queue" :size="15" /> Lire ensuite</button>
      <button type="button" class="pill pill-accent" :disabled="!ui.selected.length" @click="addToPlaylists"><Icon name="playlist" :size="15" /> Ajouter à des playlists</button>
    </div>
  </Transition>
</template>

<style scoped>
.selection-bar {
  position: fixed;
  left: 50%;
  bottom: calc(var(--selection-bottom, 24px) + env(safe-area-inset-bottom, 0px));
  z-index: 46;
  display: flex;
  align-items: center;
  gap: var(--space-2);
  max-width: calc(100vw - 24px);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-lg);
  background: rgba(var(--surface-rgb), 0.97);
  box-shadow: var(--shadow), inset 0 0 0 0.5px rgba(var(--fg-rgb), 0.14);
  transform: translateX(-50%);
  overflow-x: auto;
}
.count {
  padding: 0 var(--space-2);
  font-weight: 700;
  white-space: nowrap;
}
.selection-bar .pill {
  white-space: nowrap;
}
.bar-enter-active,
.bar-leave-active {
  transition: transform 0.2s var(--ease), opacity 0.2s;
}
.bar-enter-from,
.bar-leave-to {
  opacity: 0;
  transform: translate(-50%, 20px);
}
@media (max-width: 760px), (pointer: coarse) and (max-width: 1100px) {
  .selection-bar {
    /* Au-dessus du mini-lecteur et des onglets du téléphone. */
    --selection-bottom: 140px;
    flex-wrap: nowrap;
  }
  .selection-bar .pill {
    padding-inline: 10px;
  }
}
</style>
