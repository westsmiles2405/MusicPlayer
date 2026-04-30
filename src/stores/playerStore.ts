import { create } from "zustand";

interface PlayerState {
  isPlaying: boolean;
  currentTrackId: number | null;
  position: number;
  duration: number;
  volume: number;
}

export const usePlayerStore = create<PlayerState>(() => ({
  isPlaying: false,
  currentTrackId: null,
  position: 0,
  duration: 0,
  volume: 0.8,
}));
