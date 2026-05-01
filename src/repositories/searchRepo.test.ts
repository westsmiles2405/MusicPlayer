import { beforeEach, describe, expect, it, vi } from "vitest";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

describe("searchRepo", () => {
  beforeEach(() => invoke.mockReset());

  it("calls library_search_all with limitPerGroup", async () => {
    const { searchRepo } = await import("./searchRepo");
    await searchRepo.search("love", 12);
    expect(invoke).toHaveBeenCalledWith("library_search_all", {
      query: "love",
      limitPerGroup: 12,
    });
  });
});
