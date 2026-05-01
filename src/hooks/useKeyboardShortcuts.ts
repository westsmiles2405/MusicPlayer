import { useEffect } from "react";
import { useNavigate } from "react-router";
import { usePlayer } from "@/hooks/usePlayer";
import { useUIStore } from "@/stores/uiStore";

function isTauriRuntime() {
  return "__TAURI_INTERNALS__" in window;
}

function shouldIgnoreSpace(target: EventTarget | null) {
  if (!(target instanceof HTMLElement)) return false;
  if (target.closest('[role="slider"]')) return true;
  return target.matches("input, textarea, [contenteditable='true']");
}

export function useKeyboardShortcuts() {
  const navigate = useNavigate();
  const player = usePlayer();
  const closeNowPlaying = useUIStore((s) => s.closeNowPlaying);

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
        if (isTauriRuntime()) {
          e.preventDefault();
          navigate("/search?focus=1");
        }
        return;
      }
      if (e.key === "Escape") {
        closeNowPlaying();
        return;
      }
      if (e.key === " " && !shouldIgnoreSpace(e.target)) {
        e.preventDefault();
        player.toggle();
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [navigate, player, closeNowPlaying]);
}
