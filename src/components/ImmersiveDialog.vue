<script setup lang="ts">
// Fenêtre du mode immersif, ouverte depuis « À l'écoute » (réglages qui jouent maintenant)
// ou depuis une playlist (son ambiance propre).
import { computed, onBeforeUnmount, onMounted } from "vue";
import Icon from "@/components/Icon.vue";
import Immersive from "@/components/Immersive.vue";
import { useImmersiveStore } from "@/stores/immersive";
import { usePlaylistsStore } from "@/stores/playlists";

const props = withDefaults(defineProps<{ playlistId?: string | null }>(), { playlistId: null });
const emit = defineEmits<{ (e: "close"): void }>();

const im = useImmersiveStore();
const playlists = usePlaylistsStore();
/** Depuis « À l'écoute » : on règle ce qui joue, donc l'ambiance de la playlist en cours si elle en a une. */
const target = computed(() => props.playlistId ?? (im.fromPlaylist ? im.profiles.context : null));
const targetName = computed(() => (target.value ? playlists.byId(target.value)?.name : null));

function onKey(e: KeyboardEvent) {
  if (e.key === "Escape") emit("close");
}
onMounted(() => window.addEventListener("keydown", onKey));
onBeforeUnmount(() => window.removeEventListener("keydown", onKey));
</script>

<template>
  <div class="overlay" role="presentation" @click.self="emit('close')">
    <section class="dialog" role="dialog" aria-modal="true" aria-labelledby="immersive-title">
      <header class="dialog-header">
        <h2 id="immersive-title"><Icon name="orbit" :size="18" /> Mode immersif<small v-if="targetName"> · {{ targetName }}</small></h2>
        <button class="icon-btn" type="button" title="Fermer" aria-label="Fermer" @click="emit('close')"><Icon name="close" :size="17" /></button>
      </header>
      <div class="dialog-body">
        <p v-if="!playlistId && target" class="muted small note">Ce morceau suit l'ambiance propre à sa playlist : vos changements s'y appliquent.</p>
        <Immersive :playlist-id="target" compact />
      </div>
    </section>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-4, 16px);
  background: rgba(0, 0, 0, 0.55);
}
.dialog {
  display: flex;
  flex-direction: column;
  width: min(560px, 100%);
  max-height: min(88vh, 900px);
  overflow: hidden;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--bg-elevated);
  color: var(--text);
}
.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px 12px 18px;
  border-bottom: 1px solid var(--border);
}
.dialog-header h2 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}
.dialog-header small {
  color: var(--text-2);
  font-weight: 500;
}
.dialog-body {
  padding: 14px 18px 18px;
  overflow-y: auto;
  overscroll-behavior: contain;
}
.note {
  margin: 0 0 var(--space-2);
}
@media (max-width: 600px) {
  .overlay {
    align-items: flex-end;
    padding: 0;
  }
  .dialog {
    width: 100%;
    max-height: 92vh;
    border-radius: var(--radius) var(--radius) 0 0;
    padding-bottom: env(safe-area-inset-bottom);
  }
}
</style>
