import { beforeEach, describe, expect, it } from "vitest";
import type { PlayerStore } from "@/stores/playerStore";
import { usePlayerStore } from "@/stores/playerStore";

beforeEach(() => {
  usePlayerStore.setState({
    status: "idle",
    current: null,
    positionMs: 0,
    durationMs: 0,
    volume: 0.8,
    muted: false,
    queueIndex: null,
    queueLen: 0,
    repeatMode: "off",
    shuffle: false,
    error: null,
    isSeeking: false,
    optimisticPositionMs: null,
  } as unknown as PlayerStore);
});

describe("playerStore", () => {
  it("progress only updates position and duration", () => {
    usePlayerStore.getState().applySnapshot({
      status: "playing",
      current: {
        id: 1,
        title: "Song",
        albumName: null,
        artistName: null,
        durationMs: 1000,
        coverPath: null,
      },
      positionMs: 0,
      durationMs: 1000,
      volume: 0.5,
      muted: false,
      queueIndex: 0,
      queueLen: 1,
      repeatMode: "off",
      shuffle: false,
    });
    usePlayerStore
      .getState()
      .applyProgress({ positionMs: 100, durationMs: 1000 });
    const state = usePlayerStore.getState();
    expect(state.current?.id).toBe(1);
    expect(state.positionMs).toBe(100);
    expect(state.volume).toBe(0.5);
  });

  it("progress does not overwrite active seek preview", () => {
    usePlayerStore.getState().beginSeek(500);
    usePlayerStore
      .getState()
      .applyProgress({ positionMs: 100, durationMs: 1000 });
    const state = usePlayerStore.getState();
    expect(state.positionMs).toBe(0);
    expect(state.optimisticPositionMs).toBe(500);
    expect(state.durationMs).toBe(1000);
  });

  it("track_changed updates current structure", () => {
    usePlayerStore.getState().applyTrackChanged({
      id: 4,
      title: "Next",
      albumName: "Album",
      artistName: "Artist",
      durationMs: 2000,
      coverPath: null,
    });
    const state = usePlayerStore.getState();
    expect(state.current?.title).toBe("Next");
    expect(state.positionMs).toBe(0);
    expect(state.durationMs).toBe(2000);
  });
});
