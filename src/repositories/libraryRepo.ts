import { invoke } from "@tauri-apps/api/core";

export interface ScanFolder {
  id: number;
  path: string;
  addedAt: number;
  lastScannedAt: number | null;
}

export interface ScanProgress {
  done: number;
  total: number;
  currentFile: string | null;
}

export interface ScanError {
  path: string;
  message: string;
}

export interface ScanReport {
  added: number;
  updated: number;
  moved: number;
  unchanged: number;
  missing: number;
  errors: ScanError[];
}

export const libraryRepo = {
  addFolder: (path: string) => invoke<ScanFolder>("add_music_folder", { path }),
  listFolders: () => invoke<ScanFolder[]>("list_music_folders"),
  removeFolder: (id: number) => invoke<void>("remove_music_folder", { id }),
  startScan: () => invoke<void>("start_scan"),
  cancelScan: () => invoke<void>("cancel_scan"),
};
