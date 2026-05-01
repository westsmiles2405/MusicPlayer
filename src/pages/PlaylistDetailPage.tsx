import { useState } from "react";
import { useParams, useNavigate } from "react-router";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { playlistRepo } from "@/repositories/playlistRepo";
import { TrackTable } from "@/components/library/TrackTable";
import { PageHeader, LoadingState, ErrorState, EmptyState } from "@/components/layout";
import { PlaylistRenameDialog, PlaylistDeleteDialog } from "@/components/playlists";
import { usePlayer } from "@/hooks/usePlayer";

export default function PlaylistDetailPage() {
  const { id } = useParams();
  const navigate = useNavigate();
  const playlistId = Number(id);
  const player = usePlayer();
  const queryClient = useQueryClient();

  const [renameOpen, setRenameOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);

  const playlists = useQuery({
    queryKey: ["playlists"],
    queryFn: playlistRepo.list,
    enabled: Number.isFinite(playlistId),
  });

  const tracks = useQuery({
    queryKey: ["playlistTracks", playlistId],
    queryFn: () => playlistRepo.tracks(playlistId),
    enabled: Number.isFinite(playlistId),
  });

  const playlist = playlists.data?.find((p) => p.id === playlistId);

  const renameMutation = useMutation({
    mutationFn: (name: string) => playlistRepo.rename(playlistId, name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: () => playlistRepo.delete(playlistId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
      navigate("/playlists", { replace: true });
    },
  });

  const removeTrackMutation = useMutation({
    mutationFn: ({
      trackId,
      position,
    }: {
      trackId: number;
      position: number;
    }) => playlistRepo.removeTrack(playlistId, trackId, position),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["playlistTracks", playlistId] });
      queryClient.invalidateQueries({ queryKey: ["playlists"] });
    },
  });

  const reorderMutation = useMutation({
    mutationFn: ({
      sourcePosition,
      destinationPosition,
    }: {
      sourcePosition: number;
      destinationPosition: number;
    }) => playlistRepo.reorder(playlistId, sourcePosition, destinationPosition),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["playlistTracks", playlistId] });
    },
  });

  if (!Number.isFinite(playlistId)) {
    return <ErrorState message="无效的播放列表 ID" />;
  }

  const playableTracks = tracks.data?.filter((t) => t.missingAt === null) ?? [];
  const canPlay = playableTracks.length > 0;

  return (
    <>
      {playlists.isLoading && <LoadingState title="加载中" />}
      {playlists.isError && (
        <ErrorState message={playlists.error?.message ?? "加载失败"} />
      )}
      {playlists.data && !playlist && (
        <ErrorState message="播放列表不存在" />
      )}
      {playlist && (
        <>
          <PageHeader
            title={playlist.name}
            action={
              <div className="flex gap-2">
                <button
                  type="button"
                  disabled={!canPlay}
                  onClick={() => {
                    if (!canPlay || playableTracks.length === 0) return;
                    const first = playableTracks[0]!;
                    player.play(
                      first.id,
                      playableTracks.map((t) => t.id),
                      0,
                    );
                  }}
                >
                  播放
                </button>
                <button type="button" onClick={() => setRenameOpen(true)}>
                  重命名
                </button>
                <button type="button" onClick={() => setDeleteOpen(true)}>
                  删除
                </button>
              </div>
            }
          />

          {tracks.isLoading && <LoadingState title="加载中" />}
          {tracks.isError && (
            <ErrorState message={tracks.error?.message ?? "加载失败"} />
          )}
          {tracks.data?.length === 0 && (
            <EmptyState title="播放列表为空" description="从歌曲页面添加音乐到此播放列表" />
          )}
          {tracks.data && tracks.data.length > 0 && (
            <TrackTable
              tracks={tracks.data}
              queueContext="playlist"
              onRemoveFromPlaylist={(row) => {
                if (row.playlistPosition !== undefined) {
                  removeTrackMutation.mutateAsync({
                    trackId: row.id,
                    position: row.playlistPosition,
                  });
                }
              }}
              onReorderPlaylist={(sourcePosition, destinationPosition) => {
                reorderMutation.mutateAsync({
                  sourcePosition,
                  destinationPosition,
                });
              }}
            />
          )}
        </>
      )}

      {playlist && (
        <>
          <PlaylistRenameDialog
            open={renameOpen}
            onClose={() => setRenameOpen(false)}
            onRename={async (name) => {
              await renameMutation.mutateAsync(name);
            }}
            initialName={playlist.name}
          />
          <PlaylistDeleteDialog
            open={deleteOpen}
            onClose={() => setDeleteOpen(false)}
            onDelete={async () => {
              await deleteMutation.mutateAsync();
            }}
            playlistName={playlist.name}
          />
        </>
      )}
    </>
  );
}
