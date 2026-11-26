<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { api } from "@/lib/api";
import { useFrame } from "@/lib/clock";
import { formatDuration } from "@/lib/format";
import { seekMask, waveformMask, type SeekStyle } from "@/lib/seekStyles";
import { useAppearanceStore } from "@/stores/appearance";
import { usePlayerStore } from "@/stores/player";

// Barre de progression : avance à chaque image, curseur et heure visée au survol (souris)
// ou pendant le glisser (doigt), saut au relâchement ; flèches ±5 s, Début et Fin au clavier.
// « hero » : écran « À l'écoute » ; « lcd » : fine ligne au bas de l'afficheur de la barre.
// Style choisi dans la personnalisation (« À l'écoute » seulement) ; `look` et `preview` :
// aperçu figé d'un style dans la page Personnalisation (position fixe, sans morceau).
const props = withDefaults(defineProps<{ variant?: "hero" | "lcd"; look?: SeekStyle | null; preview?: number | null }>(), {
  variant: "hero",
  look: null,
  preview: null,
});
const emit = defineEmits<{ (e: "dragging", value: number | null): void }>();

const player = usePlayerStore();
const appearance = useAppearanceStore();
const style = computed<SeekStyle>(() => props.look ?? (props.variant === "hero" ? appearance.settings.seekStyle : "classic"));
/** Styles dessinés par un masque : fond discret + copie colorée découpée jusqu'à la position. */
const masked = computed(() => ["wave", "dots", "segments", "waveform"].includes(style.value));

// Vraie forme d'onde du morceau (calculée une fois par le moteur, puis en cache).
const bars = ref<number[] | null>(null);
watch(
  () => (style.value === "waveform" && props.preview === null ? (player.currentTrack?.id ?? null) : null),
  async (id) => {
    bars.value = null;
    if (!id) return;
    try {
      const got = await api.trackWaveform(id);
      if (player.currentTrack?.id === id) bars.value = got;
    } catch {
      // morceau illisible pour l'analyse : barres régulières
    }
  },
  { immediate: true },
);
// Aperçu : une forme d'onde inventée mais régulière (couplet, refrain, fin).
const previewBars = Array.from({ length: 160 }, (_, i) => Math.round(40 + 200 * Math.abs(Math.sin(i / 9) * Math.sin(i / 37))));
const maskStyle = computed(() => {
  if (style.value === "waveform") {
    const image = waveformMask(props.preview !== null ? previewBars : bars.value);
    return { maskImage: image, WebkitMaskImage: image, maskSize: "100% 100%", WebkitMaskSize: "100% 100%" };
  }
  const mask = seekMask(style.value);
  return mask ? { maskImage: mask.image, WebkitMaskImage: mask.image, maskSize: mask.size, WebkitMaskSize: mask.size } : undefined;
});
const duration = computed(() => player.currentTrack?.duration_secs ?? 0);
const dragging = ref<number | null>(null);
/** Survol à la souris : position (0 à 1) et heure correspondante. */
const hover = ref<number | null>(null);

const track = ref<HTMLElement | null>(null);
const fill = ref<HTMLElement | null>(null);
const knob = ref<HTMLElement | null>(null);

function paint(pos: number) {
  const ratio = props.preview !== null ? props.preview : duration.value ? Math.min(1, Math.max(0, pos / duration.value)) : 0;
  if (fill.value) {
    if (masked.value) {
      fill.value.style.transform = "";
      fill.value.style.clipPath = `inset(0 ${((1 - ratio) * 100).toFixed(2)}% 0 0)`;
    } else {
      fill.value.style.clipPath = "";
      fill.value.style.transform = `scaleX(${ratio})`;
    }
  }
  if (knob.value) knob.value.style.left = `${ratio * 100}%`;
}
useFrame((t) => paint(dragging.value ?? player.positionAt(t)));

function ratioOf(e: PointerEvent): number {
  const rect = track.value!.getBoundingClientRect();
  return Math.min(1, Math.max(0, (e.clientX - rect.left) / rect.width));
}
function down(e: PointerEvent) {
  if (!duration.value || props.preview !== null) return;
  (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  dragging.value = ratioOf(e) * duration.value;
  emit("dragging", dragging.value);
  paint(dragging.value);
}
function move(e: PointerEvent) {
  if (e.pointerType === "mouse") hover.value = ratioOf(e);
  if (dragging.value === null) return;
  dragging.value = ratioOf(e) * duration.value;
  emit("dragging", dragging.value);
  paint(dragging.value);
}
async function up() {
  if (dragging.value === null) return;
  const target = dragging.value;
  await player.seek(target);
  dragging.value = null;
  emit("dragging", null);
}
function cancel() {
  dragging.value = null;
  emit("dragging", null);
}
function key(e: KeyboardEvent) {
  const pos = player.positionAt(performance.now());
  const jumps: Record<string, number> = { ArrowRight: pos + 5, ArrowLeft: pos - 5, Home: 0, End: duration.value - 1 };
  if (!(e.key in jumps) || !duration.value) return;
  e.preventDefault();
  void player.seek(Math.min(duration.value, Math.max(0, jumps[e.key])));
}

/** Bulle : heure visée pendant le glisser, sinon heure sous la souris. */
const bubble = computed(() => {
  if (dragging.value !== null && duration.value) return { at: dragging.value / duration.value, time: dragging.value };
  if (hover.value !== null && duration.value) return { at: hover.value, time: hover.value * duration.value };
  return null;
});
</script>

<template>
  <div
    class="seek"
    :class="[variant, `look-${style}`, { masked, dragging: dragging !== null, hovering: hover !== null, playing: !!player.currentTrack && !player.isPaused && preview === null, preview: preview !== null }]"
    role="slider"
    tabindex="0"
    aria-label="Position dans le morceau"
    :aria-valuemin="0"
    :aria-valuemax="Math.round(duration)"
    :aria-valuenow="Math.round(dragging ?? player.positionSecs)"
    :aria-valuetext="formatDuration(dragging ?? player.positionSecs)"
    @pointerdown="down"
    @pointermove="move"
    @pointerup="up"
    @pointercancel="cancel"
    @pointerleave="hover = null"
    @keydown="key"
  >
    <div ref="track" class="track">
      <div v-if="masked" class="base" :style="maskStyle" />
      <div v-if="hover !== null && dragging === null && !masked" class="ghost" :style="{ transform: `scaleX(${hover})` }" />
      <div ref="fill" class="fill" :style="masked ? maskStyle : undefined" />
    </div>
    <span ref="knob" class="knob" />
    <span v-if="bubble" class="bubble" :style="{ left: `${bubble.at * 100}%` }">{{ formatDuration(bubble.time) }}</span>
  </div>
</template>

<style scoped>
.seek {
  position: relative;
  cursor: pointer;
  touch-action: none;
  outline: none;
  -webkit-tap-highlight-color: transparent;
}
.track {
  position: relative;
  overflow: hidden;
  transition: height 0.15s ease;
}
.fill,
.ghost {
  position: absolute;
  inset: 0;
  transform-origin: left;
}
.knob {
  position: absolute;
  top: 50%;
  width: 14px;
  height: 14px;
  margin-left: -7px;
  border-radius: 50%;
  transform: translateY(-50%) scale(0);
  transition: transform 0.15s ease;
  pointer-events: none;
}
.seek:hover .knob,
.seek:focus-visible .knob,
.seek.dragging .knob {
  transform: translateY(-50%) scale(1);
}
.bubble {
  position: absolute;
  bottom: calc(100% + 8px);
  padding: 3px 7px;
  border-radius: var(--radius-sm);
  transform: translateX(-50%);
  font-size: 11px;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
  pointer-events: none;
}

/* « À l'écoute » : zone tactile de 34 px autour d'une barre de 6 px. */
.hero {
  padding: 14px 0;
  margin: -14px 0;
}
.hero .track {
  height: 6px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.2);
}
.hero:hover .track,
.hero.dragging .track {
  height: 8px;
}
.hero .ghost {
  background: rgba(255, 255, 255, 0.18);
}
.hero .fill {
  background: linear-gradient(90deg, color-mix(in srgb, var(--visual, #fff) 55%, #fff), #fff);
  box-shadow: 0 0 12px color-mix(in srgb, var(--visual, #fff) 60%, transparent);
}
.hero .knob {
  background: #fff;
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--visual, #fff) 35%, transparent), 0 2px 6px rgba(0, 0, 0, 0.4);
}
.hero .bubble {
  background: rgba(0, 0, 0, 0.6);
  color: #fff;
  backdrop-filter: blur(8px);
}
.hero:focus-visible .track {
  box-shadow: 0 0 0 2px rgba(255, 255, 255, 0.6);
}

/* Barre du haut : fine ligne au bas de l'afficheur, plus épaisse au survol. */
.lcd {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: 11px;
  display: flex;
  align-items: flex-end;
}
.lcd .track {
  width: 100%;
  height: 3px;
  background: rgba(var(--fg-rgb), 0.1);
}
.lcd:hover .track,
.lcd.dragging .track {
  height: 5px;
}
.lcd .ghost {
  background: rgba(var(--fg-rgb), 0.12);
}
.lcd .fill {
  background: var(--text-2);
}
.lcd:hover .fill,
.lcd.dragging .fill,
.lcd:focus-visible .fill {
  background: var(--accent);
}
.lcd .knob {
  top: auto;
  bottom: -3.5px;
  width: 12px;
  height: 12px;
  margin-left: -6px;
  transform: scale(0);
  background: var(--accent);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.35);
}
.lcd:hover .knob,
.lcd.dragging .knob,
.lcd:focus-visible .knob {
  transform: scale(1);
}
.lcd .bubble {
  bottom: 14px;
  background: var(--accent);
  color: #fff;
}

/* ---- Styles de « À l'écoute » (lot 0.13.0) ---- */
.hero.masked .track {
  overflow: visible;
  border-radius: 0;
  background: none;
}
.hero.masked:hover .track,
.hero.masked.dragging .track {
  height: auto;
}
.base,
.hero.masked .fill {
  position: absolute;
  inset: 0;
  mask-repeat: repeat-x;
  -webkit-mask-repeat: repeat-x;
  mask-position: 0 50%;
  -webkit-mask-position: 0 50%;
}
.base {
  background: rgba(255, 255, 255, 0.26);
}
.hero.masked .fill {
  box-shadow: none;
  filter: drop-shadow(0 0 6px color-mix(in srgb, var(--visual, #fff) 55%, transparent));
}
.hero.look-wave .track {
  height: 16px;
}
.hero.look-dots .track {
  height: 10px;
}
.hero.look-segments .track {
  height: 12px;
}
.hero.look-waveform .track {
  height: 36px;
}
.hero.look-waveform .base,
.hero.look-waveform .fill {
  mask-repeat: no-repeat;
  -webkit-mask-repeat: no-repeat;
}
/* Vague : elle ondule tant que le morceau joue, s'arrête en pause. */
.hero.look-wave.playing .base,
.hero.look-wave.playing .fill {
  animation: seek-wave 1.1s linear infinite;
}
@keyframes seek-wave {
  to {
    mask-position: 24px 50%;
    -webkit-mask-position: 24px 50%;
  }
}
.hero.look-neon .track {
  height: 5px;
  border-radius: 3px;
  overflow: visible;
}
.hero.look-neon .fill {
  border-radius: 3px;
  background: var(--visual, #7df9ff);
  box-shadow:
    0 0 6px var(--visual, #7df9ff),
    0 0 16px color-mix(in srgb, var(--visual, #7df9ff) 70%, transparent),
    0 0 30px color-mix(in srgb, var(--visual, #7df9ff) 45%, transparent);
}
.hero.look-waveform .knob,
.hero.look-segments .knob {
  width: 3px;
  height: 120%;
  margin-left: -1.5px;
  border-radius: 2px;
}
.preview {
  pointer-events: none;
}
.preview .knob {
  transform: translateY(-50%) scale(1);
}
:global(:root[data-perf="lite"]) .look-wave .base,
:global(:root[data-perf="lite"]) .look-wave .fill {
  animation: none !important;
}
@media (prefers-reduced-motion: reduce) {
  .track,
  .knob {
    transition: none;
  }
  .look-wave .base,
  .look-wave .fill {
    animation: none !important;
  }
}
</style>
