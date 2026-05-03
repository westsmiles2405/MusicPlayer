import { useDynamicColors } from '@/hooks/useDynamicColors';

interface Props {
  coverUrl: string | null;
}

export function NowPlayingBackground({ coverUrl }: Props) {
  const colors = useDynamicColors(coverUrl);

  return (
    <div
      className="np-bg"
      style={{
        background: `
          radial-gradient(ellipse at 50% 0%, ${colors.primary}40 0%, transparent 70%),
          linear-gradient(180deg, ${colors.background} 0%, #000 100%)
        `,
        transition: 'background 0.5s ease',
      }}
    />
  );
}
