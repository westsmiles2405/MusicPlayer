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

  it("toggleNowPlaying cycles correctly", () => {
    const store = useUIStore.getState();
    store.toggleNowPlaying();
    expect(useUIStore.getState().isNowPlayingOpen).toBe(true);
    store.toggleNowPlaying();
    expect(useUIStore.getState().isNowPlayingOpen).toBe(false);
    store.toggleNowPlaying();
    expect(useUIStore.getState().isNowPlayingOpen).toBe(true);
  });

  it("setResolvedTheme updates correctly", () => {
    useUIStore.getState().setResolvedTheme("dark");
    expect(useUIStore.getState().resolvedTheme).toBe("dark");
    useUIStore.getState().setResolvedTheme("light");
    expect(useUIStore.getState().resolvedTheme).toBe("light");
  });

  it("initial state is correct", () => {
    expect(useUIStore.getState().isNowPlayingOpen).toBe(false);
    expect(useUIStore.getState().sidebarCollapsed).toBe(false);
    expect(useUIStore.getState().theme).toBe("system");
    expect(useUIStore.getState().resolvedTheme).toBe("light");
  });
});
