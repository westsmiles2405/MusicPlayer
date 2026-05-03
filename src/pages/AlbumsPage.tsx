import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router";
import { albumRepo } from "@/repositories/albumRepo";
import { AlbumGrid } from "@/components/library/AlbumGrid";
import { PageHeader, LoadingState, ErrorState } from "@/components/layout";
import { DopamineEmptyState } from "@/components/ui";

export default function AlbumsPage() {
  const navigate = useNavigate();
  const albums = useQuery({
    queryKey: ["albums"],
    queryFn: albumRepo.list,
  });

  return (
    <>
      <PageHeader title="专辑" />
      {albums.isLoading && <LoadingState title="加载中" />}
      {albums.isError && (
        <ErrorState message={albums.error?.message ?? "加载失败"} />
      )}
      {albums.data?.length === 0 && <DopamineEmptyState context="albums" title="还没有专辑" description="添加音乐后专辑会出现在这里" />}
      {albums.data && albums.data.length > 0 && (
        <AlbumGrid
          albums={albums.data.map((a) => ({
            id: a.id,
            name: a.name,
            albumArtistName: a.albumArtistName,
            coverPath: a.coverPath,
          }))}
          onOpen={(id) => navigate(`/library/albums/${id}`)}
        />
      )}
    </>
  );
}
