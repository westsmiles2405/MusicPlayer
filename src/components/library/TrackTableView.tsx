export interface TrackTableRow {
  id: number;
  title: string;
  albumName: string | null;
  primaryArtistName: string | null;
  durationMs: number;
  missingAt: number | null;
  playlistPosition?: number;
}

export function TrackTableView({
  rows,
  queueContext,
  onPlay,
  onRemove,
  renderActions,
}: {
  rows: TrackTableRow[];
  queueContext: "recent" | "songs" | "album" | "artist" | "playlist";
  onPlay: (row: TrackTableRow) => void;
  onRemove?: (row: TrackTableRow) => void;
  renderActions?: (row: TrackTableRow) => React.ReactNode;
}) {
  return (
    <table className="track-table">
      <thead>
        <tr>
          <th>标题</th>
          <th>艺人</th>
          <th>专辑</th>
          <th>状态</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row) => (
          <tr
            key={`${row.id}-${row.playlistPosition ?? queueContext}`}
            data-missing={row.missingAt !== null}
          >
            <td>{row.title}</td>
            <td>{row.primaryArtistName ?? "未知艺人"}</td>
            <td>{row.albumName ?? "未知专辑"}</td>
            <td>{row.missingAt === null ? "" : "文件缺失"}</td>
            <td>
              <button
                type="button"
                onClick={() => onPlay(row)}
                disabled={row.missingAt !== null}
                aria-label={`播放 ${row.title}`}
              >
                播放
              </button>
              {renderActions?.(row)}
              {onRemove && (
                <button type="button" onClick={() => onRemove(row)}>
                  移除
                </button>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
