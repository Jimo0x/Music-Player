<script setup lang="ts">
// Barre de la sélection de plusieurs morceaux : combien, tout cocher, ajouter à des
// playlists, lire, envoyer à un proche, annuler.
import { ref, watch } from "vue";
import { MAX_TRACKS_AT_ONCE, useFriendsStore } from "@/stores/friends";
import { usePlayerStore } from "@/stores/player";
import { useUiStore } from "@/stores/ui";
import FriendAvatar from "@/components/FriendAvatar.vue";
import Icon from "@/components/Icon.vue";

const ui = useUiStore();
const player = usePlayerStore();
const friends = useFriendsStore();

/** Liste des proches ouverte (« Envoyer »). */
const picking = ref(false);
watch(
  () => ui.selecting,
  (on) => {
    if (!on) picking.value = false;
  },
);

/** Sons choisis → discussion avec ce proche, où ils sont déjà joints (#8). */
function sendTo(code: string) {
  if (!ui.selected.length) return;
  if (ui.selected.length > MAX_TRACKS_AT_ONCE) ui.notify(`${MAX_TRACKS_AT_ONCE} sons au plus par envoi : les ${MAX_TRACKS_AT_ONCE} premiers sont joints.`);
  friends.outbox = { code, trackIds: ui.selected.slice(0, MAX_TRACKS_AT_ONCE) };
  picking.value = false;
  ui.clearSelection();
  friends.openChat(code);
}

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
      <button v-if="friends.friends.length" type="button" class="pill pill-ghost" :disabled="!ui.selected.length" :aria-expanded="picking" @click="picking = !picking">
        <Icon name="share" :size="15" /> Envoyer
      </button>
      <button type="button" class="pill pill-accent" :disabled="!ui.selected.length" @click="addToPlaylists"><Icon name="playlist" :size="15" /> Ajouter à des playlists</button>
    </div>
  </Transition>
  <div v-if="ui.selecting && picking" class="send-to" role="dialog" aria-label="Envoyer à un proche">
    <strong class="send-to-title">Envoyer {{ ui.selected.length }} son{{ ui.selected.length > 1 ? "s" : "" }} à…</strong>
    <button v-for="f in friends.friends" :key="f.code" type="button" class="friend" @click="sendTo(f.code)">
      <FriendAvatar :name="f.name" :src="f.avatar" :size="28" />
      <span>{{ f.name }}</span>
    </button>
  </div>
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
/* Liste des proches, posée au-dessus de la barre (la barre défile, elle couperait la liste). */
.send-to {
  position: fixed;
  left: 50%;
  bottom: calc(var(--selection-bottom, 24px) + 64px + env(safe-area-inset-bottom, 0px));
  z-index: 47;
  display: grid;
  gap: 2px;
  width: min(320px, calc(100vw - 24px));
  max-height: 50vh;
  padding: var(--space-2);
  border-radius: var(--radius-lg);
  background: rgba(var(--surface-rgb), 0.98);
  box-shadow: var(--shadow), inset 0 0 0 0.5px rgba(var(--fg-rgb), 0.14);
  transform: translateX(-50%);
  overflow-y: auto;
}
.send-to-title {
  padding: 4px 8px 6px;
  font-size: 13px;
}
.friend {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 8px;
  border: 0;
  border-radius: var(--radius);
  background: none;
  color: var(--text);
  font: inherit;
  text-align: left;
  cursor: pointer;
}
@media (hover: hover) {
  .friend:hover {
    background: rgba(var(--fg-rgb), 0.07);
  }
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
  .send-to {
    --selection-bottom: 140px;
  }
}
</style>
