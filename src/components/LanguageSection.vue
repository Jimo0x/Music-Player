<script setup lang="ts">
import { ref } from "vue";
import { t, localePref, setLocalePref, type LocalePref } from "@/lib/i18n";
import Icon from "./Icon.vue";

// Choix de la langue de l'interface (#23) : français, anglais, ou celle du système.
const pref = ref<LocalePref>(localePref());
const options: { value: LocalePref; label: string }[] = [
  { value: "systeme", label: "Langue du système" },
  { value: "fr", label: "Français" },
  { value: "en", label: "Anglais" },
];

function choose(value: LocalePref) {
  pref.value = value;
  setLocalePref(value);
}
</script>

<template>
  <section class="section-card">
    <h2><Icon name="globe" :size="17" /> {{ t("Langue") }}</h2>
    <p class="muted">
      {{
        t(
          "L'interface s'affiche dans cette langue ; « Langue du système » la reprend sur celle de votre appareil (français si ce n'est pas l'anglais).",
        )
      }}
    </p>
    <label v-for="opt in options" :key="opt.value" class="toggle row">
      <input type="radio" name="langue" :checked="pref === opt.value" @change="choose(opt.value)" />
      <span>{{ t(opt.label) }}</span>
    </label>
  </section>
</template>

<style scoped>
.row {
  align-items: center;
  margin-top: 10px;
}
</style>
