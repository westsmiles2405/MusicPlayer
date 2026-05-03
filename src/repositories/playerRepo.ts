import { invoke } from "@/lib/tauri";

export type PlaybackStatus =
  | "idle"
  | "loading"
  | "buffering"
  | "playing"
  | "paused"
  | "stopped"
  | "ended"
  | "error";

export type RepeatMode = "off" | "one" | "all";

export interface NowPlayingTrack {
  id: number;
  title: string;
  albumName: string | null;
  artistName: string | null;
  durationMs: number;
  coverPath: string | null;
}

export interface PlayerSnapshot {
  status: PlaybackStatus;
  current: NowPlayingTrack | null;
  positionMs: number;
  durationMs: number;
  volume: number;
  muted: boolean;
  queueIndex: number | null;
  queueLen: number;
  repeatMode: RepeatMode;
  shuffle: boolean;
}

export interface PlaybackProgress {
  positionMs: number;
  durationMs: number;
}

export type PlaybackErrorCode =
  | "fileNotFound"
  | "permissionDenied"
  | "decodeFailed"
  | "outputUnavailable"
  | "streamError"
  | "invalidInput"
  | "unknown";

export interface PlaybackError {
  trackId?: number;
  code: PlaybackErrorCode;
  message: string;
  recoverable: boolean;
}

export interface PlayParams {
  trackId: number;
  queueTrackIds?: number[];
  queueIndex?: number;
}

export const playerRepo = {
  play: ({ trackId, queueTrackIds, queueIndex }: PlayParams) =>
    invoke<void>("player_play", { trackId, queueTrackIds, queueIndex }),
  pause: () => invoke<void>("player_pause"),
  resume: () => invoke<void>("player_resume"),
  toggle: () => invoke<void>("player_toggle"),
  stop: () => invoke<void>("player_stop"),
  seek: (positionMs: number) => invoke<void>("player_seek", { positionMs }),
  next: () => invoke<void>("player_next"),
  previous: () => invoke<void>("player_previous"),
  setVolume: (volume: number) => invoke<void>("player_set_volume", { volume }),
  setMuted: (muted: boolean) => invoke<void>("player_set_muted", { muted }),
  toggleMute: () => invoke<void>("player_toggle_mute"),
  getState: () => invoke<PlayerSnapshot>("player_get_state"),
};
