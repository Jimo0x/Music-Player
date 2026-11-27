import { beforeEach, describe, expect, it, vi } from "vitest";
import { createPinia, setActivePinia } from "pinia";
import type { ImmersiveProfiles, ImmersiveSettings } from "@/lib/types";

// Moteur simulé : garde les réglages comme le ferait le Rust.
const engine: ImmersiveProfiles = { global: {} as ImmersiveSettings, playlists: {}, context: null };
const copy = () => JSON.parse(JSON.stringify(engine)) as ImmersiveProfiles;
vi.mock("@/lib/api", () => ({
  api: {
    setImmersive: vi.fn(async (s: ImmersiveSettings) => ((engine.global = s), copy())),
    setPlaylistImmersive: vi.fn(async (id: string, s: ImmersiveSettings | null) => {
      if (s) engine.playlists[id] = s;
      else delete engine.playlists[id];
      return copy();
    }),
    setImmersiveContext: vi.fn(async (id: string | null) => ((engine.context = id), copy())),
  },
}));

import { DEFAULT_IMMERSIVE, useImmersiveStore } from "./immersive";

describe("mode immersif en un geste", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    engine.global = { ...DEFAULT_IMMERSIVE };
    engine.playlists = {};
    engine.context = null;
  });

  it("allume puis éteint les réglages globaux en gardant le reste", async () => {
    const im = useImmersiveStore();
    await im.update(null, { speed: 25, room: true });
    expect(await im.toggle()).toBe(true);
    expect(im.effective).toMatchObject({ enabled: true, speed: 25, room: true });
    expect(await im.toggle()).toBe(false);
    expect(im.effective).toMatchObject({ enabled: false, speed: 25, room: true });
  });

  it("sans aucun effet coché, l'allumage coche la rotation 8D", async () => {
    const im = useImmersiveStore();
    await im.update(null, { rotation: false, room: false });
    await im.toggle();
    expect(im.effective).toMatchObject({ enabled: true, rotation: true });
  });

  it("dans une playlist qui a son ambiance, c'est elle qui bascule", async () => {
    const im = useImmersiveStore();
    await im.setPlaylistOwn("p1", true);
    await im.setContext("p1");
    expect(await im.toggle()).toBe(false);
    expect(im.profiles.playlists.p1.enabled).toBe(false);
    expect(im.profiles.global.enabled).toBe(false);
    expect(await im.toggle()).toBe(true);
    expect(im.profiles.playlists.p1.enabled).toBe(true);
    expect(im.profiles.global.enabled).toBe(false);
  });
});
