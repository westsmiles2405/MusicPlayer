import { useMemo } from "react";
import type { Track } from "@/repositories/trackRepo";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { recentPlayRepo } from "@/repositories/recentPlayRepo";
import { playlistRepo } from "@/repositories/playlistRepo";
import { TrackTable } from "@/components/library/TrackTable";
import {
  PageHeader,
  LoadingState,
  ErrorState,
} from "@/components/layout";
import { DopamineEmptyState } from "@/components/ui";
import { useToggleFavoriteMutation } from "@/hooks/useToggleFavoriteMutation";

export default function RecentPlaysPage() {
  const queryClient = useQueryClient();
  const toggleFavorite = useToggleFavoriteMutation();

  const recentPlays = useQuery({
    queryKey: ["recentPlays"],
    queryFn: () => recentPlayRepo.list(50),
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

  const tracks = useMemo(
    () =>
      recentPlays.data?.map(
        ({ lastPlayedAt: _lastPlayedAt, ...track }): Track => track as Track,
      ) ?? [],
    [recentPlays.data],
  );

  return (
    <>
      <PageHeader title="最近播放" />
      {recentPlays.isLoading && <LoadingState title="加载中" />}
      {recentPlays.isError && (
        <ErrorState message={recentPlays.error?.message ?? "加载失败"} />
      )}
      {recentPlays.data?.length === 0 && (
        <DopamineEmptyState
          context="recent"
          title="还没有最近播放记录"
          description="播放过的歌曲会出现在这里"
        />
      )}
      {recentPlays.data && recentPlays.data.length > 0 && (
        <TrackTable
          tracks={tracks}
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
