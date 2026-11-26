<script setup lang="ts">
// « Ajouter à des playlists » : un ou plusieurs morceaux (sélection, morceau en cours, menu
// d'un morceau) ajoutés à une ou plusieurs playlists d'un coup, avec création sur place.
import { computed, nextTick, ref, watch } from "vue";
import { usePlaylistsStore } from "@/stores/playlists";
import { useUiStore } from "@/stores/ui";
import PlaylistCover from "@/components/PlaylistCover.vue";
import Icon from "@/components/Icon.vue";

const ui = useUiStore();
const playlists = usePlaylistsStore();

const ids = computed(() => ui.addToPlaylists ?? []);
const chosen = ref<string[]>([]);
const newName = ref("");
const creating = ref(false);
const saving = ref(false);
const nameInput = ref<HTMLInputElement | null>(null);

watch(
  () => ui.addToPlaylists,
  (open) => {
    if (!open) return;
    chosen.value = [];
    newName.value = "";
  },
);

/** Morceaux déjà présents dans une playlist : « Déjà dedans » ou « 3 sur 5 déjà dedans ». */
function already(trackIds: string[]): string {
  const inside = ids.value.filter((id) => trackIds.includes(id)).length;
  if (!inside) return "";
  return inside === ids.value.length ? "Déjà dedans" : `${inside} sur ${ids.value.length} déjà dedans`;
}

function toggle(id: string) {
  chosen.value = chosen.value.includes(id) ? chosen.value.filter((x) => x !== id) : [...chosen.value, id];
}

async function startCreate() {
  creating.value = true;
  await nextTick();
  nameInput.value?.focus();
}

async function create() {
  const name = newName.value.trim();
  if (!name) return;
  const id = await playlists.create(name);
  if (id) {
    chosen.value = [...chosen.value, id];
    newName.value = "";
    creating.value = false;
  }
}

function close() {
  ui.addToPlaylists = null;
}

async function add() {
  if (!chosen.value.length || saving.value) return;
  saving.value = true;
  let added = 0;
  for (const id of chosen.value) added += await playlists.addTracks(id, ids.value);
  saving.value = false;
  const n = chosen.value.length;
  ui.notify(
    added
      ? `${added} ajout${added > 1 ? "s" : ""} dans ${n} playlist${n > 1 ? "s" : ""}`
      : "Ces morceaux étaient déjà dans ces playlists",
  );
  close();
  ui.clearSelection();
}
</script>

<template>
  <div v-if="ui.addToPlaylists" class="scrim" @mousedown.self="close">
    <div class="dialog" role="dialog" aria-modal="true" aria-label="Ajouter à des playlists">
      <header class="head">
        <h2>Ajouter à des playlists</h2>
        <p class="muted">{{ ids.length }} morceau{{ ids.length > 1 ? "x" : "" }} · plusieurs playlists possibles</p>
      </header>

      <ul class="list" role="listbox" aria-multiselectable="true">
        <li v-if="creating" class="create">
          <form @submit.prevent="create">
            <input ref="nameInput" v-model="newName" class="fieldbox" maxlength="80" placeholder="Nom de la playlist" aria-label="Nom de la nouvelle playlist" />
            <button type="submit" class="pill pill-accent" :disabled="!newName.trim()">Créer</button>
          </form>
        </li>
        <li v-else>
          <button type="button" class="row new" @click="startCreate">
            <span class="cover plus"><Icon name="plus" :size="18" /></span>
            <span class="name">Nouvelle playlist</span>
          </button>
        </li>
        <li v-for="p in playlists.items" :key="p.id">
          <button type="button" class="row" role="option" :aria-selected="chosen.includes(p.id)" :class="{ on: chosen.includes(p.id) }" @click="toggle(p.id)">
            <span class="cover"><PlaylistCover :name="p.name" :theme="p.theme" :styling="p.style" :track-ids="p.track_ids" :radius="6" mini /></span>
            <span class="text">
              <span class="name">{{ p.name }}</span>
              <span class="sub">{{ already(p.track_ids) || `${p.track_ids.length} morceau${p.track_ids.length > 1 ? "x" : ""}` }}</span>
            </span>
            <span class="box" :class="{ on: chosen.includes(p.id) }" aria-hidden="true"><Icon v-if="chosen.includes(p.id)" name="check" :size="14" /></span>
          </button>
        </li>
      </ul>

      <footer class="actions">
        <button type="button" class="pill pill-ghost" @click="close">Annuler</button>
        <button type="button" class="pill pill-accent" :disabled="!chosen.length || saving" @click="add">
          {{ saving ? "Ajout…" : chosen.length > 1 ? `Ajouter à ${chosen.length} playlists` : "Ajouter" }}
        </button>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.scrim {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.45);
  backdrop-filter: blur(6px);
}
.dialog {
  display: flex;
  flex-direction: column;
  width: min(440px, calc(100vw - 32px));
  max-height: min(640px, calc(100vh - 40px));
  border-radius: var(--radius-lg);
  background: rgba(var(--surface-rgb), 0.97);
  box-shadow: var(--shadow), inset 0 0 0 0.5px rgba(var(--fg-rgb), 0.12);
  overflow: hidden;
}
.head {
  padding: var(--space-5) var(--space-5) var(--space-3);
}
.head h2 {
  margin: 0 0 var(--space-1);
  font-size: 18px;
}
.head p {
  margin: 0;
  font-size: 13px;
}
.list {
  flex: 1;
  min-height: 0;
  margin: 0;
  padding: 0 var(--space-3);
  overflow-y: auto;
  list-style: none;
}
.row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  width: 100%;
  padding: var(--space-2);
  border: 0;
  border-radius: var(--radius);
  background: none;
  color: var(--text);
  text-align: left;
}
.row:hover {
  background: var(--bg-hover);
}
.row.on {
  background: var(--accent-soft);
}
.cover {
  flex: 0 0 auto;
  width: 44px;
  height: 44px;
  border-radius: var(--radius-sm);
  overflow: hidden;
}
.cover.plus {
  display: grid;
  place-items: center;
  background: rgba(var(--fg-rgb), 0.08);
  color: var(--accent);
}
.text {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-width: 0;
}
.name {
  overflow: hidden;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.new .name {
  color: var(--accent);
}
.sub {
  font-size: 12px;
  color: var(--text-2);
}
.box {
  display: grid;
  place-items: center;
  flex: 0 0 auto;
  width: 22px;
  height: 22px;
  border-radius: var(--radius-sm);
  box-shadow: inset 0 0 0 2px rgba(var(--fg-rgb), 0.3);
  color: var(--on-accent);
}
.box.on {
  background: var(--accent);
  box-shadow: none;
}
.create form {
  display: flex;
  gap: var(--space-2);
  padding: var(--space-2);
}
.create input {
  flex: 1;
  min-width: 0;
}
.actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-5) var(--space-5);
  border-top: 1px solid var(--separator);
}
</style>
