export interface Artist {
  id: number;
  name: string;
}

export async function getArtists(): Promise<Artist[]> {
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<Artist[]>("get_artists");
}
