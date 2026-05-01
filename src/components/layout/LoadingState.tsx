export function LoadingState({
  title,
  description,
}: {
  title: string;
  description?: string;
}) {
  return (
    <div className="state state--loading">
      <h2>{title}</h2>
      {description && <p>{description}</p>}
    </div>
  );
}
