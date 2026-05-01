import { useParams } from "react-router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { artistRepo } from "@/repositories/artistRepo";
import { trackRepo } from "@/repositories/trackRepo";
import { playlistRepo } from "@/repositories/playlistRepo";
import { TrackTable } from "@/components/library/TrackTable";
import { LoadingState, ErrorState, PageHeader } from "@/components/layout";
import { usePlayer } from "@/hooks/usePlayer";

export default function ArtistDetailPage() {
  const { id } = useParams();
  const artistId = Number(id);
  const player = usePlayer();
  const queryClient = useQueryClient();

  const artist = useQuery({
    queryKey: ["artist", artistId],
    queryFn: () => artistRepo.get(artistId),
    enabled: Number.isFinite(artistId),
  });

  const tracks = useQuery({
    queryKey: ["artistTracks", artistId],
    queryFn: () => trackRepo.byArtist(artistId),
    enabled: Number.isFinite(artistId),
  });

  const playlists = useQuery({
    queryKey: ["playlists"],
    queryFn: playlistRepo.list,
  });

  const addToPlaylist = useMutation({
    mutationFn: ({ trackId, playlistId }: { trackId: number; playlistId: number }) =>
      playlistRepo.addTrack(playlistId, trackId),
    onSuccess: (_pos, vars) => {
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
      queryClient.invalidateQueries({ queryKey: ["playlistTracks", vars.playlistId] });
    },
  });

  const playable = tracks.data?.filter((t) => t.missingAt === null) ?? [];

  return (
    <>
      {artist.isLoading && <LoadingState title="加载中" />}
      {artist.isError && <ErrorState message={artist.error?.message ?? "加载失败"} />}
      {artist.data && (
        <>
          <PageHeader
            title={artist.data.name}
            action={
              <button
                type="button"
                disabled={playable.length === 0}
                onClick={() => {
                  const [first] = playable;
                  if (!first) return;
                  player.play(first.id, playable.map((t) => t.id), 0);
                }}
              >
                播放全部
              </button>
            }
          />
          {tracks.data && tracks.data.length > 0 && (
            <TrackTable
              tracks={tracks.data}
              queueContext="artist"
              playlists={playlists.data ?? []}
              onAddToPlaylist={(track, playlistId) =>
                addToPlaylist.mutateAsync({ trackId: track.id, playlistId })
              }
            />
          )}
        </>
      )}
    </>
  );
}
