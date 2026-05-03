import { useCallback, useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { listen } from "@/lib/tauri";
import {
  playerRepo,
  type NowPlayingTrack,
  type PlaybackError,
  type PlaybackProgress,
  type PlayerSnapshot,
} from "@/repositories/playerRepo";
import { usePlayerStore } from "@/stores/playerStore";

export function usePlayerEvents() {
  const queryClient = useQueryClient();
  const applySnapshot = usePlayerStore((s) => s.applySnapshot);
  const applyProgress = usePlayerStore((s) => s.applyProgress);
  const applyTrackChanged = usePlayerStore((s) => s.applyTrackChanged);
  const applyError = usePlayerStore((s) => s.applyError);

  useEffect(() => {
    let active = true;
    const unlisteners = Promise.all([
      listen<PlayerSnapshot>("playback_state", (event) =>
        applySnapshot(event.payload),
      ).catch(() => null),
      listen<PlaybackProgress>("playback_progress", (event) =>
        applyProgress(event.payload),
      ).catch(() => null),
      listen<NowPlayingTrack | null>("track_changed", (event) => {
        applyTrackChanged(event.payload);
        queryClient.invalidateQueries({ queryKey: ["recentPlays"] });
      }).catch(() => null),
      listen<PlaybackError>("playback_error", (event) =>
        applyError(event.payload),
      ).catch(() => null),
    ]);

    // Polling fallback: sync state every 200ms in case events are missed
    const poll = setInterval(() => {
      if (!active) return;
      playerRepo.getState().then((s) => {
        if (active) applySnapshot(s);
      }).catch(() => {});
    }, 200);

    playerRepo
      .getState()
      .then((snapshot) => {
        if (active) applySnapshot(snapshot);
      })
      .catch(() => {});

    return () => {
      active = false;
      clearInterval(poll);
      unlisteners.then((callbacks) =>
        callbacks.forEach((unlisten) => unlisten?.()),
      );
    };
  }, [applyError, applyProgress, applySnapshot, applyTrackChanged]);
}

export function usePlayer() {
  usePlayerEvents();
  const snapshot = usePlayerStore();

  const play = useCallback(
    (trackId: number, queueTrackIds?: number[], queueIndex?: number) =>
      playerRepo.play({ trackId, queueTrackIds, queueIndex }),
    [],
  );

  return {
    ...snapshot,
    play,
    pause: playerRepo.pause,
    resume: playerRepo.resume,
    toggle: playerRepo.toggle,
    stop: playerRepo.stop,
    seek: playerRepo.seek,
    next: playerRepo.next,
    previous: playerRepo.previous,
    setVolume: playerRepo.setVolume,
    setMuted: playerRepo.setMuted,
    toggleMute: playerRepo.toggleMute,
  };
}
