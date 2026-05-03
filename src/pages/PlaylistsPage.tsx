import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router";
import { playlistRepo } from "@/repositories/playlistRepo";
import { useCreatePlaylistMutation } from "@/hooks/useCreatePlaylistMutation";
import { PlaylistCreateDialog } from "@/components/playlists";
import {
  PageHeader,
  LoadingState,
  ErrorState,
} from "@/components/layout";
import { DopamineEmptyState } from "@/components/ui";

export default function PlaylistsPage() {
  const navigate = useNavigate();
  const [createOpen, setCreateOpen] = useState(false);
  const createMutation = useCreatePlaylistMutation();

  const playlists = useQuery({
    queryKey: ["playlists"],
    queryFn: playlistRepo.list,
  });

  return (
    <>
      <PageHeader
        title="播放列表"
        action={
          <button type="button" onClick={() => setCreateOpen(true)}>
            创建播放列表
          </button>
        }
      />

      {playlists.isLoading && <LoadingState title="加载中" />}
      {playlists.isError && (
        <ErrorState message={playlists.error?.message ?? "加载失败"} />
      )}
      {playlists.data?.length === 0 && (
        <DopamineEmptyState
          context="playlists"
          title="还没有播放列表"
          description="创建一个播放列表来整理你的音乐"
        />
      )}
      {playlists.data && playlists.data.length > 0 && (
        <ul className="playlist-list">
          {playlists.data.map((p) => (
            <li key={p.id}>
              <button
                type="button"
                className="playlist-list__item"
                onClick={() => navigate(`/playlists/${p.id}`)}
              >
                <span className="playlist-list__name">{p.name}</span>
                <span className="playlist-list__count">
                  {p.trackCount} 首歌曲
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}

      <PlaylistCreateDialog
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        onCreate={async (name) => {
          await createMutation.mutateAsync(name);
        }}
      />
    </>
  );
}
