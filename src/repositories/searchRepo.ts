import { invoke } from "@tauri-apps/api/core";
import type { Track } from "./trackRepo";
import type { Album } from "./albumRepo";
import type { Artist } from "./artistRepo";
import type { Playlist } from "./playlistRepo";

export interface SearchResult {
  tracks: Track[];
  albums: Album[];
  artists: Artist[];
  playlists: Playlist[];
}

export const searchRepo = {
  search: (query: string, limitPerGroup = 10) =>
    invoke<SearchResult>("library_search_all", {
      query,
      limitPerGroup,
    }),
};
