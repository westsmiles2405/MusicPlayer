import { invoke } from "@/lib/tauri";

export interface PlayHistoryEntry {
  id: number;
  trackId: number;
  playedAt: number;
  durationPlayedMs: number;
  completed: boolean;
  trackTitle: string;
  albumName: string | null;
  primaryArtistName: string | null;
}

export const historyRepo = {
  recent: (limit = 50) =>
    invoke<PlayHistoryEntry[]>("get_recent_plays", { limit }),
};
