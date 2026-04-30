import { invoke } from "@tauri-apps/api/core";

export type TrackSort = "title" | "artist" | "album" | "addedAt" | "lastPlayed";

// Track + denormalized album/artist names from Rust TrackView (#[serde(flatten)])
export interface Track {
  id: number;
  filePath: string;
  fileSize: number;
  fileModifiedAt: number;
  hash: string | null;
  title: string;
  albumId: number | null;
  primaryArtistId: number | null;
  albumArtistId: number | null;
  trackNo: number | null;
  discNo: number | null;
  year: number | null;
  genre: string | null;
  durationMs: number;
  bitrate: number | null;
  sampleRate: number | null;
  channels: number | null;
  codec: string | null;
  isFavorite: boolean;
  playCount: number;
  lastPlayedAt: number | null;
  lastSeenAt: number;
  missingAt: number | null;
  addedAt: number;
  updatedAt: number;
  albumName: string | null;
  primaryArtistName: string | null;
}

export interface ListParams {
  sort?: TrackSort;
  limit?: number;
  offset?: number;
}

export const trackRepo = {
  list: (params?: ListParams) => invoke<Track[]>("get_tracks", { params }),
  byAlbum: (albumId: number) => invoke<Track[]>("get_album_tracks", { albumId }),
  byArtist: (artistId: number) => invoke<Track[]>("get_artist_tracks", { artistId }),
  recentlyAdded: (limit = 50) => invoke<Track[]>("get_recently_added", { limit }),
  setFavorite: (trackId: number, favorite: boolean) =>
    invoke<void>("set_favorite", { trackId, favorite }),
  recordPlay: (trackId: number, durationPlayedMs: number, completed: boolean) =>
    invoke<number>("record_play", { trackId, durationPlayedMs, completed }),
};
