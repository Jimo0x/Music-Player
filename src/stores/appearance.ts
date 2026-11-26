import { defineStore } from "pinia";
import { convertFileSrc } from "@tauri-apps/api/core";
import { api } from "@/lib/api";
import { ensureFont, fontStack, SYSTEM_FONT, type CustomFont } from "@/lib/fonts";
import { BUILTIN_THEMES, DEFAULT_THEME_ID, isLight, themeVariables, withScheme, type FontRole, type Theme } from "@/lib/themes";
import { defaultBackdrop, normalizeBackdrop, prepareBackdropImage, type BackdropSettings, type BackdropSource } from "@/lib/backdrop";
import { defaultPattern, normalizePattern, type PatternSettings } from "@/lib/patterns";
import { iconNameOf, normalizeIcons, sanitizeSvg, setCustomIcons, type CustomIcon } from "@/lib/iconPack";
import { buildThemePack, type ParsedThemePack } from "@/lib/themePack";

// Personnalisation de l'application : thème et mode clair / sombre, polices, boutons et
// champs, icônes, profil, icône de l'application. Enregistrée côté Rust (appearance.json,
// survit à une réinstallation de la vue web) et recopiée dans le stockage local pour être
// appliquée dès le lancement, sans éclair aux couleurs d'origine (applyCachedAppearance).

/** « theme » : le mode du thème choisi ; « auto » : celui du système. */
export type SchemeMode = "theme" | "light" | "dark" | "auto";
export type ButtonShape = "pill" | "round" | "square";
export type ButtonStyle = "gradient" | "flat" | "outline" | "glass";
export type FieldShape = "round" | "pill" | "square";
export type FieldStyle = "filled" | "outline" | "underline";
export type IconStyle = "classic" | "thin" | "bold" | "square" | "duo";

export interface AppearanceSettings {
  themeId: string;
  customThemes: Theme[];
  mode: SchemeMode;
  fonts: Record<FontRole, string>;
  customFonts: CustomFont[];
  /** Taille des paroles (multiplicateur, 0,8 à 1,4). */
  lyricsScale: number;
  /** Couleur d'accent tirée de la pochette en cours. */
  followArtwork: boolean;
  buttons: { shape: ButtonShape; style: ButtonStyle };
  fields: { shape: FieldShape; style: FieldStyle };
  icons: IconStyle;
  profile: { name: string; avatar: string | null };
  /** Icône de l'application : variante dessinée (public/app-icons) ou « photo ». */
  appIcon: string;
  /** Photo choisie comme icône de fenêtre (ordinateur). */
  appIconImage: string | null;
  /** Arrière-plan de l'application (lot 0.12.0). */
  backdrop: BackdropSettings;
  /** Motif posé sur le fond (lot 0.12.0). */
  pattern: PatternSettings;
  /** Jeu d'icônes importé, par nom d'icône (lot 0.12.0). */
  customIcons: Record<string, CustomIcon>;
}

const CACHE_KEY = "mp:appearance-vars";
const ON_ANDROID = /Android/i.test(navigator.userAgent);

export function defaultAppearance(): AppearanceSettings {
  return {
    themeId: DEFAULT_THEME_ID,
    customThemes: [],
    mode: "theme",
    fonts: { ui: SYSTEM_FONT, display: SYSTEM_FONT, lyrics: SYSTEM_FONT },
    customFonts: [],
    lyricsScale: 1,
    followArtwork: false,
    buttons: { shape: "round", style: "gradient" },
    fields: { shape: "round", style: "filled" },
    icons: "classic",
    profile: { name: "", avatar: null },
    appIcon: "origine",
    appIconImage: null,
    backdrop: defaultBackdrop(),
    pattern: defaultPattern(),
    customIcons: {},
  };
}

function oneOf<T extends string>(value: unknown, allowed: readonly T[], fallback: T): T {
  return allowed.includes(value as T) ? (value as T) : fallback;
}

/** Réglages lus depuis le disque, complétés et vérifiés (fichier éventuellement ancien). */
function normalize(raw: unknown): AppearanceSettings {
  const base = defaultAppearance();
  if (!raw || typeof raw !== "object") return base;
  const o = raw as Partial<AppearanceSettings>;
  const num = (v: unknown, lo: number, hi: number, d: number) =>
    typeof v === "number" && Number.isFinite(v) ? Math.max(lo, Math.min(hi, v)) : d;
  const str = (v: unknown) => (typeof v === "string" && v ? v : null);
  return {
    themeId: typeof o.themeId === "string" ? o.themeId : base.themeId,
    customThemes: Array.isArray(o.customThemes) ? o.customThemes.filter((t) => t && t.id && t.colors) : [],
    mode: oneOf(o.mode, ["theme", "light", "dark", "auto"] as const, base.mode),
    fonts: { ...base.fonts, ...(o.fonts ?? {}) },
    customFonts: Array.isArray(o.customFonts) ? o.customFonts.filter((f) => f && f.id && f.file) : [],
    lyricsScale: num(o.lyricsScale, 0.8, 1.4, 1),
    followArtwork: o.followArtwork === true,
    buttons: {
      shape: oneOf(o.buttons?.shape, ["pill", "round", "square"] as const, base.buttons.shape),
      style: oneOf(o.buttons?.style, ["gradient", "flat", "outline", "glass"] as const, base.buttons.style),
    },
    fields: {
      shape: oneOf(o.fields?.shape, ["round", "pill", "square"] as const, base.fields.shape),
      style: oneOf(o.fields?.style, ["filled", "outline", "underline"] as const, base.fields.style),
    },
    icons: oneOf(o.icons, ["classic", "thin", "bold", "square", "duo"] as const, base.icons),
    profile: {
      name: typeof o.profile?.name === "string" ? o.profile.name.slice(0, 40) : "",
      avatar: str(o.profile?.avatar),
    },
    appIcon: typeof o.appIcon === "string" && /^[a-z]{2,20}$/.test(o.appIcon) ? o.appIcon : base.appIcon,
    appIconImage: str(o.appIconImage),
    backdrop: normalizeBackdrop(o.backdrop),
    pattern: normalizePattern(o.pattern),
    customIcons: normalizeIcons(o.customIcons),
  };
}

type Applied = { vars: Record<string, string>; light: boolean; attrs: Record<string, string> };

/** Applique au lancement le dernier rendu, avant même le premier affichage. */
export function applyCachedAppearance() {
  try {
    const cached = JSON.parse(localStorage.getItem(CACHE_KEY) ?? "null") as Applied | null;
    if (cached) setVariables(cached);
  } catch {
    // stockage indisponible : l'apparence s'appliquera à la lecture des réglages
  }
}

let appliedKeys: string[] = [];

function setVariables({ vars, light, attrs }: Applied) {
  const root = document.documentElement;
  for (const key of appliedKeys) if (!(key in vars)) root.style.removeProperty(key);
  for (const [key, value] of Object.entries(vars)) root.style.setProperty(key, value);
  appliedKeys = Object.keys(vars);
  if ("--bg" in vars) root.dataset.themed = "1";
  else delete root.dataset.themed;
  root.dataset.scheme = light ? "light" : "dark";
  root.style.colorScheme = light ? "light" : "dark";
  for (const [key, value] of Object.entries(attrs ?? {})) root.dataset[key] = value;
}

/** Le système est-il en mode sombre ? (Android le dit directement : la vue web ne suit pas
 *  toujours le réglage du téléphone.) */
function systemPrefersDark(): boolean {
  const android = window.AndroidLook?.systemDark?.();
  if (typeof android === "boolean") return android;
  return window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? true;
}

let saveTimer: ReturnType<typeof setTimeout> | undefined;

export const useAppearanceStore = defineStore("appearance", {
  state: () => ({
    settings: defaultAppearance(),
    customDir: "",
    systemDark: true,
    /** Le dernier enregistrement a échoué (affiché dans la personnalisation). */
    saveError: null as string | null,
  }),
  getters: {
    themes(state): Theme[] {
      return [...BUILTIN_THEMES, ...state.settings.customThemes];
    },
    theme(): Theme {
      return this.themes.find((t) => t.id === this.settings.themeId) ?? BUILTIN_THEMES[0];
    },
    /** Mode clair effectivement affiché. */
    light(): boolean {
      const mode = this.settings.mode;
      if (mode === "light") return true;
      if (mode === "dark") return false;
      if (mode === "auto") return !this.systemDark;
      return isLight(this.theme);
    },
    /** Le thème dans le mode affiché (version claire ou sombre calculée au besoin). */
    effectiveTheme(): Theme {
      return withScheme(this.theme, this.light);
    },
  },
  actions: {
    setFromInitialState(raw: unknown, customDir: string) {
      this.settings = normalize(raw);
      this.customDir = customDir;
      this.systemDark = systemPrefersDark();
      // Mode « automatique » : suit le système quand il change (coucher du soleil…).
      window.matchMedia?.("(prefers-color-scheme: dark)").addEventListener?.("change", () => this.onSystemScheme());
      window.__mpSystemDark = (dark: boolean) => this.onSystemScheme(dark);
      this.apply();
      void this.applyWindowIcon();
    },
    onSystemScheme(dark?: boolean) {
      this.systemDark = dark ?? systemPrefersDark();
      if (this.settings.mode === "auto") this.apply();
    },
    /** Adresse d'un fichier importé (police, image), servie par le protocole « asset ». */
    assetUrl(file: string): string {
      return convertFileSrc(this.assetPath(file));
    },
    assetPath(file: string): string {
      const sep = this.customDir.includes("\\") ? "\\" : "/";
      return `${this.customDir}${sep}${file}`;
    },
    fontStackFor(id: string | null | undefined, fallback: "ui" | "display" = "display"): string {
      return fontStack(id, fallback);
    },
    loadFont(id: string | null | undefined): Promise<void> {
      return ensureFont(id, (file) => this.assetUrl(file), this.settings.customFonts);
    },
    /** Calcule et pose toutes les variables CSS et attributs de la racine. */
    apply() {
      const s = this.settings;
      const theme = this.effectiveTheme;
      // Thème d'origine en sombre : la feuille de style s'applique telle quelle.
      const original = theme.id === DEFAULT_THEME_ID && theme === this.theme && !this.light;
      const vars: Record<string, string> = original ? {} : themeVariables(theme);
      if (s.followArtwork) {
        const fallback = vars["--accent"] ?? "#3aa8ff";
        vars["--accent"] = `var(--amb-glow, ${fallback})`;
        vars["--accent-soft"] = `color-mix(in srgb, var(--amb-glow, ${fallback}) 16%, transparent)`;
        vars["--accent-on-dark"] = `var(--amb-glow, ${vars["--accent-on-dark"] ?? fallback})`;
      }
      if (s.fonts.ui !== SYSTEM_FONT) vars["--font"] = fontStack(s.fonts.ui, "ui");
      if (s.fonts.display !== SYSTEM_FONT) vars["--font-display"] = fontStack(s.fonts.display, "display");
      if (s.fonts.lyrics !== SYSTEM_FONT) vars["--font-lyrics"] = fontStack(s.fonts.lyrics, "display");
      if (s.lyricsScale !== 1) vars["--lyrics-scale"] = s.lyricsScale.toFixed(2);
      for (const id of Object.values(s.fonts)) void this.loadFont(id);
      setCustomIcons(s.customIcons);
      const applied: Applied = {
        vars,
        light: this.light,
        attrs: {
          btnShape: s.buttons.shape,
          btnStyle: s.buttons.style,
          fieldShape: s.fields.shape,
          fieldStyle: s.fields.style,
          icons: s.icons,
          backdrop: s.backdrop.source,
          pattern: s.pattern.id,
        },
      };
      setVariables(applied);
      try {
        localStorage.setItem(CACHE_KEY, JSON.stringify(applied));
      } catch {
        // pas de cache : simple éclair aux couleurs d'origine au prochain lancement
      }
    },
    /** Applique tout de suite, enregistre un peu plus tard (curseurs, pipettes). */
    changed() {
      this.apply();
      clearTimeout(saveTimer);
      saveTimer = setTimeout(() => {
        api
          .setAppearance(JSON.parse(JSON.stringify(this.settings)))
          .then(() => (this.saveError = null))
          .catch((e) => (this.saveError = String(e)));
      }, 400);
    },
    selectTheme(id: string, withFonts = false) {
      this.settings.themeId = id;
      const theme = this.theme;
      if (withFonts && theme.fonts) {
        for (const role of ["ui", "display", "lyrics"] as FontRole[]) {
          this.settings.fonts[role] = theme.fonts[role] ?? SYSTEM_FONT;
        }
      }
      this.changed();
    },
    setMode(mode: SchemeMode) {
      this.settings.mode = mode;
      this.changed();
    },
    /** Bouton soleil / lune : bascule immédiate entre clair et sombre. */
    toggleScheme() {
      this.setMode(this.light ? "dark" : "light");
    },
    saveTheme(theme: Theme) {
      const list = this.settings.customThemes;
      const i = list.findIndex((t) => t.id === theme.id);
      if (i >= 0) list.splice(i, 1, theme);
      else list.push(theme);
      this.settings.themeId = theme.id;
      this.changed();
    },
    deleteTheme(id: string) {
      this.settings.customThemes = this.settings.customThemes.filter((t) => t.id !== id);
      if (this.settings.themeId === id) this.settings.themeId = DEFAULT_THEME_ID;
      this.changed();
    },
    setFont(role: FontRole, id: string) {
      this.settings.fonts[role] = id;
      this.changed();
    },
    /** Importe un fichier de police ; renvoie son identifiant (« custom:… »). */
    async importFont(file: File): Promise<string> {
      const ext = (file.name.split(".").pop() ?? "").toLowerCase();
      const name = await api.importAsset("font", ext, new Uint8Array(await file.arrayBuffer()));
      const id = `custom:${name}`;
      const label = file.name.replace(/\.[^.]+$/, "").replace(/[-_]+/g, " ").trim() || "Ma police";
      this.settings.customFonts.push({ id, label, file: name });
      this.changed();
      return id;
    },
    async removeCustomFont(id: string) {
      const font = this.settings.customFonts.find((f) => f.id === id);
      this.settings.customFonts = this.settings.customFonts.filter((f) => f.id !== id);
      for (const role of ["ui", "display", "lyrics"] as FontRole[]) {
        if (this.settings.fonts[role] === id) this.settings.fonts[role] = SYSTEM_FONT;
      }
      this.changed();
      if (font) await api.deleteAsset(font.file).catch(() => {});
    },
    /** Enregistre une image (photo de profil, icône) et renvoie son nom ; l'ancienne est
     *  supprimée. */
    async replaceImage(blob: Blob, previous: string | null): Promise<string> {
      const name = await api.importAsset("image", blob.type === "image/png" ? "png" : "jpg", new Uint8Array(await blob.arrayBuffer()));
      if (previous && previous !== name) void api.deleteAsset(previous).catch(() => {});
      return name;
    },
    async setAvatar(blob: Blob | null) {
      const previous = this.settings.profile.avatar;
      if (!blob) {
        this.settings.profile.avatar = null;
        if (previous) void api.deleteAsset(previous).catch(() => {});
      } else {
        this.settings.profile.avatar = await this.replaceImage(blob, previous);
      }
      this.changed();
    },
    /**
     * Icône de l'appli sur ordinateur, partout : la fenêtre (titre, barre des tâches, Alt+Tab)
     * et les raccourcis Windows (menu Démarrer, Bureau, épinglés). `force` : réapplique même si
     * c'est déjà fait (au moment où l'on choisit une icône).
     */
    async applyWindowIcon(force = false) {
      if (ON_ANDROID || !("__TAURI_INTERNALS__" in window)) return;
      const s = this.settings;
      const original = s.appIcon === "origine" || (s.appIcon === "photo" && !s.appIconImage);
      try {
        const url = original ? "/app-icons/origine.png" : s.appIcon === "photo" ? this.assetUrl(s.appIconImage!) : `/app-icons/${s.appIcon}.png`;
        const bytes = new Uint8Array(await (await fetch(url)).arrayBuffer());
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        // Revenir à « origine » remet aussi l'icône d'origine sur la fenêtre (avant, l'ancienne restait).
        await getCurrentWindow().setIcon(bytes);
        await api.syncAppIcon(original ? null : bytes, force);
      } catch {
        // l'icône actuelle est conservée
      }
    },
    /** Change l'icône de l'application. Android : l'icône de l'écran d'accueil (variantes
     *  dessinées uniquement) ; ordinateur : celle de la fenêtre, photo comprise. */
    async setAppIcon(id: string, photo?: Blob): Promise<boolean> {
      if (photo) {
        this.settings.appIconImage = await this.replaceImage(photo, this.settings.appIconImage);
      }
      if (ON_ANDROID) {
        const ok = window.AndroidLook?.setIcon?.(id) ?? false;
        if (!ok) return false;
      }
      this.settings.appIcon = id;
      this.changed();
      await this.applyWindowIcon(true);
      return true;
    },
    /** Fichiers SVG choisis : ceux qui portent le nom d'une icône de l'appli la remplacent. */
    async importIcons(files: File[]): Promise<{ added: string[]; ignored: string[] }> {
      const added: string[] = [];
      const ignored: string[] = [];
      const icons = { ...this.settings.customIcons };
      for (const file of files) {
        const name = iconNameOf(file.name);
        const icon = name && file.size <= 64 * 1024 ? sanitizeSvg(await file.text()) : null;
        if (name && icon) {
          icons[name] = icon;
          added.push(name);
        } else {
          ignored.push(file.name);
        }
      }
      if (added.length) {
        this.settings.customIcons = icons;
        this.changed();
      }
      return { added, ignored };
    },
    clearIcons() {
      this.settings.customIcons = {};
      this.changed();
    },
    /** Paquet de thème complet de l'apparence actuelle, prêt à enregistrer (texte JSON). */
    async exportThemePack(): Promise<string> {
      const s = this.settings;
      const pack = await buildThemePack(
        {
          theme: this.theme,
          fonts: s.fonts,
          customFonts: s.customFonts,
          buttons: s.buttons,
          fields: s.fields,
          icons: s.icons,
          pattern: s.pattern,
          backdrop: s.backdrop,
          customIcons: s.customIcons,
        },
        async (file) => new Uint8Array(await (await fetch(this.assetUrl(file))).arrayBuffer()),
      );
      return JSON.stringify(pack);
    },
    /** Applique un paquet reçu (déjà vérifié) : thème ajouté et choisi, polices et image
     *  jointes enregistrées sur cet appareil, motif, fond, icônes et styles. */
    async importThemePack(pack: ParsedThemePack) {
      const s = this.settings;
      const fontIds: (string | null)[] = [];
      for (const f of pack.embeddedFonts) {
        if (!f) {
          fontIds.push(null);
          continue;
        }
        const name = await api.importAsset("font", f.ext, f.bytes);
        const id = `custom:${name}`;
        s.customFonts.push({ id, label: f.label || "Police du thème", file: name });
        fontIds.push(id);
      }
      for (const role of ["ui", "display", "lyrics"] as FontRole[]) {
        const f = pack.fonts[role];
        s.fonts[role] = "catalog" in f ? f.catalog : "embedded" in f ? (fontIds[f.embedded] ?? SYSTEM_FONT) : SYSTEM_FONT;
      }
      s.buttons = pack.look.buttons as AppearanceSettings["buttons"];
      s.fields = pack.look.fields as AppearanceSettings["fields"];
      s.icons = pack.look.icons as IconStyle;
      s.pattern = pack.look.pattern;
      s.customIcons = pack.look.customIcons;
      const previousImage = s.backdrop.image;
      let image = previousImage;
      if (pack.look.backdropImage) {
        image = await api.importAsset("image", pack.look.backdropImage.ext, pack.look.backdropImage.bytes);
        if (previousImage) void api.deleteAsset(previousImage).catch(() => {});
      }
      s.backdrop = { ...pack.look.backdrop, image, source: pack.look.backdrop.source === "image" && !pack.look.backdropImage ? "none" : pack.look.backdrop.source };
      this.saveTheme({ ...pack.theme, id: `perso-${Date.now().toString(36)}` });
    },
    setBackdropSource(source: BackdropSource) {
      if (source === "image" && !this.settings.backdrop.image) return;
      this.settings.backdrop.source = source;
      this.changed();
    },
    /** Image de fond choisie : réduite, enregistrée, et l'ancienne supprimée. */
    async setBackdropImage(file: Blob) {
      const blob = await prepareBackdropImage(file);
      const b = this.settings.backdrop;
      b.image = await this.replaceImage(blob, b.image);
      b.source = "image";
      b.x = 50;
      b.y = 50;
      this.changed();
    },
    clearBackdropImage() {
      const b = this.settings.backdrop;
      const previous = b.image;
      b.image = null;
      if (b.source === "image") b.source = "none";
      this.changed();
      if (previous) void api.deleteAsset(previous).catch(() => {});
    },
    /** Apparence d'une sauvegarde : tout sauf les fichiers propres à l'autre appareil (photo
     *  de profil, polices importées), qui gardent ceux de cet appareil. */
    restore(raw: unknown) {
      const incoming = normalize(raw);
      const mine = this.settings;
      const knownFonts = new Set(mine.customFonts.map((f) => f.id));
      const font = (id: string, fallback: string) => (incoming.customFonts.some((f) => f.id === id) && !knownFonts.has(id) ? fallback : id);
      this.settings = {
        ...incoming,
        customFonts: mine.customFonts,
        fonts: {
          ui: font(incoming.fonts.ui, mine.fonts.ui),
          display: font(incoming.fonts.display, mine.fonts.display),
          lyrics: font(incoming.fonts.lyrics, mine.fonts.lyrics),
        },
        profile: { name: incoming.profile.name || mine.profile.name, avatar: mine.profile.avatar },
        appIcon: incoming.appIcon === "photo" ? mine.appIcon : incoming.appIcon,
        appIconImage: mine.appIconImage,
        // L'image de fond est un fichier de l'autre appareil : on garde celle d'ici.
        backdrop: { ...incoming.backdrop, image: mine.backdrop.image, source: incoming.backdrop.source === "image" && !mine.backdrop.image ? "none" : incoming.backdrop.source },
      };
      this.changed();
    },
    reset() {
      const keep = this.settings.customThemes;
      const fonts = this.settings.customFonts;
      const profile = this.settings.profile;
      const appIcon = this.settings.appIcon;
      const appIconImage = this.settings.appIconImage;
      // L'image de fond importée reste disponible (le fond, lui, revient à « aucun »).
      const backdrop = { ...defaultBackdrop(), image: this.settings.backdrop.image };
      this.settings = { ...defaultAppearance(), customThemes: keep, customFonts: fonts, profile, appIcon, appIconImage, backdrop };
      this.changed();
    },
  },
});
