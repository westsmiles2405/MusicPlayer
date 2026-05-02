import { useEffect, useRef } from "react";
import { Outlet } from "react-router";
import { useQueryClient } from "@tanstack/react-query";
import { MiniPlayer, NowPlayingOverlay } from "@/components/player";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import { useScanProgress } from "@/hooks/useScanProgress";
import { useSystemTheme } from "@/hooks/useSystemTheme";
import { useUIStore } from "@/stores/uiStore";
import { ScanProgressBar } from "./ScanProgressBar";
import { Sidebar } from "./Sidebar";
import { invalidateAfterScan } from "./queryInvalidation";

export function AppShell() {
  useKeyboardShortcuts();
  useSystemTheme();
  const { phase } = useScanProgress();
  const queryClient = useQueryClient();
  const lastPhase = useRef<typeof phase | null>(null);
  const isNowPlayingOpen = useUIStore((s) => s.isNowPlayingOpen);
  const closeNowPlaying = useUIStore((s) => s.closeNowPlaying);

  useEffect(() => {
    if (phase === "done" && lastPhase.current !== "done") {
      invalidateAfterScan(queryClient);
    }
    lastPhase.current = phase;
  }, [phase, queryClient]);

  return (
    <div className="app-shell">
      <Sidebar />
      <main className="app-shell__main">
        <Outlet />
      </main>
      <MiniPlayer />
      <ScanProgressBar />
      <NowPlayingOverlay open={isNowPlayingOpen} onClose={closeNowPlaying} />
    </div>
  );
}
