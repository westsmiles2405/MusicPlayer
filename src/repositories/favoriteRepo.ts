import { invoke } from "@/lib/tauri";
import type { Track } from "./trackRepo";

export const favoriteRepo = {
  list: () => invoke<Track[]>("library_get_favorite_tracks"),
  set: (trackId: number, favorite: boolean) =>
    invoke<void>("set_favorite", { trackId, favorite }),
};
