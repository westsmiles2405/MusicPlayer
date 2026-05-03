import React, { useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

const isJsdom =
  typeof navigator !== "undefined" && navigator.userAgent.includes("jsdom");

export interface TrackTableRow {
  id: number;
  title: string;
  albumName: string | null;
  primaryArtistName: string | null;
  durationMs: number;
  missingAt: number | null;
  playlistPosition?: number;
  trackNo?: number;
  isFavorite?: boolean;
  isFavoritePending?: boolean;
}

function TrackRow({
  row,
  index,
  onPlay,
  onRemove,
  onToggleFavorite,
  onReorderPlaylist,
  rowsLength,
  renderActions,
  currentTrackId,
  onPlayRow,
}: {
  row: TrackTableRow;
  index: number;
  onPlay: (row: TrackTableRow, index: number) => void;
  onRemove?: (row: TrackTableRow) => void;
  onToggleFavorite?: (row: TrackTableRow) => void;
  onReorderPlaylist?: (
    sourcePosition: number,
    destinationPosition: number,
  ) => void;
  rowsLength: number;
  renderActions?: (row: TrackTableRow) => React.ReactNode;
  currentTrackId?: number;
  onPlayRow?: (row: TrackTableRow, index: number) => void;
}) {
  const isPlaying = currentTrackId === row.id;
  return (
    <tr
      data-testid="track-row"
      data-missing={row.missingAt !== null}
      className={`track-table__row${isPlaying ? ' track-table__row--playing' : ''}`}
      aria-current={isPlaying ? 'true' : undefined}
      onDoubleClick={() => onPlayRow?.(row, index)}
    >
      <td className="track-table__number-cell">
        <span className="track-table__number">
          <span className="track-table__number-text">
            {row.trackNo || index + 1}
          </span>
          <span className="track-table__number-icon" aria-hidden="true">
            ▶
          </span>
        </span>
      </td>
      <td className="track-table__title">{row.title}</td>
      <td>{row.primaryArtistName ?? "未知艺人"}</td>
      <td>{row.albumName ?? "未知专辑"}</td>
      <td>{row.missingAt === null ? "" : "文件缺失"}</td>
      <td>
        {onToggleFavorite && row.isFavorite !== undefined && (
          <button
            type="button"
            disabled={row.isFavoritePending}
            onClick={() => onToggleFavorite(row)}
            aria-label={row.isFavorite ? "取消收藏" : "收藏"}
          >
            {row.isFavoritePending ? "..." : row.isFavorite ? "已喜欢" : "喜欢"}
          </button>
        )}
        <button
          type="button"
          onClick={() => onPlay(row, index)}
          disabled={row.missingAt !== null}
          aria-label={`播放 ${row.title}`}
        >
          播放
        </button>
        {renderActions?.(row)}
        {onReorderPlaylist && row.playlistPosition !== undefined && (
          <>
            <button
              type="button"
              disabled={row.playlistPosition <= 0}
              onClick={() =>
                onReorderPlaylist(
                  row.playlistPosition!,
                  row.playlistPosition! - 1,
                )
              }
              aria-label="上移"
            >
              ↑
            </button>
            <button
              type="button"
              disabled={row.playlistPosition >= rowsLength - 1}
              onClick={() =>
                onReorderPlaylist(
                  row.playlistPosition!,
                  row.playlistPosition! + 1,
                )
              }
              aria-label="下移"
            >
              ↓
            </button>
          </>
        )}
        {onRemove && (
          <button type="button" onClick={() => onRemove(row)}>
            移除
          </button>
        )}
      </td>
    </tr>
  );
}

function VirtualizedTableBody({
  rows,
  queueContext,
  onPlay,
  onRemove,
  onToggleFavorite,
  onReorderPlaylist,
  renderActions,
  currentTrackId,
  onPlayRow,
}: {
  rows: TrackTableRow[];
  queueContext: string;
  onPlay: (row: TrackTableRow, index: number) => void;
  onRemove?: (row: TrackTableRow) => void;
  onToggleFavorite?: (row: TrackTableRow) => void;
  onReorderPlaylist?: (
    sourcePosition: number,
    destinationPosition: number,
  ) => void;
  renderActions?: (row: TrackTableRow) => React.ReactNode;
  currentTrackId?: number;
  onPlayRow?: (row: TrackTableRow, index: number) => void;
}) {
  const parentRef = useRef<HTMLTableSectionElement>(null);

  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 40,
    overscan: 5,
  });

  const virtualItems = virtualizer.getVirtualItems();

  return (
    <tbody
      ref={parentRef}
      style={{
        display: "block",
        overflow: "auto",
        maxHeight: "100%",
      }}
    >
      {virtualItems.length > 0 && (
        <tr
          style={{
            display: "table",
            tableLayout: "fixed",
            width: "100%",
            height: `${virtualItems[0]!.start}px`,
          }}
          aria-hidden="true"
        >
          <td colSpan={6} style={{ padding: 0, border: "none" }} />
        </tr>
      )}
      {virtualItems.map((virtualRow) => {
        const row = rows[virtualRow.index]!;
        const isPlaying = currentTrackId === row.id;
        return (
          <tr
            key={`${row.id}-${row.playlistPosition ?? queueContext}`}
            data-testid="track-row"
            data-missing={row.missingAt !== null}
            className={`track-table__row${isPlaying ? ' track-table__row--playing' : ''}`}
            aria-current={isPlaying ? 'true' : undefined}
            onDoubleClick={() => onPlayRow?.(row, virtualRow.index)}
            style={{
              display: "table",
              tableLayout: "fixed",
              width: "100%",
            }}
          >
            <td className="track-table__number-cell">
              <span className="track-table__number">
                <span className="track-table__number-text">
                  {row.trackNo || virtualRow.index + 1}
                </span>
                <span className="track-table__number-icon" aria-hidden="true">
                  ▶
                </span>
              </span>
            </td>
            <td className="track-table__title">{row.title}</td>
            <td>{row.primaryArtistName ?? "未知艺人"}</td>
            <td>{row.albumName ?? "未知专辑"}</td>
            <td>{row.missingAt === null ? "" : "文件缺失"}</td>
            <td>
              {onToggleFavorite && row.isFavorite !== undefined && (
                <button
                  type="button"
                  disabled={row.isFavoritePending}
                  onClick={() => onToggleFavorite(row)}
                  aria-label={row.isFavorite ? "取消收藏" : "收藏"}
                >
                  {row.isFavoritePending
                    ? "..."
                    : row.isFavorite
                      ? "已喜欢"
                      : "喜欢"}
                </button>
              )}
              <button
                type="button"
                onClick={() => onPlay(row, virtualRow.index)}
                disabled={row.missingAt !== null}
                aria-label={`播放 ${row.title}`}
              >
                播放
              </button>
              {renderActions?.(row)}
              {onReorderPlaylist && row.playlistPosition !== undefined && (
                <>
                  <button
                    type="button"
                    disabled={row.playlistPosition <= 0}
                    onClick={() =>
                      onReorderPlaylist(
                        row.playlistPosition!,
                        row.playlistPosition! - 1,
                      )
                    }
                    aria-label="上移"
                  >
                    ↑
                  </button>
                  <button
                    type="button"
                    disabled={row.playlistPosition >= rows.length - 1}
                    onClick={() =>
                      onReorderPlaylist(
                        row.playlistPosition!,
                        row.playlistPosition! + 1,
                      )
                    }
                    aria-label="下移"
                  >
                    ↓
                  </button>
                </>
              )}
              {onRemove && (
                <button type="button" onClick={() => onRemove(row)}>
                  移除
                </button>
              )}
            </td>
          </tr>
        );
      })}
      {virtualItems.length > 0 && (
        <tr
          style={{
            display: "table",
            tableLayout: "fixed",
            width: "100%",
            height: `${virtualizer.getTotalSize() - virtualItems[virtualItems.length - 1]!.end}px`,
          }}
          aria-hidden="true"
        >
          <td colSpan={6} style={{ padding: 0, border: "none" }} />
        </tr>
      )}
    </tbody>
  );
}

export function TrackTableView({
  rows,
  queueContext,
  onPlay,
  onRemove,
  onToggleFavorite,
  onReorderPlaylist,
  renderActions,
  virtual = false,
  currentTrackId,
  onPlayRow,
}: {
  rows: TrackTableRow[];
  queueContext: "recent" | "songs" | "album" | "artist" | "playlist";
  onPlay: (row: TrackTableRow, index: number) => void;
  onRemove?: (row: TrackTableRow) => void;
  onToggleFavorite?: (row: TrackTableRow) => void;
  onReorderPlaylist?: (
    sourcePosition: number,
    destinationPosition: number,
  ) => void;
  renderActions?: (row: TrackTableRow) => React.ReactNode;
  virtual?: boolean;
  currentTrackId?: number;
  onPlayRow?: (row: TrackTableRow, index: number) => void;
}) {
  if (virtual && !isJsdom) {
    return (
      <table className="track-table">
        <thead>
          <tr>
            <th>#</th>
            <th>标题</th>
            <th>艺人</th>
            <th>专辑</th>
            <th>状态</th>
            <th>操作</th>
          </tr>
        </thead>
        <VirtualizedTableBody
          rows={rows}
          queueContext={queueContext}
          onPlay={onPlay}
          onRemove={onRemove}
          onToggleFavorite={onToggleFavorite}
          onReorderPlaylist={onReorderPlaylist}
          renderActions={renderActions}
          currentTrackId={currentTrackId}
          onPlayRow={onPlayRow}
        />
      </table>
    );
  }

  return (
    <table className="track-table">
      <thead>
        <tr>
          <th>#</th>
          <th>标题</th>
          <th>艺人</th>
          <th>专辑</th>
          <th>状态</th>
          <th>操作</th>
        </tr>
      </thead>
      <tbody>
        {rows.map((row, index) => (
          <TrackRow
            key={`${row.id}-${row.playlistPosition ?? queueContext}`}
            row={row}
            index={index}
            onPlay={onPlay}
            onRemove={onRemove}
            onToggleFavorite={onToggleFavorite}
            onReorderPlaylist={onReorderPlaylist}
            rowsLength={rows.length}
            renderActions={renderActions}
            currentTrackId={currentTrackId}
            onPlayRow={onPlayRow}
          />
        ))}
      </tbody>
    </table>
  );
}
