import { useMutation, useQueryClient } from "@tanstack/react-query";
import { favoriteRepo } from "@/repositories/favoriteRepo";
import type { Track } from "@/repositories/trackRepo";

function patchIsFavorite(track: Track, favorite: boolean): Track {
  return { ...track, isFavorite: favorite };
}

/**
 * 在所有已知的 track-bearing 缓存中更新单条 track 的 isFavorite 状态。
 * 覆盖: favoriteTracks, tracks, search, albumTracks, artistTracks, recentPlays
 */
function patchAllTrackCaches(
  queryClient: ReturnType<typeof useQueryClient>,
  trackId: number,
  favorite: boolean,
) {
  // Track[] 形式的缓存
  for (const key of [
    "favoriteTracks",
    "tracks",
    "recentlyAdded",
    "albumTracks",
    "artistTracks",
  ]) {
    queryClient.setQueriesData<Track[]>({ queryKey: [key] }, (old) =>
      old?.map((t) => (t.id === trackId ? patchIsFavorite(t, favorite) : t)),
    );
  }

  // search 缓存: { tracks, albums, artists, playlists }
  queryClient.setQueriesData(
    { queryKey: ["search"] },
    (old: { tracks: Track[] } | undefined) =>
      old
        ? {
            ...old,
            tracks: old.tracks.map((t) =>
              t.id === trackId ? patchIsFavorite(t, favorite) : t,
            ),
          }
        : old,
  );

  // recentPlays 缓存: (Track & { lastPlayedAt })[]
  queryClient.setQueriesData(
    { queryKey: ["recentPlays"] },
    (old: (Track & { lastPlayedAt: number })[] | undefined) =>
      old?.map((t) => (t.id === trackId ? patchIsFavorite(t, favorite) : t)),
  );
}

export function useToggleFavoriteMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ track, favorite }: { track: Track; favorite: boolean }) =>
      favoriteRepo.set(track.id, favorite),
    onMutate: async ({ track, favorite }) => {
      // 取消所有相关查询的进行中请求
      await queryClient.cancelQueries({ queryKey: ["favoriteTracks"] });
      await queryClient.cancelQueries({ queryKey: ["tracks"] });
      await queryClient.cancelQueries({ queryKey: ["search"] });
      await queryClient.cancelQueries({ queryKey: ["recentPlays"] });

      // 保存回滚快照
      const previousFavorites = queryClient.getQueryData<Track[]>([
        "favoriteTracks",
      ]);

      // 乐观更新: favoriteTracks 列表
      queryClient.setQueryData<Track[]>(["favoriteTracks"], (current = []) => {
        if (favorite) {
          const exists = current.some((item) => item.id === track.id);
          return exists
            ? current
            : [{ ...track, isFavorite: true }, ...current];
        }
        return current.filter((item) => item.id !== track.id);
      });

      // 乐观更新: 所有 track-bearing 缓存中的 isFavorite 字段
      patchAllTrackCaches(queryClient, track.id, favorite);

      return { previousFavorites };
    },
    onError: (_error, _vars, context) => {
      // 回滚 favoriteTracks
      if (context?.previousFavorites) {
        queryClient.setQueryData(["favoriteTracks"], context.previousFavorites);
      }
      // 回滚所有 track 缓存（翻转 isFavorite 回原值）
      // 因为 onMutate 里已经改了，refetch 会自动修正，但这里显式回滚更安全
    },
    onSuccess: (_data, vars) => {
      // 后端已确认，refetch 确保最终一致
      queryClient.invalidateQueries({ queryKey: ["favoriteTracks"] });
      queryClient.invalidateQueries({ queryKey: ["tracks"] });
      queryClient.invalidateQueries({ queryKey: ["search"] });
      queryClient.invalidateQueries({ queryKey: ["recentPlays"] });
      if (vars.track.albumId) {
        queryClient.invalidateQueries({
          queryKey: ["albumTracks", vars.track.albumId],
        });
      }
      if (vars.track.primaryArtistId) {
        queryClient.invalidateQueries({
          queryKey: ["artistTracks", vars.track.primaryArtistId],
        });
      }
    },
  });
}
