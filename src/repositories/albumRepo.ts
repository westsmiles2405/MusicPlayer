export interface Album {
  id: number;
  name: string;
  artistName: string;
  year?: number;
  coverPath?: string;
}

export async function getAlbums(): Promise<Album[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<Album[]>("get_albums");
}
