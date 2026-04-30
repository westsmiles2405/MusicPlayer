import { invoke } from "@tauri-apps/api/core";
import type { Track } from "./trackRepo";

export const searchRepo = {
  query: (query: string, limit = 50) => invoke<Track[]>("search", { query, limit }),
};
