import { useState } from "react";
import { usePlayer } from "@/hooks/usePlayer";
import { usePlayerStore } from "@/stores/playerStore";
import type { Track } from "@/repositories/trackRepo";
import type { Playlist, PlaylistTrack } from "@/repositories/playlistRepo";
import { TrackTableView } from "./TrackTableView";
import type { TrackTableRow } from "./TrackTableView";

interface TrackTableProps {
  tracks: Track[];
  queueContext: "recent" | "songs" | "album" | "artist" | "playlist";
  playlists?: Playlist[];
  onAddToPlaylist?: (track: Track, playlistId: number) => Promise<unknown>;
  onRemoveFromPlaylist?: (track: TrackTableRow) => void;
  onReorderPlaylist?: (
    sourcePosition: number,
    destinationPosition: number,
  ) => void;
  showFavorite?: boolean;
  onToggleFavorite?: (track: Track, favorite: boolean) => void;
  virtual?: boolean;
}

export function TrackTable({
  tracks,
  queueContext,
  playlists,
  onAddToPlaylist,
  onRemoveFromPlaylist,
  onReorderPlaylist,
  showFavorite,
  onToggleFavorite,
  virtual,
}: TrackTableProps) {
  const { play } = usePlayer();
  const currentTrackId = usePlayerStore((s) => s.current?.id);
  const [pendingFavoriteIds, setPendingFavoriteIds] = useState<Set<number>>(
    () => new Set(),
  );

  const playableTracks = tracks.filter((t) => t.missingAt === null);
  const playableIds = playableTracks.map((t) => t.id);

  // Map each playable row to its queue index, correctly handling duplicate track IDs
  // by tracking which occurrence of each track ID we've seen.
  const queueIndexByRow: number[] = [];
  const seen = new Map<number, number>();
  for (const t of tracks) {
    if (t.missingAt !== null) {
      queueIndexByRow.push(-1);
      continue;
    }
    const nth = seen.get(t.id) ?? 0;
    let queueIdx = -1;
    let count = 0;
    for (let i = 0; i < playableIds.length; i++) {
      if (playableIds[i] === t.id) {
        if (count === nth) {
          queueIdx = i;
          break;
        }
        count++;
      }
    }
    queueIndexByRow.push(queueIdx);
    seen.set(t.id, nth + 1);
  }

  const handlePlay = (row: TrackTableRow, rowIndex: number) => {
    const queueIndex = queueIndexByRow[rowIndex] ?? -1;
    play(row.id, playableIds, queueIndex >= 0 ? queueIndex : undefined);
  };

  const handleToggleFavorite = onToggleFavorite
    ? (row: TrackTableRow) => {
        const track = tracks.find((t) => t.id === row.id);
        if (!track || pendingFavoriteIds.has(track.id)) return;
        onToggleFavorite(track, !track.isFavorite);
        // 如果调用方没有在 mutation 的 onSuccess/onError 里清除 pending，
        // 这里用乐观超时兜底：2s 后自动移除 pending 状态
        setPendingFavoriteIds((prev) => new Set(prev).add(track.id));
        setTimeout(() => {
          setPendingFavoriteIds((prev) => {
            const next = new Set(prev);
            next.delete(track.id);
            return next;
          });
        }, 2000);
      }
    : undefined;

  const rows: TrackTableRow[] = tracks.map((t) => {
    const playlistPosition =
      "playlistPosition" in t
        ? (t as PlaylistTrack).playlistPosition
        : undefined;
    return {
      id: t.id,
      title: t.title,
      albumName: t.albumName,
      primaryArtistName: t.primaryArtistName,
      durationMs: t.durationMs,
      missingAt: t.missingAt,
      playlistPosition,
      trackNo: t.trackNo ?? undefined,
      isFavorite: showFavorite ? t.isFavorite : undefined,
      isFavoritePending: pendingFavoriteIds.has(t.id),
    };
  });

  const renderActions = (row: TrackTableRow) => {
    if (!playlists || !onAddToPlaylist || row.missingAt !== null) return null;
    const track = tracks.find((t) => t.id === row.id);
    if (!track) return null;
    return (
      <select
        aria-label="选择播放列表"
        defaultValue=""
        onChange={(e) => {
          const playlistId = Number(e.target.value);
          if (playlistId) onAddToPlaylist(track, playlistId);
        }}
      >
        <option value="" disabled>
          添加到...
        </option>
        {playlists.map((p) => (
          <option key={p.id} value={p.id}>
            {p.name}
          </option>
        ))}
      </select>
    );
  };

  return (
    <TrackTableView
      rows={rows}
      queueContext={queueContext}
      onPlay={handlePlay}
      onRemove={onRemoveFromPlaylist}
      onToggleFavorite={handleToggleFavorite}
      onReorderPlaylist={onReorderPlaylist}
      renderActions={renderActions}
      virtual={virtual}
      currentTrackId={currentTrackId}
      onPlayRow={(row) => {
        const visibleIndex = rows.findIndex((item) => item.id === row.id);
        if (visibleIndex >= 0) {
          handlePlay(row, visibleIndex);
        }
      }}
    />
  );
}
