# Apple Music 设计对标实施计划（修订版 v1.1）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 通过 7 个主线任务 + 1 个 Spike，使 MusicPlayer 在视觉和交互层面更接近 Apple Music。

**Architecture:** 修改集中在 CSS 层（globals.css）和少量组件文件（TrackTableView、MiniPlayer、NowPlayingOverlay）。不动数据层、不改路由、不加新依赖（Spike 除外）。

**Tech Stack:** CSS custom properties, Framer Motion LayoutGroup + layoutId, ARIA attributes

**Spec:** `docs/design-upgrade/specs/2026-05-03-apple-music-设计对标规范.md`

---

## File Map

| 文件 | 操作 | 涉及任务 |
|---|---|---|
| `src/styles/globals.css` | Modify | T1, T2, T3, T4, T7 |
| `src/components/library/TrackTableView.tsx` | Modify | T1 |
| `src/components/library/TrackTable.tsx` | Modify | T1 |
| `src/components/library/TrackTable.test.tsx` | Modify | T1 |
| `src/components/player/MiniPlayer.tsx` | Modify | T2, T5 |
| `src/components/player/NowPlayingOverlay.tsx` | Modify | T4, T5 |
| `src/components/player/NowPlayingOverlay.test.tsx` | Modify | T5 |
| `src/App.tsx` 或 `src/components/layout/AppShell.tsx` | Modify | T6 |

## 依赖关系

```
T1 独立
T2 独立（改 globals.css，建议在 T1 后串行避免冲突）
T3 独立（改 globals.css，建议在 T2 后串行）
T4 独立（改 globals.css，建议在 T3 后串行）
T5 独立（需确认 MiniPlayer 和 NowPlayingOverlay 在同一 LayoutGroup 下）
T6 独立（建议在 T4 后，减少 CSS 冲突）
T7 独立（建议在 T4 后）
T8 独立 Spike，不阻塞主线
```

---

### Task 1: Track Table 交互升级（P0）

**Files:**
- Modify: `src/styles/globals.css`
- Modify: `src/components/library/TrackTableView.tsx`
- Modify: `src/components/library/TrackTable.tsx`
- Modify: `src/components/library/TrackTable.test.tsx`

**前置确认：** 在开始前，先读取 `TrackTableView.tsx` 和 `TrackTable.tsx`，确认：
- 现有 `<tr>` 的渲染方式和 props
- 播放回调是 `onPlayIndex(idx)` 还是 `playTrack(id)`
- `playableIds` 是数组还是 Set

- [ ] **Step 1: 写失败测试 — class hook 存在**

在 `TrackTable.test.tsx` 中添加：

```tsx
it('renders table rows with interaction class hooks', () => {
  const tracks = [makeTrack({ id: 1, title: 'Song A' })];
  render(<TrackTable tracks={tracks} playableIds={[1]} queueIndexMap={{}} />);

  const rows = screen.getAllByTestId('track-row');
  expect(rows[0]).toHaveClass('track-table__row');
});
```

- [ ] **Step 2: 运行测试确认失败**

Run: `pnpm test -- --run src/components/library/TrackTable.test.tsx`
Expected: FAIL — `track-table__row` class 不存在

- [ ] **Step 3: 给 `<tr>` 添加 BEM class**

在 `TrackTableView.tsx` 中，给每行 `<tr>` 添加 class：

```tsx
<tr
  key={row.id}
  data-testid="track-row"
  data-missing={row.isMissing}
  className={`track-table__row ${currentTrackId === row.id ? 'track-table__row--playing' : ''}`}
  aria-current={currentTrackId === row.id ? 'true' : undefined}
  onDoubleClick={() => onPlayRow?.(row)}
>
```

- [ ] **Step 4: 运行测试确认通过**

Run: `pnpm test -- --run src/components/library/TrackTable.test.tsx`
Expected: PASS

- [ ] **Step 5: 写失败测试 — aria-current 属性**

```tsx
it('sets aria-current="true" on the currently playing row', () => {
  const tracks = [
    makeTrack({ id: 1, title: 'Song A' }),
    makeTrack({ id: 2, title: 'Song B' }),
  ];
  render(
    <TrackTable
      tracks={tracks}
      playableIds={[1, 2]}
      queueIndexMap={{}}
      currentTrackId={2}
    />
  );

  expect(screen.getByRole('row', { current: true })).toHaveTextContent('Song B');
});
```

- [ ] **Step 6: 运行测试确认失败**

Run: `pnpm test -- --run src/components/library/TrackTable.test.tsx`
Expected: FAIL — `currentTrackId` prop 不存在

- [ ] **Step 7: 传递 currentTrackId prop**

在 `TrackTable.tsx` 中，从 playerStore 获取 currentTrackId 并传给 TrackTableView：

```tsx
// TrackTable.tsx
import { usePlayerStore } from '@/stores/playerStore';

// 在组件内部
const currentTrackId = usePlayerStore((s) => s.currentTrack?.id);

// 传给 TrackTableView
<TrackTableView
  ...
  currentTrackId={currentTrackId}
/>
```

在 `TrackTableView.tsx` 的 props 接口中添加：

```tsx
interface TrackTableViewProps {
  // ... 现有 props
  currentTrackId?: number;
}
```

- [ ] **Step 8: 运行测试确认通过**

Run: `pnpm test -- --run src/components/library/TrackTable.test.tsx`
Expected: PASS

- [ ] **Step 9: 写失败测试 — 双击播放**

```tsx
it('calls onPlayRow with track data when a row is double-clicked', () => {
  const onPlayRow = vi.fn();
  const tracks = [makeTrack({ id: 1, title: 'Song A' })];
  render(
    <TrackTable
      tracks={tracks}
      playableIds={[1]}
      queueIndexMap={{}}
      onPlayRow={onPlayRow}
    />
  );

  const rows = screen.getAllByTestId('track-row');
  fireEvent.doubleClick(rows[0]);
  expect(onPlayRow).toHaveBeenCalledWith(expect.objectContaining({ id: 1 }));
});
```

- [ ] **Step 10: 运行测试确认失败**

Run: `pnpm test -- --run src/components/library/TrackTable.test.tsx`
Expected: FAIL — `onPlayRow` 未被调用

- [ ] **Step 11: 实现双击播放**

在 `TrackTableView.tsx` 中添加 `onPlayRow` prop：

```tsx
interface TrackTableViewProps {
  // ... 现有 props
  onPlayRow?: (row: TrackTableRow) => void;
}
```

在 `TrackTable.tsx` 中传入（使用 visibleIndex 而非 trackId）：

```tsx
<TrackTableView
  ...
  onPlayRow={(row) => {
    // 按当前列表中的可见位置播放，而非按 trackId
    const visibleIndex = rows.findIndex((item) => item.id === row.id);
    if (visibleIndex >= 0) {
      onPlayIndex(visibleIndex);
    }
  }}
/>
```

> **设计决策：** 使用 visibleIndex 而非 trackId，因为 `onPlayIndex` 是队列播放的入口，需要知道曲目在当前列表中的位置。

- [ ] **Step 12: 运行测试确认通过**

Run: `pnpm test -- --run src/components/library/TrackTable.test.tsx`
Expected: PASS

- [ ] **Step 13: 添加 CSS — hover 高亮、编号切换**

在 `globals.css` 中追加：

```css
/* ===== Track Table 交互升级 ===== */

/* 行基础样式 */
.track-table__row {
  transition: background 0.15s ease;
}

/* hover 高亮 */
.track-table__row:hover {
  background: rgba(0, 0, 0, 0.04);
}

[data-theme="dark"] .track-table__row:hover {
  background: rgba(255, 255, 255, 0.06);
}

/* 正在播放行 */
.track-table__row--playing {
  background: rgba(250, 35, 62, 0.08);
}

[data-theme="dark"] .track-table__row--playing {
  background: rgba(250, 35, 62, 0.12);
}

/* 正在播放标题变色 */
.track-table__row--playing .track-table__title {
  color: var(--color-apple-red);
}

/* 编号单元格 — 作为指示条的定位上下文 */
.track-table__number-cell {
  position: relative;
  padding-left: 8px;
}

/* 正在播放指示条 — 挂在第一列而非 <tr> */
.track-table__row--playing .track-table__number-cell::before {
  content: '';
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 3px;
  background: var(--color-apple-red);
  border-radius: 0 2px 2px 0;
}

/* 编号内部容器 */
.track-table__number {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
}

.track-table__number-text {
  transition: opacity 0.15s ease;
}

.track-table__number-icon {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: opacity 0.15s ease;
}

/* hover 时切换：文字消失，图标出现 */
.track-table__row:hover .track-table__number-text {
  opacity: 0;
}

.track-table__row:hover .track-table__number-icon {
  opacity: 1;
}

/* 正在播放行始终显示图标 */
.track-table__row--playing .track-table__number-text {
  opacity: 0;
}

.track-table__row--playing .track-table__number-icon {
  opacity: 1;
}
```

- [ ] **Step 14: 更新 TrackTableView 渲染编号列**

在 `TrackTableView.tsx` 中，将编号列改为三层结构：

```tsx
<td className="track-table__number-cell">
  <span className="track-table__number">
    <span className="track-table__number-text">
      {row.trackNo || index + 1}
    </span>
    <span className="track-table__number-icon" aria-hidden="true">
      ▶
    </span>
  </span>
</td>
```

- [ ] **Step 15: 写测试 — 编号结构**

```tsx
it('renders number cell with text and icon elements', () => {
  const tracks = [makeTrack({ id: 1, title: 'Song A', trackNo: 1 })];
  render(<TrackTable tracks={tracks} playableIds={[1]} queueIndexMap={{}} />);

  const numberCell = screen.getByText('1').closest('.track-table__number-cell');
  expect(numberCell).toBeTruthy();
  expect(numberCell!.querySelector('.track-table__number-icon')).toBeTruthy();
});
```

- [ ] **Step 16: 运行全部 TrackTable 测试**

Run: `pnpm test -- --run src/components/library/TrackTable.test.tsx`
Expected: ALL PASS

- [ ] **Step 17: Commit**

```bash
git add src/components/library/TrackTableView.tsx src/components/library/TrackTable.tsx src/components/library/TrackTable.test.tsx src/styles/globals.css
git commit -m "feat(ui): add hover highlight, playing indicator, and double-click to TrackTable"
```

---

### Task 2: 边框处理（P0）

**Files:**
- Modify: `src/styles/globals.css`
- Modify: `src/components/player/MiniPlayer.tsx`（确认 border-top 存在）

**验证方式：** 本任务不写自动化测试（jsdom 对 CSS 计算样式支持有限）。验收通过 `pnpm build` + DevTools 手动检查。

- [ ] **Step 1: 移除侧边栏实线边框**

在 `globals.css` 中找到 `.sidebar` 规则，删除 `border-right` 行：

```css
/* 之前 */
.sidebar {
  border-right: 1px solid var(--color-border);
  background: color-mix(in srgb, var(--color-sidebar) 50%, transparent);
  /* ... */
}

/* 之后 */
.sidebar {
  background: color-mix(in srgb, var(--color-sidebar) 50%, transparent);
  /* ... */
}
```

- [ ] **Step 2: 移除 MiniPlayer 实线边框，添加渐变分隔**

在 `globals.css` 中找到 `.mini-player` 规则：

```css
/* 之前 */
.mini-player {
  border-top: 1px solid var(--color-border);
  /* ... */
}

/* 之后 */
.mini-player {
  position: relative;
  /* 删除 border-top */
}

.mini-player::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgba(0, 0, 0, 0.1) 20%,
    rgba(0, 0, 0, 0.1) 80%,
    transparent 100%
  );
  pointer-events: none;
}

[data-theme="dark"] .mini-player::before {
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgba(255, 255, 255, 0.1) 20%,
    rgba(255, 255, 255, 0.1) 80%,
    transparent 100%
  );
}
```

- [ ] **Step 3: 运行构建验证**

Run: `pnpm build`
Expected: SUCCESS

- [ ] **Step 4: 手动验收**

DevTools 检查：
- `.sidebar` 无 `border-right` 属性
- `.mini-player` 无 `border-top` 属性
- `.mini-player::before` 存在，`height: 1px`，`pointer-events: none`

- [ ] **Step 5: Commit**

```bash
git add src/styles/globals.css
git commit -m "feat(ui): remove solid borders, add gradient separators for native feel"
```

---

### Task 3: 信息密度调整（P1）

**Files:**
- Modify: `src/styles/globals.css`

- [ ] **Step 1: 添加 CSS 自定义属性**

在 `globals.css` 的 `:root` 或 `@theme` 块中添加：

```css
:root {
  /* 信息密度 — 对齐 macOS 8pt 网格 */
  --sidebar-width: 220px;
  --content-padding-y: 20px;
  --content-padding-x: 24px;
  --grid-gap: 12px;
  --track-row-height: 34px;
  --page-header-size: 22px;
  --page-header-margin: 16px;
}
```

- [ ] **Step 2: 更新 AppShell grid 使用新变量**

在 `.app-shell` 规则中：

```css
/* 之前 */
.app-shell {
  grid-template-columns: 240px minmax(0, 1fr);
  /* ... */
}

/* 之后 */
.app-shell {
  grid-template-columns: var(--sidebar-width) minmax(0, 1fr);
  /* ... */
}
```

- [ ] **Step 3: 更新内容区内边距**

```css
/* 之前 — 找到 main 区域的 padding */
.main-content {
  padding: 24px 28px 96px;
}

/* 之后 */
.main-content {
  padding: var(--content-padding-y) var(--content-padding-x) 96px;
}
```

- [ ] **Step 4: 更新封面网格间距**

```css
/* 之前 */
.album-grid {
  gap: 16px;
}

/* 之后 */
.album-grid {
  gap: var(--grid-gap);
}
```

- [ ] **Step 5: 更新页面标题**

```css
/* 之前 */
.page-header h1 {
  font-size: 24px;
  margin-bottom: 20px;
}

/* 之后 */
.page-header h1 {
  font-size: var(--page-header-size);
  margin-bottom: var(--page-header-margin);
}
```

- [ ] **Step 6: 应用 track-row-height 变量**

```css
/* 新增 — 将变量应用到歌曲行 */
.track-table td {
  height: var(--track-row-height);
  padding: 0 12px;
  vertical-align: middle;
}
```

> **注意：** 这会覆盖 Task 4 中的 `padding: 6px 12px`。由于 T3 在 T4 之前执行，T4 应在此基础上调整而非重复设置。

- [ ] **Step 7: 运行前端构建验证**

Run: `pnpm build`
Expected: SUCCESS（无 CSS 错误）

- [ ] **Step 8: Commit**

```bash
git add src/styles/globals.css
git commit -m "feat(ui): tighten spacing to match macOS 8pt grid (220px sidebar, 34px rows)"
```

---

### Task 4: 字体排版精调（P1）

**Files:**
- Modify: `src/styles/globals.css`
- Modify: `src/components/player/NowPlayingOverlay.tsx`（确认 class 名匹配）

- [ ] **Step 1: 更新页面标题样式**

```css
/* 之前 */
.page-header h1 {
  font-size: var(--page-header-size);
  margin-bottom: var(--page-header-margin);
}

/* 之后 — 添加紧排 */
.page-header h1 {
  font-size: var(--page-header-size);
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1.2;
  margin-bottom: var(--page-header-margin);
}
```

- [ ] **Step 2: 更新 Now Playing 标题**

在修改前，先读取 `NowPlayingOverlay.tsx` 确认 `.now-playing__title` 的实际 class 名。

```css
/* 之前 */
.now-playing__title {
  font-size: 22px;
  font-weight: 700;
}

/* 之后 */
.now-playing__title {
  font-size: 28px;
  font-weight: 700;
  letter-spacing: -0.03em;
  line-height: 1.1;
}
```

- [ ] **Step 3: 更新 Now Playing 副标题**

```css
/* 之前 */
.now-playing__subtitle {
  font-size: 15px;
}

/* 之后 */
.now-playing__subtitle {
  font-size: 16px;
  letter-spacing: -0.01em;
}
```

- [ ] **Step 4: 更新歌曲表格行高**

> **注意：** Task 3 Step 6 已设置 `height: var(--track-row-height)` 和 `padding: 0 12px`。此处只补充 `line-height`。

```css
.track-table td {
  /* 继承 Task 3 的 height 和 padding */
  font-size: 13px;
  line-height: 1.3;
}
```

- [ ] **Step 5: 更新侧边栏标题**

```css
/* 之前 */
.sidebar__heading {
  font-size: 12px;
  font-weight: 600;
}

/* 之后 */
.sidebar__heading {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--color-text-secondary);
}
```

- [ ] **Step 6: 运行前端构建验证**

Run: `pnpm build`
Expected: SUCCESS

- [ ] **Step 7: Commit**

```bash
git add src/styles/globals.css
git commit -m "feat(ui): refine typography — tighter letter-spacing, larger Now Playing title"
```

---

### Task 5: Now Playing 展开动画（P2）

**Files:**
- Modify: `src/components/player/MiniPlayer.tsx`
- Modify: `src/components/player/NowPlayingOverlay.tsx`
- Modify: `src/components/player/NowPlayingOverlay.test.tsx`

**前置确认：**
1. 读取 `MiniPlayer.tsx` 和 `NowPlayingOverlay.tsx`，确认两者是否已被同一父组件（如 `AppShell`）渲染
2. 如果不在同一父组件，需要在共同祖先添加 `<LayoutGroup>`

- [ ] **Step 1: 添加 LayoutGroup 包裹**

在 `AppShell.tsx` 或 `App.tsx` 中，确保 MiniPlayer 和 NowPlayingOverlay 被 `<LayoutGroup>` 包裹：

```tsx
import { LayoutGroup } from 'framer-motion';

// 在渲染 MiniPlayer 和 NowPlayingOverlay 的共同祖先中
<LayoutGroup id="now-playing">
  <main>
    <Outlet />
  </main>
  <MiniPlayer />
  <NowPlayingOverlay />
</LayoutGroup>
```

- [ ] **Step 2: 写失败测试 — 封面 motion 元素存在**

```tsx
// NowPlayingOverlay.test.tsx
it('renders cover art wrapped in motion element', () => {
  render(<NowPlayingOverlay isOpen={true} onClose={vi.fn()} />);

  // 验证封面容器存在（motion.div 会被渲染为 div）
  const cover = document.querySelector('.now-playing__cover');
  expect(cover).toBeInTheDocument();
});
```

> **注意：** React Testing Library 无法直接验证 Framer Motion 的 `layoutId` prop。共享元素动画的正确性通过代码审查 + 手动视觉测试验证。

- [ ] **Step 3: 运行测试确认通过**

Run: `pnpm test -- --run src/components/player/NowPlayingOverlay.test.tsx`
Expected: PASS（测试只验证元素存在，不验证 layoutId）

- [ ] **Step 4: 给 MiniPlayer 封面添加 layoutId**

在 `MiniPlayer.tsx` 中：

```tsx
import { motion } from 'framer-motion';

// 找到 mini-player__art 的 div
// 之前
<div className="mini-player__art" aria-hidden="true" />

// 之后
<motion.div
  className="mini-player__art"
  aria-hidden="true"
  layoutId="now-playing-cover"
  style={{ borderRadius: 8 }}
/>
```

- [ ] **Step 5: 给 NowPlayingOverlay 封面添加 layoutId**

在 `NowPlayingOverlay.tsx` 中：

```tsx
// 找到 now-playing__cover 的 div
// 之前
<div className="now-playing__cover">
  {coverUrl ? <img ... /> : <div ... />}
</div>

// 之后
<motion.div className="now-playing__cover" layoutId="now-playing-cover">
  {coverUrl ? <img ... /> : <div ... />}
</motion.div>
```

- [ ] **Step 6: 更新 NowPlayingOverlay 动画参数**

```tsx
// 之前
<motion.section
  className="now-playing"
  initial={{ opacity: 0, y: 40 }}
  animate={{ opacity: 1, y: 0 }}
  exit={{ opacity: 0, y: 40 }}
  transition={{ duration: 0.25, ease: "easeOut" }}
>

// 之后 — 仅控制背景淡入，封面由 layoutId 处理
<motion.section
  className="now-playing"
  initial={{ opacity: 0 }}
  animate={{ opacity: 1 }}
  exit={{ opacity: 0 }}
  transition={{ duration: 0.3, ease: [0.32, 0.72, 0, 1] }}
>
```

- [ ] **Step 7: 运行测试确认通过**

Run: `pnpm test -- --run src/components/player/NowPlayingOverlay.test.tsx`
Expected: PASS

- [ ] **Step 8: 手动视觉验收**

1. 打开应用，播放一首歌
2. 点击 MiniPlayer 封面区域
3. 预期：封面从 MiniPlayer 位置无缝放大到 NowPlaying 中央
4. 关闭 NowPlaying
5. 预期：封面缩小回到 MiniPlayer 位置

- [ ] **Step 9: Commit**

```bash
git add src/components/player/MiniPlayer.tsx src/components/player/NowPlayingOverlay.tsx src/components/player/NowPlayingOverlay.test.tsx src/components/layout/AppShell.tsx
git commit -m "feat(ui): add shared element transition for Now Playing cover expansion"
```

---

### Task 6: 页面切换动画（P3）

**Files:**
- Modify: `src/App.tsx` 或 `src/components/layout/AppShell.tsx`

**注意：** 本任务仅使用 Framer Motion，不写 CSS keyframes。项目已统一使用 Framer Motion 做动画，避免两套系统混用。

- [ ] **Step 1: 在路由出口包装动画容器**

在 `App.tsx` 或 `AppShell.tsx` 中，用 `<AnimatePresence>` 包裹路由：

```tsx
import { useLocation } from 'react-router-dom';
import { AnimatePresence, motion } from 'framer-motion';

// 在路由出口处
const location = useLocation();

<AnimatePresence mode="wait">
  <motion.div
    key={location.pathname}
    initial={{ opacity: 0, x: 20 }}
    animate={{ opacity: 1, x: 0 }}
    exit={{ opacity: 0, x: -20 }}
    transition={{ duration: 0.22, ease: [0.32, 0.72, 0, 1] }}
  >
    <Outlet />
  </motion.div>
</AnimatePresence>
```

- [ ] **Step 2: 运行前端构建验证**

Run: `pnpm build`
Expected: SUCCESS

- [ ] **Step 3: 手动视觉验收**

1. 在侧边栏点击不同页面
2. 预期：页面有 220ms 的滑入/滑出过渡
3. 快速切换多个页面
4. 预期：动画不卡顿，无闪烁

- [ ] **Step 4: Commit**

```bash
git add src/App.tsx
git commit -m "feat(ui): add page transition animation with Framer Motion"
```

---

### Task 7: 进度条弹性反馈（P3）

**Files:**
- Modify: `src/styles/globals.css`
- Modify: `src/components/player/MiniPlayer.tsx`（如需添加 class）
- Modify: `src/components/player/NowPlayingOverlay.tsx`（如需添加 class）

**前置确认（必须先做）：** 读取 `MiniPlayer.tsx` 和 `NowPlayingOverlay.tsx`，确认进度条的 DOM 结构：

- **如果是原生 `<input type="range">`**：使用 `::-webkit-slider-thumb` 伪元素
- **如果是自定义 DOM slider**（`<div>` + `<span>`）：使用 `.track` / `.thumb` class

以下按两种情况分别写 CSS。

- [ ] **Step 1a: 原生 input range 的 CSS**

如果进度条是 `<input type="range" className="progress-slider" />`：

```css
/* ===== 进度条弹性反馈（原生 range） ===== */

.progress-slider {
  height: 4px;
  transition: height 0.15s cubic-bezier(0.32, 0.72, 0, 1);
}

.progress-slider:hover {
  height: 6px;
}

.progress-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--color-text);
  transform: scale(1);
  transition: transform 0.15s cubic-bezier(0.32, 0.72, 0, 1);
}

.progress-slider:hover::-webkit-slider-thumb {
  transform: scale(1.3);
}

.progress-slider:active::-webkit-slider-thumb {
  transform: scale(1.5);
}

/* 音量滑块 */
.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--color-text);
  transform: scale(1);
  transition: transform 0.15s cubic-bezier(0.32, 0.72, 0, 1);
}

.volume-slider:hover::-webkit-slider-thumb {
  transform: scale(1.2);
}
```

- [ ] **Step 1b: 自定义 DOM slider 的 CSS**

如果进度条是自定义 DOM 结构（`<div className="progress-slider"><div className="progress-slider__track" /><div className="progress-slider__thumb" /></div>`）：

```css
/* ===== 进度条弹性反馈（自定义 DOM） ===== */

.progress-slider__track {
  height: 4px;
  transition: height 0.15s cubic-bezier(0.32, 0.72, 0, 1);
}

.progress-slider:hover .progress-slider__track {
  height: 6px;
}

.progress-slider__thumb {
  width: 12px;
  height: 12px;
  transition: transform 0.15s cubic-bezier(0.32, 0.72, 0, 1);
}

.progress-slider:hover .progress-slider__thumb {
  transform: scale(1.3);
}

.progress-slider:active .progress-slider__thumb {
  transform: scale(1.5);
}
```

- [ ] **Step 2: 确认 class 名匹配**

读取 `MiniPlayer.tsx` 和 `NowPlayingOverlay.tsx`，确认进度条元素的 className 与 CSS 选择器匹配。如果不匹配，修改组件添加正确的 class。

- [ ] **Step 3: 运行前端构建验证**

Run: `pnpm build`
Expected: SUCCESS

- [ ] **Step 4: Commit**

```bash
git add src/styles/globals.css
git commit -m "feat(ui): add elastic feedback to progress and volume sliders"
```

---

### Task 8: 动态色彩系统 Spike（Spike）

**Files:**
- Create: `src/hooks/useDynamicColors.ts`
- Modify: `src/components/player/NowPlayingBackground.tsx`

**Spike 限制：**
- 必须在独立分支 `spike/dynamic-colors` 完成，不 merge 到主线
- `extract-colors` 不写入 `package.json` 主线（仅在 Spike 分支安装）
- 结果必须 Map 缓存（相同 coverUrl 不重复提取）
- 失败时必须回退 `defaultColors`

> **注意：** 本任务为 Spike 验证，不纳入主线验收。可独立进行，不阻塞其他任务。

- [ ] **Step 1: 创建 Spike 分支**

```bash
git checkout -b spike/dynamic-colors
```

- [ ] **Step 2: 安装 extract-colors**

```bash
pnpm add extract-colors
```

- [ ] **Step 3: 创建 useDynamicColors hook**

```tsx
// src/hooks/useDynamicColors.ts
import { useState, useEffect, useRef } from 'react';
import { extractColors } from 'extract-colors';

interface DynamicColors {
  primary: string;
  background: string;
  text: string;
}

const defaultColors: DynamicColors = {
  primary: '#fa233e',
  background: '#1c1c1e',
  text: '#f5f5f7',
};

// 缓存：相同 coverUrl 不重复提取
const colorCache = new Map<string, DynamicColors>();

function adjustBrightness(hex: string, amount: number): string {
  const num = parseInt(hex.replace('#', ''), 16);
  const r = Math.max(0, Math.min(255, (num >> 16) + amount));
  const g = Math.max(0, Math.min(255, ((num >> 8) & 0x00ff) + amount));
  const b = Math.max(0, Math.min(255, (num & 0x0000ff) + amount));
  return `#${(r << 16 | g << 8 | b).toString(16).padStart(6, '0')}`;
}

function getContrastColor(hex: string): string {
  const num = parseInt(hex.replace('#', ''), 16);
  const r = num >> 16;
  const g = (num >> 8) & 0x00ff;
  const b = num & 0x0000ff;
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return luminance > 0.5 ? '#1d1d1f' : '#f5f5f7';
}

export function useDynamicColors(coverUrl: string | null): DynamicColors {
  const [colors, setColors] = useState<DynamicColors>(defaultColors);

  useEffect(() => {
    if (!coverUrl) {
      setColors(defaultColors);
      return;
    }

    // 检查缓存
    const cached = colorCache.get(coverUrl);
    if (cached) {
      setColors(cached);
      return;
    }

    extractColors(coverUrl)
      .then((palette) => {
        if (!palette.length) return;

        const dominant = palette[0];
        const vibrant = palette.find((c) => c.saturation > 0.5) || dominant;

        const result: DynamicColors = {
          primary: vibrant.hex,
          background: adjustBrightness(dominant.hex, -60),
          text: getContrastColor(dominant.hex),
        };

        // 写入缓存
        colorCache.set(coverUrl, result);
        setColors(result);
      })
      .catch(() => {
        setColors(defaultColors);
      });
  }, [coverUrl]);

  return colors;
}
```

- [ ] **Step 4: 更新 NowPlayingBackground 使用动态色彩**

```tsx
// src/components/player/NowPlayingBackground.tsx
import { useDynamicColors } from '@/hooks/useDynamicColors';

interface Props {
  coverUrl: string | null;
}

export function NowPlayingBackground({ coverUrl }: Props) {
  const colors = useDynamicColors(coverUrl);

  return (
    <div
      className="np-bg"
      style={{
        background: `
          radial-gradient(ellipse at 50% 0%, ${colors.primary}40 0%, transparent 70%),
          linear-gradient(180deg, ${colors.background} 0%, #000 100%)
        `,
        transition: 'background 0.5s ease',
      }}
    />
  );
}
```

- [ ] **Step 5: 验证性能**

手动测试：
1. 打开 Now Playing
2. 快速切换歌曲（至少 5 首）
3. 观察色彩过渡是否流畅（≥ 30fps）
4. 检查文字对比度是否可读（人工检查，不依赖 WCAG 计算）
5. 检查相同封面是否命中缓存（console.log 验证）

- [ ] **Step 6: Spike 结论**

记录验证结果：
- 色彩提取耗时：___ms
- 过渡动画帧率：___fps
- 文字可读性：通过/不通过
- 缓存命中：是/否
- 是否值得升级为主线功能：是/否

- [ ] **Step 7: Commit（仅 Spike 分支）**

```bash
git add src/hooks/useDynamicColors.ts src/components/player/NowPlayingBackground.tsx
git commit -m "spike: dynamic color extraction from album cover for Now Playing"
```

---

## 验收检查清单

### 主线验收（P0-P3）

| 任务 | 验收项 | 验证方法 |
|---|---|---|
| T1 | 行有 `track-table__row` class | 测试通过 |
| T1 | 播放行 `aria-current="true"` | `screen.getByRole('row', { current: true })` |
| T1 | 播放行指示条 3px `#fa233e`，挂在第一列 | DevTools 检查 `::before` |
| T1 | 双击调用 `onPlayRow(row)` | 测试通过 |
| T1 | hover 编号文字消失、图标出现 | DevTools 检查 opacity |
| T2 | 侧边栏无 `border-right` | DevTools |
| T2 | MiniPlayer 无 `border-top`，有 `::before` 渐变 | DevTools |
| T3 | `--sidebar-width: 220px` 应用到 `.app-shell` | DevTools |
| T3 | `--track-row-height: 34px` 应用到 `td` | DevTools |
| T4 | Now Playing 标题 `28px`, `-0.03em` | DevTools |
| T4 | 歌曲行 `line-height: 1.3` | DevTools |
| T5 | LayoutGroup 包裹 MiniPlayer + NowPlayingOverlay | 代码审查 |
| T5 | 封面 `layoutId="now-playing-cover"` | 代码审查 |
| T5 | 展开动画 300ms, ease `[0.32, 0.72, 0, 1]` | DevTools |
| T5 | 视觉：封面从 MiniPlayer 无缝放大 | 手动测试 |
| T6 | 页面切换 220ms 进入 / 220ms 退出 | DevTools |
| T7 | 进度条 hover 滑块 `scale(1.3)` | DevTools |

### Spike 验收

| 指标 | 标准 | 结果 |
|---|---|---|
| 色彩提取耗时 | ≤ 200ms | ___ |
| 过渡帧率 | ≥ 30fps | ___ |
| 文字可读性 | 人工检查通过 | ___ |
| 缓存命中 | 相同 coverUrl 不重复提取 | ___ |
