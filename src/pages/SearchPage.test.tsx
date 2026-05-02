import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react/pure";
import { MemoryRouter } from "react-router";
import type { ReactNode } from "react";
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

const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider
    client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
  >
    <MemoryRouter>{children}</MemoryRouter>
  </QueryClientProvider>
);

const emptyResults = {
  tracks: [],
  albums: [],
  artists: [],
  playlists: [],
};

describe("SearchPage", () => {
  beforeEach(() => {
    search.mockReset();
    cleanup();
  });

  it("does not search on empty input", async () => {
    const { default: SearchPage } = await import("./SearchPage");
    await act(async () => {
      render(<SearchPage />, { wrapper });
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
      render(<SearchPage />, { wrapper });
    });

    await act(async () => {
      fireEvent.change(screen.getByTestId("search-input"), {
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

  it("shows empty state when query is empty", async () => {
    const { default: SearchPage } = await import("./SearchPage");
    render(<SearchPage />, { wrapper });
    expect(await screen.findByText("开始搜索")).toBeInTheDocument();
  });

  it("shows loading state while searching", async () => {
    search.mockResolvedValue(emptyResults);
    const { default: SearchPage } = await import("./SearchPage");
    render(<SearchPage />, { wrapper });
    act(() => {
      fireEvent.change(screen.getByTestId("search-input"), {
        target: { value: "test" },
      });
    });
    expect(screen.getByText("搜索中")).toBeInTheDocument();
  });

  it("shows error state on search failure", async () => {
    vi.useFakeTimers();
    let rejectSearch!: (reason: unknown) => void;
    search.mockImplementation(
      () =>
        new Promise((_resolve, reject) => {
          rejectSearch = reject;
        }),
    );
    const { default: SearchPage } = await import("./SearchPage");
    render(<SearchPage />, { wrapper });
    fireEvent.change(screen.getByTestId("search-input"), {
      target: { value: "test" },
    });
    // Flush effects so the query starts
    await vi.advanceTimersByTimeAsync(0);
    rejectSearch("fail");
    // Flush effects so the error state renders
    await vi.advanceTimersByTimeAsync(0);
    expect(screen.getByText("搜索失败")).toBeInTheDocument();
    // Restore real timers; cleanup will run in beforeEach for next test
    vi.useRealTimers();
  });
});
