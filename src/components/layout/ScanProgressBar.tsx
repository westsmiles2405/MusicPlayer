import { useScanProgress } from "@/hooks/useScanProgress";

export function ScanProgressBar() {
  const { phase, progress } = useScanProgress();
  if (phase !== "scanning" || !progress) return null;
  const pct =
    progress.total > 0 ? Math.round((progress.done / progress.total) * 100) : 0;
  return (
    <div className="fixed bottom-20 left-0 right-0 z-50 px-4">
      <div className="bg-black/80 text-white rounded-md px-3 py-2 backdrop-blur-md text-xs flex items-center gap-3">
        <span className="font-mono">
          {progress.done} / {progress.total}
        </span>
        <div className="flex-1 h-1 bg-white/20 rounded overflow-hidden">
          <div className="h-full bg-white" style={{ width: `${pct}%` }} />
        </div>
        <span className="truncate max-w-xs">{progress.currentFile ?? ""}</span>
      </div>
    </div>
  );
}
