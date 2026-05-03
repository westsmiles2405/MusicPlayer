import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { open } from "@tauri-apps/plugin-dialog";
import { libraryRepo, type ScanFolder } from "@/repositories/libraryRepo";

export default function SettingsPage() {
  const qc = useQueryClient();
  const [manualPath, setManualPath] = useState("");
  const folders = useQuery({
    queryKey: ["scan-folders"],
    queryFn: () => libraryRepo.listFolders(),
  });

  const addM = useMutation({
    mutationFn: async () => {
      const picked = await open({ directory: true, multiple: false });
      if (typeof picked !== "string") return;
      await libraryRepo.addFolder(picked);
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ["scan-folders"] }),
  });

  const addManual = useMutation({
    mutationFn: async () => {
      const p = manualPath.trim();
      if (!p) return;
      await libraryRepo.addFolder(p);
      setManualPath("");
    },
    onSuccess: () => qc.invalidateQueries({ queryKey: ["scan-folders"] }),
  });

  const removeM = useMutation({
    mutationFn: (id: number) => libraryRepo.removeFolder(id),
    onSuccess: () =>
      Promise.all([
        qc.invalidateQueries({ queryKey: ["scan-folders"] }),
        qc.invalidateQueries({ queryKey: ["tracks"] }),
      ]),
  });

  const rescan = () => libraryRepo.startScan();

  return (
    <div className="p-8 max-w-2xl space-y-6">
      <h1 className="text-2xl font-semibold">设置</h1>

      <section>
        <h2 className="text-lg font-medium mb-2">音乐文件夹</h2>
        <ul className="space-y-1 mb-3">
          {folders.data?.map((f: ScanFolder) => (
            <li
              key={f.id}
              className="flex items-center justify-between gap-3 text-sm"
            >
              <span className="truncate">{f.path}</span>
              <button
                onClick={() => removeM.mutate(f.id)}
                className="text-red-500 hover:underline"
              >
                移除
              </button>
            </li>
          ))}
          {(folders.data?.length ?? 0) === 0 && (
            <li className="text-sm text-gray-500">尚未添加文件夹</li>
          )}
        </ul>
        <div className="flex gap-2 items-center">
          <input
            type="text"
            value={manualPath}
            onChange={(e) => setManualPath(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && addManual.mutate()}
            placeholder="输入文件夹路径，如 /Users/xxx/Music"
            className="flex-1 px-3 py-1 rounded border text-sm bg-transparent"
          />
          <button
            onClick={() => addManual.mutate()}
            disabled={!manualPath.trim()}
            className="px-3 py-1 rounded bg-black text-white text-sm disabled:opacity-40"
          >
            添加
          </button>
          <button
            onClick={() => addM.mutate()}
            className="px-3 py-1 rounded border text-sm"
          >
            浏览选择
          </button>
          <button onClick={rescan} className="px-3 py-1 rounded border text-sm">
            {"↻"} 重新扫描
          </button>
        </div>
      </section>
    </div>
  );
}
