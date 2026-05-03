import { useEffect, useState } from "react";
import { listen } from "@/lib/tauri";
import {
  libraryRepo,
  type ScanProgress,
  type ScanReport,
} from "@/repositories/libraryRepo";

export type ScanPhase = "idle" | "scanning" | "done";

export function useScanProgress(): {
  phase: ScanPhase;
  progress: ScanProgress | null;
  report: ScanReport | null;
  cancel: () => void;
} {
  const [phase, setPhase] = useState<ScanPhase>("idle");
  const [progress, setProgress] = useState<ScanProgress | null>(null);
  const [report, setReport] = useState<ScanReport | null>(null);

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    listen<ScanProgress>("scan_progress", (e) => {
      setPhase("scanning");
      setProgress(e.payload);
    }).then((u) => unsubs.push(u));
    listen<ScanReport>("scan_done", (e) => {
      setPhase("done");
      setReport(e.payload);
    }).then((u) => unsubs.push(u));
    return () => {
      for (const u of unsubs) u();
    };
  }, []);

  return {
    phase,
    progress,
    report,
    cancel: () => {
      libraryRepo.cancelScan().catch(() => {});
    },
  };
}
