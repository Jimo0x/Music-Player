// Sélection de plusieurs morceaux : chaque liste annonce ses morceaux pour « Tout sélectionner ».

import { onBeforeUnmount, watch } from "vue";
import { useUiStore } from "@/stores/ui";

export function useSelectionPool(ids: () => string[]) {
  const ui = useUiStore();
  watch(ids, (list) => (ui.selectionPool = list), { immediate: true });
  onBeforeUnmount(() => (ui.selectionPool = []));
}
