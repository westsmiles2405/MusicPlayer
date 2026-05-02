import { describe, expect, it, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import type { Track } from "@/repositories/trackRepo";

const setFavorite = vi.fn().mockResolvedValue(undefined);
vi.mock("@/repositories/favoriteRepo", () => ({
  favoriteRepo: { set: setFavorite },
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const baseTrack: Track = {
  id: 1,
  filePath: "/a.mp3",
  fileSize: 1,
  fileModifiedAt: 1,
  hash: null,
  title: "Song A",
  albumId: null,
  primaryArtistId: null,
  albumArtistId: null,
  trackNo: null,
  discNo: null,
  year: null,
  genre: null,
  durationMs: 1000,
  bitrate: null,
  sampleRate: null,
  channels: null,
  codec: null,
  isFavorite: false,
  playCount: 0,
  lastPlayedAt: null,
  lastSeenAt: 1,
  missingAt: null,
  addedAt: 1,
  updatedAt: 1,
  rootFolderId: null,
  albumName: null,
  primaryArtistName: null,
};

describe("useToggleFavoriteMutation", () => {
  let qc: QueryClient;
  let wrapper: ({ children }: { children: ReactNode }) => React.ReactElement;

  beforeEach(() => {
    setFavorite.mockClear();
    qc = new QueryClient({
      defaultOptions: {
        queries: { retry: false },
        mutations: { retry: false },
      },
    });
    wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={qc}>{children}</QueryClientProvider>
    );
  });

  it("calls favoriteRepo.set with correct args", async () => {
    const { useToggleFavoriteMutation } =
      await import("./useToggleFavoriteMutation");
    const { result } = renderHook(() => useToggleFavoriteMutation(), {
      wrapper,
    });
    await act(async () => {
      result.current.mutate({ track: baseTrack, favorite: true });
    });
    await waitFor(() => expect(setFavorite).toHaveBeenCalledWith(1, true));
  });

  it("optimistically adds to favoriteTracks cache", async () => {
    qc.setQueryData<Track[]>(["favoriteTracks"], []);
    const { useToggleFavoriteMutation } =
      await import("./useToggleFavoriteMutation");
    const { result } = renderHook(() => useToggleFavoriteMutation(), {
      wrapper,
    });
    await act(async () => {
      result.current.mutate({ track: baseTrack, favorite: true });
    });
    const cached = qc.getQueryData<Track[]>(["favoriteTracks"]);
    expect(cached?.some((t) => t.id === 1)).toBe(true);
  });

  it("rolls back on error", async () => {
    setFavorite.mockRejectedValueOnce(new Error("network"));
    qc.setQueryData<Track[]>(["favoriteTracks"], []);
    const { useToggleFavoriteMutation } =
      await import("./useToggleFavoriteMutation");
    const { result } = renderHook(() => useToggleFavoriteMutation(), {
      wrapper,
    });
    await act(async () => {
      result.current.mutate({ track: baseTrack, favorite: true });
    });
    await waitFor(() => expect(result.current.isError).toBe(true));
    const cached = qc.getQueryData<Track[]>(["favoriteTracks"]);
    expect(cached?.some((t) => t.id === 1)).toBe(false);
  });

  it("patches isFavorite in tracks cache", async () => {
    qc.setQueryData<Track[]>(["tracks"], [{ ...baseTrack, isFavorite: false }]);
    const { useToggleFavoriteMutation } =
      await import("./useToggleFavoriteMutation");
    const { result } = renderHook(() => useToggleFavoriteMutation(), {
      wrapper,
    });
    await act(async () => {
      result.current.mutate({ track: baseTrack, favorite: true });
    });
    const cached = qc.getQueryData<Track[]>(["tracks"]);
    expect(cached?.[0]?.isFavorite).toBe(true);
  });
});
