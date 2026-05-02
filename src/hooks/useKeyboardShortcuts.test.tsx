import { renderHook } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";
import { useUIStore } from "@/stores/uiStore";

const toggle = vi.fn();
const navigate = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
vi.mock("@/hooks/usePlayer", () => ({ usePlayer: () => ({ toggle }) }));
vi.mock("react-router", async (orig) => {
  const actual: Record<string, unknown> = await orig();
  return { ...actual, useNavigate: () => navigate };
});

const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider client={qc}>{children}</QueryClientProvider>
);

describe("useKeyboardShortcuts", () => {
  beforeEach(() => {
    toggle.mockReset();
    navigate.mockReset();
    useUIStore.setState({
      isNowPlayingOpen: true,
      sidebarCollapsed: false,
      theme: "system",
      resolvedTheme: "light",
    });
    delete (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;
  });

  it("ignores space when input is focused", async () => {
    const { useKeyboardShortcuts } = await import("./useKeyboardShortcuts");
    const input = document.createElement("input");
    document.body.appendChild(input);
    input.focus();
    renderHook(() => useKeyboardShortcuts(), { wrapper });
    input.dispatchEvent(
      new KeyboardEvent("keydown", { key: " ", bubbles: true }),
    );
    expect(toggle).not.toHaveBeenCalled();
    input.remove();
  });

  it("toggles playback on space in normal context", async () => {
    const { useKeyboardShortcuts } = await import("./useKeyboardShortcuts");
    renderHook(() => useKeyboardShortcuts(), { wrapper });
    window.dispatchEvent(
      new KeyboardEvent("keydown", { key: " ", bubbles: true }),
    );
    expect(toggle).toHaveBeenCalledTimes(1);
  });

  it("ignores space when slider is focused", async () => {
    const { useKeyboardShortcuts } = await import("./useKeyboardShortcuts");
    const slider = document.createElement("div");
    slider.setAttribute("role", "slider");
    document.body.appendChild(slider);
    slider.focus();
    renderHook(() => useKeyboardShortcuts(), { wrapper });
    slider.dispatchEvent(
      new KeyboardEvent("keydown", { key: " ", bubbles: true }),
    );
    expect(toggle).not.toHaveBeenCalled();
    slider.remove();
  });

  it("handles cmd+f in tauri runtime", async () => {
    (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ = {};
    const { useKeyboardShortcuts } = await import("./useKeyboardShortcuts");
    renderHook(() => useKeyboardShortcuts(), { wrapper });
    window.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "f",
        metaKey: true,
        bubbles: true,
      }),
    );
    expect(navigate).toHaveBeenCalledWith("/search?focus=1");
  });

  it("does not hijack cmd+f in web runtime", async () => {
    const { useKeyboardShortcuts } = await import("./useKeyboardShortcuts");
    renderHook(() => useKeyboardShortcuts(), { wrapper });
    window.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "f",
        metaKey: true,
        bubbles: true,
      }),
    );
    expect(navigate).not.toHaveBeenCalled();
  });

  it("esc closes now playing", async () => {
    const { useKeyboardShortcuts } = await import("./useKeyboardShortcuts");
    renderHook(() => useKeyboardShortcuts(), { wrapper });
    window.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Escape", bubbles: true }),
    );
    expect(useUIStore.getState().isNowPlayingOpen).toBe(false);
  });

  it("space does not toggle when textarea is focused", async () => {
    const { useKeyboardShortcuts } = await import("./useKeyboardShortcuts");
    const textarea = document.createElement("textarea");
    document.body.appendChild(textarea);
    textarea.focus();
    renderHook(() => useKeyboardShortcuts(), { wrapper });
    textarea.dispatchEvent(
      new KeyboardEvent("keydown", { key: " ", bubbles: true }),
    );
    expect(toggle).not.toHaveBeenCalled();
    textarea.remove();
  });
});
