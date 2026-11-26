<script setup lang="ts">
// Arrière-plan de l'application (lot 0.12.0) : l'image choisie ou la pochette du morceau en
// cours, sous toute l'interface (.app l'isole : le calque passe sous les panneaux, au-dessus du
// fond uni). Un voile de la couleur de fond du thème garde le texte lisible ; le mode léger
// retire le flou.
import { computed, ref, watch } from "vue";
import { coverUrl } from "@/lib/covers";
import { useAppearanceStore } from "@/stores/appearance";
import { usePlayerStore } from "@/stores/player";

const appearance = useAppearanceStore();
const player = usePlayerStore();

const b = computed(() => appearance.settings.backdrop);
const artwork = ref<string | null>(null);

watch(
  () => (b.value.source === "artwork" ? player.currentTrack : null),
  async (track) => {
    const id = track?.id ?? null;
    const url = track ? await coverUrl(track) : null;
    // Un autre morceau a pu partir pendant la lecture de la pochette.
    if ((player.currentTrack?.id ?? null) === id || !track) artwork.value = url;
  },
  { immediate: true },
);

const url = computed(() => {
  if (b.value.source === "image" && b.value.image) return appearance.assetUrl(b.value.image);
  if (b.value.source === "artwork") return artwork.value;
  return null;
});

const imageStyle = computed(() => ({
  backgroundImage: url.value ? `url("${url.value}")` : "none",
  backgroundPosition: `${b.value.x}% ${b.value.y}%`,
  "--backdrop-blur": `${b.value.blur}px`,
}));
const veilStyle = computed(() => ({ opacity: String(b.value.dim) }));
</script>

<template>
  <div v-if="b.source !== 'none'" class="app-backdrop" aria-hidden="true">
    <Transition name="backdrop-fade">
      <div v-if="url" :key="url" class="image" :style="imageStyle" />
    </Transition>
    <div class="veil" :style="veilStyle" />
  </div>
</template>

<style scoped>
.app-backdrop {
  position: fixed;
  inset: 0;
  z-index: -1;
  overflow: hidden;
  pointer-events: none;
  background: var(--bg-content);
}
.image {
  position: absolute;
  /* Débord : le flou ne laisse pas de bord clair. */
  inset: calc(var(--backdrop-blur, 0px) * -2);
  background-size: cover;
  background-repeat: no-repeat;
  filter: blur(var(--backdrop-blur, 0px));
}
.veil {
  position: absolute;
  inset: 0;
  background: var(--bg-content);
}
.backdrop-fade-enter-active,
.backdrop-fade-leave-active {
  transition: opacity 0.6s ease;
}
.backdrop-fade-enter-from,
.backdrop-fade-leave-to {
  opacity: 0;
}
:global(:root[data-perf="lite"]) .image {
  inset: 0;
  filter: none;
}
@media (prefers-reduced-motion: reduce) {
  .backdrop-fade-enter-active,
  .backdrop-fade-leave-active {
    transition: none;
  }
}
</style>
