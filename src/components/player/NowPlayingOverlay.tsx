import { AnimatePresence, motion } from "framer-motion";
import {
  Pause,
  Play,
  SkipBack,
  SkipForward,
  Volume2,
  VolumeX,
  X,
} from "lucide-react";
import { usePlayer } from "@/hooks/usePlayer";
import { selectDisplayPositionMs, usePlayerStore } from "@/stores/playerStore";

function formatTime(ms: number) {
  const total = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

export function NowPlayingOverlay({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const player = usePlayer();
  const displayPosition = usePlayerStore(selectDisplayPositionMs);
  const current = player.current;
  const isPlaying = player.status === "playing";
  const canControl = player.status !== "idle" && player.status !== "stopped";
  const progressMax = Math.max(player.durationMs, 1);

  return (
    <AnimatePresence>
      {open && (
        <motion.section
          className="now-playing"
          role="dialog"
          aria-label="Now Playing"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.3, ease: [0.32, 0.72, 0, 1] }}
        >
          <div className="np-bg">
            <div className="np-bg__gradient" />
            {current?.coverPath && (
              <div
                className="np-bg__blurred-cover"
                style={{ backgroundImage: `url(${current.coverPath})` }}
              />
            )}
            <div className="np-bg__glass" />
          </div>

          <header className="now-playing__header">
            <button
              type="button"
              onClick={onClose}
              aria-label="关闭 Now Playing"
            >
              <X size={20} />
            </button>
          </header>

          {current ? (
            <div className="now-playing__body">
              <motion.div
                className="now-playing__cover"
                data-testid="now-playing-cover"
                layoutId="now-playing-cover"
              >
                {current.coverPath ? (
                  <img src={current.coverPath} alt="" />
                ) : (
                  <div className="now-playing__cover-placeholder" />
                )}
              </motion.div>

              <div className="now-playing__info">
                <div className="now-playing__title">{current.title}</div>
                <div className="now-playing__subtitle">
                  {current.artistName}
                </div>
                {current.albumName && (
                  <div className="now-playing__album">{current.albumName}</div>
                )}
              </div>

              <div className="now-playing__progress">
                <input
                  aria-label="播放进度"
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
                    const pos = Number(event.currentTarget.value);
                    player
                      .seek(pos)
                      .finally(() => usePlayerStore.getState().endSeek());
                  }}
                  disabled={!canControl}
                />
                <div className="now-playing__time">
                  <span>{formatTime(displayPosition)}</span>
                  <span>{formatTime(player.durationMs)}</span>
                </div>
              </div>

              <div className="now-playing__controls">
                <button
                  type="button"
                  onClick={player.previous}
                  disabled={!canControl}
                  aria-label="上一首"
                >
                  <SkipBack size={24} />
                </button>
                <button
                  type="button"
                  className="now-playing__play-main"
                  onClick={player.toggle}
                  disabled={!canControl}
                  aria-label={isPlaying ? "暂停" : "播放"}
                >
                  {isPlaying ? <Pause size={28} /> : <Play size={28} />}
                </button>
                <button
                  type="button"
                  onClick={player.next}
                  disabled={!canControl}
                  aria-label="下一首"
                >
                  <SkipForward size={24} />
                </button>
              </div>

              <div className="now-playing__volume">
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
            </div>
          ) : (
            <div className="now-playing__empty">暂无播放内容</div>
          )}
        </motion.section>
      )}
    </AnimatePresence>
  );
}
