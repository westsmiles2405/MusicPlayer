import {
  Pause,
  Play,
  SkipBack,
  SkipForward,
  Volume2,
  VolumeX,
} from "lucide-react";
import { motion } from "framer-motion";
import { useMemo } from "react";
import { usePlayer } from "@/hooks/usePlayer";
import { useUIStore } from "@/stores/uiStore";
import { selectDisplayPositionMs, usePlayerStore } from "@/stores/playerStore";

function formatTime(ms: number) {
  const total = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function MiniPlayer() {
  const player = usePlayer();
  const openNowPlaying = useUIStore((s) => s.openNowPlaying);
  const displayPosition = usePlayerStore(selectDisplayPositionMs);
  const title = player.current?.title ?? "未播放";
  const subtitle = [player.current?.artistName, player.current?.albumName]
    .filter(Boolean)
    .join(" - ");
  const canControl = player.current !== null || (player.status !== "idle" && player.status !== "stopped");
  const progressMax = Math.max(player.durationMs, 1);
  const errorText = player.error?.message;

  const progressLabel = useMemo(
    () => `${formatTime(displayPosition)} / ${formatTime(player.durationMs)}`,
    [displayPosition, player.durationMs],
  );

  const isPlaying = player.status === "playing";

  return (
    <footer className="mini-player" aria-label="播放器">
      <div
        className="mini-player__track"
        role="button"
        tabIndex={0}
        onClick={openNowPlaying}
        onKeyDown={(e) => {
          if (e.key === "Enter") openNowPlaying();
        }}
      >
        <motion.div
          className="mini-player__art"
          aria-hidden="true"
          layoutId="now-playing-cover"
          style={{ borderRadius: 8 }}
        />
        <div className="mini-player__meta">
          <div className="mini-player__title">{title}</div>
          <div className="mini-player__subtitle">{errorText ?? subtitle}</div>
        </div>
      </div>

      <div className="mini-player__controls">
        <button
          type="button"
          onClick={player.previous}
          disabled={!canControl}
          aria-label="上一首"
        >
          <SkipBack size={18} />
        </button>
        <button
          type="button"
          onClick={player.toggle}
          disabled={!canControl}
          aria-label={isPlaying ? "暂停" : "播放"}
        >
          {isPlaying ? <Pause size={19} /> : <Play size={19} />}
        </button>
        <button
          type="button"
          onClick={player.next}
          disabled={!canControl}
          aria-label="下一首"
        >
          <SkipForward size={18} />
        </button>
      </div>

      <div className="mini-player__progress">
        <span>{formatTime(displayPosition)}</span>
        <input
          aria-label={progressLabel}
          type="range"
          min={0}
          max={progressMax}
          value={Math.min(displayPosition, progressMax)}
          onPointerDown={() =>
            usePlayerStore.getState().beginSeek(displayPosition)
          }
          onChange={(event) =>
            usePlayerStore
              .getState()
              .updateSeekPreview(Number(event.currentTarget.value))
          }
          onPointerUp={(event) => {
            const nextPosition = Number(event.currentTarget.value);
            player
              .seek(nextPosition)
              .finally(() => usePlayerStore.getState().endSeek());
          }}
          disabled={!canControl}
        />
        <span>{formatTime(player.durationMs)}</span>
      </div>

      <div className="mini-player__volume">
        <button
          type="button"
          onClick={player.toggleMute}
          aria-label={player.muted ? "取消静音" : "静音"}
        >
          {player.muted ? <VolumeX size={18} /> : <Volume2 size={18} />}
        </button>
        <input
          aria-label="音量"
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={player.volume}
          onChange={(event) =>
            player.setVolume(Number(event.currentTarget.value))
          }
        />
      </div>
    </footer>
  );
}
