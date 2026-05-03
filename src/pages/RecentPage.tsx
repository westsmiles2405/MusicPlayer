import { useQuery } from "@tanstack/react-query";
import { trackRepo } from "@/repositories/trackRepo";
import { TrackTable } from "@/components/library/TrackTable";
import {
  PageHeader,
  LoadingState,
  ErrorState,
} from "@/components/layout";
import { DopamineEmptyState } from "@/components/ui";

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
        <DopamineEmptyState
          context="recent"
          title="最近添加为空"
          description="新添加的音乐会出现在这里"
        />
      )}
      {tracks.data && tracks.data.length > 0 && (
        <TrackTable tracks={tracks.data} queueContext="songs" />
      )}
    </>
  );
}
