<script setup lang="ts">
// Mode immersif (lot 0.14.0) : une tête vue de dessus, le son qui tourne autour (8D) à la
// vitesse et dans le sens réglés, et la salle dessinée en ondes qui s'éloignent (taille,
// dosage). Même esprit que l'égaliseur : on règle en voyant, « Comparer » maintenu fait
// entendre le son sans effet. Avec `playlistId`, le panneau règle l'ambiance propre à cette
// playlist (appliquée quand la lecture est lancée depuis elle).
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { matchingRoom, PREDELAY_MAX, ROOM_PRESETS, SPEED_MAX, SPEED_MIN, useImmersiveStore } from "@/stores/immersive";
import type { ImmersiveSettings } from "@/lib/types";

const props = withDefaults(defineProps<{ playlistId?: string | null; compact?: boolean }>(), { playlistId: null, compact: false });

const im = useImmersiveStore();
const s = computed<ImmersiveSettings>(() => im.settingsFor(props.playlistId));
const own = computed(() => !!props.playlistId && im.hasOwn(props.playlistId));
/** Une playlist sans ambiance propre suit les réglages globaux : rien à régler ici. */
const editable = computed(() => !props.playlistId || own.value);
const set = (patch: Partial<ImmersiveSettings>) => im.update(props.playlistId, patch);
const room = computed(() => matchingRoom(s.value));

// ---- Animation de la vue de dessus (même vitesse que le son, sans être synchronisée) ----
const reduced = typeof window !== "undefined" && !!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;
const angle = ref(Math.PI / 4); // sens trigonométrique, 0 = devant (en haut du dessin)
let raf = 0;
let last = 0;
function tick(t: number) {
  raf = requestAnimationFrame(tick);
  const dt = last ? Math.min(0.1, (t - last) / 1000) : 0;
  last = t;
  const v = s.value;
  if (!v.enabled || !v.rotation || im.bypass) return;
  angle.value += (v.clockwise ? -1 : 1) * ((2 * Math.PI * v.speed) / 60) * dt;
}
onMounted(() => {
  if (!reduced) raf = requestAnimationFrame(tick);
});
onBeforeUnmount(() => {
  cancelAnimationFrame(raf);
  void im.setBypass(false);
});

// Repère du dessin : 300 × 300, tête au centre, « devant » en haut.
const C = 150;
const orbitR = computed(() => 62 + 50 * s.value.intensity);
const source = computed(() => ({ x: C - Math.sin(angle.value) * orbitR.value, y: C - Math.cos(angle.value) * orbitR.value }));
const active = computed(() => s.value.enabled && !im.bypass);
const roomR = computed(() => 84 + 46 * s.value.room_size);
/** Durée d'une onde de salle : plus longue dans une grande salle, comme la résonance. */
const waveSecs = computed(() => (1.6 + 3 * s.value.room_size).toFixed(2));

const pct = (v: number) => `${Math.round(v * 100)} %`;
const num = (e: Event) => +(e.target as HTMLInputElement).value;
</script>

<template>
  <div class="immersive" :class="{ off: !active, compact }">
    <div v-if="playlistId" class="own">
      <label class="toggle">
        <input type="checkbox" :checked="own" @change="im.setPlaylistOwn(playlistId, ($event.target as HTMLInputElement).checked)" />
        <span>Ambiance propre à cette playlist</span>
      </label>
      <p class="muted small">
        {{ own ? "Elle s'applique quand vous lancez la lecture depuis cette playlist, puis les réglages habituels reviennent." : "Sinon, la playlist suit les réglages habituels du mode immersif (Réglages)." }}
      </p>
    </div>

    <template v-if="editable">
      <label class="toggle master">
        <input type="checkbox" :checked="s.enabled" @change="set({ enabled: ($event.target as HTMLInputElement).checked })" />
        <span><strong>Mode immersif</strong> : le son tourne autour de vous et prend l'acoustique d'une salle. Au casque, c'est saisissant.</span>
      </label>

      <div class="stage" aria-hidden="true">
        <svg viewBox="0 0 300 300">
          <defs>
            <radialGradient id="im-glow">
              <stop offset="0%" stop-color="var(--accent)" stop-opacity="0.9" />
              <stop offset="100%" stop-color="var(--accent)" stop-opacity="0" />
            </radialGradient>
          </defs>
          <!-- Salle : contour et ondes qui s'éloignent -->
          <g v-if="s.room" class="room" :style="{ opacity: 0.35 + s.room_mix }">
            <rect :x="C - roomR" :y="C - roomR" :width="roomR * 2" :height="roomR * 2" :rx="roomR * (0.5 - s.room_damping * 0.35)" class="walls" />
            <circle v-for="i in 3" :key="i" :cx="C" :cy="C" r="40" class="wave" :style="{ animationDuration: `${waveSecs}s`, animationDelay: `${(-(i - 1) * +waveSecs) / 3}s`, '--to': roomR / 40 }" />
          </g>
          <!-- Orbite du son -->
          <circle v-if="s.rotation" :cx="C" :cy="C" :r="orbitR" class="orbit" />
          <!-- Tête vue de dessus : nez vers l'avant (en haut), deux oreilles -->
          <g class="head">
            <ellipse :cx="C - 33" :cy="C" rx="6" ry="11" />
            <ellipse :cx="C + 33" :cy="C" rx="6" ry="11" />
            <path :d="`M${C - 7} ${C - 34} L${C} ${C - 44} L${C + 7} ${C - 34} Z`" />
            <ellipse :cx="C" :cy="C" rx="31" ry="36" />
          </g>
          <text :x="C" y="12" class="front">devant</text>
          <!-- Source sonore -->
          <g v-if="s.rotation" class="source">
            <circle :cx="source.x" :cy="source.y" r="22" fill="url(#im-glow)" />
            <circle :cx="source.x" :cy="source.y" r="7" class="dot" />
          </g>
        </svg>
        <span class="badge">{{ s.rotation ? (s.engine === "hrtf" ? "Tête mesurée" : "Tête simplifiée") : "Sans rotation" }}<template v-if="s.room"> · {{ room?.name ?? "Salle sur mesure" }}</template></span>
      </div>

      <section class="block" :class="{ dim: !s.rotation }">
        <label class="toggle">
          <input type="checkbox" :checked="s.rotation" @change="set({ rotation: ($event.target as HTMLInputElement).checked })" />
          <span><strong>Rotation 8D</strong> : le son fait le tour de votre tête.</span>
        </label>
        <div class="seg wide" role="radiogroup" aria-label="Moteur de la rotation">
          <button type="button" role="radio" :aria-checked="s.engine === 'head'" :class="{ on: s.engine === 'head' }" @click="set({ engine: 'head' })">Tête simplifiée</button>
          <button type="button" role="radio" :aria-checked="s.engine === 'hrtf'" :class="{ on: s.engine === 'hrtf' }" @click="set({ engine: 'hrtf' })">Tête mesurée (HRTF)</button>
        </div>
        <p class="muted small">
          {{ s.engine === "hrtf" ? "Mesures d'une vraie tête : devant et derrière se distinguent mieux. Un peu plus de calcul." : "Calculée : décalage et ombre entre les deux oreilles. Très légère, idéale sur un téléphone modeste." }}
        </p>
        <label class="field">
          <span>Vitesse <em>{{ s.speed }} tours/min</em></span>
          <input type="range" :min="SPEED_MIN" :max="SPEED_MAX" step="1" :value="s.speed" @input="set({ speed: num($event) })" />
        </label>
        <label class="field">
          <span>Intensité <em>{{ pct(s.intensity) }}</em></span>
          <input type="range" min="0" max="100" :value="Math.round(s.intensity * 100)" @input="set({ intensity: num($event) / 100 })" />
        </label>
        <div class="seg" role="radiogroup" aria-label="Sens de rotation">
          <button type="button" role="radio" :aria-checked="s.clockwise" :class="{ on: s.clockwise }" @click="set({ clockwise: true })">↻ Horaire</button>
          <button type="button" role="radio" :aria-checked="!s.clockwise" :class="{ on: !s.clockwise }" @click="set({ clockwise: false })">↺ Antihoraire</button>
        </div>
      </section>

      <section class="block" :class="{ dim: !s.room }">
        <label class="toggle">
          <input type="checkbox" :checked="s.room" @change="set({ room: ($event.target as HTMLInputElement).checked })" />
          <span><strong>Acoustique de salle</strong> : la musique résonne comme dans un vrai lieu.</span>
        </label>
        <div class="rooms" role="radiogroup" aria-label="Salle">
          <button
            v-for="p in ROOM_PRESETS"
            :key="p.id"
            type="button"
            role="radio"
            class="room-card"
            :class="{ on: s.room && room?.id === p.id }"
            :aria-checked="s.room && room?.id === p.id"
            :title="p.hint"
            @click="im.applyRoom(playlistId, p)"
          >
            <svg viewBox="0 0 60 28" aria-hidden="true">
              <rect :x="30 - 6 - p.size * 22" :y="14 - 3 - p.size * 9" :width="(6 + p.size * 22) * 2" :height="(3 + p.size * 9) * 2" :rx="(3 + p.size * 9) * (1 - p.damping * 0.8)" />
              <circle cx="30" cy="14" r="2.6" />
            </svg>
            <span>{{ p.name }}</span>
          </button>
        </div>
        <label class="field">
          <span>Taille de la salle <em>{{ pct(s.room_size) }}</em></span>
          <input type="range" min="0" max="100" :value="Math.round(s.room_size * 100)" @input="set({ room_size: num($event) / 100 })" />
        </label>
        <label class="field">
          <span>Dosage <em>{{ pct(s.room_mix) }}</em></span>
          <input type="range" min="0" max="100" :value="Math.round(s.room_mix * 100)" @input="set({ room_mix: num($event) / 100 })" />
        </label>
        <label class="field">
          <span>Murs mats <em>{{ pct(s.room_damping) }}</em></span>
          <input type="range" min="0" max="100" :value="Math.round(s.room_damping * 100)" @input="set({ room_damping: num($event) / 100 })" />
        </label>
        <label class="field">
          <span>Distance des murs <em>{{ Math.round(s.room_predelay_ms) }} ms</em></span>
          <input type="range" min="0" :max="PREDELAY_MAX" :value="s.room_predelay_ms" @input="set({ room_predelay_ms: num($event) })" />
        </label>
      </section>

      <div class="actions">
        <button
          type="button"
          class="pill pill-ghost compare"
          :class="{ on: im.bypass }"
          :disabled="!s.enabled"
          @pointerdown="im.setBypass(true)"
          @pointerup="im.setBypass(false)"
          @pointerleave="im.setBypass(false)"
          @pointercancel="im.setBypass(false)"
          @keydown.space.prevent="im.setBypass(true)"
          @keyup.space.prevent="im.setBypass(false)"
        >
          {{ im.bypass ? "Son d'origine…" : "Maintenir pour comparer" }}
        </button>
      </div>
      <p v-if="!compact" class="muted small credit">
        Tête mesurée : base SADIE II, AudioLab de l'Université d'York (doi.org/10.3390/app8112029), licence Apache 2.0.
      </p>
    </template>
  </div>
</template>

<style scoped>
.immersive {
  display: grid;
  gap: var(--space-3);
}
.own {
  padding: var(--space-3);
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.04);
}
.own .small {
  margin: 6px 0 0 26px;
}
.stage {
  position: relative;
  display: grid;
  place-items: center;
  height: 260px;
  border-radius: var(--radius);
  background: radial-gradient(circle at 50% 50%, color-mix(in srgb, var(--accent) 16%, transparent), transparent 65%), rgba(var(--fg-rgb), 0.03);
  overflow: hidden;
}
.compact .stage {
  height: 200px;
}
.stage svg {
  height: 100%;
  aspect-ratio: 1;
}
.walls {
  fill: none;
  stroke: rgba(var(--fg-rgb), 0.22);
  stroke-width: 1.5;
  stroke-dasharray: 5 5;
  transition: all 0.3s var(--ease);
}
.wave {
  fill: none;
  stroke: var(--accent-2, var(--accent));
  stroke-width: 1.2;
  vector-effect: non-scaling-stroke;
  transform-origin: 150px 150px;
  animation: wave linear infinite;
  opacity: 0;
}
@keyframes wave {
  0% {
    transform: scale(1);
    opacity: 0.55;
  }
  100% {
    transform: scale(var(--to));
    opacity: 0;
  }
}
.orbit {
  fill: none;
  stroke: rgba(var(--fg-rgb), 0.16);
  stroke-width: 1.5;
  stroke-dasharray: 2 6;
  stroke-linecap: round;
  transition: r 0.3s var(--ease);
}
.head ellipse,
.head path {
  fill: var(--bg-content);
  stroke: rgba(var(--fg-rgb), 0.45);
  stroke-width: 2;
}
.front {
  fill: var(--text-3);
  font-size: 11px;
  text-anchor: middle;
}
.dot {
  fill: var(--accent);
  stroke: var(--bg-content);
  stroke-width: 2;
}
.off .source,
.off .room {
  opacity: 0.25 !important;
}
.off .wave {
  animation-play-state: paused;
}
.badge {
  position: absolute;
  bottom: 10px;
  left: 50%;
  transform: translateX(-50%);
  padding: 3px 10px;
  border-radius: 999px;
  background: rgba(var(--fg-rgb), 0.08);
  color: var(--text-2);
  font-size: 12px;
  white-space: nowrap;
}
.block {
  display: grid;
  gap: var(--space-2);
  padding: var(--space-3);
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.03);
}
.block.dim > :not(.toggle) {
  opacity: 0.45;
}
.field {
  display: grid;
  gap: 4px;
  font-size: 13px;
}
.field span {
  display: flex;
  justify-content: space-between;
  color: var(--text-2);
}
.field em {
  font-style: normal;
  color: var(--text);
  font-variant-numeric: tabular-nums;
}
.field input {
  width: 100%;
  accent-color: var(--accent);
}
.seg {
  display: inline-flex;
  flex-wrap: wrap;
  gap: var(--space-1);
  padding: 3px;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.06);
  justify-self: start;
}
.seg.wide {
  display: flex;
  justify-self: stretch;
}
.seg.wide button {
  flex: 1 1 auto;
}
.seg button {
  height: 32px;
  padding: 0 14px;
  border: 0;
  border-radius: 8px;
  background: none;
  font-size: 13px;
  color: var(--text-2);
}
.seg button.on {
  background: var(--accent);
  color: var(--on-accent);
}
.rooms {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(76px, 1fr));
  gap: 6px;
}
.room-card {
  display: grid;
  gap: 2px;
  padding: 6px 6px 8px;
  border: 0;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.05);
  color: var(--text-2);
  font: inherit;
  font-size: 12px;
  transition: background 0.15s ease, transform 0.15s ease;
}
@media (max-width: 600px) {
  .rooms {
    grid-template-columns: repeat(3, 1fr);
  }
}
.room-card svg {
  width: 100%;
  height: 28px;
}
.room-card rect {
  fill: none;
  stroke: currentColor;
  stroke-width: 1.6;
}
.room-card circle {
  fill: currentColor;
}
.room-card.on {
  background: var(--accent-soft);
  color: var(--accent);
  box-shadow: inset 0 0 0 1.5px var(--accent);
}
@media (hover: hover) {
  .room-card:hover {
    transform: translateY(-2px);
  }
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}
.compare.on {
  background: var(--accent);
  color: var(--on-accent);
}
.credit {
  margin: 0;
}
@media (prefers-reduced-motion: reduce) {
  .wave {
    animation: none;
    opacity: 0.3;
  }
  .walls,
  .orbit,
  .room-card {
    transition: none;
  }
}
</style>
