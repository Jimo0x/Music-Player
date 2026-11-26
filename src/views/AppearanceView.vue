<script setup lang="ts">
import { type ArtStyle, useVisualsStore, type VinylColor, type VisualColors } from "@/stores/visuals";
import { computed, onMounted, ref } from "vue";
import { renderCrop, type Crop } from "@/lib/imageCrop";
import { catalogFont, SYSTEM_FONT } from "@/lib/fonts";
import { FONTS } from "@/lib/fontCatalog";
import { BUILTIN_THEMES, cloneTheme, contrast, parseHex, parseThemeFile, themeFile, themeVariables, type AdvancedColorKey, type FontRole, type Theme } from "@/lib/themes";
import { useAppearanceStore, type ButtonShape, type ButtonStyle, type FieldShape, type FieldStyle, type IconStyle, type SchemeMode } from "@/stores/appearance";
import { parseThemePack } from "@/lib/themePack";
import { api } from "@/lib/api";
import { shareFile } from "@/lib/androidMedia";
import { SEEK_STYLES, type SeekStyle } from "@/lib/seekStyles";
import SeekBar from "@/components/SeekBar.vue";
import { ICON_NAMES } from "@/lib/iconPack";
import { PATTERNS, patternMask, type PatternId, type PatternSettings } from "@/lib/patterns";
import type { BackdropSource } from "@/lib/backdrop";
import { useUiStore } from "@/stores/ui";
import { useBaladeurStore } from "@/stores/baladeur";
import { SKINS } from "@/lib/baladeur";
import CropDialog from "@/components/CropDialog.vue";
import FontPicker from "@/components/FontPicker.vue";
import Icon from "@/components/Icon.vue";

// Personnalisation : thèmes (prêts à l'emploi ou créés), éditeur de couleurs, polices de
// l'interface, des titres et des paroles, taille des paroles.
const appearance = useAppearanceStore();
const ui = useUiStore();
const baladeur = useBaladeurStore();

const s = computed(() => appearance.settings);
const current = computed(() => appearance.theme);
const editable = computed(() => !current.value.builtin);
const themeInput = ref<HTMLInputElement | null>(null);
// Android : pas de filtre (un .json téléchargé y a souvent un type générique).
const themeAccept = /Android/i.test(navigator.userAgent) ? undefined : ".mptheme,.json,application/json,text/plain";

// Police des titres de chaque thème, pour l'aperçu « Aa » de sa vignette.
for (const t of appearance.themes) if (t.fonts?.display) void appearance.loadFont(t.fonts.display);

const ROLES: { role: FontRole; label: string; hint: string; decorative: boolean; sample: string }[] = [
  { role: "ui", label: "Interface", hint: "Menus, listes, boutons", decorative: false, sample: "Morceaux · Albums 123" },
  { role: "display", label: "Titres", hint: "Titres des pages, albums, playlists", decorative: true, sample: "Mes playlists" },
  { role: "lyrics", label: "Paroles", hint: "Paroles synchronisées", decorative: true, sample: "Sous les néons de la ville" },
];
const openRole = ref<FontRole | null>(null);

function fontLabel(id: string): string {
  if (!id || id === SYSTEM_FONT) return "Police du système";
  if (id.startsWith("custom:")) return appearance.settings.customFonts.find((f) => f.id === id)?.label ?? "Police importée";
  return catalogFont(id)?.label ?? "Police du système";
}

const suggested = computed(() => {
  const f = current.value.fonts;
  if (!f) return null;
  const same = (["ui", "display", "lyrics"] as FontRole[]).every((r) => (f[r] ?? SYSTEM_FONT) === s.value.fonts[r]);
  return same ? null : Object.values(f).map(fontLabel).join(" · ");
});

function preview(t: Theme) {
  return {
    background: t.colors.bg,
    color: t.colors.text,
    borderRadius: `${Math.min(t.radius, 14)}px`,
  };
}

function customize() {
  const base = current.value;
  const copy = cloneTheme(base, base.builtin ? `${base.name} (perso)` : `${base.name} (copie)`);
  appearance.saveTheme(copy);
  ui.notify("Thème copié : modifiez-le à votre goût");
}

function update(patch: Partial<Theme>, colors?: Partial<Theme["colors"]>) {
  const t = current.value;
  if (t.builtin) return;
  appearance.saveTheme({ ...t, ...patch, colors: { ...t.colors, ...(colors ?? {}) } });
}

function remove() {
  const t = current.value;
  if (t.builtin || !window.confirm(`Supprimer le thème « ${t.name} » ?`)) return;
  appearance.deleteTheme(t.id);
}

const readability = computed(() => {
  const bg = parseHex(current.value.colors.bg);
  const text = parseHex(current.value.colors.text);
  return bg && text ? contrast(bg, text) : 21;
});

// Paquet complet (.mptheme) : le thème et tout le reste de l'apparence, polices et fond joints.
const exportingPack = ref(false);
async function exportPack() {
  exportingPack.value = true;
  try {
    const path = await api.saveThemePack(await appearance.exportThemePack(), `Thème ${current.value.name}`);
    if (!path) return;
    if (/Android/i.test(navigator.userAgent)) shareFile(path);
    else ui.notify("Thème complet enregistré : il s'importe avec « Importer un thème »");
  } catch (e) {
    ui.notify(`Export impossible : ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    exportingPack.value = false;
  }
}

async function copyTheme() {
  try {
    await navigator.clipboard.writeText(themeFile(current.value));
    ui.notify("Thème copié dans le presse-papiers");
  } catch {
    ui.notify("Copie impossible sur cet appareil");
  }
}
async function pasteTheme() {
  try {
    await importTheme(await navigator.clipboard.readText());
  } catch {
    ui.notify("Lecture du presse-papiers impossible : importez plutôt un fichier");
  }
}
async function importTheme(text: string) {
  // Paquet complet (.mptheme) : polices, fond, motif, icônes et styles avec le thème.
  const pack = parseThemePack(text);
  if (pack) {
    try {
      await appearance.importThemePack(pack);
      ui.notify(`Thème complet « ${pack.name} » appliqué`);
    } catch (e) {
      ui.notify(`Thème impossible à appliquer : ${e instanceof Error ? e.message : String(e)}`);
    }
    return;
  }
  const theme = parseThemeFile(text);
  if (!theme) {
    ui.notify("Ce n'est pas un thème valide");
    return;
  }
  appearance.saveTheme(theme);
  ui.notify(`Thème « ${theme.name} » ajouté`);
}
async function onThemeFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (file) await importTheme(await file.text());
}

function resetAll() {
  if (!window.confirm("Revenir à l'apparence d'origine (thème Nuit bleue, polices du système) ? Vos thèmes créés et polices importées sont conservés.")) return;
  appearance.reset();
}

// ---------------------------------------------------------------- mode, boutons, icônes

const MODES: { id: SchemeMode; label: string }[] = [
  { id: "theme", label: "Selon le thème" },
  { id: "light", label: "Clair" },
  { id: "dark", label: "Sombre" },
  { id: "auto", label: "Comme le système" },
];
const BUTTON_SHAPES: { id: ButtonShape; label: string }[] = [
  { id: "round", label: "Arrondis" },
  { id: "pill", label: "Pilules" },
  { id: "square", label: "Carrés" },
];
const BUTTON_STYLES: { id: ButtonStyle; label: string }[] = [
  { id: "gradient", label: "Dégradé" },
  { id: "flat", label: "Plein" },
  { id: "outline", label: "Contour" },
  { id: "glass", label: "Verre" },
];
const FIELD_SHAPES: { id: FieldShape; label: string }[] = [
  { id: "round", label: "Arrondis" },
  { id: "pill", label: "Pilules" },
  { id: "square", label: "Carrés" },
];
const FIELD_STYLES: { id: FieldStyle; label: string }[] = [
  { id: "filled", label: "Remplis" },
  { id: "outline", label: "Contour" },
  { id: "underline", label: "Soulignés" },
];
// Chaque style d'icônes, avec ses variables pour l'aperçu de son propre bouton.
const ICON_STYLES: { id: IconStyle; label: string; vars: Record<string, string> }[] = [
  { id: "classic", label: "Classiques", vars: {} },
  { id: "thin", label: "Fines", vars: { "--icon-stroke": "1.25" } },
  { id: "bold", label: "Épaisses", vars: { "--icon-stroke": "2.3" } },
  { id: "square", label: "Angles vifs", vars: { "--icon-stroke": "1.9", "--icon-cap": "square", "--icon-join": "miter" } },
  { id: "duo", label: "Deux tons", vars: { "--icon-fill": "currentColor", "--icon-fill-opacity": "0.22" } },
];
const SAMPLE_ICONS = ["shuffle", "repeat", "lyrics", "queue", "playlist", "settings"];

// ---- Jeu d'icônes importé (lot 0.12.0) ----
const iconsInput = ref<HTMLInputElement | null>(null);
const showIconNames = ref(false);
const customIconCount = computed(() => Object.keys(s.value.customIcons).length);
async function onIconFiles(event: Event) {
  const input = event.target as HTMLInputElement;
  const files = [...(input.files ?? [])];
  input.value = "";
  if (!files.length) return;
  const { added, ignored } = await appearance.importIcons(files);
  if (!added.length) {
    ui.notify("Aucune icône reconnue : chaque fichier doit porter le nom d'une icône de l'appli (play.svg, playlist.svg…).");
  } else {
    ui.notify(`${added.length} icône${added.length > 1 ? "s" : ""} remplacée${added.length > 1 ? "s" : ""}${ignored.length ? ` · ${ignored.length} fichier${ignored.length > 1 ? "s" : ""} ignoré${ignored.length > 1 ? "s" : ""}` : ""}`);
  }
}
async function copyIconNames() {
  try {
    await navigator.clipboard.writeText(ICON_NAMES.join("\n"));
    ui.notify("Liste des noms copiée");
  } catch {
    ui.notify("Copie impossible");
  }
}

function setButtons(patch: Partial<typeof s.value.buttons>) {
  appearance.settings.buttons = { ...s.value.buttons, ...patch };
  appearance.changed();
}
function setFields(patch: Partial<typeof s.value.fields>) {
  appearance.settings.fields = { ...s.value.fields, ...patch };
  appearance.changed();
}
function setIcons(id: IconStyle) {
  appearance.settings.icons = id;
  appearance.changed();
}

// ---------------------------------------------------------------- profil et icône

const onAndroid = /Android/i.test(navigator.userAgent);
const iconVariants = ref<{ id: string; label: string }[]>([]);
onMounted(async () => {
  try {
    iconVariants.value = await (await fetch("/app-icons/variants.json")).json();
  } catch {
    iconVariants.value = [{ id: "origine", label: "Original" }];
  }
});
const canPin = computed(() => onAndroid && (window.AndroidLook?.canPinShortcut?.() ?? false));

// ---- Arrière-plan (lot 0.12.0) ----
const BACKDROPS: { id: BackdropSource; label: string }[] = [
  { id: "none", label: "Aucun" },
  { id: "image", label: "Mon image" },
  { id: "artwork", label: "Pochette en cours" },
];
const backdropInput = ref<HTMLInputElement | null>(null);
const backdropBusy = ref(false);
function chooseBackdrop(id: BackdropSource) {
  if (id === "image" && !s.value.backdrop.image) backdropInput.value?.click();
  else appearance.setBackdropSource(id);
}
async function onBackdropFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file) return;
  if (!file.type.startsWith("image/")) {
    ui.notify("Choisissez une image (JPG, PNG, WebP…).");
    return;
  }
  backdropBusy.value = true;
  try {
    await appearance.setBackdropImage(file);
    ui.notify("Arrière-plan enregistré");
  } catch (e) {
    ui.notify(`Image impossible à utiliser : ${e instanceof Error ? e.message : String(e)}`);
  } finally {
    backdropBusy.value = false;
  }
}
function setBackdrop(key: "blur" | "dim" | "x" | "y", event: Event) {
  const value = +(event.target as HTMLInputElement).value;
  s.value.backdrop[key] = key === "dim" ? value / 100 : value;
  appearance.changed();
}

// ---- Motif du fond (lot 0.12.0) ----
const PATTERN_CHOICES = [{ id: "none" as PatternId, label: "Aucun" }, ...PATTERNS.map((x) => ({ id: x.id as PatternId, label: x.label }))];
function patternPreview(id: PatternId) {
  const mask = patternMask(id, 0.8);
  if (!mask) return {};
  return { maskImage: mask.image, WebkitMaskImage: mask.image, maskSize: mask.size, WebkitMaskSize: mask.size };
}
function setPattern(change: Partial<PatternSettings>) {
  Object.assign(s.value.pattern, change);
  appearance.changed();
}

// ---- Barre de progression (lot 0.13.0) ----
function setSeekStyle(id: SeekStyle) {
  s.value.seekStyle = id;
  appearance.changed();
}

type CropPurpose = "avatar" | "pc-icon" | "shortcut";
const cropping = ref<{ purpose: CropPurpose; url: string } | null>(null);
const imageInput = ref<HTMLInputElement | null>(null);
let nextPurpose: CropPurpose = "avatar";
const busy = ref(false);

function pickImage(purpose: CropPurpose) {
  nextPurpose = purpose;
  imageInput.value?.click();
}
function onImageFile(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = "";
  if (!file) return;
  if (!file.type.startsWith("image/")) {
    ui.notify("Choisissez une image (JPG, PNG, WebP…).");
    return;
  }
  cropping.value = { purpose: nextPurpose, url: URL.createObjectURL(file) };
}
function closeCrop() {
  if (cropping.value) URL.revokeObjectURL(cropping.value.url);
  cropping.value = null;
}
const cropTitle = computed(() =>
  cropping.value?.purpose === "avatar" ? "Photo de profil" : cropping.value?.purpose === "pc-icon" ? "Icône de la fenêtre" : "Raccourci avec ma photo",
);

async function confirmCrop(crop: Crop) {
  const job = cropping.value;
  if (!job) return;
  busy.value = true;
  try {
    if (job.purpose === "avatar") {
      await appearance.setAvatar(await renderCrop(job.url, crop, { size: 512 }));
      ui.notify("Photo de profil enregistrée");
    } else if (job.purpose === "pc-icon") {
      await appearance.setAppIcon("photo", await renderCrop(job.url, crop, { size: 256, round: 0.22 }));
      ui.notify("Icône de la fenêtre changée");
    } else {
      // Icône adaptative : le lanceur ne montre que les deux tiers du centre.
      const blob = await renderCrop(job.url, crop, { size: 432, inset: 0.667, type: "image/png" });
      const name = await appearance.replaceImage(blob, null);
      const label = s.value.profile.name.trim() || "Music Player";
      const ok = window.AndroidLook?.pinShortcut?.(label, appearance.assetPath(name)) ?? false;
      ui.notify(ok ? "Confirmez l'ajout du raccourci sur l'écran d'accueil" : "Ce téléphone ne permet pas d'ajouter de raccourci");
    }
  } catch (e) {
    ui.notify(`Impossible : ${e}`);
  } finally {
    busy.value = false;
    closeCrop();
  }
}

function setName(event: Event) {
  appearance.settings.profile = { ...s.value.profile, name: (event.target as HTMLInputElement).value.trim().slice(0, 40) };
  appearance.changed();
}

async function chooseIcon(id: string) {
  if (id === s.value.appIcon) return;
  if (onAndroid && !window.confirm("L'icône de l'écran d'accueil va changer. Selon le téléphone, l'appli peut se fermer un instant et son raccourci se déplacer dans la liste des applis.")) return;
  const ok = await appearance.setAppIcon(id);
  ui.notify(ok ? "Icône de l'application changée" : "Ce téléphone n'a pas accepté le changement d'icône");
}

// Couleurs avancées : facultatives ; « Auto » les rend au calcul à partir des couleurs de base.
const ADVANCED_FIELDS: { key: AdvancedColorKey; label: string }[] = [
  { key: "sidebar", label: "Barre latérale" },
  { key: "bar", label: "Barre du lecteur" },
  { key: "text2", label: "Texte secondaire" },
  { key: "success", label: "Réussite" },
  { key: "danger", label: "Erreur" },
];
const advancedOpen = ref(false);
/** Couleur affichée dans le sélecteur quand rien n'est choisi : celle calculée par le thème. */
function advancedValue(key: AdvancedColorKey): string {
  const chosen = current.value.colors[key];
  if (chosen) return chosen;
  const vars = themeVariables(current.value);
  const css = { sidebar: vars["--bg-sidebar"], bar: vars["--bg"], text2: current.value.colors.text, success: "#3cc878", danger: "#ff6b7d" }[key];
  return /^#[0-9a-f]{6}$/i.test(css ?? "") ? css! : "#808080";
}
function resetAdvanced(key: AdvancedColorKey) {
  const t = current.value;
  if (t.builtin) return;
  const colors = { ...t.colors };
  delete colors[key];
  appearance.saveTheme({ ...t, colors });
}

const COLOR_FIELDS: { key: keyof Theme["colors"]; label: string }[] = [
  { key: "bg", label: "Fond" },
  { key: "surface", label: "Surfaces" },
  { key: "text", label: "Texte" },
  { key: "accent", label: "Accent" },
  { key: "accent2", label: "Halo" },
];

// « À l'écoute » : présentation de la pochette et couleurs des visuels.
const visuals = useVisualsStore();
const ART_STYLES: { id: ArtStyle; label: string }[] = [
  { id: "auto", label: "Automatique" },
  { id: "cover", label: "Pochette" },
  { id: "vinyl", label: "Vinyle" },
  { id: "cd", label: "CD 3D" },
];
const VISUAL_COLORS: { id: VisualColors; label: string }[] = [
  { id: "pochette", label: "De la pochette" },
  { id: "theme", label: "Du thème" },
];
const VINYL_COLORS: { id: VinylColor; label: string }[] = [
  { id: "noir", label: "Noir" },
  { id: "teinte", label: "Coloré" },
];
</script>

<template>
  <div class="page appearance">
    <div class="head">
      <h1 class="page-title">Personnalisation</h1>
      <button type="button" class="pill pill-ghost" @click="resetAll">Réinitialiser</button>
    </div>

    <section v-if="ui.isMobile" class="card">
      <h2>Mode baladeur</h2>
      <p class="muted small">
        Le téléphone devient un lecteur de poche : écran encastré, molette tactile qu'on fait tourner du doigt, clics et
        vibrations, paroles taillées pour le petit écran. Pour en sortir : Menu, puis « Quitter le baladeur ».
      </p>
      <div class="seg wide" role="radiogroup" aria-label="Finition du baladeur">
        <button v-for="skin in SKINS" :key="skin.id" type="button" role="radio" :aria-checked="baladeur.skinId === skin.id" :class="{ on: baladeur.skinId === skin.id }" @click="baladeur.skinId = skin.id">
          {{ skin.name }}
        </button>
      </div>
      <button type="button" class="pill pill-accent baladeur-go" @click="baladeur.enabled = true">Passer en mode baladeur</button>
    </section>

    <section class="card">
      <h2>Mode clair ou sombre</h2>
      <div class="seg wide" role="radiogroup" aria-label="Mode clair ou sombre">
        <button v-for="m in MODES" :key="m.id" type="button" role="radio" :aria-checked="s.mode === m.id" :class="{ on: s.mode === m.id }" @click="appearance.setMode(m.id)">
          {{ m.label }}
        </button>
      </div>
      <p class="muted small">
        Chaque thème existe en clair et en sombre : sa version inverse est calculée à partir de ses couleurs. Bascule rapide : bouton
        <Icon :name="appearance.light ? 'moon' : 'sun'" :size="14" /> en haut de l'écran.
      </p>
    </section>

    <section class="card">
      <h2>Thème</h2>
      <div class="themes">
        <button
          v-for="t in appearance.themes"
          :key="t.id"
          type="button"
          class="theme"
          :class="{ on: t.id === s.themeId }"
          :aria-pressed="t.id === s.themeId"
          @click="appearance.selectTheme(t.id)"
        >
          <span class="mock" :style="preview(t)">
            <span class="mock-side" :style="{ background: t.colors.surface }" />
            <span class="mock-body">
              <span class="mock-title" :style="{ fontFamily: appearance.fontStackFor(t.fonts?.display) }">Aa</span>
              <span class="mock-line" :style="{ background: t.colors.text }" />
              <span class="mock-line short" :style="{ background: t.colors.text }" />
              <span class="mock-pill" :style="{ background: t.colors.accent }" />
            </span>
          </span>
          <span class="theme-name">
            <Icon v-if="t.id === s.themeId" name="check" :size="13" class="tick" />
            {{ t.name }}
          </span>
        </button>
      </div>

      <div v-if="suggested" class="suggest">
        <span>Polices de ce thème : <strong>{{ suggested }}</strong></span>
        <button type="button" class="link" @click="appearance.selectTheme(s.themeId, true)">Les utiliser</button>
      </div>

      <div class="actions">
        <button type="button" class="pill pill-accent" @click="customize"><Icon name="brush" :size="15" /> {{ current.builtin ? "Personnaliser ce thème" : "Dupliquer" }}</button>
        <button type="button" class="pill pill-ghost" @click="themeInput?.click()">Importer un thème</button>
        <button type="button" class="pill pill-ghost" @click="pasteTheme">Coller un thème</button>
        <button type="button" class="pill pill-ghost" @click="copyTheme">Copier ce thème</button>
        <button type="button" class="pill pill-ghost" :disabled="exportingPack" @click="exportPack"><Icon name="share" :size="15" /> {{ exportingPack ? "Préparation…" : "Exporter le thème complet" }}</button>
        <input ref="themeInput" type="file" :accept="themeAccept" hidden @change="onThemeFile" />
      </div>
    </section>

    <section v-if="editable" class="card">
      <h2>Modifier « {{ current.name }} »</h2>
      <label class="field">
        <span>Nom</span>
        <input :value="current.name" class="fieldbox" maxlength="40" @change="update({ name: ($event.target as HTMLInputElement).value.trim() || current.name })" />
      </label>
      <div class="colors">
        <label v-for="f in COLOR_FIELDS" :key="f.key" class="color">
          <input
            type="color"
            :value="current.colors[f.key] ?? current.colors.accent"
            :aria-label="f.label"
            @input="update({}, { [f.key]: ($event.target as HTMLInputElement).value })"
          />
          <span>{{ f.label }}</span>
        </label>
      </div>
      <p v-if="readability < 4.5" class="warn">Texte peu lisible sur ce fond (contraste {{ readability.toFixed(1) }} : 4,5 conseillé).</p>
      <button type="button" class="link advanced-toggle" :aria-expanded="advancedOpen" @click="advancedOpen = !advancedOpen">
        {{ advancedOpen ? "Masquer les couleurs avancées" : "Couleurs avancées" }}
      </button>
      <div v-if="advancedOpen" class="advanced">
        <p class="muted small">Chaque couleur laissée sur « Auto » est calculée à partir des couleurs ci-dessus.</p>
        <div class="colors">
          <div v-for="f in ADVANCED_FIELDS" :key="f.key" class="color">
            <input type="color" :value="advancedValue(f.key)" :aria-label="f.label" @input="update({}, { [f.key]: ($event.target as HTMLInputElement).value })" />
            <span>{{ f.label }}</span>
            <button v-if="current.colors[f.key]" type="button" class="link small" @click="resetAdvanced(f.key)">Remettre en Auto</button>
            <span v-else class="muted small">Auto</span>
          </div>
        </div>
      </div>
      <label class="slider">
        <span>Arrondi des coins <em>{{ current.radius }} px</em></span>
        <input type="range" min="0" max="20" :value="current.radius" @input="update({ radius: +($event.target as HTMLInputElement).value })" />
      </label>
      <label class="slider">
        <span>Halos et lueurs <em>{{ Math.round(current.glow * 100) }} %</em></span>
        <input type="range" min="0" max="100" :value="Math.round(current.glow * 100)" @input="update({ glow: +($event.target as HTMLInputElement).value / 100 })" />
      </label>
      <div class="actions">
        <button type="button" class="pill pill-ghost danger" @click="remove"><Icon name="trash" :size="15" /> Supprimer ce thème</button>
      </div>
    </section>

    <section class="card">
      <h2>Arrière-plan</h2>
      <div class="seg wide" role="radiogroup" aria-label="Arrière-plan">
        <button v-for="o in BACKDROPS" :key="o.id" type="button" role="radio" :aria-checked="s.backdrop.source === o.id" :class="{ on: s.backdrop.source === o.id }" :disabled="backdropBusy" @click="chooseBackdrop(o.id)">
          {{ o.label }}
        </button>
      </div>
      <div v-if="s.backdrop.image || s.backdrop.source === 'image'" class="backdrop-pick">
        <span class="backdrop-thumb" :style="s.backdrop.image ? { backgroundImage: `url(&quot;${appearance.assetUrl(s.backdrop.image)}&quot;)`, backgroundPosition: `${s.backdrop.x}% ${s.backdrop.y}%` } : {}" />
        <div class="row">
          <button type="button" class="pill pill-ghost" :disabled="backdropBusy" @click="backdropInput?.click()">
            <Icon name="image" :size="15" /> {{ backdropBusy ? "Préparation…" : s.backdrop.image ? "Changer l'image" : "Choisir une image" }}
          </button>
          <button v-if="s.backdrop.image" type="button" class="pill pill-ghost danger" :disabled="backdropBusy" @click="appearance.clearBackdropImage()">Retirer</button>
        </div>
      </div>
      <template v-if="s.backdrop.source !== 'none'">
        <label class="slider">
          <span>Flou <em>{{ s.backdrop.blur }} px</em></span>
          <input type="range" min="0" max="30" step="1" :value="s.backdrop.blur" @input="setBackdrop('blur', $event)" />
        </label>
        <label class="slider">
          <span>Assombrissement <em>{{ Math.round(s.backdrop.dim * 100) }} %</em></span>
          <input type="range" min="0" max="90" step="5" :value="Math.round(s.backdrop.dim * 100)" @input="setBackdrop('dim', $event)" />
        </label>
        <template v-if="s.backdrop.source === 'image'">
          <label class="slider">
            <span>Cadrage horizontal <em>{{ s.backdrop.x }} %</em></span>
            <input type="range" min="0" max="100" step="1" :value="s.backdrop.x" @input="setBackdrop('x', $event)" />
          </label>
          <label class="slider">
            <span>Cadrage vertical <em>{{ s.backdrop.y }} %</em></span>
            <input type="range" min="0" max="100" step="1" :value="s.backdrop.y" @input="setBackdrop('y', $event)" />
          </label>
        </template>
        <p class="muted small">
          Un voile de la couleur du thème garde le texte lisible : augmentez l'assombrissement si une image claire gêne la lecture. En affichage léger, le flou est retiré
          pour ménager le téléphone.
        </p>
      </template>
      <input ref="backdropInput" type="file" accept="image/*" hidden @change="onBackdropFile" />
    </section>

    <section class="card">
      <h2>Motif</h2>
      <div class="patterns" role="radiogroup" aria-label="Motif du fond">
        <button v-for="o in PATTERN_CHOICES" :key="o.id" type="button" role="radio" class="pattern-choice" :class="{ on: s.pattern.id === o.id }" :aria-checked="s.pattern.id === o.id" @click="setPattern({ id: o.id })">
          <span class="pattern-sample"><span v-if="o.id !== 'none'" class="pattern-fill" :style="patternPreview(o.id)" /></span>
          <span class="pattern-label">{{ o.label }}</span>
        </button>
      </div>
      <template v-if="s.pattern.id !== 'none'">
        <label class="slider">
          <span>Intensité <em>{{ Math.round(s.pattern.opacity * 100) }} %</em></span>
          <input type="range" min="2" max="40" step="1" :value="Math.round(s.pattern.opacity * 100)" @input="setPattern({ opacity: +($event.target as HTMLInputElement).value / 100 })" />
        </label>
        <label class="slider">
          <span>Taille <em>× {{ s.pattern.scale.toFixed(1) }}</em></span>
          <input type="range" min="5" max="30" step="1" :value="Math.round(s.pattern.scale * 10)" @input="setPattern({ scale: +($event.target as HTMLInputElement).value / 10 })" />
        </label>
        <div class="seg" role="radiogroup" aria-label="Couleur du motif">
          <button type="button" role="radio" :aria-checked="s.pattern.color === 'text'" :class="{ on: s.pattern.color === 'text' }" @click="setPattern({ color: 'text' })">Couleur du texte</button>
          <button type="button" role="radio" :aria-checked="s.pattern.color === 'accent'" :class="{ on: s.pattern.color === 'accent' }" @click="setPattern({ color: 'accent' })">Couleur d'accent</button>
        </div>
      </template>
    </section>

    <section class="card">
      <h2>Polices</h2>
      <div v-for="r in ROLES" :key="r.role" class="role">
        <button type="button" class="role-head" :aria-expanded="openRole === r.role" @click="openRole = openRole === r.role ? null : r.role">
          <span class="role-text">
            <span class="role-label">{{ r.label }}</span>
            <span class="role-hint">{{ r.hint }}</span>
          </span>
          <span class="role-value" :style="{ fontFamily: appearance.fontStackFor(s.fonts[r.role], r.role === 'ui' ? 'ui' : 'display') }">{{ fontLabel(s.fonts[r.role]) }}</span>
          <span class="chev" :class="{ open: openRole === r.role }">›</span>
        </button>
        <FontPicker
          v-if="openRole === r.role"
          :model-value="s.fonts[r.role]"
          :decorative="r.decorative"
          :sample="r.sample"
          @update:model-value="(id) => appearance.setFont(r.role, id)"
        />
      </div>
    </section>

    <section class="card">
      <h2>Boutons et champs</h2>
      <p class="label">Forme des boutons</p>
      <div class="seg" role="radiogroup">
        <button v-for="o in BUTTON_SHAPES" :key="o.id" type="button" role="radio" :aria-checked="s.buttons.shape === o.id" :class="{ on: s.buttons.shape === o.id }" @click="setButtons({ shape: o.id })">{{ o.label }}</button>
      </div>
      <p class="label">Style des boutons</p>
      <div class="seg" role="radiogroup">
        <button v-for="o in BUTTON_STYLES" :key="o.id" type="button" role="radio" :aria-checked="s.buttons.style === o.id" :class="{ on: s.buttons.style === o.id }" @click="setButtons({ style: o.id })">{{ o.label }}</button>
      </div>
      <div class="demo">
        <button type="button" class="pill pill-accent"><Icon name="play" :size="13" /> Lire</button>
        <button type="button" class="pill pill-ghost"><Icon name="shuffle" :size="14" /> Aléatoire</button>
        <button type="button" class="icon-btn" aria-label="Exemple"><Icon name="more" :size="18" /></button>
      </div>
      <p class="label">Forme des champs</p>
      <div class="seg" role="radiogroup">
        <button v-for="o in FIELD_SHAPES" :key="o.id" type="button" role="radio" :aria-checked="s.fields.shape === o.id" :class="{ on: s.fields.shape === o.id }" @click="setFields({ shape: o.id })">{{ o.label }}</button>
      </div>
      <p class="label">Style des champs</p>
      <div class="seg" role="radiogroup">
        <button v-for="o in FIELD_STYLES" :key="o.id" type="button" role="radio" :aria-checked="s.fields.style === o.id" :class="{ on: s.fields.style === o.id }" @click="setFields({ style: o.id })">{{ o.label }}</button>
      </div>
      <label class="demo-field fieldbox">
        <Icon name="search" :size="15" />
        <input placeholder="Rechercher" aria-label="Exemple de champ" />
      </label>
    </section>

    <section class="card">
      <h2>À l'écoute</h2>
      <p class="label">Présentation de la pochette</p>
      <div class="seg" role="radiogroup">
        <button v-for="o in ART_STYLES" :key="o.id" type="button" role="radio" :aria-checked="visuals.artStyle === o.id" :class="{ on: visuals.artStyle === o.id }" @click="visuals.setArtStyle(o.id)">{{ o.label }}</button>
      </div>
      <p class="label">Couleurs des visuels (visualiseur, vinyle, CD, barre)</p>
      <div class="seg" role="radiogroup">
        <button v-for="o in VISUAL_COLORS" :key="o.id" type="button" role="radio" :aria-checked="visuals.colors === o.id" :class="{ on: visuals.colors === o.id }" @click="visuals.setColors(o.id)">{{ o.label }}</button>
      </div>
      <p class="label">Vinyle</p>
      <div class="seg" role="radiogroup">
        <button v-for="o in VINYL_COLORS" :key="o.id" type="button" role="radio" :aria-checked="visuals.vinyl === o.id" :class="{ on: visuals.vinyl === o.id }" @click="visuals.setVinyl(o.id)">{{ o.label }}</button>
      </div>
      <label class="toggle visualizer-toggle">
        <input type="checkbox" :checked="visuals.visualizer" @change="visuals.setVisualizer(($event.target as HTMLInputElement).checked)" />
        <span>Visualiseur du son (couronne autour du vinyle, barres en bas de l'écran)</span>
      </label>
    </section>

    <section class="card">
      <h2>Barre de progression</h2>
      <p class="muted small">Sur l'écran « À l'écoute ». La forme d'onde est celle du vrai morceau, calculée une fois puis gardée.</p>
      <div class="seek-styles" role="radiogroup" aria-label="Style de la barre de progression">
        <button v-for="o in SEEK_STYLES" :key="o.id" type="button" role="radio" class="seek-style" :class="{ on: s.seekStyle === o.id }" :aria-checked="s.seekStyle === o.id" @click="setSeekStyle(o.id)">
          <span class="seek-demo"><SeekBar variant="hero" :look="o.id" :preview="0.42" /></span>
          <span class="seek-label">{{ o.label }}</span>
        </button>
      </div>
    </section>

    <section class="card">
      <h2>Icônes</h2>
      <div class="icon-styles">
        <button v-for="o in ICON_STYLES" :key="o.id" type="button" class="icon-style" :class="{ on: s.icons === o.id }" :style="o.vars" :aria-pressed="s.icons === o.id" @click="setIcons(o.id)">
          <span class="icon-row"><Icon v-for="n in SAMPLE_ICONS" :key="n" :name="n" :size="19" /></span>
          <span class="icon-label">{{ o.label }}</span>
        </button>
      </div>
      <h3 class="sub">Mon jeu d'icônes</h3>
      <p class="muted small">
        Importez des fichiers SVG nommés comme les icônes de l'appli (play.svg, pause.svg, playlist.svg…) : chacun remplace l'icône du même nom, les autres restent.
        Les couleurs suivent le thème ; tout ce qui n'est pas un dessin (scripts, styles, liens, images) est retiré.
      </p>
      <p v-if="customIconCount" class="small">{{ customIconCount }} icône{{ customIconCount > 1 ? "s" : "" }} remplacée{{ customIconCount > 1 ? "s" : "" }} sur {{ ICON_NAMES.length }}.</p>
      <div class="row">
        <button type="button" class="pill pill-ghost" @click="iconsInput?.click()"><Icon name="import" :size="15" /> Importer des icônes (.svg)</button>
        <button v-if="customIconCount" type="button" class="pill pill-ghost danger" @click="appearance.clearIcons()">Revenir aux icônes d'origine</button>
        <button type="button" class="link" :aria-expanded="showIconNames" @click="showIconNames = !showIconNames">{{ showIconNames ? "Masquer les noms" : "Voir les noms des icônes" }}</button>
      </div>
      <div v-if="showIconNames" class="icon-names">
        <span v-for="n in ICON_NAMES" :key="n" class="icon-name" :class="{ mine: s.customIcons[n] }"><Icon :name="n" :size="16" /> {{ n }}</span>
        <button type="button" class="link small" @click="copyIconNames">Copier la liste</button>
      </div>
      <input ref="iconsInput" type="file" accept=".svg,image/svg+xml" multiple hidden @change="onIconFiles" />
    </section>

    <section class="card">
      <h2>Profil</h2>
      <div class="profile">
        <button type="button" class="avatar" :aria-label="s.profile.avatar ? 'Changer la photo de profil' : 'Choisir une photo de profil'" @click="pickImage('avatar')">
          <img v-if="s.profile.avatar" :src="appearance.assetUrl(s.profile.avatar)" alt="" />
          <Icon v-else name="user" :size="34" />
        </button>
        <div class="profile-fields">
          <label class="field">
            <span>Ton nom (affiché dans l'appli)</span>
            <input class="fieldbox" :value="s.profile.name" maxlength="40" placeholder="Prénom ou pseudo" @change="setName" />
          </label>
          <div class="row">
            <button type="button" class="pill pill-ghost" @click="pickImage('avatar')"><Icon name="image" :size="15" /> {{ s.profile.avatar ? "Changer la photo" : "Choisir une photo" }}</button>
            <button v-if="s.profile.avatar" type="button" class="pill pill-ghost danger" @click="appearance.setAvatar(null)">Retirer</button>
          </div>
        </div>
      </div>
    </section>

    <section class="card">
      <h2>Icône de l'application</h2>
      <div class="app-icons">
        <button v-for="v in iconVariants" :key="v.id" type="button" class="app-icon" :class="{ on: s.appIcon === v.id }" :aria-pressed="s.appIcon === v.id" @click="chooseIcon(v.id)">
          <img :src="`/app-icons/${v.id}.png`" alt="" />
          <span>{{ v.label }}</span>
        </button>
        <button v-if="!onAndroid" type="button" class="app-icon" :class="{ on: s.appIcon === 'photo' }" @click="pickImage('pc-icon')">
          <img v-if="s.appIconImage" :src="appearance.assetUrl(s.appIconImage)" alt="" />
          <span v-else class="add"><Icon name="image" :size="26" /></span>
          <span>Ma photo</span>
        </button>
      </div>
      <p v-if="onAndroid" class="muted small">
        Android n'accepte comme icône principale que des dessins prévus dans l'appli. Pour une photo à toi, ajoute un raccourci sur l'écran d'accueil :
      </p>
      <p v-else class="muted small">
        L'icône change partout : la fenêtre (barre des tâches, Alt+Tab) et les raccourcis du menu Démarrer, du Bureau et de la barre des tâches. Si l'un d'eux
        garde l'ancienne icône quelques instants, c'est Windows qui la redessine : elle suit au prochain rafraîchissement.
      </p>
      <div v-if="onAndroid" class="row">
        <button type="button" class="pill pill-accent" :disabled="!canPin || busy" @click="pickImage('shortcut')"><Icon name="image" :size="15" /> Raccourci avec ma photo</button>
        <span v-if="!canPin" class="muted small">Ce lanceur ne permet pas d'ajouter de raccourci.</span>
      </div>
    </section>

    <section class="card">
      <h2>Paroles et couleurs</h2>
      <label class="slider">
        <span>Taille des paroles <em>{{ Math.round(s.lyricsScale * 100) }} %</em></span>
        <input type="range" min="80" max="140" step="5" :value="Math.round(s.lyricsScale * 100)" @input="s.lyricsScale = +($event.target as HTMLInputElement).value / 100; appearance.changed()" />
      </label>
      <p class="lyrics-preview" :style="{ fontSize: `calc(22px * ${s.lyricsScale})` }">Sous les néons de la ville</p>
      <label class="toggle">
        <input type="checkbox" :checked="s.followArtwork" @change="s.followArtwork = ($event.target as HTMLInputElement).checked; appearance.changed()" />
        <span>
          Couleur d'accent tirée de la pochette en cours
          <small>Boutons et curseurs prennent la teinte du morceau qui joue.</small>
        </span>
      </label>
      <p v-if="appearance.saveError" class="warn">Enregistrement impossible : {{ appearance.saveError }}</p>
    </section>

    <input ref="imageInput" type="file" accept="image/*" hidden @change="onImageFile" />
    <CropDialog
      :src="cropping?.url ?? null"
      :title="cropTitle"
      :shape="cropping?.purpose === 'avatar' ? 'circle' : 'rounded'"
      :confirm-label="busy ? 'Enregistrement…' : 'Valider'"
      @confirm="confirmCrop"
      @cancel="closeCrop"
    />

    <p class="muted foot">{{ BUILTIN_THEMES.length }} thèmes fournis, {{ FONTS.length }} polices libres de droits livrées avec l'application (hors ligne).</p>
  </div>
</template>

<style scoped>
.visualizer-toggle {
  margin-top: 14px;
}
.appearance {
  max-width: 880px;
}
.head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}
.card {
  margin-bottom: 16px;
  padding: 18px 20px;
  border-radius: 12px;
  background: rgba(var(--fg-rgb), 0.04);
  box-shadow: inset 0 0 0 0.5px var(--separator);
}
h2 {
  margin: 0 0 14px;
  font-size: 15px;
}
.themes {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(132px, 1fr));
  gap: var(--space-3);
}
.theme {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: 6px;
  border: 0;
  border-radius: 12px;
  background: none;
  text-align: left;
  -webkit-tap-highlight-color: transparent;
}
.theme:hover {
  background: rgba(var(--fg-rgb), 0.05);
}
.theme.on {
  background: var(--accent-soft);
  box-shadow: inset 0 0 0 2px var(--accent);
}
.mock {
  display: flex;
  height: 78px;
  overflow: hidden;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.3), inset 0 0 0 0.5px rgba(128, 128, 128, 0.3);
}
.mock-side {
  width: 24%;
}
.mock-body {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 8px;
}
.mock-title {
  font-size: 17px;
  font-weight: 800;
  line-height: 1;
}
.mock-line {
  height: 4px;
  width: 80%;
  border-radius: 2px;
  opacity: 0.5;
}
.mock-line.short {
  width: 50%;
  opacity: 0.3;
}
.mock-pill {
  margin-top: auto;
  width: 46%;
  height: 10px;
  border-radius: 5px;
}
.theme-name {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 12.5px;
  font-weight: 600;
  padding: 0 2px 2px;
}
.tick {
  color: var(--accent);
}
.suggest {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  margin-top: 14px;
  padding: 10px 12px;
  border-radius: 9px;
  background: var(--accent-soft);
  font-size: 13px;
}
.actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  margin-top: 16px;
}
.link {
  border: 0;
  padding: 0;
  background: none;
  color: var(--accent);
  font-weight: 600;
}
.danger {
  color: var(--danger);
}
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-width: 320px;
  margin-bottom: 14px;
}
.field span,
.slider span {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-2);
}
.field input {
  height: 36px;
  padding: 0 10px;
  border: 0;
  border-radius: 8px;
  background: rgba(var(--fg-rgb), 0.08);
  color: var(--text);
  outline: none;
}
.field input:focus {
  box-shadow: 0 0 0 2px var(--accent);
}
.advanced-toggle {
  display: inline-block;
  margin: 4px 0 10px;
}
.advanced {
  padding: var(--space-3);
  margin-bottom: var(--space-3);
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.04);
}
.colors {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin-bottom: 12px;
}
.color {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-2);
  cursor: pointer;
}
.color input {
  width: 52px;
  height: 52px;
  padding: 0;
  border: 0;
  border-radius: 50%;
  background: none;
  cursor: pointer;
}
.color input::-webkit-color-swatch-wrapper {
  padding: 0;
}
.color input::-webkit-color-swatch {
  border: 2px solid rgba(var(--fg-rgb), 0.25);
  border-radius: 50%;
}
.warn {
  margin: 6px 0 10px;
  font-size: 12.5px;
  color: var(--warning);
}
.slider {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-width: 420px;
  margin: 12px 0;
}
.slider em {
  font-style: normal;
  color: var(--text-3);
  margin-left: 6px;
}
.slider input {
  accent-color: var(--accent);
}
.role + .role {
  border-top: 1px solid var(--separator);
}
.role {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 4px 0 10px;
}
.role-head {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  width: 100%;
  padding: 10px 0 0;
  border: 0;
  background: none;
  text-align: left;
}
.role-text {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}
.role-label {
  font-weight: 600;
}
.role-hint {
  font-size: 12px;
  color: var(--text-3);
}
.role-value {
  font-size: 17px;
  color: var(--text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 50%;
}
.chev {
  font-size: 20px;
  color: var(--text-3);
  transition: transform 0.2s;
}
.chev.open {
  transform: rotate(90deg);
}
.lyrics-preview {
  margin: 4px 0 16px;
  font-family: var(--font-lyrics);
  font-weight: 800;
  color: var(--text);
  line-height: 1.2;
}
.toggle {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  cursor: pointer;
}
.toggle input {
  margin-top: 3px;
  accent-color: var(--accent);
  width: 18px;
  height: 18px;
}
.toggle small {
  display: block;
  font-size: 12px;
  color: var(--text-3);
}
.foot {
  margin: 8px 0 24px;
  font-size: 12px;
}
.small {
  font-size: 12.5px;
  line-height: 1.5;
}
.small :deep(svg) {
  vertical-align: -2px;
}
.label {
  margin: 14px 0 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-2);
}
.seg {
  display: inline-flex;
  flex-wrap: wrap;
  gap: var(--space-1);
  padding: 3px;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.06);
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
.seg.wide {
  display: flex;
}
.seg.wide button {
  flex: 1 1 auto;
}
.demo {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 14px 0 4px;
  padding: 14px;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.03);
}
.demo-field {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  max-width: 320px;
  height: 38px;
  margin-top: 4px;
  padding: 0 12px;
  border-radius: 9px;
  background: rgba(var(--fg-rgb), 0.08);
  color: var(--text-2);
}
.demo-field input {
  flex: 1;
  min-width: 0;
  border: 0;
  background: none;
  outline: none;
  color: var(--text);
}
.demo-field:focus-within {
  box-shadow: 0 0 0 2px var(--accent);
}
.icon-styles {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 10px;
}
.icon-style {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: 12px;
  border: 0;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.05);
  color: var(--text);
  text-align: left;
}
.icon-style.on {
  background: var(--accent-soft);
  box-shadow: inset 0 0 0 2px var(--accent);
}
.icon-row {
  display: flex;
  gap: 10px;
  color: var(--text);
}
.icon-label {
  font-size: 12.5px;
  font-weight: 600;
  color: var(--text-2);
}
.profile {
  display: flex;
  align-items: center;
  gap: 18px;
  flex-wrap: wrap;
}
.avatar {
  display: grid;
  place-items: center;
  width: 92px;
  height: 92px;
  padding: 0;
  border: 0;
  border-radius: 50%;
  overflow: hidden;
  background: rgba(var(--fg-rgb), 0.08);
  color: var(--text-3);
  box-shadow: 0 0 0 2px var(--accent-soft);
}
.avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.profile-fields {
  flex: 1;
  min-width: 220px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.profile-fields .field {
  margin: 0;
}
.sub {
  margin: var(--space-4) 0 var(--space-2);
  font-size: 14px;
}
.icon-names {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: var(--space-3);
}
.icon-name {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 8px;
  border-radius: var(--radius-sm);
  background: rgba(var(--fg-rgb), 0.05);
  font-size: 12px;
  font-family: ui-monospace, monospace;
  user-select: text;
}
.icon-name.mine {
  background: var(--accent-soft);
  color: var(--accent);
}
.seek-styles {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: var(--space-2);
}
.seek-style {
  display: grid;
  gap: 8px;
  padding: 8px;
  border: 0;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.04);
  color: var(--text);
  font: inherit;
}
.seek-style.on {
  background: var(--accent-soft);
  box-shadow: inset 0 0 0 1.5px var(--accent);
}
.seek-demo {
  display: flex;
  align-items: center;
  height: 64px;
  padding: 0 14px;
  border-radius: calc(var(--radius) * 0.7);
  /* Les barres de « À l'écoute » sont faites pour un fond sombre (la pochette floutée). */
  background: linear-gradient(135deg, #1a1430, #0b0b14);
  --visual: var(--accent);
}
.seek-demo > * {
  flex: 1;
}
.seek-label {
  font-size: 12px;
  text-align: center;
}
.patterns {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(84px, 1fr));
  gap: var(--space-2);
  margin-bottom: var(--space-3);
}
.pattern-choice {
  display: grid;
  gap: 6px;
  padding: 6px;
  border: 0;
  border-radius: var(--radius);
  background: rgba(var(--fg-rgb), 0.04);
  color: var(--text);
  font: inherit;
}
.pattern-choice.on {
  background: var(--accent-soft);
  box-shadow: inset 0 0 0 1.5px var(--accent);
}
.pattern-sample {
  position: relative;
  height: 46px;
  border-radius: calc(var(--radius) * 0.7);
  background: var(--bg-content);
  overflow: hidden;
}
.pattern-fill {
  position: absolute;
  inset: 0;
  background: var(--text);
  opacity: 0.45;
  mask-repeat: repeat;
  -webkit-mask-repeat: repeat;
}
.pattern-label {
  font-size: 12px;
  text-align: center;
}
.backdrop-pick {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin: var(--space-3) 0;
}
.backdrop-thumb {
  flex: none;
  width: 96px;
  height: 60px;
  border-radius: var(--radius);
  background-color: rgba(var(--fg-rgb), 0.08);
  background-size: cover;
  box-shadow: inset 0 0 0 1px var(--separator);
}
.row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-2);
}
.app-icons {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(88px, 1fr));
  gap: 10px;
  margin-bottom: 8px;
}
.app-icon {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 10px 6px;
  border: 0;
  border-radius: 12px;
  background: none;
  font-size: 12px;
  color: var(--text-2);
}
.app-icon img,
.app-icon .add {
  width: 60px;
  height: 60px;
  border-radius: 22%;
  object-fit: cover;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.3);
}
.app-icon .add {
  display: grid;
  place-items: center;
  background: rgba(var(--fg-rgb), 0.08);
  box-shadow: inset 0 0 0 1.5px rgba(var(--fg-rgb), 0.2);
}
.app-icon:hover {
  background: rgba(var(--fg-rgb), 0.05);
}
.app-icon.on {
  background: var(--accent-soft);
  box-shadow: inset 0 0 0 2px var(--accent);
  color: var(--text);
}
@media (max-width: 760px), (pointer: coarse) and (max-width: 1100px) {
  .themes {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
  .actions .pill {
    flex: 1 1 auto;
  }
}
.baladeur-go {
  margin-top: var(--space-3);
}
</style>
