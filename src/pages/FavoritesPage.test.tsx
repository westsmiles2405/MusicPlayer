import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, cleanup, render, screen } from "@testing-library/react/pure";
import { MemoryRouter } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

const list = vi.fn();
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

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

vi.mock("@/repositories/favoriteRepo", () => ({
  favoriteRepo: { list },
}));

vi.mock("@/repositories/playlistRepo", () => ({
  playlistRepo: {
    list: vi.fn().mockResolvedValue([]),
    addTrack: vi.fn().mockResolvedValue(0),
  },
}));

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider
    client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
  >
    <MemoryRouter>{children}</MemoryRouter>
  </QueryClientProvider>
);

describe("FavoritesPage", () => {
  beforeEach(() => {
    list.mockReset();
    cleanup();
  });

  it("renders empty state when there are no favorites", async () => {
    list.mockResolvedValue([]);
    const { default: FavoritesPage } = await import("./FavoritesPage");
    await act(async () => {
      render(<FavoritesPage />, { wrapper });
    });
    expect(await screen.findByText("还没有收藏")).toBeInTheDocument();
  });

  it("renders favorite tracks list", async () => {
    list.mockResolvedValue([
      {
        id: 1,
        filePath: "/a.mp3",
        fileSize: 1,
        fileModifiedAt: 1,
        hash: null,
        title: "Fav Song",
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
        isFavorite: true,
        playCount: 0,
        lastPlayedAt: null,
        lastSeenAt: 1,
        missingAt: null,
        addedAt: 1,
        updatedAt: 1,
        rootFolderId: null,
        albumName: "Album",
        primaryArtistName: "Artist",
      },
    ]);
    const { default: FavoritesPage } = await import("./FavoritesPage");
    await act(async () => {
      render(<FavoritesPage />, { wrapper });
    });
    expect(await screen.findByText("Fav Song")).toBeInTheDocument();
  });

  it("shows loading state", async () => {
    list.mockReturnValue(new Promise(() => {})); // never resolves
    const { default: FavoritesPage } = await import("./FavoritesPage");
    render(<FavoritesPage />, { wrapper });
    expect(screen.getByText("加载中")).toBeInTheDocument();
  });

  it("shows error state", async () => {
    vi.useFakeTimers();
    let rejectList!: (reason: unknown) => void;
    list.mockImplementation(
      () =>
        new Promise((_resolve, reject) => {
          rejectList = reject;
        }),
    );
    const { default: FavoritesPage } = await import("./FavoritesPage");
    render(<FavoritesPage />, { wrapper });
    await vi.advanceTimersByTimeAsync(0);
    rejectList("fail");
    await vi.advanceTimersByTimeAsync(0);
    expect(screen.getByText("加载失败")).toBeInTheDocument();
    vi.useRealTimers();
  });
});
