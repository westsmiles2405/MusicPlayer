import { invoke } from "@tauri-apps/api/core";

export interface Artist {
  id: number;
  name: string;
  addedAt: number;
  updatedAt: number;
}

export const artistRepo = {
  list: () => invoke<Artist[]>("get_artists"),
};
