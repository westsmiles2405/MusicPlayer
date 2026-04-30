# MusicPlayer · Claude Code 项目上下文

> 这个文件是给 Claude Code 看的，每次会话自动加载。请保持简洁、高信号。

## 1. 项目是什么

**Apple Music 风格的桌面音乐播放器**，技术栈 Tauri 2.x + React 19 + Rust。

- **目标平台**：macOS（首发，Mac App Store）+ Web（GitHub Pages）；Windows 推迟到 v1.x。
- **项目性质**：个人作品集 → 演进为公开发布产品（GitHub 公开，MIT License）。
- **当前阶段**：Phase 1 / v1.0 — **本地音乐播放器**（无后端、无 DRM）。
- **完整设计**：见 `docs/superpowers/specs/2026-04-30-apple-music-style-player-design.md`。
- **背景调研**：`Apple Music 风格音乐播放器分析报告.pdf`（项目根，14 页）。

## 2. 三阶段路线（避免被 Phase 1 决策锁死未来）

| 阶段 | 版本 | 内容 | 后端 | DRM |
|---|---|---|---|---|
| **Phase 1** | v1.0 | 本地音乐播放器 | 无 | 无 |
| **Phase 2** | v2.0 | 自建后端 + 自有/CC 内容 + 云同步收藏 | Node/Rust + PostgreSQL + S3 | 无 |
| **Phase 3** | v3.0 | 完整流媒体 OR 第三方平台客户端 | 视方向 | 视方向 |

**关键解耦点**：所有数据查询走 `src/repositories/` 抽象层，Phase 2 切到 HTTP API 时只改这一层。

## 3. 技术栈速查

### 客户端

| 层 | 选择 |
|---|---|
| 桌面壳 | Tauri 2.x |
| 前端 | React 19 + TypeScript (strict) + Vite |
| 样式 | Tailwind CSS v4 |
| UI primitives | Radix UI + 自写 Apple Music 风组件（不用 shadcn 默认皮肤） |
| 动画 | Framer Motion + 少量 CSS |
| 状态 | Zustand（UI 状态）+ TanStack Query（服务端状态/缓存） |
| 路由 | React Router v7 |
| i18n | i18next（v1.0 仅 zh，en 推迟到 v1.1） |
| 图标 | Lucide |

### 音频与数据

| 关注点 | 选择 |
|---|---|
| 解码 / 播放 | Rust `symphonia` + `cpal`（PCM 不过 IPC，直接写音频设备） |
| 元数据 | Rust `lofty` |
| 文件监听 | Rust `notify` |
| 数据库 | SQLite via `rusqlite`（WAL 模式） |
| 全文搜索 | SQLite FTS5 |
| 封面取色 | JS `extract-colors` |

### 工程

| 项 | 选择 |
|---|---|
| 包管理 | pnpm |
| Lint / Format | ESLint + Prettier + cargo fmt + cargo clippy |
| 测试 | Vitest（前端）+ cargo test（Rust）+ Playwright（推迟） |
| CI | GitHub Actions |

## 4. 目录速查

```
MusicPlayer/
├── src/                        # React 前端
│   ├── pages/                  # 路由页（Library / Albums / AlbumDetail / ... / Settings）
│   ├── components/{layout,player,library,ui,effects}/
│   ├── stores/                 # Zustand: playerStore / uiStore / prefsStore
│   ├── hooks/                  # usePlayer / useScanProgress / useColorTint
│   ├── repositories/           # ⚠️ 关键解耦层（Phase 2 切 HTTP 时只改这里）
│   ├── lib/  i18n/  styles/
│
├── src-tauri/                  # Rust 核心
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   ├── migrations/             # SQL 迁移文件 V1__init.sql ...
│   └── src/
│       ├── main.rs
│       ├── commands/           # Tauri IPC 命令（player/library/playlist/prefs）
│       ├── player/             # symphonia + cpal + 队列 + Gapless
│       ├── library/            # scanner / watcher / indexer
│       ├── metadata/           # lofty 读取 + 封面缓存
│       ├── db/                 # SQLite schema + 各表查询
│       ├── system/now_playing.rs   # macOS MPNowPlayingInfoCenter 桥接
│       └── error.rs            # 统一 AppError
│
├── docs/superpowers/specs/     # 设计文档（spec）
├── .github/workflows/          # ci.yml / release.yml / pages.yml
└── CLAUDE.md  README.md  LICENSE
```

## 5. 关键架构原则（不可违反）

1. **音频流不过 IPC**。解码后 PCM 由 Rust 通过 cpal 直接输出到系统音频设备；前端只通过事件订阅播放进度。
2. **库扫描在 Rust 后台线程**。前端通过 `scan_progress` / `scan_done` 事件订阅。
3. **`src/repositories/` 是唯一允许调 Tauri 命令的地方**。`pages/` 和 `components/` 不直接调 IPC。
4. **数据存放**：`~/Library/Application Support/<bundle-id>/`（用 Tauri `app_data_dir()`），Mac App Store 沙盒兼容。
5. **`tracks.file_path` 是业务主键，`tracks.hash` 用于文件移动后的身份保留**。
6. **错误分三档**：轻微（log）/ 中等（toast）/ 严重（模态阻塞）。统一从 `src-tauri/src/error.rs::AppError` 流到前端 ErrorBoundary。

## 6. 工程规范

| 项 | 规则 |
|---|---|
| 分支 | Trunk-based：`main` + `feat/*` / `fix/*` 短分支；PR 必走，CI 全绿才合 |
| 提交 | [Conventional Commits](https://www.conventionalcommits.org/)：`feat:` / `fix:` / `refactor:` / `docs:` / `chore:` / `test:` |
| 版本号 | SemVer。v0.x 开发期 → v1.0.0 首发 |
| 发版 | `git tag vX.Y.Z && git push --tags` 触发 `release.yml` |
| 代码注释 | 只写"为什么"。不写"做什么"——代码本身已经说明了 |
| TS | strict；函数式组件 + 自定义 hooks 优先 |
| Rust | 按领域切模块（不要"utils.rs"大杂烩）；用 `thiserror` 定义错误 |
| 文档 / Issue / PR / 注释语言 | 中文为主。README、ISSUE_TEMPLATE 双语 |

## 7. v1.0 必做功能（不要走偏）

1. 本地音乐扫描（mp3/m4a/flac/wav/aac，递归 + ID3/iTunes 标签）
2. 资料库浏览（侧边栏：资料库 → 最近添加 / 艺人 / 专辑 / 歌曲）
3. 专辑 / 艺人详情页（封面取色背景）
4. 用户播放列表（CRUD + 拖拽排序）
5. 播放控制（播放/暂停/上下首/进度/音量）
6. 播放队列（即将播放、历史、随机、循环）
7. 底部 Mini Player + 上拉 Now Playing 大屏
8. 全局搜索（FTS5）
9. 最近播放 + 收藏
10. macOS Now Playing 系统集成（媒体栏 / 控制中心 / 媒体键 / AirPods 暂停）
11. 毛玻璃 UI（系统材质 NSVisualEffectView，Web 端用 backdrop-filter 兜底）
12. **Gapless 无缝播放**（双缓冲，作品集亮点）
13. **键盘快捷键**（空格 / ⌘F / 媒体键）
14. **i18n 框架**（zh 优先）
15. **主题跟随系统**（不做手动切换）

**不做（推迟到 v1.1+）**：均衡器、智能播放列表、iTunes XML 导入、音频可视化、外部歌词抓取、桌面歌词浮窗。

## 8. 常用开发命令（项目搭好后填充）

```bash
# 开发
pnpm tauri dev               # 启动桌面开发
pnpm dev                     # 仅启动 Web 端开发服务

# 测试
pnpm test                    # Vitest 前端单元
cargo test --manifest-path src-tauri/Cargo.toml   # Rust 测试
pnpm lint                    # ESLint + Prettier --check
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# 出包
pnpm tauri build             # 本地出 DMG
pnpm tauri build --target universal-apple-darwin   # Intel + ARM 通用
```

> 项目脚手架尚未生成，以上命令在 v0.1.0 完成后启用。

## 9. 关键风险（开发时留意）

- **Mac App Store 沙盒**：访问音乐文件夹需用户授权 + security-scoped bookmark；写在 Rust 侧 macOS-specific 模块。
- **macOS Now Playing 集成**：需 objc 桥接（`objc2` crate），Tauri 现成 plugin 不一定够用。
- **FLAC / ALAC 兼容性**：symphonia 对部分变体支持有限，必要时 fallback 到 ffmpeg-rs（Phase 1.x）。
- **Web 端音频**：用 Web Audio API 兜底，体验弱化（无 Gapless / 无低延迟）。
- **Tauri 2.x 仍在迭代**：锁版本、关注 changelog。

## 10. Claude 工作约定

- 任何"为何这样设计"的问题，先看本文件 + spec。
- 提议改架构 / 加依赖前，明确说出影响哪些已有模块。
- 写代码前先看 `repositories/` 是否已有抽象——避免在 `pages/` 里直接调 Tauri 命令。
- 测试覆盖关键路径，**不追求 100%**；不写快照测试。
- 用户偏好简洁回复 + 不输出冗长总结。

## 11. 文档参考

- 设计文档：`docs/superpowers/specs/2026-04-30-apple-music-style-player-design.md`
- 调研报告：`Apple Music 风格音乐播放器分析报告.pdf`
- Tauri 2 文档：https://tauri.app/start/
- React 19：https://react.dev/
- symphonia：https://github.com/pdeljanov/Symphonia
- lofty：https://github.com/Serial-ATA/lofty-rs
- TanStack Query：https://tanstack.com/query/latest
