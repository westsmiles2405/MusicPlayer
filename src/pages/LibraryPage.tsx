import { useQuery, useQueryClient } from "@tanstack/react-query";
import { open } from "@tauri-apps/plugin-dialog";
import { useEffect } from "react";
import { trackRepo } from "@/repositories/trackRepo";
import { albumRepo } from "@/repositories/albumRepo";
import { artistRepo } from "@/repositories/artistRepo";
import { playlistRepo } from "@/repositories/playlistRepo";
import { libraryRepo } from "@/repositories/libraryRepo";
import { useScanProgress } from "@/hooks/useScanProgress";

export default function LibraryPage() {
  const qc = useQueryClient();
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
  const { phase, report } = useScanProgress();

  useEffect(() => {
    if (phase === "done") {
      qc.invalidateQueries({ queryKey: ["tracks"] });
      qc.invalidateQueries({ queryKey: ["albums"] });
      qc.invalidateQueries({ queryKey: ["artists"] });
    }
  }, [phase, qc]);

  const onAddFolder = async () => {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked !== "string") return;
    await libraryRepo.addFolder(picked);
    await libraryRepo.startScan();
  };

  if (tracks.isLoading) return <div className="p-8">载入中…</div>;
  if (tracks.error)
    return <div className="p-8 text-red-500">错误: {String(tracks.error)}</div>;

  if ((tracks.data?.length ?? 0) === 0 && phase !== "scanning") {
    return (
      <div className="flex flex-col items-center justify-center h-full p-8 text-center gap-4">
        <div className="text-6xl" aria-hidden="true">
          {"🎵"}
        </div>
        <h2 className="text-xl font-semibold">尚未添加音乐</h2>
        <p className="text-sm text-gray-500 max-w-sm">
          选择一个文件夹，开始构建你的资料库。MusicPlayer 会扫描其中的 mp3 / m4a
          / flac / wav / aac 文件。
        </p>
        <button
          onClick={onAddFolder}
          className="px-4 py-2 rounded-md bg-black text-white hover:bg-black/80"
        >
          添加音乐文件夹
        </button>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-2">
      <h1 className="text-2xl font-semibold">资料库</h1>
      <div>曲目: {tracks.data?.length ?? 0}</div>
      <div>专辑: {albums.data?.length ?? 0}</div>
      <div>艺人: {artists.data?.length ?? 0}</div>
      <div>播放列表: {playlists.data?.length ?? 0}</div>
      {report && phase === "done" && (
        <div className="text-sm text-gray-500">
          上次扫描：添加 {report.added} / 更新 {report.updated} / 移动{" "}
          {report.moved} / 缺失 {report.missing}
          {report.errors.length > 0 && ` / ${report.errors.length} 个错误`}
        </div>
      )}
    </div>
  );
}
