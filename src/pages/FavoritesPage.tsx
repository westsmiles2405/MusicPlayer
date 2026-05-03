import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { favoriteRepo } from "@/repositories/favoriteRepo";
import { playlistRepo } from "@/repositories/playlistRepo";
import { TrackTable } from "@/components/library/TrackTable";
import {
  PageHeader,
  LoadingState,
  ErrorState,
} from "@/components/layout";
import { DopamineEmptyState } from "@/components/ui";
import { useToggleFavoriteMutation } from "@/hooks/useToggleFavoriteMutation";

export default function FavoritesPage() {
  const queryClient = useQueryClient();
  const toggleFavorite = useToggleFavoriteMutation();

  const favorites = useQuery({
    queryKey: ["favoriteTracks"],
    queryFn: favoriteRepo.list,
  });

  const playlists = useQuery({
    queryKey: ["playlists"],
    queryFn: playlistRepo.list,
  });

  const addToPlaylist = useMutation({
    mutationFn: ({
      trackId,
      playlistId,
    }: {
      trackId: number;
      playlistId: number;
    }) => playlistRepo.addTrack(playlistId, trackId),
    onSuccess: (_pos, vars) => {
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
      queryClient.invalidateQueries({
        queryKey: ["playlistTracks", vars.playlistId],
      });
    },
  });

  return (
    <>
      <PageHeader title="收藏" />
      {favorites.isLoading && <LoadingState title="加载中" />}
      {favorites.isError && (
        <ErrorState message={favorites.error?.message ?? "加载失败"} />
      )}
      {favorites.data?.length === 0 && (
        <DopamineEmptyState
          context="favorites"
          title="还没有收藏"
          description="点击心形图标收藏你喜欢的歌曲"
        />
      )}
      {favorites.data && favorites.data.length > 0 && (
        <TrackTable
          tracks={favorites.data}
          queueContext="songs"
          playlists={playlists.data ?? []}
          showFavorite
          onToggleFavorite={(track, favorite) =>
            toggleFavorite.mutate({ track, favorite })
          }
          onAddToPlaylist={(track, playlistId) =>
            addToPlaylist.mutateAsync({ trackId: track.id, playlistId })
          }
        />
      )}
    </>
  );
}
