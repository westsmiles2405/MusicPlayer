import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";

const unlistenA = vi.fn();
const unlistenB = vi.fn();
const unlistenC = vi.fn();
const unlistenD = vi.fn();
const listen = vi
  .fn()
  .mockResolvedValueOnce(unlistenA)
  .mockResolvedValueOnce(unlistenB)
  .mockResolvedValueOnce(unlistenC)
  .mockResolvedValueOnce(unlistenD)
  .mockResolvedValue(vi.fn());

vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("@/repositories/playerRepo", () => ({
  playerRepo: {
    getState: vi.fn().mockResolvedValue({
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
    play: vi.fn(),
    pause: vi.fn(),
    resume: vi.fn(),
    toggle: vi.fn(),
    stop: vi.fn(),
    seek: vi.fn(),
    next: vi.fn(),
    previous: vi.fn(),
    setVolume: vi.fn(),
    setMuted: vi.fn(),
    toggleMute: vi.fn(),
  },
}));

describe("usePlayerEvents", () => {
  it("registers and cleans playback listeners", async () => {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    const wrapper = ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={qc}>{children}</QueryClientProvider>
    );
    const { usePlayerEvents } = await import("@/hooks/usePlayer");
    const hook = renderHook(() => usePlayerEvents(), { wrapper });
    await waitFor(() => expect(listen).toHaveBeenCalledTimes(4));
    hook.unmount();
    await waitFor(() => expect(unlistenD).toHaveBeenCalled());
    expect(unlistenA).toHaveBeenCalled();
    expect(unlistenB).toHaveBeenCalled();
    expect(unlistenC).toHaveBeenCalled();
  });
});

describe("usePlayer", () => {
  function makeWrapper() {
    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });
    return ({ children }: { children: ReactNode }) => (
      <QueryClientProvider client={qc}>{children}</QueryClientProvider>
    );
  }

  it("exposes play function", async () => {
    const { usePlayer } = await import("@/hooks/usePlayer");
    const { result } = renderHook(() => usePlayer(), {
      wrapper: makeWrapper(),
    });
    expect(typeof result.current.play).toBe("function");
    expect(typeof result.current.toggle).toBe("function");
    expect(typeof result.current.pause).toBe("function");
  });

  it("exposes volume controls", async () => {
    const { usePlayer } = await import("@/hooks/usePlayer");
    const { result } = renderHook(() => usePlayer(), {
      wrapper: makeWrapper(),
    });
    expect(typeof result.current.setVolume).toBe("function");
    expect(typeof result.current.toggleMute).toBe("function");
    expect(typeof result.current.setMuted).toBe("function");
  });

  it("exposes navigation functions", async () => {
    const { usePlayer } = await import("@/hooks/usePlayer");
    const { result } = renderHook(() => usePlayer(), {
      wrapper: makeWrapper(),
    });
    expect(typeof result.current.next).toBe("function");
    expect(typeof result.current.previous).toBe("function");
    expect(typeof result.current.seek).toBe("function");
  });
});
