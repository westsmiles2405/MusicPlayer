import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { describe, expect, it, vi } from "vitest";

const invoke = vi.fn().mockResolvedValue([]);
vi.mock("@tauri-apps/api/core", () => ({ invoke }));

const listen = vi.fn().mockResolvedValue(() => {});
vi.mock("@tauri-apps/api/event", () => ({ listen }));

describe("Sidebar", () => {
  it("renders navigation links for library, playlists, and settings", async () => {
    const { Sidebar } = await import("./Sidebar");

    const qc = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    render(
      <QueryClientProvider client={qc}>
        <MemoryRouter>
          <Sidebar />
        </MemoryRouter>
      </QueryClientProvider>,
    );

    expect(
      screen.getByRole("navigation", { name: "侧边栏" }),
    ).toBeInTheDocument();

    expect(screen.getByRole("link", { name: "搜索" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "最近添加" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "歌曲" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "专辑" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "艺人" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "收藏" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "最近播放" })).toBeInTheDocument();
    expect(
      screen.getByRole("link", { name: "全部播放列表" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "设置" })).toBeInTheDocument();
  });
});
