import { create } from "zustand";

interface PrefsState {
  scanFolders: string[];
  language: string;
}

export const usePrefsStore = create<PrefsState>(() => ({
  scanFolders: [],
  language: "zh",
}));
