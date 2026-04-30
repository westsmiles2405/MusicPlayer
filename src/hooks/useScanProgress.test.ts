import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";

const listeners: Record<string, Array<(payload: unknown) => void>> = {};
vi.mock("@tauri-apps/api/event", () => ({
  listen: (event: string, handler: (e: { payload: unknown }) => void) => {
    listeners[event] = listeners[event] ?? [];
    const wrapped = (payload: unknown) => handler({ payload } as never);
    listeners[event].push(wrapped);
    return Promise.resolve(() => {
      const arr = listeners[event];
      if (!arr) return;
      const idx = arr.indexOf(wrapped);
      if (idx >= 0) arr.splice(idx, 1);
    });
  },
}));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

import { useScanProgress } from "./useScanProgress";

const fire = (event: string, payload: unknown) => {
  for (const h of listeners[event] ?? []) h(payload);
};

describe("useScanProgress", () => {
  beforeEach(() => {
    Object.keys(listeners).forEach((k) => delete listeners[k]);
  });

  it("starts in idle", () => {
    const { result } = renderHook(() => useScanProgress());
    expect(result.current.phase).toBe("idle");
    expect(result.current.progress).toBeNull();
    expect(result.current.report).toBeNull();
  });

  it("transitions idle -> scanning -> done", async () => {
    const { result } = renderHook(() => useScanProgress());
    await waitFor(() =>
      expect(listeners.scan_progress?.length ?? 0).toBeGreaterThan(0),
    );

    act(() =>
      fire("scan_progress", { done: 1, total: 5, currentFile: "a.mp3" }),
    );
    expect(result.current.phase).toBe("scanning");
    expect(result.current.progress?.done).toBe(1);

    act(() =>
      fire("scan_done", {
        added: 5,
        updated: 0,
        moved: 0,
        unchanged: 0,
        missing: 0,
        errors: [],
      }),
    );
    expect(result.current.phase).toBe("done");
    expect(result.current.report?.added).toBe(5);
  });
});
