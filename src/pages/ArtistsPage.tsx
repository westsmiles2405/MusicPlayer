import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "react-router";
import { artistRepo } from "@/repositories/artistRepo";
import { ArtistList } from "@/components/library/ArtistList";
import { PageHeader, EmptyState, LoadingState, ErrorState } from "@/components/layout";

export default function ArtistsPage() {
  const navigate = useNavigate();
  const artists = useQuery({
    queryKey: ["artists"],
    queryFn: artistRepo.list,
  });

  return (
    <>
      <PageHeader title="艺人" />
      {artists.isLoading && <LoadingState title="加载中" />}
      {artists.isError && <ErrorState message={artists.error?.message ?? "加载失败"} />}
      {artists.data?.length === 0 && <EmptyState title="没有艺人" />}
      {artists.data && artists.data.length > 0 && (
        <ArtistList
          artists={artists.data.map((a) => ({ id: a.id, name: a.name }))}
          onOpen={(id) => navigate(`/library/artists/${id}`)}
        />
      )}
    </>
  );
}
