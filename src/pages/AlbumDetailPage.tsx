import { useParams } from "react-router";
import { PageHeader } from "@/components/layout";

export default function AlbumDetailPage() {
  const { id } = useParams();
  return <PageHeader title={`专辑详情 ${id ?? ""}`} />;
}
