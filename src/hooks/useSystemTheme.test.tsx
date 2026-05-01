import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { useUIStore } from "@/stores/uiStore";

describe("useSystemTheme", () => {
  it("syncs resolved theme with system preference", async () => {
    const listeners: Array<(e: MediaQueryListEvent) => void> = [];
    vi.stubGlobal("matchMedia", (_query: string) => ({
      matches: false,
      media: "(prefers-color-scheme: dark)",
      addEventListener: (_: string, cb: (e: MediaQueryListEvent) => void) =>
        listeners.push(cb),
      removeEventListener: vi.fn(),
    }));
    const { useSystemTheme } = await import("./useSystemTheme");
    renderHook(() => useSystemTheme());
    listeners.forEach((cb) => cb({ matches: true } as MediaQueryListEvent));
    expect(useUIStore.getState().resolvedTheme).toBe("dark");
  });

  it("falls back to light when matchMedia is unavailable", async () => {
    vi.stubGlobal("matchMedia", undefined);
    const { useSystemTheme } = await import("./useSystemTheme");
    renderHook(() => useSystemTheme());
    expect(useUIStore.getState().resolvedTheme).toBe("light");
  });
});
