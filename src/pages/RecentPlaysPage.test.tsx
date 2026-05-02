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

vi.mock("@/repositories/recentPlayRepo", () => ({
  recentPlayRepo: { list },
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

describe("RecentPlaysPage", () => {
  beforeEach(() => {
    list.mockReset();
    cleanup();
  });

  it("renders an empty state when no recent plays exist", async () => {
    list.mockResolvedValue([]);
    const { default: RecentPlaysPage } = await import("./RecentPlaysPage");
    await act(async () => {
      render(<RecentPlaysPage />, { wrapper });
    });
    expect(await screen.findByText("还没有最近播放记录")).toBeInTheDocument();
  });

  it("renders recent plays list", async () => {
    list.mockResolvedValue([
      {
        id: 1,
        filePath: "/a.mp3",
        fileSize: 1,
        fileModifiedAt: 1,
        hash: null,
        title: "Recent Song",
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
        playCount: 1,
        lastPlayedAt: 1000,
        lastSeenAt: 1,
        missingAt: null,
        addedAt: 1,
        updatedAt: 1,
        rootFolderId: null,
        albumName: "Album",
        primaryArtistName: "Artist",
      },
    ]);
    const { default: RecentPlaysPage } = await import("./RecentPlaysPage");
    await act(async () => {
      render(<RecentPlaysPage />, { wrapper });
    });
    expect(await screen.findByText("Recent Song")).toBeInTheDocument();
  });

  it("shows loading state", async () => {
    list.mockReturnValue(new Promise(() => {})); // never resolves
    const { default: RecentPlaysPage } = await import("./RecentPlaysPage");
    render(<RecentPlaysPage />, { wrapper });
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
    const { default: RecentPlaysPage } = await import("./RecentPlaysPage");
    render(<RecentPlaysPage />, { wrapper });
    await vi.advanceTimersByTimeAsync(0);
    rejectList("fail");
    await vi.advanceTimersByTimeAsync(0);
    expect(screen.getByText("加载失败")).toBeInTheDocument();
    vi.useRealTimers();
  });
});
