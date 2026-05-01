import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("recentPlayRepo", () => {
  beforeEach(() => invoke.mockReset());

  it("lists recent played tracks", async () => {
    const { recentPlayRepo } = await import("./recentPlayRepo");
    await recentPlayRepo.list(25);
    expect(invoke).toHaveBeenCalledWith("library_get_recent_played_tracks", {
      limit: 25,
    });
  });
});
