import { useState } from "react";
import { Link } from "react-router";
import { useMutation, useQueryClient, useQuery } from "@tanstack/react-query";
import {
  PageHeader,
  EmptyState,
  LoadingState,
  ErrorState,
} from "@/components/layout";
import { TrackTable } from "@/components/library/TrackTable";
import { searchRepo } from "@/repositories/searchRepo";
import { playlistRepo } from "@/repositories/playlistRepo";
import { useDebouncedValue } from "@/hooks/useDebouncedValue";
import { useToggleFavoriteMutation } from "@/hooks/useToggleFavoriteMutation";

export default function SearchPage() {
  const [query, setQuery] = useState("");
  const trimmed = query.trim();
  const debouncedQuery = useDebouncedValue(trimmed, 250);
  const toggleFavorite = useToggleFavoriteMutation();
  const queryClient = useQueryClient();

  const results = useQuery({
    queryKey: ["search", debouncedQuery],
    queryFn: () => searchRepo.search(debouncedQuery, 10),
    enabled: debouncedQuery.length > 0,
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

  return (
    <>
      <PageHeader title="搜索" />
      <label className="search-input">
        <span className="sr-only">搜索</span>
        <input
          type="text"
          aria-label="搜索"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="搜索歌曲、专辑、艺人或播放列表"
        />
      </label>

      {trimmed.length === 0 && (
        <EmptyState
          title="开始搜索"
          description="输入关键词以搜索歌曲、专辑、艺人或播放列表"
        />
      )}
      {debouncedQuery.length > 0 && results.isLoading && (
        <LoadingState title="搜索中" />
      )}
      {debouncedQuery.length > 0 && results.isError && (
        <ErrorState message={results.error?.message ?? "搜索失败"} />
      )}
      {debouncedQuery.length > 0 && results.data && (
        <div className="search-results">
          <section>
            <h2>歌曲</h2>
            {results.data.tracks.length > 0 ? (
              <TrackTable
                tracks={results.data.tracks}
                queueContext="songs"
                playlists={playlists.data ?? []}
                showFavorite
                onToggleFavorite={(track, favorite) =>
                  toggleFavorite.mutate({ track, favorite })
                }
                onAddToPlaylist={(track, playlistId) =>
                  addToPlaylist.mutateAsync({
                    trackId: track.id,
                    playlistId,
                  })
                }
              />
            ) : (
              <p>没有匹配的歌曲</p>
            )}
          </section>
          <section>
            <h2>专辑</h2>
            {results.data.albums.length > 0 ? (
              <ul>
                {results.data.albums.map((album) => (
                  <li key={album.id}>
                    <Link to={`/library/albums/${album.id}`}>{album.name}</Link>
                  </li>
                ))}
              </ul>
            ) : (
              <p>没有匹配的专辑</p>
            )}
          </section>
          <section>
            <h2>艺人</h2>
            {results.data.artists.length > 0 ? (
              <ul>
                {results.data.artists.map((artist) => (
                  <li key={artist.id}>
                    <Link to={`/library/artists/${artist.id}`}>
                      {artist.name}
                    </Link>
                  </li>
                ))}
              </ul>
            ) : (
              <p>没有匹配的艺人</p>
            )}
          </section>
          <section>
            <h2>播放列表</h2>
            {results.data.playlists.length > 0 ? (
              <ul>
                {results.data.playlists.map((playlist) => (
                  <li key={playlist.id}>
                    <Link to={`/playlists/${playlist.id}`}>
                      {playlist.name}
                    </Link>
                  </li>
                ))}
              </ul>
            ) : (
              <p>没有匹配的播放列表</p>
            )}
          </section>
        </div>
      )}
    </>
  );
}
