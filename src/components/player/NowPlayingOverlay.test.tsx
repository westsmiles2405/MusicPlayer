import { act, fireEvent, render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ReactNode } from "react";

vi.mock("framer-motion", () => ({
  AnimatePresence: ({ children }: { children: ReactNode }) => <>{children}</>,
  motion: new Proxy({} as Record<string, React.FC<Record<string, unknown>>>, {
    get: (_target, tag: string) => {
      const Component = ({
        children,
        initial: _initial,
        animate: _animate,
        exit: _exit,
        transition: _transition,
        layoutId: _layoutId,
        ...domProps
      }: Record<string, unknown>) => {
        return <div {...domProps}>{children as ReactNode}</div>;
      };
      Component.displayName = `motion.${tag}`;
      return Component;
    },
  }),
}));

const invokeMock = vi.fn().mockResolvedValue({
  status: "playing",
  current: {
    id: 1,
    title: "Track A",
    artistName: "Artist A",
    albumName: "Album A",
    durationMs: 180000,
    coverPath: null,
  },
  positionMs: 30000,
  durationMs: 180000,
  volume: 0.8,
  muted: false,
  queueIndex: 0,
  queueLen: 3,
  repeatMode: "off",
  shuffle: false,
});

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider client={qc}>{children}</QueryClientProvider>
);

beforeEach(() => {
  invokeMock.mockResolvedValue({
    status: "playing",
    current: {
      id: 1,
      title: "Track A",
      artistName: "Artist A",
      albumName: "Album A",
      durationMs: 180000,
      coverPath: null,
    },
    positionMs: 30000,
    durationMs: 180000,
    volume: 0.8,
    muted: false,
    queueIndex: 0,
    queueLen: 3,
    repeatMode: "off",
    shuffle: false,
  });
});

describe("NowPlayingOverlay", () => {
  it("renders full structure when open", async () => {
    const { NowPlayingOverlay } = await import("./NowPlayingOverlay");
    const onClose = vi.fn();
    await act(async () => {
      render(<NowPlayingOverlay open onClose={onClose} />, { wrapper });
    });
    expect(
      screen.getByRole("dialog", { name: "Now Playing" }),
    ).toBeInTheDocument();
    expect(screen.getByTestId("now-playing-cover")).toBeInTheDocument();
    expect(screen.getByText("Track A")).toBeInTheDocument();
    expect(screen.getByText("Artist A")).toBeInTheDocument();
    expect(screen.getByText("Album A")).toBeInTheDocument();
    expect(
      screen.getByRole("slider", { name: "播放进度" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "上一首" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "暂停" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "下一首" })).toBeInTheDocument();
    expect(screen.getByRole("slider", { name: "音量" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "关闭 Now Playing" }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("renders empty state when no current track", async () => {
    invokeMock.mockResolvedValue({
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
    });
    const { NowPlayingOverlay } = await import("./NowPlayingOverlay");
    await act(async () => {
      render(<NowPlayingOverlay open onClose={() => {}} />, { wrapper });
    });
    expect(screen.getByText("暂无播放内容")).toBeInTheDocument();
  });

  it("renders volume slider", async () => {
    const { NowPlayingOverlay } = await import("./NowPlayingOverlay");
    await act(async () => {
      render(<NowPlayingOverlay open onClose={() => {}} />, { wrapper });
    });
    expect(screen.getByRole("slider", { name: "音量" })).toBeInTheDocument();
  });

  it("renders progress slider", async () => {
    const { NowPlayingOverlay } = await import("./NowPlayingOverlay");
    await act(async () => {
      render(<NowPlayingOverlay open onClose={() => {}} />, { wrapper });
    });
    expect(screen.getByRole("slider", { name: "播放进度" })).toBeInTheDocument();
  });

  it("renders play/pause button", async () => {
    const { NowPlayingOverlay } = await import("./NowPlayingOverlay");
    await act(async () => {
      render(<NowPlayingOverlay open onClose={() => {}} />, { wrapper });
    });
    // The mock returns status: "playing", so button should say "暂停"
    expect(screen.getByRole("button", { name: "暂停" })).toBeInTheDocument();
  });
});
