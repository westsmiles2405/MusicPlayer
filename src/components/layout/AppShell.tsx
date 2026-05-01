import { Outlet } from "react-router";
import { MiniPlayer } from "@/components/player";
import { ScanProgressBar } from "./ScanProgressBar";
import { Sidebar } from "./Sidebar";

export function AppShell() {
  return (
    <div className="app-shell">
      <Sidebar />
      <main className="app-shell__main">
        <Outlet />
      </main>
      <MiniPlayer />
      <ScanProgressBar />
    </div>
  );
}
