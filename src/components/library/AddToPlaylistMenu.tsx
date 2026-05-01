import { useState } from "react";

interface Playlist {
  id: number;
  name: string;
}

interface AddToPlaylistMenuProps {
  playlists: Playlist[];
  onAdd: (playlistId: number) => void;
  disabled?: boolean;
}

export function AddToPlaylistMenu({
  playlists,
  onAdd,
  disabled = false,
}: AddToPlaylistMenuProps) {
  const [selectedId, setSelectedId] = useState<number | null>(null);

  return (
    <div style={{ display: "flex", gap: 8, alignItems: "center" }}>
      <select
        aria-label="选择播放列表"
        value={selectedId ?? ""}
        onChange={(e) => {
          const val = e.target.value;
          setSelectedId(val ? Number(val) : null);
        }}
      >
        <option value="">选择播放列表...</option>
        {playlists.map((pl) => (
          <option key={pl.id} value={pl.id}>
            {pl.name}
          </option>
        ))}
      </select>
      <button
        type="button"
        disabled={disabled || selectedId === null}
        onClick={() => {
          if (selectedId !== null) onAdd(selectedId);
        }}
      >
        加入播放列表
      </button>
    </div>
  );
}
