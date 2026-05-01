export function ErrorState({
  message,
  onRetry,
  title,
  description,
}: {
  message: string;
  onRetry?: () => void;
  title?: string;
  description?: string;
}) {
  return (
    <div className="state state--error">
      <h2>{title ?? "出错了"}</h2>
      {description && <p>{description}</p>}
      <p>{message}</p>
      {onRetry && <button onClick={onRetry}>重试</button>}
    </div>
  );
}
