export function PageHeader({
  title,
  action,
}: {
  title: string;
  action?: React.ReactNode;
}) {
  return (
    <div className="page-header">
      <h1>{title}</h1>
      {action}
    </div>
  );
}
