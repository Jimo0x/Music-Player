<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { TOUR_MAX_CARDS, tourCards } from "@/lib/whatsNewTour";
import { useUiStore } from "@/stores/ui";
import Icon from "./Icon.vue";

// « Quoi de neuf ? » : après une mise à jour (une fois), depuis les Réglages (tout
// l'historique), ou avant de mettre à jour (notes de la version proposée). Peu de
// nouveautés (lot 0.13.0) : une carte par nouveauté, à faire défiler (flèches, glisser du
// doigt, clavier), avec « Essayer » qui y mène ; l'historique complet reste une liste.
const ui = useUiStore();

const cards = computed(() => (ui.whatsNew ? tourCards(ui.whatsNew.releases) : []));
const listMode = ref(false);
const index = ref(0);
watch(
  () => ui.whatsNew,
  (wn) => {
    index.value = 0;
    listMode.value = !wn || tourCards(wn.releases).length > TOUR_MAX_CARDS;
  },
  { immediate: true },
);
const card = computed(() => cards.value[index.value] ?? null);
const last = computed(() => index.value >= cards.value.length - 1);
/** Sens du dernier passage, pour l'animation (la carte arrive du bon côté). */
const direction = ref<"next" | "prev">("next");

function goTo(i: number) {
  if (i < 0 || i >= cards.value.length || i === index.value) return;
  direction.value = i > index.value ? "next" : "prev";
  index.value = i;
}
function next() {
  if (last.value) ui.closeWhatsNew();
  else goTo(index.value + 1);
}
function tryIt() {
  const route = card.value?.route;
  ui.closeWhatsNew();
  if (route) ui.go(route);
}
function key(e: KeyboardEvent) {
  if (listMode.value) return;
  if (e.key === "ArrowRight") goTo(index.value + 1);
  else if (e.key === "ArrowLeft") goTo(index.value - 1);
  else return;
  e.preventDefault();
}
let startX: number | null = null;
function swipeStart(e: PointerEvent) {
  startX = e.clientX;
}
function swipeEnd(e: PointerEvent) {
  if (startX === null) return;
  const dx = e.clientX - startX;
  startX = null;
  if (Math.abs(dx) < 50) return;
  if (dx < 0) goTo(index.value + 1);
  else goTo(index.value - 1);
}

function date(iso: string): string {
  const d = new Date(`${iso}T12:00:00`);
  return Number.isNaN(d.getTime()) ? "" : d.toLocaleDateString("fr-FR", { day: "numeric", month: "long", year: "numeric" });
}
</script>

<template>
  <Transition name="dialog">
    <div v-if="ui.whatsNew" class="wn-scrim" @click.self="ui.closeWhatsNew()">
      <section class="wn-card" role="dialog" aria-modal="true" aria-labelledby="wn-title" tabindex="-1" @keydown="key">
        <header>
          <span class="wn-icon" aria-hidden="true"><Icon name="sparkle" :size="22" /></span>
          <h2 id="wn-title">{{ ui.whatsNew.title }}</h2>
          <button type="button" class="icon-btn" aria-label="Fermer" @click="ui.closeWhatsNew()"><Icon name="close" :size="16" /></button>
        </header>
        <div v-if="!listMode && card" class="tour" @pointerdown="swipeStart" @pointerup="swipeEnd" @pointercancel="swipeEnd">
          <Transition :name="`slide-${direction}`" mode="out-in">
            <article :key="index" class="tour-card" aria-live="polite">
              <span class="tour-art" aria-hidden="true">
                <span class="ring r1" /><span class="ring r2" />
                <Icon :name="card.icon" :size="40" />
              </span>
              <span class="tour-version">Version {{ card.version }}</span>
              <h3>{{ card.title }}</h3>
              <p>{{ card.text }}</p>
            </article>
          </Transition>
          <div class="dots" role="tablist" aria-label="Nouveautés">
            <button
              v-for="(c, i) in cards"
              :key="i"
              type="button"
              role="tab"
              class="dot"
              :class="{ on: i === index }"
              :aria-selected="i === index"
              :aria-label="`Nouveauté ${i + 1} sur ${cards.length} : ${c.title}`"
              @click="goTo(i)"
            />
          </div>
        </div>
        <div v-else class="wn-body">
          <article v-for="r in ui.whatsNew.releases" :key="r.version" class="wn-release">
            <h3>
              Version {{ r.version }}<small v-if="date(r.date)"> · {{ date(r.date) }}</small>
            </h3>
            <ul>
              <li v-for="(item, i) in r.items" :key="i">{{ item }}</li>
            </ul>
          </article>
        </div>
        <footer v-if="!listMode && card">
          <button type="button" class="link small" @click="listMode = true">Tout voir</button>
          <span class="grow" />
          <button type="button" class="icon-btn" :disabled="index === 0" aria-label="Nouveauté précédente" @click="goTo(index - 1)"><Icon name="prev" :size="16" /></button>
          <button v-if="card.route" type="button" class="pill pill-ghost" @click="tryIt">Essayer</button>
          <button type="button" class="pill pill-accent" @click="next">{{ last ? "Compris" : "Suivante" }}</button>
        </footer>
        <footer v-else>
          <button v-if="cards.length && cards.length <= TOUR_MAX_CARDS" type="button" class="link small" @click="listMode = false">Une par une</button>
          <span class="grow" />
          <button type="button" class="pill pill-accent" @click="ui.closeWhatsNew()">Compris</button>
        </footer>
      </section>
    </div>
  </Transition>
</template>

<style scoped>
.wn-scrim {
  position: fixed;
  inset: 0;
  z-index: 95;
  display: grid;
  place-items: center;
  padding: calc(var(--safe-top) + 20px) calc(var(--safe-right) + 16px) calc(var(--safe-bottom) + 20px)
    calc(var(--safe-left) + 16px);
  background: rgba(2, 6, 20, 0.72);
}
.wn-card {
  display: flex;
  flex-direction: column;
  width: min(100%, 520px);
  max-height: min(80vh, 640px);
  border-radius: var(--radius-lg, 18px);
  background: var(--bg-elevated, #0d2357);
  border: 1px solid var(--separator);
  box-shadow: 0 24px 60px rgba(0, 0, 0, 0.55);
  color: var(--text);
}
header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-4) var(--space-4) var(--space-2);
}
header h2 {
  flex: 1;
  margin: 0;
  font-size: 19px;
}
.wn-icon {
  display: grid;
  place-items: center;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: var(--accent-soft);
  color: var(--accent);
}
.wn-body {
  flex: 1;
  overflow-y: auto;
  padding: 0 var(--space-4);
}
.wn-release + .wn-release {
  margin-top: var(--space-3);
  padding-top: var(--space-3);
  border-top: 1px solid var(--separator);
}
.wn-release h3 {
  margin: var(--space-2) 0;
  font-size: 15px;
}
.wn-release small {
  color: var(--text-3);
  font-weight: 500;
}
ul {
  display: grid;
  gap: 6px;
  margin: 0;
  padding-left: 18px;
  color: var(--text-2);
  font-size: 14px;
  line-height: 1.45;
}
footer {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4) var(--space-4);
}
.grow {
  flex: 1;
}
.link {
  border: 0;
  background: none;
  color: var(--accent);
  font: inherit;
  cursor: pointer;
}
.small {
  font-size: 13px;
}

/* ---- Cartes une à une ---- */
.tour {
  padding: 0 var(--space-4);
  touch-action: pan-y;
  user-select: none;
}
.tour-card {
  display: grid;
  justify-items: center;
  gap: var(--space-2);
  min-height: 250px;
  padding: var(--space-3) 0 var(--space-2);
  text-align: center;
}
.tour-art {
  position: relative;
  display: grid;
  place-items: center;
  width: 108px;
  height: 108px;
  margin-bottom: var(--space-2);
  border-radius: 50%;
  background: radial-gradient(circle, color-mix(in srgb, var(--accent) 34%, transparent), transparent 70%);
  color: var(--accent);
}
.ring {
  position: absolute;
  inset: 0;
  border-radius: 50%;
  border: 1.5px solid color-mix(in srgb, var(--accent) 45%, transparent);
  animation: wn-pulse 2.6s ease-out infinite;
}
.ring.r2 {
  animation-delay: 1.3s;
}
@keyframes wn-pulse {
  from {
    transform: scale(0.7);
    opacity: 0.9;
  }
  to {
    transform: scale(1.35);
    opacity: 0;
  }
}
.tour-version {
  padding: 2px 10px;
  border-radius: 999px;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 11px;
  font-weight: 600;
}
.tour-card h3 {
  margin: 4px 0 0;
  font-size: 20px;
}
.tour-card p {
  max-width: 400px;
  margin: 0;
  color: var(--text-2);
  font-size: 15px;
  line-height: 1.5;
}
.dots {
  display: flex;
  justify-content: center;
  flex-wrap: wrap;
  gap: 6px;
  padding: var(--space-2) 0 0;
}
.dot {
  width: 8px;
  height: 8px;
  padding: 0;
  border: 0;
  border-radius: 999px;
  background: rgba(var(--fg-rgb), 0.2);
  cursor: pointer;
  transition: width 0.2s ease, background 0.2s ease;
}
.dot.on {
  width: 22px;
  background: var(--accent);
}
.slide-next-enter-active,
.slide-next-leave-active,
.slide-prev-enter-active,
.slide-prev-leave-active {
  transition: opacity 0.22s ease, transform 0.22s ease;
}
.slide-next-enter-from,
.slide-prev-leave-to {
  opacity: 0;
  transform: translateX(40px);
}
.slide-next-leave-to,
.slide-prev-enter-from {
  opacity: 0;
  transform: translateX(-40px);
}
@media (prefers-reduced-motion: reduce) {
  .ring {
    animation: none;
    opacity: 0.4;
  }
  .slide-next-enter-active,
  .slide-next-leave-active,
  .slide-prev-enter-active,
  .slide-prev-leave-active,
  .dot {
    transition: none;
  }
}
</style>
