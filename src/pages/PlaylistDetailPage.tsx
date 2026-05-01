import { useParams } from "react-router";
import { PageHeader } from "@/components/layout";

export default function PlaylistDetailPage() {
  const { id } = useParams();
  return <PageHeader title={`播放列表详情 ${id ?? ""}`} />;
}
