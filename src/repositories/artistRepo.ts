import { invoke } from "@/lib/tauri";

export interface Artist {
  id: number;
  name: string;
  addedAt: number;
  updatedAt: number;
}

export const artistRepo = {
  list: () => invoke<Artist[]>("get_artists"),
  get: (artistId: number) => invoke<Artist | null>("get_artist", { artistId }),
};
