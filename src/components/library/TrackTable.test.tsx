import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { TrackTable } from "./TrackTable";
import type { Track } from "@/repositories/trackRepo";

const play = vi.fn();
vi.mock("@/hooks/usePlayer", () => ({
  usePlayer: () => ({ play }),
}));

const baseTrack: Track = {
  id: 1,
  filePath: "/a.mp3",
  fileSize: 1,
  fileModifiedAt: 1,
  hash: null,
  title: "A",
  albumId: null,
  primaryArtistId: null,
  albumArtistId: null,
  trackNo: null,
  discNo: null,
  year: null,
  genre: null,
  durationMs: 1000,
  bitrate: null,
  sampleRate: null,
  channels: null,
  codec: null,
  isFavorite: false,
  playCount: 0,
  lastPlayedAt: null,
  lastSeenAt: 1,
  missingAt: null,
  addedAt: 1,
  updatedAt: 1,
  rootFolderId: null,
  albumName: "Album",
  primaryArtistName: "Artist",
};

describe("TrackTable", () => {
  it("plays clicked track with non-missing queue", () => {
    play.mockReset();
    const tracks = [
      baseTrack,
      { ...baseTrack, id: 2, title: "B", missingAt: 2 },
      { ...baseTrack, id: 3, title: "C" },
    ];
    render(<TrackTable tracks={tracks} queueContext="songs" />);
    fireEvent.click(screen.getByRole("button", { name: "播放 C" }));
    expect(play).toHaveBeenCalledWith(3, [1, 3], 1);
  });

  it("does not play missing track", () => {
    play.mockReset();
    render(
      <TrackTable
        tracks={[{ ...baseTrack, missingAt: 2 }]}
        queueContext="playlist"
      />,
    );
    expect(screen.getByRole("button", { name: "播放 A" })).toBeDisabled();
  });

  it("renders favorite toggles when showFavorite is true", () => {
    const onToggleFavorite = vi.fn();
    render(
      <TrackTable
        tracks={[baseTrack]}
        queueContext="songs"
        showFavorite
        onToggleFavorite={onToggleFavorite}
      />,
    );
    expect(screen.getByRole("button", { name: "收藏" })).toBeInTheDocument();
  });

  it("calls onToggleFavorite with next favorite value", () => {
    const onToggleFavorite = vi.fn();
    render(
      <TrackTable
        tracks={[{ ...baseTrack, isFavorite: false }]}
        queueContext="songs"
        showFavorite
        onToggleFavorite={onToggleFavorite}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "收藏" }));
    expect(onToggleFavorite).toHaveBeenCalledWith(
      expect.objectContaining({ id: baseTrack.id }),
      true,
    );
  });

  it("shows cancel favorite when already favorited", () => {
    const onToggleFavorite = vi.fn();
    render(
      <TrackTable
        tracks={[{ ...baseTrack, isFavorite: true }]}
        queueContext="songs"
        showFavorite
        onToggleFavorite={onToggleFavorite}
      />,
    );
    expect(
      screen.getByRole("button", { name: "取消收藏" }),
    ).toBeInTheDocument();
  });
});
