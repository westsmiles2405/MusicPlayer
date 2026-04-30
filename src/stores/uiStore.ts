import { create } from "zustand";

interface UIState {
  isNowPlayingOpen: boolean;
  sidebarCollapsed: boolean;
}

export const useUIStore = create<UIState>(() => ({
  isNowPlayingOpen: false,
  sidebarCollapsed: false,
}));
