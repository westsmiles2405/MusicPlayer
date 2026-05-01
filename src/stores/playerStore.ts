import { create } from "zustand";
import type {
  NowPlayingTrack,
  PlaybackError,
  PlaybackProgress,
  PlayerSnapshot,
  PlaybackStatus,
  RepeatMode,
} from "@/repositories/playerRepo";

export interface PlayerStore extends PlayerSnapshot {
  error: PlaybackError | null;
  isSeeking: boolean;
  optimisticPositionMs: number | null;
  applySnapshot: (snapshot: PlayerSnapshot) => void;
  applyProgress: (progress: PlaybackProgress) => void;
  applyTrackChanged: (track: NowPlayingTrack | null) => void;
  applyError: (error: PlaybackError) => void;
  beginSeek: (positionMs: number) => void;
  updateSeekPreview: (positionMs: number) => void;
  endSeek: () => void;
}

const initialState: PlayerSnapshot = {
  status: "idle" as PlaybackStatus,
  current: null,
  positionMs: 0,
  durationMs: 0,
  volume: 0.8,
  muted: false,
  queueIndex: null,
  queueLen: 0,
  repeatMode: "off" as RepeatMode,
  shuffle: false,
};

export const usePlayerStore = create<PlayerStore>((set) => ({
  ...initialState,
  error: null,
  isSeeking: false,
  optimisticPositionMs: null,

  applySnapshot: (snapshot) =>
    set((state) => ({
      ...snapshot,
      positionMs: state.isSeeking ? state.positionMs : snapshot.positionMs,
      error: snapshot.status === "playing" ? null : state.error,
    })),

  applyProgress: (progress) =>
    set((state) =>
      state.isSeeking
        ? { durationMs: progress.durationMs }
        : {
            positionMs: progress.positionMs,
            durationMs: progress.durationMs,
          },
    ),

  applyTrackChanged: (track) =>
    set({
      current: track,
      positionMs: 0,
      durationMs: track?.durationMs ?? 0,
      optimisticPositionMs: null,
      isSeeking: false,
    }),

  applyError: (error) =>
    set({
      error,
      status: error.recoverable ? undefined : ("error" as PlaybackStatus),
    } as Partial<PlayerStore>),

  beginSeek: (positionMs) =>
    set({ isSeeking: true, optimisticPositionMs: positionMs }),

  updateSeekPreview: (positionMs) => set({ optimisticPositionMs: positionMs }),

  endSeek: () => set({ isSeeking: false, optimisticPositionMs: null }),
}));

export const selectDisplayPositionMs = (state: PlayerStore) =>
  state.optimisticPositionMs ?? state.positionMs;
