// src/components/ui/DopamineEmptyState.tsx
import {
  MusicNotes,
  Heart,
  Playlist,
  VinylRecord,
  MicrophoneStage,
  ClockCounterClockwise,
  MagnifyingGlass,
  Guitar,
} from "@phosphor-icons/react";
import type { ComponentType, CSSProperties, ReactNode } from "react";
import "../../styles/dopamine-empty.css";

// Derive icon type from a known icon to avoid IconProps version mismatches
type IconWeight = "thin" | "light" | "regular" | "bold" | "fill" | "duotone";

type PhosphorIcon = ComponentType<{
  size?: number;
  weight?: IconWeight;
  className?: string;
  "aria-hidden"?: boolean;
}>;

type DopamineEmptyStateContext =
  | "library"
  | "favorites"
  | "playlists"
  | "albums"
  | "artists"
  | "recent"
  | "search"
  | "default";

type AnimationLevel = "full" | "subtle" | "minimal";

interface Preset {
  gradient: [string, string, string];
  Icon: PhosphorIcon;
  iconWeight: string;
  accent: string;
  sparkleDensity: "medium" | "low" | "none";
  animation: AnimationLevel;
}

const PRESETS: Record<DopamineEmptyStateContext, Preset> = {
  library: {
    gradient: ["#7c3aed", "#ec4899", "#06b6d4"],
    Icon: MusicNotes,
    iconWeight: "duotone",
    accent: "#a855f7",
    sparkleDensity: "medium",
    animation: "full",
  },
  favorites: {
    gradient: ["#fb7185", "#ec4899", "#fb923c"],
    Icon: Heart,
    iconWeight: "fill",
    accent: "#ec4899",
    sparkleDensity: "medium",
    animation: "full",
  },
  playlists: {
    gradient: ["#8b5cf6", "#6366f1", "#3b82f6"],
    Icon: Playlist,
    iconWeight: "duotone",
    accent: "#8b5cf6",
    sparkleDensity: "medium",
    animation: "full",
  },
  albums: {
    gradient: ["#f59e0b", "#f97316", "#fdba74"],
    Icon: VinylRecord,
    iconWeight: "duotone",
    accent: "#f59e0b",
    sparkleDensity: "low",
    animation: "full",
  },
  artists: {
    gradient: ["#06b6d4", "#14b8a6", "#22c55e"],
    Icon: MicrophoneStage,
    iconWeight: "duotone",
    accent: "#06b6d4",
    sparkleDensity: "medium",
    animation: "full",
  },
  recent: {
    gradient: ["#7c3aed", "#6366f1", "#3b82f6"],
    Icon: ClockCounterClockwise,
    iconWeight: "duotone",
    accent: "#a78bfa",
    sparkleDensity: "low",
    animation: "subtle",
  },
  search: {
    gradient: ["#334155", "#4f46e5", "#1e3a8a"],
    Icon: MagnifyingGlass,
    iconWeight: "regular",
    accent: "#818cf8",
    sparkleDensity: "none",
    animation: "minimal",
  },
  default: {
    gradient: ["#7c3aed", "#ec4899", "#06b6d4"],
    Icon: Guitar,
    iconWeight: "duotone",
    accent: "#a855f7",
    sparkleDensity: "medium",
    animation: "full",
  },
};

function Sparkles({ density }: { density: Preset["sparkleDensity"] }) {
  if (density === "none") return null;
  const count = density === "medium" ? 3 : 2;
  return (
    <div className="dopamine-empty__sparkles" aria-hidden="true">
      {Array.from({ length: count }, (_, i) => (
        <span
          key={i}
          className={`dopamine-empty__sparkle dopamine-empty__sparkle--${i + 1}`}
        />
      ))}
    </div>
  );
}

function Notes({ density }: { density: Preset["sparkleDensity"] }) {
  if (density !== "medium") return null;
  return (
    <div className="dopamine-empty__notes" aria-hidden="true">
      <span className="dopamine-empty__note dopamine-empty__note--1">♪</span>
      <span className="dopamine-empty__note dopamine-empty__note--2">♫</span>
    </div>
  );
}

interface DopamineEmptyStateProps {
  context?: DopamineEmptyStateContext;
  title: string;
  description?: string;
  action?: ReactNode;
  className?: string;
}

export function DopamineEmptyState({
  context = "default",
  title,
  description,
  action,
  className,
}: DopamineEmptyStateProps) {
  const preset = PRESETS[context];
  const { Icon, iconWeight, accent, sparkleDensity, animation } = preset;

  const presetVars = {
    "--gradient-1": preset.gradient[0],
    "--gradient-2": preset.gradient[1],
    "--gradient-3": preset.gradient[2],
    "--accent": accent,
  } as CSSProperties;

  return (
    <div
      className={`dopamine-empty${className ? ` ${className}` : ""}`}
      data-context={context}
      data-animation={animation}
      style={presetVars}
    >
      <div className="dopamine-empty__orbs" aria-hidden="true">
        <span className="dopamine-empty__orb dopamine-empty__orb--one" />
        <span className="dopamine-empty__orb dopamine-empty__orb--two" />
        <span className="dopamine-empty__orb dopamine-empty__orb--three" />
      </div>

      <div className="dopamine-empty__content">
        <div className="dopamine-empty__icon-wrap">
          <div className="dopamine-empty__glow" aria-hidden="true" />
          <Icon
            size={80}
            weight={iconWeight as IconWeight}
            className="dopamine-empty__icon"
            aria-hidden
          />
          <Sparkles density={sparkleDensity} />
          <Notes density={sparkleDensity} />
        </div>

        <h2 className="dopamine-empty__title">{title}</h2>
        {description && (
          <p className="dopamine-empty__desc">{description}</p>
        )}
        {action && <div className="dopamine-empty__action">{action}</div>}
      </div>
    </div>
  );
}
