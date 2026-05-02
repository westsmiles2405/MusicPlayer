import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { MemoryRouter } from "react-router";
import type { ReactNode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

// ---------------------------------------------------------------------------
// Mocks (hoisted by vi.mock)
// ---------------------------------------------------------------------------

const playMock = vi.fn();
const toggleMock = vi.fn();

const usePlayerMock = vi.fn(() => ({
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
  error: null,
  play: playMock,
  toggle: toggleMock,
}));
vi.mock("@/hooks/usePlayer", () => ({
  usePlayer: usePlayerMock,
}));

const invoke = vi.fn((cmd: string) => {
  if (cmd === "get_playlists") return Promise.resolve([]);
  if (cmd === "get_tracks") return Promise.resolve([]);
  return Promise.resolve({
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
  });
}) as any;
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const listen = vi.fn().mockResolvedValue(() => {});
vi.mock("@tauri-apps/api/event", () => ({ listen }));

const search = vi.fn();
vi.mock("@/repositories/searchRepo", () => ({
  searchRepo: { search },
}));

const favoriteSet = vi.fn();
vi.mock("@/repositories/favoriteRepo", () => ({
  favoriteRepo: { list: vi.fn().mockResolvedValue([]), set: favoriteSet },
}));

vi.mock("@/repositories/playlistRepo", () => ({
  playlistRepo: {
    list: vi.fn().mockResolvedValue([]),
    addTrack: vi.fn().mockResolvedValue(0),
    tracks: vi.fn().mockResolvedValue([]),
  },
}));

vi.mock("@/hooks/useDebouncedValue", () => ({
  useDebouncedValue: (value: string) => value,
}));

// Mock @tanstack/react-virtual to bypass virtualization in JSDOM
vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({
    count,
  }: {
    count: number;
    getScrollElement: () => HTMLElement | null;
    estimateSize: () => number;
    overscan?: number;
  }) => {
    const items = Array.from({ length: count }, (_, i) => ({
      key: i,
      index: i,
      start: i * 40,
      end: (i + 1) * 40,
      size: 40,
    }));
    return {
      getVirtualItems: () => items,
      getTotalSize: () => count * 40,
    };
  },
}));

vi.mock("framer-motion", () => ({
  AnimatePresence: ({ children }: { children: ReactNode }) => <>{children}</>,
  motion: new Proxy({} as Record<string, React.FC<Record<string, unknown>>>, {
    get: (_target, tag: string) => {
      const Component = ({
        children,
        initial: _initial,
        animate: _animate,
        exit: _exit,
        transition: _transition,
        layoutId: _layoutId,
        ...domProps
      }: Record<string, unknown>) => {
        return <div {...domProps}>{children as ReactNode}</div>;
      };
      Component.displayName = `motion.${tag}`;
      return Component;
    },
  }),
}));

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider
    client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
  >
    <MemoryRouter>{children}</MemoryRouter>
  </QueryClientProvider>
);

const makeTrack = (overrides: Partial<Record<string, unknown>> = {}) => ({
  id: 1,
  filePath: "/music/song.mp3",
  fileSize: 1024,
  fileModifiedAt: 1,
  hash: null,
  title: "Test Song",
  albumId: null,
  primaryArtistId: null,
  albumArtistId: null,
  trackNo: null,
  discNo: null,
  year: null,
  genre: null,
  durationMs: 180_000,
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
  albumName: "Test Album",
  primaryArtistName: "Test Artist",
  ...overrides,
});

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("E2E integration flows", () => {
  beforeEach(() => {
    invoke.mockReset();
    search.mockReset();
    favoriteSet.mockReset();
    playMock.mockReset();
    toggleMock.mockReset();
    usePlayerMock.mockReset();
    // Restore default return value after reset
    usePlayerMock.mockReturnValue({
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
      error: null,
      play: playMock,
      toggle: toggleMock,
    });
    cleanup();
  });

  // -- Flow 1: Scan -> Browse -------------------------------------------------
  it("scan -> browse: library shows tracks after scan", async () => {
    const tracks = [
      makeTrack({ id: 1, title: "Song A" }),
      makeTrack({ id: 2, title: "Song B" }),
      makeTrack({ id: 3, title: "Song C" }),
    ];
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "get_tracks") return Promise.resolve(tracks);
      if (cmd === "get_playlists") return Promise.resolve([]);
      return Promise.resolve({
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
      });
    });

    const { default: SongsPage } = await import("@/pages/SongsPage");
    await act(async () => {
      render(<SongsPage />, { wrapper });
    });

    await waitFor(() => {
      expect(screen.getByText("Song A")).toBeInTheDocument();
    });
    expect(screen.getByText("Song B")).toBeInTheDocument();
    expect(screen.getByText("Song C")).toBeInTheDocument();
  });

  // -- Flow 2: Playback Control ----------------------------------------------
  it("playback: click play button triggers play", async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "get_tracks")
        return Promise.resolve([makeTrack({ id: 42, title: "Play Me" })]);
      if (cmd === "get_playlists") return Promise.resolve([]);
      return Promise.resolve({
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
      });
    });

    const { default: SongsPage } = await import("@/pages/SongsPage");
    await act(async () => {
      render(<SongsPage />, { wrapper });
    });

    await waitFor(() => {
      expect(screen.getByText("Play Me")).toBeInTheDocument();
    });

    const playButton = screen.getByRole("button", { name: "播放 Play Me" });
    await act(async () => {
      fireEvent.click(playButton);
    });

    expect(playMock).toHaveBeenCalledOnce();
    expect(playMock).toHaveBeenCalledWith(42, [42], 0);
  });

  // -- Flow 3: Search -> Favorite ---------------------------------------------
  it("search -> favorite: search results show and favorite works", async () => {
    search.mockResolvedValue({
      tracks: [
        {
          id: 7,
          title: "Found Song",
          albumName: null,
          primaryArtistName: null,
          durationMs: 1000,
          missingAt: null,
          isFavorite: false,
        },
      ],
      albums: [],
      artists: [],
      playlists: [],
    });
    favoriteSet.mockResolvedValue(undefined);

    const { default: SearchPage } = await import("@/pages/SearchPage");
    await act(async () => {
      render(<SearchPage />, { wrapper });
    });

    // Type a search query
    await act(async () => {
      fireEvent.change(screen.getByTestId("search-input"), {
        target: { value: "found" },
      });
    });

    // Results should appear
    await waitFor(() => expect(search).toHaveBeenCalledWith("found", 10));
    await waitFor(() => {
      expect(screen.getByText("Found Song")).toBeInTheDocument();
    });

    // Click favorite button (aria-label is "收藏")
    const favButton = screen.getByRole("button", { name: "收藏" });
    await act(async () => {
      fireEvent.click(favButton);
    });

    expect(favoriteSet).toHaveBeenCalledOnce();
    expect(favoriteSet).toHaveBeenCalledWith(7, true);
  });

  // -- Flow 4: Playlist CRUD -------------------------------------------------
  it("playlist CRUD: create playlist and verify dialog exists", async () => {
    invoke.mockImplementation((cmd: string) => {
      if (cmd === "get_playlists") return Promise.resolve([]);
      return Promise.resolve({
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
      });
    });

    const { default: PlaylistsPage } = await import("@/pages/PlaylistsPage");
    await act(async () => {
      render(<PlaylistsPage />, { wrapper });
    });

    // Empty state should be visible
    await waitFor(() => {
      expect(screen.getByText("没有播放列表")).toBeInTheDocument();
    });

    // "创建播放列表" button should exist
    const createButton = screen.getByRole("button", {
      name: "创建播放列表",
    });
    expect(createButton).toBeInTheDocument();

    // Click to open create dialog
    await act(async () => {
      fireEvent.click(createButton);
    });

    // Dialog should appear
    await waitFor(() => {
      expect(
        screen.getByRole("dialog", { name: "创建播放列表" }),
      ).toBeInTheDocument();
    });
    expect(
      screen.getByRole("textbox", { name: "播放列表名称" }),
    ).toBeInTheDocument();
  });

  // -- Flow 5: Now Playing ---------------------------------------------------
  it("now playing: overlay opens and shows track info", async () => {
    const { NowPlayingOverlay } =
      await import("@/components/player/NowPlayingOverlay");

    // Override usePlayer for this test to return active playback state
    usePlayerMock.mockReturnValue({
      status: "playing",
      current: {
        id: 1,
        title: "Now Playing Track",
        artistName: "Now Playing Artist",
        albumName: "Now Playing Album",
        durationMs: 240_000,
        coverPath: null,
      },
      positionMs: 60_000,
      durationMs: 240_000,
      volume: 0.8,
      muted: false,
      queueIndex: 0,
      queueLen: 5,
      repeatMode: "off",
      shuffle: false,
      error: null,
      play: playMock,
      pause: vi.fn(),
      resume: vi.fn(),
      toggle: toggleMock,
      stop: vi.fn(),
      seek: vi.fn(),
      next: vi.fn(),
      previous: vi.fn(),
      setVolume: vi.fn(),
      setMuted: vi.fn(),
      toggleMute: vi.fn(),
    } as any);

    const onClose = vi.fn();

    await act(async () => {
      render(<NowPlayingOverlay open onClose={onClose} />, { wrapper });
    });

    // Dialog should be visible with track info
    expect(
      screen.getByRole("dialog", { name: "Now Playing" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Now Playing Track")).toBeInTheDocument();
    expect(screen.getByText("Now Playing Artist")).toBeInTheDocument();
    expect(screen.getByText("Now Playing Album")).toBeInTheDocument();

    // Control buttons should exist
    expect(screen.getByRole("button", { name: "上一首" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "暂停" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "下一首" })).toBeInTheDocument();

    // Close button should work
    fireEvent.click(screen.getByRole("button", { name: "关闭 Now Playing" }));
    expect(onClose).toHaveBeenCalledOnce();
  });
});
