import { create } from "zustand";

type ResolvedTheme = "light" | "dark";

interface UIState {
  isNowPlayingOpen: boolean;
  sidebarCollapsed: boolean;
  theme: "system";
  resolvedTheme: ResolvedTheme;
  openNowPlaying: () => void;
  closeNowPlaying: () => void;
  toggleNowPlaying: () => void;
  setResolvedTheme: (theme: ResolvedTheme) => void;
}

export const useUIStore = create<UIState>((set) => ({
  isNowPlayingOpen: false,
  sidebarCollapsed: false,
  theme: "system",
  resolvedTheme: "light",
  openNowPlaying: () => set({ isNowPlayingOpen: true }),
  closeNowPlaying: () => set({ isNowPlayingOpen: false }),
  toggleNowPlaying: () =>
    set((state) => ({ isNowPlayingOpen: !state.isNowPlayingOpen })),
  setResolvedTheme: (resolvedTheme) => set({ resolvedTheme }),
}));
