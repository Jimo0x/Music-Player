<script setup lang="ts">
// Icônes vectorielles maison (aucune police d'icônes externe), dessinées sur une grille 24×24.
import { computed } from "vue";
import { PATHS } from "@/lib/iconPaths";
import { customIcon } from "@/lib/iconPack";

const props = withDefaults(defineProps<{ name: string; size?: number }>(), { size: 18 });
// Icône importée par la personne (jeu d'icônes, lot 0.12.0), nettoyée à l'import.
const custom = computed(() => customIcon(props.name));


const FILLED = new Set(["play", "pause", "next", "prev"]);
const STROKE_STYLE =
  "stroke-width: var(--icon-stroke, 1.7); stroke-linecap: var(--icon-cap, round); " +
  "stroke-linejoin: var(--icon-join, round); fill: var(--icon-fill, none); fill-opacity: var(--icon-fill-opacity, 1)";
</script>

<template>
  <!-- Traits réglables dans la personnalisation (--icon-stroke, --icon-cap…, style.css). -->
  <svg
    v-if="custom"
    :width="props.size"
    :height="props.size"
    :viewBox="custom.viewBox"
    aria-hidden="true"
    :fill="custom.fill"
    :stroke="custom.stroke"
    :stroke-width="custom.strokeWidth"
    stroke-linecap="round"
    stroke-linejoin="round"
    v-html="custom.body"
  />
  <svg
    v-else
    :width="props.size"
    :height="props.size"
    viewBox="0 0 24 24"
    aria-hidden="true"
    :fill="FILLED.has(props.name) ? 'currentColor' : 'none'"
    :stroke="FILLED.has(props.name) ? 'none' : 'currentColor'"
    stroke-width="1.7"
    stroke-linecap="round"
    stroke-linejoin="round"
    :style="FILLED.has(props.name) ? undefined : STROKE_STYLE"
  >
    <path :d="PATHS[props.name] ?? ''" />
  </svg>
</template>
