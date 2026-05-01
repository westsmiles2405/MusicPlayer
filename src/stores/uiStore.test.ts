import { beforeEach, describe, expect, it } from "vitest";
import { useUIStore } from "./uiStore";

describe("uiStore", () => {
  beforeEach(() => {
    useUIStore.setState({
      isNowPlayingOpen: false,
      sidebarCollapsed: false,
      theme: "system",
      resolvedTheme: "light",
    });
  });

  it("opens and closes now playing", () => {
    useUIStore.getState().openNowPlaying();
    expect(useUIStore.getState().isNowPlayingOpen).toBe(true);
    useUIStore.getState().closeNowPlaying();
    expect(useUIStore.getState().isNowPlayingOpen).toBe(false);
  });

  it("toggles now playing", () => {
    useUIStore.getState().toggleNowPlaying();
    expect(useUIStore.getState().isNowPlayingOpen).toBe(true);
    useUIStore.getState().toggleNowPlaying();
    expect(useUIStore.getState().isNowPlayingOpen).toBe(false);
  });

  it("resolves theme from system", () => {
    useUIStore.getState().setResolvedTheme("dark");
    expect(useUIStore.getState().resolvedTheme).toBe("dark");
  });
});
