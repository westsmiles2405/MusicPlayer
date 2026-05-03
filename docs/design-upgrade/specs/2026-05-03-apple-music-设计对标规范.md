# Apple Music 设计对标规范

- 创建日期：2026-05-03
- 项目代号：MusicPlayer
- 文档状态：Draft v1.1（已按 review 反馈修订）
- 分支：`feat/apple-music-design-upgrade`
- 依据：前端设计审查 + Apple Music 实际界面对比

---

## 1. 背景与目标

### 1.1 背景

MusicPlayer v0.5.0 已完成基础功能（资料库浏览、播放列表、播放控制），毛玻璃效果和系统字体等 Apple 风格元素已到位。但在细节层面与真正的 Apple Music 仍有可感知的差距。

### 1.2 目标

通过本次设计升级，使 MusicPlayer 在视觉和交互层面更接近 Apple Music 的设计语言，同时保留项目的原创特色（鼠标跟踪光效、Dopamine 空状态）。

### 1.3 设计原则

1. **原生感优先** — 用 macOS 原生设计语言替代 Web 思维
2. **物理感交互** — 动效遵循惯性、弹性、阻尼的物理规律
3. **动态色彩** — UI 色彩跟随当前播放内容变化
4. **信息密度** — 对齐 macOS 8pt 网格，提升内容密度
5. **保留特色** — 鼠标跟踪光效和 Dopamine 空状态是差异化亮点，保留

---

## 2. 差距分析与改进项

### 2.1 Track Table（最高优先级）

**现状问题：**
- 无 hover 状态反馈
- 无正在播放行指示
- 无交互反馈（双击、选中）
- 按钮为原生样式

**对标 Apple Music：**

| 特性 | Apple Music | 目标实现 |
|---|---|---|
| 行 hover | 半透明高亮 + 播放按钮浮现 | ✅ 实现 |
| 正在播放行 | 左侧红色指示条 + 标题变色 | ✅ 实现 |
| 曲目编号 | 自动编号，hover 变播放图标 | ✅ 实现 |
| 选中行 | 蓝色高亮（多选） | ⏸️ 推迟到 v1.1 |
| 右键菜单 | 原生 macOS 菜单 | ⏸️ 推迟到 v1.1 |
| 排序 | 点击列头排序 | ⏸️ 推迟到 v1.1 |
| 分组 | 按光盘分组（多碟专辑） | ⏸️ 推迟到 v1.1 |

**具体规范：**

```css
/* 行 hover 状态 */
.track-row:hover {
  background: rgba(0, 0, 0, 0.04); /* 浅色主题 */
  /* dark: rgba(255, 255, 255, 0.06) */
}

/* 正在播放行 */
.track-row--playing {
  background: rgba(250, 35, 62, 0.08); /* 微红背景 */
}

.track-row--playing .track-row__title {
  color: var(--color-apple-red);
}

.track-row--playing::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--color-apple-red);
  border-radius: 0 2px 2px 0;
}

/* 曲目编号 hover 变播放图标 */
.track-row__number {
  transition: opacity 0.15s ease;
}

.track-row:hover .track-row__number--text {
  display: none;
}

.track-row:hover .track-row__number--icon {
  display: block;
}

/* 播放按钮浮现 */
.track-row__play-btn {
  opacity: 0;
  transition: opacity 0.15s ease;
}

.track-row:hover .track-row__play-btn {
  opacity: 1;
}
```

**交互规范：**
- 双击行：播放该曲目并加入队列
- hover 行：显示播放按钮，曲目编号变为播放图标
- 正在播放行：左侧红色指示条 + 标题变红，设置 `aria-current="true"`
- 键盘导航：支持上下箭头移动焦点，Enter 播放

---

### 2.2 动效系统

**现状问题：**
- Now Playing 展开是简单淡入，缺少空间连续性
- 页面切换瞬时跳转
- 进度条无弹性反馈
- 封面切换无过渡

**对标 Apple Music：**

> **注意：** Now Playing 展开动画与动态色彩系统（§2.3）完全独立，可并行开发。

#### 2.2.1 Now Playing 展开动画

**当前实现：**
```tsx
initial={{ opacity: 0, y: 40 }}
animate={{ opacity: 1, y: 0 }}
exit={{ opacity: 0, y: 40 }}
```

**目标实现：从封面无缝放大**
```tsx
// 使用 Framer Motion layoutId 实现共享元素过渡
// MiniPlayer 封面 → NowPlaying 封面

// MiniPlayer 中
<motion.div layoutId="now-playing-cover">
  <CoverArt ... />
</motion.div>

// NowPlaying 中
<motion.div layoutId="now-playing-cover">
  <CoverArt ... />
</motion.div>

// 背景层使用 AnimatePresence
<AnimatePresence>
  {isOpen && (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.3 }}
    >
      <NowPlayingBackground />
    </motion.div>
  )}
</AnimatePresence>
```

**物理参数：**
- 展开时长：300ms
- 缓动曲线：`cubic-bezier(0.32, 0.72, 0, 1)`（Apple 标准缓动）
- 封面放大：从 MiniPlayer 位置放大到 `min(60vw, 320px)`
- 背景淡入：与封面放大同时，200ms

#### 2.2.2 页面切换动画

**规范：**
```css
/* 页面进入 */
.page-enter {
  animation: page-slide-in 0.25s cubic-bezier(0.32, 0.72, 0, 1);
}

@keyframes page-slide-in {
  from {
    opacity: 0;
    transform: translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

/* 页面退出 */
.page-exit {
  animation: page-slide-out 0.2s cubic-bezier(0.32, 0.72, 0, 1);
}

@keyframes page-slide-out {
  from {
    opacity: 1;
    transform: translateX(0);
  }
  to {
    opacity: 0;
    transform: translateX(-20px);
  }
}
```

#### 2.2.3 进度条弹性反馈

**规范：**
```css
/* 进度滑块 hover 放大 */
.progress-slider__thumb {
  width: 12px;
  height: 12px;
  transition: transform 0.15s cubic-bezier(0.32, 0.72, 0, 1);
}

.progress-slider:hover .progress-slider__thumb {
  transform: scale(1.3);
}

/* 拖动时进一步放大 */
.progress-slider:active .progress-slider__thumb {
  transform: scale(1.5);
}

/* 进度条 hover 高亮 */
.progress-slider__track {
  height: 4px;
  transition: height 0.15s ease;
}

.progress-slider:hover .progress-slider__track {
  height: 6px;
}
```

#### 2.2.4 封面切换动画

**规范：**
```tsx
// 使用 AnimatePresence 实现交叉淡入
<AnimatePresence mode="wait">
  <motion.div
    key={trackId}
    initial={{ opacity: 0, scale: 0.95 }}
    animate={{ opacity: 1, scale: 1 }}
    exit={{ opacity: 0, scale: 1.05 }}
    transition={{ duration: 0.2 }}
  >
    <CoverArt src={coverUrl} />
  </motion.div>
</AnimatePresence>
```

---

### 2.3 动态色彩系统（Spike）

> **状态：Spike 验证** — 本节为技术验证性质，不纳入主线验收标准。验证成功后可升级为正式功能。

**现状问题：**
- 强调色固定为 `#fa233e`（Apple Red）
- 封面取色仅用于 Dopamine 空状态
- UI 不随播放内容变化

**对标 Apple Music：**

Apple Music 的 Now Playing 界面会根据封面主色动态调整：
- 背景渐变色
- 进度条颜色
- 控制按钮高亮色
- 文字颜色（确保对比度）

**Spike 目标：**
- 验证 `extract-colors` 在封面切换时的性能表现
- 验证动态色彩在暗色/浅色主题下的可读性
- 验证色彩过渡动画的流畅度

**实现方案：**

```typescript
// hooks/useDynamicColors.ts
import { extractColors } from 'extract-colors';

export function useDynamicColors(coverUrl: string | null) {
  const [colors, setColors] = useState<DynamicColors>(defaultColors);

  useEffect(() => {
    if (!coverUrl) {
      setColors(defaultColors);
      return;
    }

    extractColors(coverUrl).then((palette) => {
      const dominant = palette[0];
      const vibrant = palette.find(c => c.saturation > 0.5) || dominant;

      setColors({
        primary: vibrant.hex,
        background: adjustBrightness(dominant.hex, -60),
        text: getContrastColor(dominant.hex),
        accent: vibrant.hex,
      });
    });
  }, [coverUrl]);

  return colors;
}

// 应用到 Now Playing 背景
function NowPlayingBackground({ coverUrl }: Props) {
  const colors = useDynamicColors(coverUrl);

  return (
    <div
      style={{
        background: `
          radial-gradient(ellipse at 50% 0%, ${colors.primary}40 0%, transparent 70%),
          linear-gradient(180deg, ${colors.background} 0%, #000 100%)
        `,
      }}
    />
  );
}
```

**色彩规范：**
- 主色：从封面提取的最鲜艳颜色
- 背景色：主色降低亮度 60%
- 文字色：根据背景色自动计算对比色（WCAG AA 标准）
- 强度：Now Playing 界面使用强动态色彩，其他页面保持静态

**Spike 验收标准：**
- [ ] 封面切换后 200ms 内完成色彩提取
- [ ] 动态色彩下文字对比度 ≥ 4.5:1（WCAG AA）
- [ ] 色彩过渡动画帧率 ≥ 30fps

---

### 2.4 边框处理

**现状问题：**
- 侧边栏使用 `1px solid var(--color-border)` 实线边框
- MiniPlayer 使用 `1px solid var(--color-border)` 实线边框

**对标 Apple Music：**

macOS 原生应用几乎**不用实线边框**，而是用：
- 背景色差区分区域
- 半透明分隔线（`rgba(0,0,0,0.1)` 或 `rgba(255,255,255,0.1)`）
- 毛玻璃效果自然分隔

**改进方案：**

```css
/* 侧边栏：移除实线边框，用背景色差 + 毛玻璃自然分隔 */
.sidebar {
  /* 移除: border-right: 1px solid var(--color-border); */
  background: color-mix(in srgb, var(--color-sidebar) 50%, transparent);
  backdrop-filter: blur(40px) saturate(180%);
}

/* MiniPlayer：移除实线边框，用顶部渐变分隔 */
.mini-player {
  /* 移除: border-top: 1px solid var(--color-border); */
  position: relative;
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
}

/* 暗色主题 */
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

---

### 2.5 信息密度

**现状问题：**
- 侧边栏 240px（Apple Music 约 200-220px）
- 歌曲行高约 40px（Apple Music 约 32px）
- 间距偏大，Web 化

**对标 macOS 8pt 网格：**

| 元素 | 当前 | 目标 | 说明 |
|---|---|---|---|
| 侧边栏宽度 | 240px | 220px | 收窄 20px |
| 歌曲行高 | ~40px | 34px | 收紧行高 |
| 封面网格间距 | 16px | 12px | 更紧凑 |
| 内容区内边距 | 24px 28px | 20px 24px | 收紧边距 |
| 页面标题字号 | 24px | 22px | 略小 |
| 页面标题下边距 | 20px | 16px | 收紧 |

**CSS 变量调整：**
```css
:root {
  --sidebar-width: 220px;
  --content-padding: 20px 24px;
  --grid-gap: 12px;
  --track-row-height: 34px;
  --page-header-size: 22px;
  --page-header-margin: 16px;
}
```

---

### 2.6 字体排版精调

**现状问题：**
- 标题 letter-spacing 默认
- Now Playing 标题偏小（22px）
- 行高未显式控制

**改进规范：**

```css
/* 页面标题：紧排 */
.page-header h1 {
  font-size: 22px;
  font-weight: 700;
  letter-spacing: -0.02em;
  line-height: 1.2;
}

/* Now Playing 标题：更大更紧 */
.now-playing__title {
  font-size: 28px; /* 从 22px 提升 */
  font-weight: 700;
  letter-spacing: -0.03em;
  line-height: 1.1;
}

/* Now Playing 副标题 */
.now-playing__subtitle {
  font-size: 16px; /* 从 15px 提升 */
  letter-spacing: -0.01em;
}

/* 歌曲表格：紧凑行高 */
.track-table td {
  font-size: 13px;
  line-height: 1.3;
  padding: 6px 12px; /* 从 8px 12px 收紧 */
}

/* 侧边栏标题：Apple Music 风格大写 */
.sidebar__heading {
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--color-text-secondary);
}
```

---

## 3. 保留的原创特色

### 3.1 鼠标跟踪光效

AppShell 的鼠标跟踪径向渐变是原创设计，Apple Music 没有此效果。**保留**，但可考虑：
- 在 Now Playing 界面时降低强度（避免与动态色彩冲突）
- 提供设置选项让用户关闭

### 3.2 Dopamine 空状态

动画丰富的空状态是原创设计。**保留**，但考虑：
- 在用户第二次看到同一空状态时降低动画强度（从 `full` 降为 `subtle`）
- 避免动画过于复杂导致性能问题

---

## 4. 实施优先级

| 优先级 | 改进项 | 影响范围 | 工作量 | 依赖 |
|---|---|---|---|---|
| P0 | Track Table 交互 | 高频交互 | 中 | 无 |
| P0 | 边框处理 | 全局视觉 | 小 | 无 |
| P1 | 信息密度调整 | 全局布局 | 小 | 无 |
| P1 | 字体排版精调 | 全局视觉 | 小 | 无 |
| P2 | Now Playing 展开动画 | 单页面 | 中 | 无 |
| P3 | 页面切换动画 | 全局 | 小 | 无 |
| P3 | 进度条弹性反馈 | 单组件 | 小 | 无 |
| Spike | 动态色彩系统 | Now Playing | 大 | 无 |

> **说明：** Now Playing 展开动画（P2）与动态色彩系统（Spike）完全独立，可并行开发。动态色彩作为 Spike 验证，不阻塞主线交付。

---

## 5. 验收标准

### 5.1 Track Table 验收

| 指标 | 可测标准 | 验证方法 |
|---|---|---|
| 行高 | `--track-row-height: 34px`，实测 ≤ 36px | DevTools 计算样式 |
| hover 延迟 | hover 后 150ms 内背景色变化 | `transition: background 0.15s`，DevTools 检查 |
| hover 背景色 | 浅色: `rgba(0,0,0,0.04)`，暗色: `rgba(255,255,255,0.06)` | 截图取色 |
| 正在播放指示条 | 宽度 3px，颜色 `#fa233e`，左侧定位 | DevTools 检查 `::before` 伪元素 |
| 正在播放标题色 | `color: var(--color-apple-red)` | DevTools 计算样式 |
| aria-current | 正在播放行 `aria-current="true"`，其他行无此属性 | `screen.getByRole('row', { current: true })` |
| 编号→图标切换 | hover 时 `.track-row__number--text` display:none，`--icon` display:block | 测试断言 + CSS 检查 |
| 播放按钮浮现 | hover 时 opacity 从 0 变为 1，过渡 150ms | DevTools 检查 `transition: opacity 0.15s` |

### 5.2 动效验收

| 指标 | 可测标准 | 验证方法 |
|---|---|---|
| Now Playing 展开时长 | 300ms | Framer Motion `transition={{ duration: 0.3 }}` |
| 缓动曲线 | `cubic-bezier(0.32, 0.72, 0, 1)` | DevTools 动画面板 |
| 封面 layoutId | MiniPlayer 和 NowPlaying 共享 `layoutId="now-playing-cover"` | 代码审查 |
| 背景淡入 | 200ms，与封面放大并行 | DevTools 动画面板 |
| 页面切换动画 | 进入 250ms，退出 200ms | CSS 动画时长检查 |
| 进度条 hover | 滑块 `scale(1.3)`，轨道高度 4px→6px | DevTools 计算样式 |
| 进度条 active | 滑块 `scale(1.5)` | DevTools 计算样式 |
| 动画属性限制 | 仅使用 `transform` 和 `opacity`，禁止触发布局重排 | Lighthouse 性能审计 |

### 5.3 布局验收

| 指标 | 可测标准 | 验证方法 |
|---|---|---|
| 侧边栏宽度 | `--sidebar-width: 220px`，实测 ≤ 224px | DevTools 计算样式 |
| 内容区内边距 | `padding: 20px 24px` | DevTools 计算样式 |
| 封面网格间距 | `gap: 12px` | DevTools 计算样式 |
| 页面标题字号 | `font-size: 22px` | DevTools 计算样式 |
| 页面标题下边距 | `margin-bottom: 16px` | DevTools 计算样式 |
| 侧边栏无实线边框 | 无 `border-right` 属性 | DevTools 检查 |
| MiniPlayer 无实线边框 | 无 `border-top` 属性，使用 `::before` 渐变分隔 | DevTools 检查 |

### 5.4 字体验收

| 指标 | 可测标准 | 验证方法 |
|---|---|---|
| Now Playing 标题 | `font-size: 28px`, `letter-spacing: -0.03em` | DevTools 计算样式 |
| Now Playing 副标题 | `font-size: 16px`, `letter-spacing: -0.01em` | DevTools 计算样式 |
| 歌曲表格行高 | `line-height: 1.3` | DevTools 计算样式 |
| 侧边栏标题 | `text-transform: uppercase`, `letter-spacing: 0.06em` | DevTools 计算样式 |

### 5.5 技术验收

| 指标 | 可测标准 | 验证方法 |
|---|---|---|
| 现有测试 | 100% 通过 | `pnpm test` |
| 新增测试 | hover、双击、aria-current 有对应测试 | 测试覆盖率报告 |
| 动画帧率 | ≥ 58fps（P95） | Chrome DevTools Performance 面板 |
| 暗色主题 | 所有改进项在 `[data-theme="dark"]` 下正常 | 手动切换主题验证 |
| 动画属性 | 仅 `transform`/`opacity`，无 `width`/`height`/`top`/`left` | 代码审查 + Lighthouse |

---

## 6. 风险与缓解

| 风险 | 描述 | 缓解 |
|---|---|---|
| 动态色彩性能 | extract-colors 在封面切换时可能卡顿 | 使用 Web Worker + 缓存 |
| 动画性能 | 多层动画可能导致掉帧 | 使用 `will-change` + GPU 加速 |
| 兼容性 | backdrop-filter 在部分浏览器不支持 | 提供 fallback 背景色 |
| 过度设计 | 改进项过多导致开发周期过长 | 严格按优先级实施，P0 先行 |

---

_本文档基于前端设计审查结果，对标 Apple Music 实际界面，结合 Karpathy 的"简洁优先"和"手术式修改"原则编写。_
