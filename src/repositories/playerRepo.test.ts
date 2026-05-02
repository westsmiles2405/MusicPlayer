import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("playerRepo", () => {
  beforeEach(() => invoke.mockReset());

  it("sends player_play with queue args", async () => {
    const { playerRepo } = await import("@/repositories/playerRepo");
    await playerRepo.play({ trackId: 2, queueTrackIds: [1, 2], queueIndex: 1 });
    expect(invoke).toHaveBeenCalledWith("player_play", {
      trackId: 2,
      queueTrackIds: [1, 2],
      queueIndex: 1,
    });
  });

  it("sends mute commands", async () => {
    const { playerRepo } = await import("@/repositories/playerRepo");
    await playerRepo.setMuted(true);
    await playerRepo.toggleMute();
    expect(invoke).toHaveBeenNthCalledWith(1, "player_set_muted", {
      muted: true,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "player_toggle_mute");
  });

  it("sends player_next command", async () => {
    const { playerRepo } = await import("@/repositories/playerRepo");
    await playerRepo.next();
    expect(invoke).toHaveBeenCalledWith("player_next");
  });

  it("sends player_previous command", async () => {
    const { playerRepo } = await import("@/repositories/playerRepo");
    await playerRepo.previous();
    expect(invoke).toHaveBeenCalledWith("player_previous");
  });

  it("sends player_seek with position", async () => {
    const { playerRepo } = await import("@/repositories/playerRepo");
    await playerRepo.seek(5000);
    expect(invoke).toHaveBeenCalledWith("player_seek", { positionMs: 5000 });
  });
});
