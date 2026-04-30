import { useQuery } from "@tanstack/react-query";
import { trackRepo } from "@/repositories/trackRepo";
import { albumRepo } from "@/repositories/albumRepo";
import { artistRepo } from "@/repositories/artistRepo";
import { playlistRepo } from "@/repositories/playlistRepo";

export default function LibraryPage() {
  const tracks = useQuery({
    queryKey: ["tracks"],
    queryFn: () => trackRepo.list(),
  });
  const albums = useQuery({
    queryKey: ["albums"],
    queryFn: () => albumRepo.list(),
  });
  const artists = useQuery({
    queryKey: ["artists"],
    queryFn: () => artistRepo.list(),
  });
  const playlists = useQuery({
    queryKey: ["playlists"],
    queryFn: () => playlistRepo.list(),
  });

  if (tracks.isLoading) return <div className="p-8">载入中…</div>;
  if (tracks.error)
    return <div className="p-8 text-red-500">错误: {String(tracks.error)}</div>;

  return (
    <div className="p-8 space-y-2">
      <h1 className="text-2xl font-semibold">资料库 (v0.2.0 数据层冒烟)</h1>
      <div>曲目: {tracks.data?.length ?? 0}</div>
      <div>专辑: {albums.data?.length ?? 0}</div>
      <div>艺人: {artists.data?.length ?? 0}</div>
      <div>播放列表: {playlists.data?.length ?? 0}</div>
      <p className="text-sm text-gray-500 mt-4">
        v0.3.0 (库扫描器) 完成后这些计数才会大于
        0。当前数据层正常工作即视为通过。
      </p>
    </div>
  );
}
