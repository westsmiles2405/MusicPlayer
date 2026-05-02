import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

const list = vi.fn();
vi.mock("@/repositories/favoriteRepo", () => ({
  favoriteRepo: { list },
}));

vi.mock("@/repositories/playlistRepo", () => ({
  playlistRepo: {
    list: vi.fn().mockResolvedValue([]),
    addTrack: vi.fn().mockResolvedValue(0),
  },
}));

describe("FavoritesPage", () => {
  beforeEach(() => list.mockReset());

  it("renders empty state when there are no favorites", async () => {
    list.mockResolvedValue([]);
    const { default: FavoritesPage } = await import("./FavoritesPage");
    await act(async () => {
      render(
        <QueryClientProvider client={new QueryClient()}>
          <MemoryRouter>
            <FavoritesPage />
          </MemoryRouter>
        </QueryClientProvider>,
      );
    });
    expect(await screen.findByText("还没有喜欢的歌曲")).toBeInTheDocument();
  });
});
