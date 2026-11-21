// Résumé d'un partage reçu (#11) : combien de morceaux sont arrivés, combien étaient déjà là
// (fusionnés), combien étaient en double dans la playlist (ignorés), combien étaient illisibles.
import type { ShareImportResult } from "./types";

const n = (count: number, one: string, many: string) => `${count} ${count > 1 ? many : one}`;

export function shareSummary(result: Pick<ShareImportResult, "name" | "kind" | "added_tracks" | "skipped_tracks" | "merged_tracks" | "duplicate_tracks">): string {
  const what = result.kind === "playlist" ? `Playlist « ${result.name} »` : `« ${result.name} »`;
  const details = [n(result.added_tracks, "morceau reçu", "morceaux reçus")];
  if (result.merged_tracks) details.push(`${n(result.merged_tracks, "déjà présent", "déjà présents")} (fusionné${result.merged_tracks > 1 ? "s" : ""})`);
  if (result.duplicate_tracks) details.push(n(result.duplicate_tracks, "doublon ignoré", "doublons ignorés"));
  if (result.skipped_tracks) details.push(n(result.skipped_tracks, "piste illisible ignorée", "pistes illisibles ignorées"));
  return `${what} : ${details.join(", ")}.`;
}
