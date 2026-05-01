import { invoke } from "@tauri-apps/api/core";
import type { Track } from "./trackRepo";

export interface RecentPlayedTrack extends Track {
  lastPlayedAt: number;
}

export const recentPlayRepo = {
  list: (limit = 50) =>
    invoke<RecentPlayedTrack[]>("library_get_recent_played_tracks", { limit }),
};
