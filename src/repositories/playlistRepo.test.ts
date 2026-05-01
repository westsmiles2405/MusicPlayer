import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("playlistRepo", () => {
  beforeEach(() => invoke.mockReset());

  it("gets playlist tracks", async () => {
    const { playlistRepo } = await import("./playlistRepo");
    await playlistRepo.tracks(7);
    expect(invoke).toHaveBeenCalledWith("get_playlist_tracks", {
      playlistId: 7,
    });
  });

  it("reorders with source and destination positions", async () => {
    const { playlistRepo } = await import("./playlistRepo");
    await playlistRepo.reorder(7, 0, 2);
    expect(invoke).toHaveBeenCalledWith("reorder_playlist", {
      playlistId: 7,
      sourcePosition: 0,
      destinationPosition: 2,
    });
  });
});
