import { usePlayer } from "@/hooks/usePlayer";
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
}

export function TrackTable({
  tracks,
  queueContext,
  playlists,
  onAddToPlaylist,
  onRemoveFromPlaylist,
  onReorderPlaylist,
}: TrackTableProps) {
  const { play } = usePlayer();

  const playableTracks = tracks.filter((t) => t.missingAt === null);

  const handlePlay = (row: TrackTableRow) => {
    const playableIds = playableTracks.map((t) => t.id);
    const queueIndex = playableIds.indexOf(row.id);
    play(row.id, playableIds, queueIndex >= 0 ? queueIndex : undefined);
  };

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
      onReorderPlaylist={onReorderPlaylist}
      renderActions={renderActions}
    />
  );
}
