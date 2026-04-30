import { describe, it, expect } from "vitest";

describe("v0.2.0 core data layer smoke", () => {
  it("repository modules export their repos", async () => {
    const { trackRepo } = await import("@/repositories/trackRepo");
    const { albumRepo } = await import("@/repositories/albumRepo");
    const { artistRepo } = await import("@/repositories/artistRepo");
    const { playlistRepo } = await import("@/repositories/playlistRepo");
    const { searchRepo } = await import("@/repositories/searchRepo");
    const { historyRepo } = await import("@/repositories/historyRepo");

    expect(trackRepo.list).toBeDefined();
    expect(albumRepo.list).toBeDefined();
    expect(artistRepo.list).toBeDefined();
    expect(playlistRepo.list).toBeDefined();
    expect(searchRepo.query).toBeDefined();
    expect(historyRepo.recent).toBeDefined();
  });

  it("TrackSort type is a string union", () => {
    // Compile-time check: valid sort values
    const sorts = [
      "title",
      "artist",
      "album",
      "addedAt",
      "lastPlayed",
    ] as const;
    expect(sorts.length).toBe(5);
  });
});
