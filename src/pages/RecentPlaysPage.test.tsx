import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { beforeEach, describe, expect, it, vi } from "vitest";

const list = vi.fn();
vi.mock("@/repositories/recentPlayRepo", () => ({
  recentPlayRepo: { list },
}));

vi.mock("@/repositories/playlistRepo", () => ({
  playlistRepo: {
    list: vi.fn().mockResolvedValue([]),
    addTrack: vi.fn().mockResolvedValue(0),
  },
}));

describe("RecentPlaysPage", () => {
  beforeEach(() => list.mockReset());

  it("renders an empty state when no recent plays exist", async () => {
    list.mockResolvedValue([]);
    const { default: RecentPlaysPage } = await import("./RecentPlaysPage");
    await act(async () => {
      render(
        <QueryClientProvider client={new QueryClient()}>
          <MemoryRouter>
            <RecentPlaysPage />
          </MemoryRouter>
        </QueryClientProvider>,
      );
    });
    expect(await screen.findByText("还没有最近播放记录")).toBeInTheDocument();
  });
});
