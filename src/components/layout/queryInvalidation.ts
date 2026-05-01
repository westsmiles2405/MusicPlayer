import type { QueryClient } from "@tanstack/react-query";

const refreshableRoots = new Set([
  "tracks",
  "recentlyAdded",
  "albums",
  "albumTracks",
  "artists",
  "artistTracks",
  "playlistTracks",
]);

export function invalidateAfterScan(queryClient: QueryClient) {
  queryClient.invalidateQueries({
    predicate: (query) => refreshableRoots.has(String(query.queryKey[0])),
  });
}
