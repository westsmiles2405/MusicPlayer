import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

const search = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    status: "idle",
    current: null,
    positionMs: 0,
    durationMs: 0,
    volume: 0.8,
    muted: false,
    queueIndex: null,
    queueLen: 0,
    repeatMode: "off",
    shuffle: false,
  }),
}));

vi.mock("@/repositories/searchRepo", () => ({
  searchRepo: { search },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@/repositories/playlistRepo", () => ({
  playlistRepo: {
    list: vi.fn().mockResolvedValue([]),
    addTrack: vi.fn().mockResolvedValue(0),
  },
}));

vi.mock("@/hooks/useDebouncedValue", () => ({
  useDebouncedValue: (value: string) => value,
}));

describe("SearchPage", () => {
  beforeEach(() => search.mockReset());

  it("does not search on empty input", async () => {
    const { default: SearchPage } = await import("./SearchPage");
    await act(async () => {
      render(
        <QueryClientProvider client={new QueryClient()}>
          <MemoryRouter>
            <SearchPage />
          </MemoryRouter>
        </QueryClientProvider>,
      );
    });
    expect(search).not.toHaveBeenCalled();
  });

  it("renders grouped results after searching", async () => {
    search.mockResolvedValue({
      tracks: [
        {
          id: 1,
          title: "Love Song",
          albumName: null,
          primaryArtistName: null,
          durationMs: 1,
          missingAt: null,
          isFavorite: false,
        },
      ],
      albums: [
        {
          id: 2,
          name: "Love Album",
          albumArtistId: 1,
          year: 2024,
          coverPath: null,
          addedAt: 1,
          updatedAt: 1,
          albumArtistName: "Artist",
          trackCount: 1,
        },
      ],
      artists: [{ id: 3, name: "Love Artist", addedAt: 1, updatedAt: 1 }],
      playlists: [
        {
          id: 4,
          name: "Love Playlist",
          description: null,
          coverPath: null,
          createdAt: 1,
          updatedAt: 1,
          trackCount: 0,
        },
      ],
    });

    const { default: SearchPage } = await import("./SearchPage");
    await act(async () => {
      render(
        <QueryClientProvider client={new QueryClient()}>
          <MemoryRouter>
            <SearchPage />
          </MemoryRouter>
        </QueryClientProvider>,
      );
    });

    await act(async () => {
      fireEvent.change(screen.getByRole("textbox", { name: "搜索" }), {
        target: { value: "love" },
      });
    });

    await waitFor(() => expect(search).toHaveBeenCalledWith("love", 10));
    await waitFor(() => {
      const headings = screen.getAllByRole("heading", { level: 2 });
      const texts = headings.map((h) => h.textContent);
      expect(texts).toContain("歌曲");
      expect(texts).toContain("专辑");
      expect(texts).toContain("艺人");
      expect(texts).toContain("播放列表");
    });
  });
});
