import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("favoriteRepo", () => {
  beforeEach(() => invoke.mockReset());

  it("lists favorite tracks", async () => {
    const { favoriteRepo } = await import("./favoriteRepo");
    await favoriteRepo.list();
    expect(invoke).toHaveBeenCalledWith("library_get_favorite_tracks");
  });

  it("toggles favorite state", async () => {
    const { favoriteRepo } = await import("./favoriteRepo");
    await favoriteRepo.set(7, true);
    expect(invoke).toHaveBeenCalledWith("set_favorite", {
      trackId: 7,
      favorite: true,
    });
  });
});
