export function NowPlayingBackground({
  coverPath,
}: {
  coverPath: string | null;
}) {
  return (
    <div className="np-bg">
      <div className="np-bg__gradient" />
      {coverPath && (
        <div
          className="np-bg__blurred-cover"
          style={{ backgroundImage: `url(${coverPath})` }}
        />
      )}
      <div className="np-bg__glass" />
    </div>
  );
}
