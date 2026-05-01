import { CoverArt } from "./CoverArt";

interface Album {
  id: number;
  name: string;
  albumArtistName: string | null;
  coverPath: string | null;
}

interface AlbumGridProps {
  albums: Album[];
  onOpen: (id: number) => void;
}

export function AlbumGrid({ albums, onOpen }: AlbumGridProps) {
  if (albums.length === 0) {
    return (
      <div className="state">
        <h2>没有专辑</h2>
        <p>导入音乐文件夹后，专辑将显示在这里</p>
      </div>
    );
  }

  return (
    <div className="album-grid">
      {albums.map((album) => (
        <button
          key={album.id}
          className="album-grid__item"
          onClick={() => onOpen(album.id)}
          type="button"
        >
          <CoverArt
            coverPath={album.coverPath}
            title={album.name}
            size="md"
          />
          <p style={{ margin: "8px 0 2px", fontWeight: 600, fontSize: 14 }}>
            {album.name}
          </p>
          {album.albumArtistName && (
            <p style={{ margin: 0, fontSize: 12, color: "var(--color-muted, #9a9a9a)" }}>
              {album.albumArtistName}
            </p>
          )}
        </button>
      ))}
    </div>
  );
}
