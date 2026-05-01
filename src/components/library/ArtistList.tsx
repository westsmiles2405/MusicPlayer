interface Artist {
  id: number;
  name: string;
}

interface ArtistListProps {
  artists: Artist[];
  onOpen: (id: number) => void;
}

export function ArtistList({ artists, onOpen }: ArtistListProps) {
  if (artists.length === 0) {
    return (
      <div className="state">
        <h2>没有艺人</h2>
        <p>导入音乐文件夹后，艺人将显示在这里</p>
      </div>
    );
  }

  return (
    <div className="artist-list">
      {artists.map((artist) => (
        <button
          key={artist.id}
          className="artist-list__item"
          onClick={() => onOpen(artist.id)}
          type="button"
        >
          {artist.name}
        </button>
      ))}
    </div>
  );
}
