import { useParams } from "react-router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { albumRepo } from "@/repositories/albumRepo";
import { trackRepo } from "@/repositories/trackRepo";
import { playlistRepo } from "@/repositories/playlistRepo";
import { TrackTable } from "@/components/library/TrackTable";
import { LoadingState, ErrorState } from "@/components/layout";
import { CoverArt } from "@/components/library/CoverArt";
import { usePlayer } from "@/hooks/usePlayer";

export default function AlbumDetailPage() {
  const { id } = useParams();
  const albumId = Number(id);
  const player = usePlayer();
  const queryClient = useQueryClient();

  const album = useQuery({
    queryKey: ["album", albumId],
    queryFn: () => albumRepo.get(albumId),
    enabled: Number.isFinite(albumId),
  });

  const tracks = useQuery({
    queryKey: ["albumTracks", albumId],
    queryFn: () => trackRepo.byAlbum(albumId),
    enabled: Number.isFinite(albumId),
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

  const playable = tracks.data?.filter((t) => t.missingAt === null) ?? [];
  const canPlay = playable.length > 0;

  return (
    <>
      {album.isLoading && <LoadingState title="加载中" />}
      {album.isError && (
        <ErrorState message={album.error?.message ?? "加载失败"} />
      )}
      {album.data && (
        <>
          <div className="flex items-center gap-4 mb-4">
            <CoverArt
              coverPath={album.data.coverPath}
              title={album.data.name}
              size="lg"
            />
            <div>
              <h1 className="text-2xl font-bold">{album.data.name}</h1>
              <p className="text-muted">{album.data.albumArtistName}</p>
            </div>
            <button
              type="button"
              disabled={!canPlay}
              onClick={() => {
                const [first] = playable;
                if (!first) return;
                player.play(
                  first.id,
                  playable.map((t) => t.id),
                  0,
                );
              }}
              aria-label="播放专辑"
            >
              播放专辑
            </button>
          </div>
          {tracks.data && tracks.data.length > 0 && (
            <TrackTable
              tracks={tracks.data}
              queueContext="album"
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
