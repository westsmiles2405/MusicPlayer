import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router";
import { albumRepo } from "@/repositories/albumRepo";
import { AlbumGrid } from "@/components/library/AlbumGrid";
import { PageHeader, EmptyState, LoadingState, ErrorState } from "@/components/layout";

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
      {albums.isError && <ErrorState message={albums.error?.message ?? "加载失败"} />}
      {albums.data?.length === 0 && <EmptyState title="没有专辑" />}
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
