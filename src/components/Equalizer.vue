<script setup lang="ts">
// Égaliseur ludique (lot 0.13.0) : une courbe qu'on façonne du doigt ou de la souris (une
// poignée par bande, flèches au clavier), le spectre du son en direct derrière, des
// préréglages illustrés par leur propre courbe, et « Comparer » : maintenu, on entend le son
// sans égaliseur. Le spectre n'est relevé que pendant l'affichage (useSpectrum).
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { BANDS as SPECTRUM_BANDS, useSpectrum } from "@/lib/spectrum";
import { BAND_FREQS, BAND_LABELS, EQ_MAX, EQ_MIN, EQ_PRESETS, useEqStore, type Gains } from "@/stores/eq";

const eq = useEqStore();
const levels = useSpectrum();

// Repère de dessin : 1000 × 300, marges pour les étiquettes.
const W = 1000;
const H = 300;
const PAD = { left: 34, right: 18, top: 16, bottom: 34 };
const plotW = W - PAD.left - PAD.right;
const plotH = H - PAD.top - PAD.bottom;
const x = (i: number) => PAD.left + (i / (BAND_FREQS.length - 1)) * plotW;
const y = (db: number) => PAD.top + ((EQ_MAX - db) / (EQ_MAX - EQ_MIN)) * plotH;

/** Courbe lisse (Catmull-Rom → Bézier) passant par chaque point. */
function smoothPath(points: [number, number][]): string {
  if (!points.length) return "";
  let d = `M${points[0][0].toFixed(1)} ${points[0][1].toFixed(1)}`;
  for (let i = 0; i < points.length - 1; i++) {
    const p0 = points[i - 1] ?? points[i];
    const p1 = points[i];
    const p2 = points[i + 1];
    const p3 = points[i + 2] ?? p2;
    const c1 = [p1[0] + (p2[0] - p0[0]) / 6, p1[1] + (p2[1] - p0[1]) / 6];
    const c2 = [p2[0] - (p3[0] - p1[0]) / 6, p2[1] - (p3[1] - p1[1]) / 6];
    d += ` C${c1[0].toFixed(1)} ${c1[1].toFixed(1)} ${c2[0].toFixed(1)} ${c2[1].toFixed(1)} ${p2[0].toFixed(1)} ${p2[1].toFixed(1)}`;
  }
  return d;
}

const points = computed<[number, number][]>(() => eq.gains.map((g, i) => [x(i), y(g)]));
const curve = computed(() => smoothPath(points.value));
const area = computed(() => `${curve.value} L${x(BAND_FREQS.length - 1)} ${y(0)} L${x(0)} ${y(0)} Z`);

/** Miniature d'un préréglage (même courbe, dans 100 × 36). */
function miniPath(gains: Gains): string {
  const pts = gains.map((g, i) => [4 + (i / (gains.length - 1)) * 92, 18 - (g / EQ_MAX) * 14] as [number, number]);
  return smoothPath(pts);
}

// ---- Spectre en direct (32 bandes de 40 Hz à 16 kHz, rafraîchies à l'image) ----
const spectrumBars = ref<number[]>(Array(SPECTRUM_BANDS).fill(0));
let raf = 0;
function tick() {
  raf = requestAnimationFrame(tick);
  spectrumBars.value = Array.from(levels);
}
onMounted(() => {
  if (!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches) raf = requestAnimationFrame(tick);
});
onBeforeUnmount(() => {
  cancelAnimationFrame(raf);
  void eq.setBypass(false);
});
const barW = plotW / SPECTRUM_BANDS;

// ---- Poignées : glisser (souris, doigt) et clavier ----
const svg = ref<SVGSVGElement | null>(null);
const active = ref<number | null>(null);
function dbAt(e: PointerEvent): number {
  const rect = svg.value!.getBoundingClientRect();
  const yy = ((e.clientY - rect.top) / rect.height) * H;
  return EQ_MAX - ((yy - PAD.top) / plotH) * (EQ_MAX - EQ_MIN);
}
function nearestBand(e: PointerEvent): number {
  const rect = svg.value!.getBoundingClientRect();
  const xx = ((e.clientX - rect.left) / rect.width) * W;
  return Math.max(0, Math.min(BAND_FREQS.length - 1, Math.round(((xx - PAD.left) / plotW) * (BAND_FREQS.length - 1))));
}
function down(e: PointerEvent) {
  (e.currentTarget as Element).setPointerCapture(e.pointerId);
  active.value = nearestBand(e);
  void eq.setGain(active.value, dbAt(e));
}
function move(e: PointerEvent) {
  if (active.value === null) return;
  void eq.setGain(active.value, dbAt(e));
}
function up() {
  active.value = null;
}
function key(i: number, e: KeyboardEvent) {
  const step = e.shiftKey ? 3 : 1;
  const next = { ArrowUp: eq.gains[i] + step, ArrowDown: eq.gains[i] - step, Home: EQ_MAX, End: EQ_MIN, Delete: 0, Backspace: 0 }[e.key];
  if (next === undefined) return;
  e.preventDefault();
  void eq.setGain(i, next);
}
const fmt = (g: number) => `${g > 0 ? "+" : ""}${g} dB`;
</script>

<template>
  <div class="equalizer" :class="{ bypass: eq.bypass }">
    <div class="presets" role="radiogroup" aria-label="Préréglages de l'égaliseur">
      <button v-for="p in EQ_PRESETS" :key="p.name" type="button" role="radio" class="preset" :class="{ on: eq.activePreset === p.name }" :aria-checked="eq.activePreset === p.name" @click="eq.applyPreset(p.gains)">
        <svg viewBox="0 0 100 36" aria-hidden="true"><path :d="miniPath(p.gains)" /></svg>
        <span>{{ p.name }}</span>
      </button>
    </div>

    <div class="stage">
      <svg ref="svg" class="graph" :viewBox="`0 0 ${W} ${H}`" preserveAspectRatio="none" @pointerdown="down" @pointermove="move" @pointerup="up" @pointercancel="up">
        <defs>
          <linearGradient id="eq-area" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="var(--accent)" stop-opacity="0.45" />
            <stop offset="100%" stop-color="var(--accent)" stop-opacity="0" />
          </linearGradient>
        </defs>
        <!-- Spectre du son, derrière la courbe -->
        <g class="spectrum">
          <rect v-for="(v, i) in spectrumBars" :key="i" :x="PAD.left + i * barW + 1" :y="PAD.top + plotH * (1 - v)" :width="barW - 2" :height="plotH * v" rx="2" />
        </g>
        <!-- Repères : 0 dB, ±6, ±12 -->
        <g class="grid">
          <line v-for="db in [12, 6, 0, -6, -12]" :key="db" :x1="PAD.left" :x2="W - PAD.right" :y1="y(db)" :y2="y(db)" :class="{ zero: db === 0 }" />
        </g>
        <path class="area" :d="area" fill="url(#eq-area)" />
        <path class="curve" :d="curve" />
      </svg>
      <!-- Étiquettes en HTML : le dessin est étiré à la largeur, un texte SVG y serait déformé. -->
      <span v-for="db in [12, 0, -12]" :key="`t${db}`" class="axis db" :style="{ top: `${(y(db) / H) * 100}%` }">{{ db > 0 ? "+" : "" }}{{ db }}</span>
      <span v-for="(l, i) in BAND_LABELS" :key="l" class="axis hz" :style="{ left: `${(x(i) / W) * 100}%` }">{{ l }}</span>
      <!-- Poignées en HTML : cibles tactiles rondes, non déformées par l'étirement du dessin. -->
      <button
        v-for="(g, i) in eq.gains"
        :key="i"
        type="button"
        class="knob"
        :class="{ on: active === i, raised: g !== 0 }"
        :style="{ left: `${(x(i) / W) * 100}%`, top: `${(y(g) / H) * 100}%` }"
        role="slider"
        :aria-label="`Bande ${BAND_LABELS[i]} Hz`"
        :aria-valuemin="EQ_MIN"
        :aria-valuemax="EQ_MAX"
        :aria-valuenow="g"
        :aria-valuetext="fmt(g)"
        tabindex="0"
        @pointerdown.stop="down($event); active = i"
        @pointermove="move"
        @pointerup="up"
        @pointercancel="up"
        @keydown="key(i, $event)"
      >
        <span v-if="active === i" class="tip">{{ fmt(g) }}</span>
      </button>
    </div>

    <div class="actions">
      <button
        type="button"
        class="pill pill-ghost compare"
        :class="{ on: eq.bypass }"
        @pointerdown="eq.setBypass(true)"
        @pointerup="eq.setBypass(false)"
        @pointerleave="eq.setBypass(false)"
        @pointercancel="eq.setBypass(false)"
        @keydown.space.prevent="eq.setBypass(true)"
        @keyup.space.prevent="eq.setBypass(false)"
      >
        {{ eq.bypass ? "Sans égaliseur…" : "Maintenir pour comparer" }}
      </button>
      <button type="button" class="pill pill-ghost" :disabled="eq.activePreset === 'Neutre'" @click="eq.reset()">Remettre à plat</button>
    </div>
    <p class="muted small">Glissez les points pour façonner le son ; le spectre derrière montre la musique qui joue. Au clavier : flèches haut et bas sur un point.</p>
  </div>
</template>

<style scoped>
.presets {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
  gap: 6px;
  margin-bottom: var(--space-3);
}
.preset {
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
.preset svg {
  width: 100%;
  height: 30px;
}
.preset path {
  fill: none;
  stroke: currentColor;
  stroke-width: 2.2;
  stroke-linecap: round;
}
.preset.on {
  background: var(--accent-soft);
  color: var(--accent);
  box-shadow: inset 0 0 0 1.5px var(--accent);
}
@media (hover: hover) {
  .preset:hover {
    transform: translateY(-2px);
  }
}
.stage {
  position: relative;
  height: 240px;
  border-radius: var(--radius);
  background: radial-gradient(ellipse 80% 90% at 50% 100%, color-mix(in srgb, var(--accent) 14%, transparent), transparent 70%), rgba(var(--fg-rgb), 0.03);
  touch-action: none;
  user-select: none;
}
.graph {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  cursor: ns-resize;
}
.spectrum rect {
  fill: var(--accent-2, var(--accent));
  opacity: 0.18;
}
.grid line {
  stroke: rgba(var(--fg-rgb), 0.08);
  stroke-width: 1;
  vector-effect: non-scaling-stroke;
}
.grid line.zero {
  stroke: rgba(var(--fg-rgb), 0.2);
  stroke-dasharray: 4 4;
}
.axis {
  position: absolute;
  color: var(--text-3);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  pointer-events: none;
}
.axis.db {
  left: 4px;
  transform: translateY(-50%);
}
.axis.hz {
  bottom: 6px;
  transform: translateX(-50%);
}
.curve {
  fill: none;
  stroke: var(--accent);
  stroke-width: 3;
  vector-effect: non-scaling-stroke;
  filter: drop-shadow(0 0 6px color-mix(in srgb, var(--accent) 60%, transparent));
}
.bypass .curve,
.bypass .area {
  opacity: 0.25;
}
.knob {
  position: absolute;
  width: 26px;
  height: 26px;
  margin: -13px 0 0 -13px;
  padding: 0;
  border: 0;
  border-radius: 50%;
  background: transparent;
  cursor: grab;
  touch-action: none;
}
.knob::after {
  content: "";
  position: absolute;
  inset: 7px;
  border-radius: 50%;
  background: var(--bg-content);
  box-shadow: 0 0 0 2.5px var(--accent);
  transition: transform 0.15s ease;
}
.knob.raised::after {
  background: var(--accent);
}
.knob.on::after,
.knob:focus-visible::after {
  transform: scale(1.35);
}
.knob.on {
  cursor: grabbing;
}
.tip {
  position: absolute;
  bottom: calc(100% + 4px);
  left: 50%;
  transform: translateX(-50%);
  padding: 2px 7px;
  border-radius: var(--radius-sm);
  background: var(--accent);
  color: var(--on-accent);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  pointer-events: none;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin: var(--space-3) 0 var(--space-2);
}
.compare.on {
  background: var(--accent);
  color: var(--on-accent);
}
@media (prefers-reduced-motion: reduce) {
  .preset,
  .knob::after {
    transition: none;
  }
}
</style>
