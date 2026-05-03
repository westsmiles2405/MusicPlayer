import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { TrackTable } from "./TrackTable";
import { TrackTableView } from "./TrackTableView";
import type { Track } from "@/repositories/trackRepo";
import type { TrackTableRow } from "./TrackTableView";

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
    render(<TrackTable tracks={tracks} queueContext="songs" virtual={false} />);
    fireEvent.click(screen.getByRole("button", { name: "播放 C" }));
    expect(play).toHaveBeenCalledWith(3, [1, 3], 1);
  });

  it("does not play missing track", () => {
    play.mockReset();
    render(
      <TrackTable
        tracks={[{ ...baseTrack, missingAt: 2 }]}
        queueContext="playlist"
        virtual={false}
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
        virtual={false}
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
        virtual={false}
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
        virtual={false}
      />,
    );
    expect(
      screen.getByRole("button", { name: "取消收藏" }),
    ).toBeInTheDocument();
  });

  it("renders empty list without error", () => {
    render(<TrackTable tracks={[]} queueContext="songs" virtual={false} />);
    expect(screen.queryAllByTestId("track-row")).toHaveLength(0);
  });

  it("renders correct number of rows", () => {
    const tracks = [
      baseTrack,
      { ...baseTrack, id: 2, title: "B" },
      { ...baseTrack, id: 3, title: "C" },
    ];
    render(<TrackTable tracks={tracks} queueContext="songs" virtual={false} />);
    expect(screen.getAllByTestId("track-row")).toHaveLength(3);
  });

  it("renders table rows with interaction class hooks", () => {
    const tracks = [baseTrack];
    render(<TrackTable tracks={tracks} queueContext="songs" virtual={false} />);
    const rows = screen.getAllByTestId("track-row");
    expect(rows[0]).toHaveClass("track-table__row");
  });

  it("sets aria-current on the currently playing row", async () => {
    const { usePlayerStore } = await import("@/stores/playerStore");
    usePlayerStore.setState({ current: { id: 2, title: "B", albumName: null, artistName: null, durationMs: 1000, coverPath: null } });
    const tracks = [
      baseTrack,
      { ...baseTrack, id: 2, title: "B" },
    ];
    render(<TrackTable tracks={tracks} queueContext="songs" virtual={false} />);
    const rows = screen.getAllByTestId("track-row");
    expect(rows[0]).not.toHaveAttribute("aria-current");
    expect(rows[1]).toHaveAttribute("aria-current", "true");
    // cleanup
    usePlayerStore.setState({ current: null });
  });

  it("applies --playing class to the currently playing row", async () => {
    const { usePlayerStore } = await import("@/stores/playerStore");
    usePlayerStore.setState({ current: { id: 2, title: "B", albumName: null, artistName: null, durationMs: 1000, coverPath: null } });
    const tracks = [
      baseTrack,
      { ...baseTrack, id: 2, title: "B" },
    ];
    render(<TrackTable tracks={tracks} queueContext="songs" virtual={false} />);
    const rows = screen.getAllByTestId("track-row");
    expect(rows[0]).not.toHaveClass("track-table__row--playing");
    expect(rows[1]).toHaveClass("track-table__row--playing");
    // cleanup
    usePlayerStore.setState({ current: null });
  });

  it("renders number cells with text and icon elements", () => {
    const tracks = [baseTrack];
    render(<TrackTable tracks={tracks} queueContext="songs" virtual={false} />);
    const numberCell = document.querySelector(".track-table__number-cell");
    expect(numberCell).toBeTruthy();
    expect(
      numberCell!.querySelector(".track-table__number-icon"),
    ).toBeTruthy();
  });

  it("calls play when a row is double-clicked", () => {
    play.mockReset();
    const tracks = [baseTrack];
    render(<TrackTable tracks={tracks} queueContext="songs" virtual={false} />);
    const rows = screen.getAllByTestId("track-row");
    fireEvent.doubleClick(rows[0]!);
    expect(play).toHaveBeenCalledWith(1, [1], 0);
  });

  it("renders all rows in non-virtual mode", () => {
    const rows: TrackTableRow[] = [
      {
        id: 1,
        title: "X",
        albumName: "Album",
        primaryArtistName: "Artist",
        durationMs: 1000,
        missingAt: null,
      },
      {
        id: 2,
        title: "Y",
        albumName: "Album",
        primaryArtistName: "Artist",
        durationMs: 2000,
        missingAt: null,
      },
      {
        id: 3,
        title: "Z",
        albumName: "Album",
        primaryArtistName: "Artist",
        durationMs: 3000,
        missingAt: null,
      },
    ];
    render(
      <TrackTableView
        rows={rows}
        queueContext="songs"
        onPlay={vi.fn()}
        virtual={false}
      />,
    );
    const renderedRows = screen.getAllByTestId("track-row");
    expect(renderedRows).toHaveLength(3);
    expect(renderedRows[0]).toHaveTextContent("X");
    expect(renderedRows[1]).toHaveTextContent("Y");
    expect(renderedRows[2]).toHaveTextContent("Z");
  });
});
