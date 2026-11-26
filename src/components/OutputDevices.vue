<script setup lang="ts">
// Sortie audio (lot 0.13.0, ordinateur) : le casque, les enceintes, l'écran… chacune en carte
// avec son pictogramme ; un clic y bascule le son, le morceau reprend au même endroit. La
// liste se relit à l'ouverture et au bouton « Actualiser » (casque branché entre-temps).
import { computed, onMounted, ref } from "vue";
import { api } from "@/lib/api";
import { useUiStore } from "@/stores/ui";
import Icon from "./Icon.vue";

const ui = useUiStore();
const devices = ref<string[]>([]);
const chosen = ref<string | null>(null);
const loading = ref(false);
const switching = ref<string | null>(null);

async function refresh() {
  loading.value = true;
  try {
    const list = await api.listOutputDevices();
    devices.value = list.devices;
    chosen.value = list.chosen;
  } catch (e) {
    ui.notify(`Sorties audio illisibles : ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    loading.value = false;
  }
}
onMounted(refresh);

async function choose(name: string | null) {
  if (name === chosen.value) return;
  switching.value = name ?? "";
  try {
    await api.setOutputDevice(name);
    chosen.value = name;
    ui.notify(name ? `Le son passe par « ${name} »` : "Le son suit la sortie du système");
  } catch (e) {
    ui.notify(`Changement impossible : ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    switching.value = null;
  }
}

/** Pictogramme deviné d'après le nom (casque, écran, enceinte). */
function kind(name: string): "headphones" | "desktop" | "volume" {
  const n = name.toLowerCase();
  if (/casque|headphone|headset|écouteur|earbud|airpods|buds|bluetooth/.test(n)) return "headphones";
  if (/hdmi|display|écran|moniteur|monitor|tv|nvidia|displayport/.test(n)) return "desktop";
  return "volume";
}
const missing = computed(() => !!chosen.value && !devices.value.includes(chosen.value));
</script>

<template>
  <div class="outputs">
    <div class="cards" role="radiogroup" aria-label="Sortie audio">
      <button type="button" role="radio" class="card-btn" :class="{ on: !chosen }" :aria-checked="!chosen" :disabled="switching !== null" @click="choose(null)">
        <Icon name="refresh" :size="22" />
        <span class="name">Comme le système</span>
        <span class="hint">Suit la sortie choisie dans Windows</span>
      </button>
      <button
        v-for="d in devices"
        :key="d"
        type="button"
        role="radio"
        class="card-btn"
        :class="{ on: chosen === d, busy: switching === d }"
        :aria-checked="chosen === d"
        :disabled="switching !== null"
        @click="choose(d)"
      >
        <Icon :name="kind(d)" :size="22" />
        <span class="name">{{ d }}</span>
        <span v-if="d === devices[0]" class="hint">Sortie par défaut du système</span>
      </button>
    </div>
    <p v-if="missing" class="warn small">« {{ chosen }} » n'est pas branchée : le son passe par la sortie du système en attendant.</p>
    <button type="button" class="pill pill-ghost" :disabled="loading" @click="refresh"><Icon name="refresh" :size="14" />{{ loading ? "Recherche…" : "Actualiser la liste" }}</button>
  </div>
</template>

<style scoped>
.cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}
.card-btn {
  display: grid;
  justify-items: start;
  gap: 4px;
  padding: 12px;
  border: 0;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.05);
  color: var(--text);
  font: inherit;
  text-align: left;
  transition: background 0.15s ease, transform 0.15s ease;
}
.card-btn .name {
  font-weight: 600;
  font-size: 13px;
  overflow-wrap: anywhere;
}
.card-btn .hint {
  font-size: 11px;
  color: var(--text-3);
}
.card-btn.on {
  background: var(--accent-soft);
  color: var(--accent);
  box-shadow: inset 0 0 0 1.5px var(--accent);
}
.card-btn.busy {
  opacity: 0.6;
}
@media (hover: hover) {
  .card-btn:not(:disabled):hover {
    transform: translateY(-2px);
  }
}
.warn {
  color: var(--danger-text);
}
@media (prefers-reduced-motion: reduce) {
  .card-btn {
    transition: none;
  }
}
</style>
