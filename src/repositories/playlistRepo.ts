import { invoke } from "@tauri-apps/api/core";
import type { Track } from "./trackRepo";

export interface Playlist {
  id: number;
  name: string;
  description: string | null;
  coverPath: string | null;
  createdAt: number;
  updatedAt: number;
  trackCount: number;
}

export const playlistRepo = {
  list: () => invoke<Playlist[]>("get_playlists"),
  tracks: (playlistId: number) => invoke<Track[]>("get_playlist_tracks", { playlistId }),
  create: (name: string, description?: string) =>
    invoke<number>("create_playlist", { name, description: description ?? null }),
  rename: (playlistId: number, name: string) =>
    invoke<void>("rename_playlist", { playlistId, name }),
  delete: (playlistId: number) => invoke<void>("delete_playlist", { playlistId }),
  addTrack: (playlistId: number, trackId: number) =>
    invoke<number>("add_to_playlist", { playlistId, trackId }),
  removeTrack: (playlistId: number, trackId: number, position: number) =>
    invoke<void>("remove_from_playlist", { playlistId, trackId, position }),
  reorder: (playlistId: number, fromPosition: number, toPosition: number) =>
    invoke<void>("reorder_playlist", { playlistId, fromPosition, toPosition }),
};
