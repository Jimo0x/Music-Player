<script setup lang="ts">
import { useUiStore } from "@/stores/ui";
import Icon from "./Icon.vue";

// « Quoi de neuf ? » : après une mise à jour (une fois), depuis les Réglages (tout
// l'historique), ou avant de mettre à jour (notes de la version proposée).
const ui = useUiStore();

function date(iso: string): string {
  const d = new Date(`${iso}T12:00:00`);
  return Number.isNaN(d.getTime()) ? "" : d.toLocaleDateString("fr-FR", { day: "numeric", month: "long", year: "numeric" });
}
</script>

<template>
  <Transition name="dialog">
    <div v-if="ui.whatsNew" class="wn-scrim" @click.self="ui.closeWhatsNew()">
      <section class="wn-card" role="dialog" aria-modal="true" aria-labelledby="wn-title">
        <header>
          <span class="wn-icon" aria-hidden="true"><Icon name="sparkle" :size="22" /></span>
          <h2 id="wn-title">{{ ui.whatsNew.title }}</h2>
          <button type="button" class="icon-btn" aria-label="Fermer" @click="ui.closeWhatsNew()"><Icon name="close" :size="16" /></button>
        </header>
        <div class="wn-body">
          <article v-for="r in ui.whatsNew.releases" :key="r.version" class="wn-release">
            <h3>
              Version {{ r.version }}<small v-if="date(r.date)"> · {{ date(r.date) }}</small>
            </h3>
            <ul>
              <li v-for="(item, i) in r.items" :key="i">{{ item }}</li>
            </ul>
          </article>
        </div>
        <footer>
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
  justify-content: flex-end;
  padding: var(--space-3) var(--space-4) var(--space-4);
}
</style>
