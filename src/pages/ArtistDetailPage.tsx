import { useParams } from "react-router";
import { PageHeader } from "@/components/layout";

export default function ArtistDetailPage() {
  const { id } = useParams();
  return <PageHeader title={`艺人详情 ${id ?? ""}`} />;
}
