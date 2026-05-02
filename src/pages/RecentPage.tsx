import { useQuery } from "@tanstack/react-query";
import { trackRepo } from "@/repositories/trackRepo";
import { TrackTable } from "@/components/library/TrackTable";
import {
  PageHeader,
  EmptyState,
  LoadingState,
  ErrorState,
} from "@/components/layout";

export default function RecentPage() {
  const tracks = useQuery({
    queryKey: ["recentlyAdded", 100],
    queryFn: () => trackRepo.recentlyAdded(100),
  });

  return (
    <>
      <PageHeader title="最近添加" />
      {tracks.isLoading && <LoadingState title="加载中" />}
      {tracks.isError && (
        <ErrorState message={tracks.error?.message ?? "加载失败"} />
      )}
      {tracks.data?.length === 0 && (
        <EmptyState
          title="没有最近添加的歌曲"
          description="扫描音乐文件夹以开始"
        />
      )}
      {tracks.data && tracks.data.length > 0 && (
        <TrackTable tracks={tracks.data} queueContext="songs" />
      )}
    </>
  );
}
