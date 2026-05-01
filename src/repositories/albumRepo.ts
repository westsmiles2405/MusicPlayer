import { invoke } from "@tauri-apps/api/core";

export interface Album {
  id: number;
  name: string;
  albumArtistId: number;
  year: number | null;
  coverPath: string | null;
  addedAt: number;
  updatedAt: number;
  albumArtistName: string;
  trackCount: number;
}

export const albumRepo = {
  list: () => invoke<Album[]>("get_albums"),
  get: (albumId: number) =>
    invoke<Album | null>("get_album", { albumId }),
};
