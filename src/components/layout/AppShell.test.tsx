import { act, render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";

const invoke = vi.fn((cmd: string) => {
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
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const listen = vi.fn().mockResolvedValue(() => {});
vi.mock("@tauri-apps/api/event", () => ({ listen }));

describe("AppShell", () => {
  it("renders sidebar, MiniPlayer, and scan progress bar", async () => {
    const { AppShell } = await import("./AppShell");

    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    await act(async () => {
      render(
        <QueryClientProvider client={qc}>
          <MemoryRouter>
            <AppShell />
          </MemoryRouter>
        </QueryClientProvider>,
      );
    });

    expect(
      screen.getByRole("navigation", { name: "侧边栏" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("contentinfo", { name: "播放器" }),
    ).toBeInTheDocument();
  }, 10_000);
});
