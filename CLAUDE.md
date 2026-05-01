# MusicPlayer · Claude Code 项目上下文

> 这个文件是给 Claude Code 看的，每次会话自动加载。请保持简洁、高信号。

## 1. 项目是什么

**Apple Music 风格的桌面音乐播放器**，技术栈 Tauri 2.x + React 19 + Rust。

- **目标平台**：macOS（首发，Mac App Store）+ Web（GitHub Pages）；Windows 推迟到 v1.x。
- **项目性质**：个人作品集 → 演进为公开发布产品（GitHub 公开，MIT License）。
- **当前版本**：v0.6.0（全局搜索 + 收藏 + 最近播放已交付）→ v0.7.0（毛玻璃 UI + 键盘快捷键）。
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
│   ├── pages/                  # SearchPage / FavoritesPage / RecentPlaysPage / LibraryPage / SettingsPage（路由 App.tsx）
│   ├── components/{layout,player,library,ui,effects}/
│   │   └── layout/ScanProgressBar.tsx   # 全局扫描进度条
│   ├── stores/                 # Zustand: playerStore / uiStore / prefsStore
│   ├── hooks/                  # usePlayer (track_changed 时 invalidate recentPlays) / useScanProgress / useToggleFavoriteMutation / useDebouncedValue
│   ├── repositories/           # ⚠️ 关键解耦层（Phase 2 切 HTTP 时只改这里）
│   │   └── playerRepo.ts      # player_* / searchRepo / favoriteRepo / recentPlayRepo
│   ├── lib/  i18n/  styles/
│
├── src-tauri/                  # Rust 核心
│   ├── tauri.conf.json
│   ├── Cargo.toml
│   ├── migrations/             # V1__init.sql / V2__fix_fts5_triggers.sql / V3__scanner_support.sql
│   ├── tests/
│   │   ├── scan_e2e.rs         # 端到端扫描测试（5 个 fixture）
│   │   └── fixtures/audio/     # ffmpeg 生成的 1 秒静音 mp3/flac/m4a/wav/no-tag
│   └── src/
│       ├── main.rs
│       ├── commands/           # player_* / library_* / playlist_* / prefs_*
│       ├── player/             # ✅ v0.4.0：engine（ringbuf+cpal）/ decoder（symphonia）/ manager（DB解析+事件桥）/ queue（确定性洗牌）/ gapless（后台预解码）/ state（双层命令+会话）
│       ├── library/            # scanner（walk→reader→indexer 管道）/ watcher（v0.3.1）
│       ├── metadata/           # reader（lofty + xxh3） / art（封面缓存，内容 hash 去重）
│       ├── db/                 # tracks（含 list_favorite_tracks）/ albums / artists / playlists（含 search_by_name）/ scan_folders / search（含 search_all 分组搜索）/ play_history（含 list_recent_played_tracks 去重）/ schema
│       ├── system/now_playing.rs
│       └── error.rs            # AppError（含 Scan / Busy / Metadata / NotFound）
│
├── .worktrees/                 # git worktree（.gitignore 已排除）
├── docs/superpowers/specs/     # 设计文档：Apple Music 风格 spec / v0.3.0 scanner spec
├── .github/workflows/          # ci.yml / release.yml / pages.yml
└── CLAUDE.md  README.md  LICENSE
```

## 5. 关键架构原则（不可违反）

1. **音频流不过 IPC**。解码后 PCM 由 Rust 通过 cpal 直接输出到系统音频设备；前端只通过事件订阅播放进度。
2. **库扫描在 Rust 后台线程**。前端通过 `scan_progress` / `scan_done` 事件订阅。
3. **`src/repositories/` 是唯一允许调 Tauri 命令的地方**。`pages/` 和 `components/` 不直接调 IPC。
4. **数据存放**：`~/Library/Application Support/<bundle-id>/`（用 Tauri `app_data_dir()`），Mac App Store 沙盒兼容。
5. **`tracks.file_path` 是业务主键，`tracks.hash` 用于文件移动后的身份保留**。扫描器 B1 快速路径：path 命中 → (mtime,size) 比较 → hash+size 查库（唯一命中则 Moved，多命中降级 Added）。
6. **`Database` 内部用 `Arc<DatabaseInner>`**（`parking_lot::Mutex<Connection>`）。`Database: Clone + Send`，允许跨 `spawn_blocking` 边界持有的 clone 调用 `conn()`。
7. **`tracks.root_folder_id` 实现多根目录隔离**。扫描时通过 `canonicalize_roots` 去重前缀过滤，删除文件夹时用 `mark_missing_by_root` + `unlink_root` 三步事务操作。
8. **错误分三档**：轻微（log）/ 中等（toast）/ 严重（模态阻塞）。统一从 `src-tauri/src/error.rs::AppError` 流到前端 ErrorBoundary。新增 `AppError::Busy` 用于扫描互斥。
9. **i18next 必须在 `src/main.tsx` 顶部 side-effect import**（`import "@/i18n";`），否则被 Vite tree-shake，`useTranslation()` 拿不到资源。
10. **Rust 子模块必须在父 `mod.rs` 里 `pub mod` 显式声明**（`commands/mod.rs` / `db/mod.rs`），否则 `tauri::generate_handler!` 找不到目标。
11. **音频回调不能拿锁**。cpal callback 用 `ringbuf` Consumer 的 `try_lock()`（非阻塞）+ 原子变量读音量/静音；若 try_lock 失败直接写静音。Engine 线程持有 Producer 推 PCM。`SharedAudioBuffer::clear()` 用 `lock()` 阻塞清空（仅在 stop/seek 时调用）。
12. **PlayerCommand 双层架构**。Manager 层接收 `PlayerCommand`（裸 track ID），解析 DB 后转发 `EngineCommand`（含已解析 `EngineTrack`）给 Engine。Engine 永远不碰 DB。
13. **Buffer 背压模型**。`push_samples()` 返回实际写入的样本数；position 仅按已写入样本推进。ringbuf 满时未写入的 chunk 尾部存入 `pending_samples`，下个 tick 优先写入。`flush_pending()` 后若仍有残留 → 跳过 decoder 本 tick。
14. **Gapless 只在失效路径 cancel**。`finish_session(Stop/Replaced/DecodeError/OutputError/Shutdown)` 取消预解码；`Completed/Next/Previous` 保留结果，供 `advance_next()` 队列推进后消费。
15. **播放列表允许重复 track ID**。`playlist_tracks` 表支持同一 `track_id` 多次出现。前端计算 queue index 时不能用 `indexOf(id)`（总是返回第一次出现），必须按行在数组中的实际位置做 occurrence-aware 映射。
16. **收藏乐观更新必须覆盖所有 track-bearing 缓存**。`useToggleFavoriteMutation` 的 `onMutate` 需要同时 patch `favoriteTracks`、`tracks`、`search`、`albumTracks`、`artistTracks`、`recentPlays`；仅在 `onSuccess` 后 `invalidateQueries` 不够——用户会看到"点不上"的延迟。点击行需有 pending 态（disabled + "..."），防止重复点击。

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
| TS 配置 | **不用** project references — 单 `tsconfig.json` 即可，`tsconfig.node.json` 只给 vite.config.ts 类型用 |
| 文档 / Issue / PR / 注释语言 | 中文为主。README、ISSUE_TEMPLATE 双语 |

## 7. v1.0 必做功能（不要走偏）

1. ~~本地音乐扫描~~ ✅ v0.3.0（mp3/m4a/flac/wav/aac，递归 + lofty 标签 + xxh3 hash + 封面缓存）
2. ~~资料库浏览~~ ✅ v0.5.0（侧边栏：最近添加 / 歌曲 / 专辑 / 艺人，React Query + TrackTable）
3. ~~专辑 / 艺人详情页~~ ✅ v0.5.0（详情页含播放按钮，加入播放列表菜单）
4. ~~用户播放列表~~ ✅ v0.5.0（CRUD 对话框 + 拖拽排序 + 缺失曲目处理）
5. ~~播放控制~~ ✅ v0.4.0（播放/暂停/上下首/进度/音量，ringbuf 背压 + pending buffer）
6. ~~播放队列~~ ✅ v0.4.0（确定性顺序/随机/循环/单曲循环，PlayQueue）
7. ~~底部 Mini Player~~ ✅ v0.4.0（传输控件/进度条/音量/错误显示）
8. ~~全局搜索（FTS5）~~ ✅ v0.6.0（分组搜索：歌曲/专辑/艺人/播放列表，250ms debounce）
9. ~~最近播放 + 收藏~~ ✅ v0.6.0（收藏乐观更新 + pending 反馈，播放后自动刷新最近播放）
10. ~~macOS Now Playing 系统集成~~ ✅ v0.4.0（MPNowPlayingInfoCenter + MPRemoteCommandCenter，媒体键/控制中心）
11. 毛玻璃 UI（系统材质 NSVisualEffectView，Web 端用 backdrop-filter 兜底）
12. ~~**Gapless 无缝播放**~~ ✅ v0.4.0（后台预解码下一首首个 chunk，<1s 剩余时触发）
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
CARGO_TARGET_DIR=/tmp/musicplayer-target cargo test --manifest-path src-tauri/Cargo.toml   # Rust 测试
pnpm lint                    # ESLint + Prettier --check
cargo fmt --manifest-path src-tauri/Cargo.toml --check
CARGO_TARGET_DIR=/tmp/musicplayer-target cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# 前端独立验证（不跑 Rust，秒级）
pnpm build                   # tsc + vite build；只想验前端配置/类型时用这个

# 出包
CARGO_TARGET_DIR=/tmp/musicplayer-target pnpm tauri build --debug --bundles app   # 仅出 .app
pnpm tauri build                         # 本地出 DMG（需签名配置）
pnpm tauri build --target universal-apple-darwin   # Intel + ARM 通用

# Rust 部分验证（不跑前端）
CARGO_TARGET_DIR=/tmp/musicplayer-target cargo test --manifest-path src-tauri/Cargo.toml
CARGO_TARGET_DIR=/tmp/musicplayer-target cargo check --manifest-path src-tauri/Cargo.toml
```

> **`CARGO_TARGET_DIR=/tmp/musicplayer-target` 是必须的**：仓库本地 `src-tauri/target` 曾出现重复产物（`libmuda-bb064cc9d7a039bb 2.rmeta`），统一用 `/tmp` 路径避免损坏。

> **已知现象**：未配置代码签名时 `pnpm tauri build` 的 DMG 步骤会失败但 `.app` 已生成 — 是预期，不是 bug。

## 9. 关键风险（开发时留意）

- **Mac App Store 沙盒**：访问音乐文件夹需用户授权 + security-scoped bookmark；写在 Rust 侧 macOS-specific 模块。
- **macOS Now Playing 集成**：✅ 已通过 objc2 0.6 + objc2-media-player 0.3 + block2 0.6 实现。注意：`msg_send!` 参数间需要逗号（objc2 0.6 语法），`NSMutableDictionary::insert` 的 `CopyingHelper` 约束对 `&NSString` 不满足（用 `msg_send!` 绕开），`RcBlock` 不是 `Send`（丢弃 `addTargetWithHandler` 返回值即可，MPRemoteCommandCenter 内部 retain）。
- **FLAC / ALAC 兼容性**：symphonia 对部分变体支持有限，必要时 fallback 到 ffmpeg-rs（Phase 1.x）。
- **Web 端音频**：用 Web Audio API 兜底，体验弱化（无 Gapless / 无低延迟）。
- **Vitest `act()` 警告**：当组件（如 MiniPlayer）通过事件监听触发异步状态更新时，测试中 `render()` 需包裹在 `await act(async () => {...})` 内，且 `invoke` mock 需按命令名返回不同数据（不能用单一 `mockResolvedValue`），否则 React Query flush 时会拿到错误的返回值类型。
- **对话框状态重置**：用 `useState(initialProp)` 初始化的受控组件，在父组件保持挂载的情况下关闭再打开不会重置状态。必须加 `useEffect` 同步 `open` 和 `initialProp` 变化。

## 10. Claude 工作约定

- 任何"为何这样设计"的问题，先看本文件 + spec。
- 提议改架构 / 加依赖前，明确说出影响哪些已有模块。
- 写代码前先看 `repositories/` 是否已有抽象——避免在 `pages/` 里直接调 Tauri 命令。
- 测试覆盖关键路径，**不追求 100%**；不写快照测试。
- 用户偏好简洁回复 + 不输出冗长总结。
- **功能开发用 git worktree**：`git worktree add .worktrees/<feature> -b feat/<feature>`，隔离 `main`。`.worktrees/` 已在 `.gitignore` 排除。
- **验收前全部门禁必须过**：`cargo test + clippy + fmt` 和 `pnpm test + build + lint`。`pnpm tauri build --debug --bundles app` 验证 `.app` 生成。

## 11. 文档参考

- 设计文档：`docs/superpowers/specs/2026-04-30-apple-music-style-player-design.md`
- v0.3.0 scanner spec：`docs/superpowers/specs/2026-05-01-v0.3.0-library-scanner-design.md`
- v0.3.0 实施计划：`docs/superpowers/plans/2026-05-01-v0.3.0-library-scanner.md`
- v0.4.0 实施计划：`docs/superpowers/plans/2026-05-01-v0.4.0-audio-engine.md`
- v0.5.0 实施计划：`docs/superpowers/plans/2026-05-01-v0.5.0-library-playlists.md`
- v0.6.0 实施计划：`docs/superpowers/plans/2026-05-02-v0.6.0-search-favorites-history.md`
- 调研报告：`Apple Music 风格音乐播放器分析报告.pdf`
- Tauri 2 文档：https://tauri.app/start/
- React 19：https://react.dev/
- symphonia：https://github.com/pdeljanov/Symphonia
- lofty：https://github.com/Serial-ATA/lofty-rs
- TanStack Query：https://tanstack.com/query/latest
