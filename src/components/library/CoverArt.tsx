import { convertFileSrc } from "@tauri-apps/api/core";

const gradients = [
  "linear-gradient(135deg, #f04d5f, #235a7a)",
  "linear-gradient(135deg, #d85a2a, #2d7c6f)",
  "linear-gradient(135deg, #7a4cc2, #c94f63)",
  "linear-gradient(135deg, #2f6f9f, #d0a642)",
];

function pickGradient(seed: string) {
  const total = Array.from(seed).reduce((sum, ch) => sum + ch.charCodeAt(0), 0);
  return gradients[total % gradients.length];
}

export function CoverArt({
  coverPath,
  title,
  seed,
  size = "md",
}: {
  coverPath: string | null;
  title: string;
  seed?: string | number;
  size?: "sm" | "md" | "lg";
}) {
  if (coverPath) {
    const src = coverPath.startsWith("/") || coverPath.startsWith("http")
      ? convertFileSrc(coverPath)
      : coverPath;
    return (
      <img
        className={`cover-art cover-art--${size}`}
        src={src}
        alt={title}
      />
    );
  }
  return (
    <div
      className={`cover-art cover-art--${size} cover-art--placeholder`}
      aria-label={title}
      role="img"
      style={{
        background: pickGradient(String(seed ?? title)),
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <svg
        width="40%"
        height="40%"
        viewBox="0 0 24 24"
        fill="none"
        stroke="rgba(255,255,255,0.6)"
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M9 18V5l12-2v13" />
        <circle cx="6" cy="18" r="3" />
        <circle cx="18" cy="16" r="3" />
      </svg>
    </div>
  );
}
