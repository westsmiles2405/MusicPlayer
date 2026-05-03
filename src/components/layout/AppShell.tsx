import { useEffect, useRef } from "react";
import { Outlet, useLocation, useNavigate } from "react-router";
import { useQueryClient } from "@tanstack/react-query";
import { AnimatePresence, LayoutGroup, motion } from "framer-motion";
import { ChevronLeft, ChevronRight } from "lucide-react";
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
  const shellRef = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();
  const location = useLocation();

  useEffect(() => {
    if (phase === "done" && lastPhase.current !== "done") {
      invalidateAfterScan(queryClient);
    }
    lastPhase.current = phase;
  }, [phase, queryClient]);

  useEffect(() => {
    const shell = shellRef.current;
    if (!shell) return;

    let raf = 0;
    const onMove = (e: MouseEvent) => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(() => {
        const x = ((e.clientX / window.innerWidth) * 100).toFixed(1);
        const y = ((e.clientY / window.innerHeight) * 100).toFixed(1);
        shell.style.setProperty("--mouse-x", `${x}%`);
        shell.style.setProperty("--mouse-y", `${y}%`);
      });
    };

    window.addEventListener("mousemove", onMove, { passive: true });
    return () => {
      window.removeEventListener("mousemove", onMove);
      cancelAnimationFrame(raf);
    };
  }, []);

  return (
    <div className="app-shell" ref={shellRef}>
      <Sidebar />
      <main className="app-shell__main">
        <div className="nav-bar">
          <button
            type="button"
            className="nav-bar__btn"
            onClick={() => navigate(-1)}
            aria-label="后退"
          >
            <ChevronLeft size={16} />
          </button>
          <button
            type="button"
            className="nav-bar__btn"
            onClick={() => navigate(1)}
            aria-label="前进"
          >
            <ChevronRight size={16} />
          </button>
        </div>
        <AnimatePresence mode="wait">
          <motion.div
            key={location.pathname}
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -20 }}
            transition={{ duration: 0.22, ease: [0.32, 0.72, 0, 1] }}
          >
            <Outlet />
          </motion.div>
        </AnimatePresence>
      </main>
      <LayoutGroup id="now-playing">
        <MiniPlayer />
        <ScanProgressBar />
        <NowPlayingOverlay open={isNowPlayingOpen} onClose={closeNowPlaying} />
      </LayoutGroup>
    </div>
  );
}
