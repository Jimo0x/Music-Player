import { beforeEach, describe, expect, it, vi } from "vitest";
import { flushPromises, mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import type { SearchResult, Track } from "@/lib/types";

// Recherche « Sur Internet » de la barre de recherche : une recherche un instant après la
// dernière lettre, un geste pour télécharger, et ce qui est déjà dans la bibliothèque se lit.
vi.mock("@/lib/api", () => ({
  api: {
    fetchAvailability: vi.fn(async () => ({ spotdl: false, ytdlp: true })),
    fetchJobs: vi.fn(async () => []),
    fetchEnqueue: vi.fn(async () => ["job-1"]),
    fetchSearch: vi.fn(),
    playQueue: vi.fn(async () => {}),
  },
  onFetchEvents: vi.fn(async () => () => {}),
}));

import { api } from "@/lib/api";
import { useLibraryStore } from "@/stores/library";
import OnlineResults from "./OnlineResults.vue";

const found: SearchResult[] = [
  { title: "Macarena", artist: "Damso", duration_secs: 190, url: "https://www.youtube.com/watch?v=aaaaaaaaaaa", thumbnail: null },
  { title: "Morose", artist: "Damso", duration_secs: 200, url: "https://www.youtube.com/watch?v=bbbbbbbbbbb", thumbnail: null },
];
const morose: Track = { id: "t1", path: "/m/t1.mp3", title: "Morose", artist: "Damso", album: "QALF", album_artist: "Damso", track_no: null, disc_no: null, year: null, genre: null, duration_secs: 200, has_cover: false, added_secs: 0 };

beforeEach(() => {
  setActivePinia(createPinia());
  vi.clearAllMocks();
  vi.useFakeTimers();
});

describe("résultats sur Internet", () => {
  it("cherche une fois la saisie finie, puis propose de télécharger ce qui manque", async () => {
    vi.mocked(api.fetchSearch).mockResolvedValue(found);
    useLibraryStore().tracks = [morose];
    const view = mount(OnlineResults, { props: { query: "da" } });
    await flushPromises();
    await view.setProps({ query: "damso" });
    await vi.advanceTimersByTimeAsync(700);
    await flushPromises();
    // Une seule recherche : la dernière saisie.
    expect(api.fetchSearch).toHaveBeenCalledTimes(1);
    expect(api.fetchSearch).toHaveBeenCalledWith("damso");

    const rows = view.findAll("li");
    expect(rows).toHaveLength(2);
    // « Morose » est déjà là : il se lit ; « Macarena » se télécharge.
    expect(rows[1].text()).toContain("Dans la bibliothèque");
    await rows[0].find("button").trigger("click");
    await flushPromises();
    expect(api.fetchEnqueue).toHaveBeenCalledWith([{ title: "Macarena", artist: "Damso", duration_secs: 190, url: found[0].url, playlist_id: null }]);
    await rows[1].find("button").trigger("click");
    expect(api.playQueue).toHaveBeenCalledWith(["t1"], "t1");
  });

  it("ne cherche rien pour une lettre seule", async () => {
    mount(OnlineResults, { props: { query: "d" } });
    await flushPromises();
    await vi.advanceTimersByTimeAsync(2000);
    expect(api.fetchSearch).not.toHaveBeenCalled();
  });
});
