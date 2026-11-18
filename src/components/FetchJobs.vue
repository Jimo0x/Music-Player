<script setup lang="ts">
import type { FetchJob } from "@/lib/types";
import { useFetchStore } from "@/stores/fetch";

// File de téléchargements (spotDL, yt-dlp) : état, avancement, erreur, arrêt. Partagée par
// « Importer de Spotify » et « Télécharger » (code venu de SpotifyImportView, déplacé tel quel).
const fetchStore = useFetchStore();

const jobStateLabel: Record<FetchJob["state"], string> = {
  queued: "En attente",
  running: "Téléchargement en cours…",
  done: "Ajouté à la bibliothèque",
  failed: "Échec",
  cancelled: "Arrêté",
};

/** « Artiste — Titre », ou le lien pour un téléchargement direct. */
function label(job: FetchJob): string {
  const w = job.wanted;
  if (!w.title && w.url) return w.url;
  return `${w.artist ? `${w.artist} — ` : ""}${w.title}`;
}
</script>

<template>
  <ul v-if="fetchStore.jobs.length" class="fetch-jobs">
    <li v-for="job in fetchStore.jobs" :key="job.id" class="fetch-job" :class="job.state">
      <div class="fetch-job-main">
        <strong>{{ label(job) }}</strong>
        <span class="muted">
          {{ jobStateLabel[job.state] }}<template v-if="job.state === 'running' && typeof fetchStore.progress[job.id] === 'number'"
            >&nbsp;· {{ Math.round((fetchStore.progress[job.id] ?? 0) * 100) }} %</template
          >
        </span>
        <!-- Barre d'avancement : réelle avec yt-dlp, animée avec spotDL (qui ne la donne pas). -->
        <span
          v-if="job.state === 'running'"
          class="job-bar"
          :class="{ indeterminate: typeof fetchStore.progress[job.id] !== 'number' }"
          ><i :style="typeof fetchStore.progress[job.id] === 'number' ? { width: `${(fetchStore.progress[job.id] ?? 0) * 100}%` } : undefined"
        /></span>
        <details v-if="job.last_error && job.state === 'failed'" class="job-error">
          <summary>Détails</summary>
          <pre>{{ job.last_error }}</pre>
        </details>
      </div>
      <button v-if="job.state === 'queued' || job.state === 'running'" type="button" class="pill pill-danger" @click="fetchStore.cancel(job.id)">
        Arrêter
      </button>
    </li>
  </ul>
</template>

<style scoped>
.fetch-jobs {
  display: grid;
  gap: 6px;
  margin: var(--space-3) 0 0;
  padding: 0;
  list-style: none;
}
.fetch-job {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: 8px 10px;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.04);
  font-size: 13px;
}
.job-bar {
  position: relative;
  display: block;
  height: 4px;
  margin-top: 4px;
  border-radius: 2px;
  background: rgba(var(--fg-rgb), 0.1);
  overflow: hidden;
}
.job-bar i {
  display: block;
  height: 100%;
  background: var(--accent);
  transition: width 0.25s;
}
.job-bar.indeterminate i {
  width: 35%;
  animation: job-slide 1.2s ease-in-out infinite;
}
@keyframes job-slide {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(300%);
  }
}
.fetch-job.done .muted {
  color: var(--success);
}
.fetch-job.failed .muted {
  color: var(--danger-text);
}
.fetch-job-main {
  display: grid;
  min-width: 0;
  gap: 2px;
}
.fetch-job-main strong {
  overflow-wrap: anywhere;
  font-weight: 600;
}
.job-error summary {
  cursor: pointer;
  font-size: 12px;
  color: var(--text-2);
}
.job-error pre {
  max-height: 220px;
  overflow: auto;
  white-space: pre-wrap;
  overflow-wrap: anywhere;
  padding: 10px;
  background: rgba(var(--fg-rgb), 0.05);
  border-radius: var(--radius);
  color: var(--text-2);
  font-size: 12px;
}

</style>
