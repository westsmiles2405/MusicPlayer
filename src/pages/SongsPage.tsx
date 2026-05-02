import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { trackRepo, type TrackSort } from "@/repositories/trackRepo";
import { playlistRepo } from "@/repositories/playlistRepo";
import { TrackTable } from "@/components/library/TrackTable";
import {
  PageHeader,
  EmptyState,
  LoadingState,
  ErrorState,
} from "@/components/layout";

const SORTS: { label: string; value: TrackSort }[] = [
  { label: "按标题", value: "title" },
  { label: "按艺人", value: "artist" },
  { label: "按专辑", value: "album" },
  { label: "按添加时间", value: "addedAt" },
];

export default function SongsPage() {
  const [sort, setSort] = useState<TrackSort>("title");
  const queryClient = useQueryClient();

  const tracks = useQuery({
    queryKey: ["tracks", sort],
    queryFn: async () => {
      const result = await trackRepo.list({ sort });
      console.log("[SongsPage] tracks result:", result.length, result);
      return result;
    },
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
      <PageHeader
        title="歌曲"
        action={
          <div className="flex gap-2">
            {SORTS.map((s) => (
              <button
                key={s.value}
                type="button"
                onClick={() => setSort(s.value)}
                aria-label={s.label}
                className={sort === s.value ? "font-bold" : ""}
              >
                {s.label}
              </button>
            ))}
          </div>
        }
      />
      {tracks.isLoading && <LoadingState title="加载中" />}
      {tracks.isError && (
        <ErrorState message={tracks.error?.message ?? "加载失败"} />
      )}
      {tracks.data?.length === 0 && (
        <EmptyState title="没有歌曲" description="扫描音乐文件夹以添加歌曲" />
      )}
      {tracks.data && tracks.data.length > 0 && (
        <TrackTable
          tracks={tracks.data}
          queueContext="songs"
          playlists={playlists.data ?? []}
          onAddToPlaylist={(track, playlistId) =>
            addToPlaylist.mutateAsync({ trackId: track.id, playlistId })
          }
        />
      )}
    </>
  );
}
