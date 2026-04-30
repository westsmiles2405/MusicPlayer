// Phase 1: calls Tauri IPC commands
// Phase 2: switches to HTTP API
// All UI code depends on this interface — never calls IPC directly

export interface Track {
  id: number;
  title: string;
  albumName?: string;
  artistName?: string;
  durationMs: number;
  filePath: string;
}

export async function getTracks(): Promise<Track[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<Track[]>("get_tracks");
}
