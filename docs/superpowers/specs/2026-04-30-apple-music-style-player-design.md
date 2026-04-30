# Apple Music 风格本地音乐播放器 · 设计文档

- 创建日期：2026-04-30
- 项目代号：MusicPlayer
- 文档状态：Draft v1.1（已完成首轮审阅修订，待最终确认 → writing-plans 出实施计划）
- 调研依据：`调研文档/Apple Music 风格音乐播放器分析报告.pdf`

---

## 1. 项目愿景与定位

打造一款**Apple Music 风格的桌面音乐播放器**，目标平台：

- **桌面端 macOS**（首发，目标 Mac App Store 上架）
- **Web 端 Demo**（部署到 GitHub Pages，公开访问；功能弱于 macOS 原生版）
- **Windows 桌面端**（v1.x 后增加，非 v1.0 范围）

项目性质：**个人作品集起步 → 演进为公开发布的产品**。GitHub 仓库公开（MIT License）。

### 1.1 三阶段路线（与 PDF 调研报告对齐）

| 阶段 | 版本 | 内容 | 后端 | DRM |
|---|---|---|---|---|
| **Phase 1** | v1.0 | 本地音乐播放器（用户从本地导入音频文件） | 无 | 无 |
| **Phase 2** | v2.0 | 自建轻量后端 + 自有/CC 内容曲库 + 云同步收藏 | Node/Rust + PostgreSQL + S3 | 无 |
| **Phase 3** | v3.0 | 完整流媒体平台 OR 第三方平台客户端（按 Phase 2 真实流量决定方向） | 完整 | 视方向 |

本文档**仅细化 Phase 1**。Phase 2/3 仅作为长期路线参考，避免 Phase 1 的设计决策把未来的可能性锁死。

### 1.2 非目标（Non-Goals）

明确不做的事情：

- ❌ 不做正版商业音乐授权与分发（无 DRM、无许可链）
- ❌ 不做完整流媒体后端（PDF 中的 Kafka / OpenSearch / CDN 体系延后）
- ❌ 不做移动端原生 App（Phase 3 视情况）
- ❌ 不做 Touch Bar、桌面歌词浮窗等小众特性
- ❌ 不要求 Web 端与 macOS 原生版功能完全一致（Phase 1 Web 仅作为作品集 Demo）

---

## 2. Phase 1 (v1.0) 功能清单

### 2.1 必做（推荐基线 12 项）

1. 本地音乐扫描（mp3 / m4a / flac / wav / aac，递归读取 ID3/iTunes 标签）
2. 资料库浏览（侧边栏：资料库 → 最近添加 / 艺人 / 专辑 / 歌曲）
3. 专辑 / 艺人详情页（封面 + 曲目 + 封面取色背景）
4. 用户播放列表（创建 / 编辑 / 拖拽排序 / 删除）
5. 播放控制（播放 / 暂停 / 上下首 / 进度条 / 音量 / 静音）
6. 播放队列（即将播放、历史、随机、循环：单曲 / 列表）
7. 底部 Mini Player（全局常驻，点击展开 Now Playing）
8. Now Playing 大屏（大封面 + 进度条 + 控制；背景跟随封面取色 + 模糊）
9. 全局搜索（跨歌曲 / 专辑 / 艺人 / 播放列表，FTS5 加速）
10. 最近播放 + 收藏（喜欢的歌曲）
11. macOS 系统集成（Now Playing 媒体栏 / 控制中心 / 键盘媒体键 / AirPods 自动暂停）
12. 毛玻璃 UI（侧边栏、Now Playing 背景、Mini Player 底栏使用 macOS 系统材质）

### 2.2 v1.0 加选

- **D. Gapless 无缝播放**（双缓冲）
- **E. 键盘快捷键**（空格暂停、⌘F 搜索等）
- **I. i18n 框架**（v1.0 仅中文，英文 v1.1 补全）
- **J. 主题跟随系统**（深色 / 浅色不做手动切换）

其中 Gapless 与 macOS 系统媒体集成属于 v1.0 亮点，但实现风险高。实施计划中需要先安排 spike 验证：若 `symphonia + cpal` 双缓冲、`MPNowPlayingInfoCenter` 桥接或媒体键监听在早期验证不稳定，则降级为 v1.1，不阻塞基础播放器 v1.0 发布。

### 2.3 推迟到 v1.1+

- 均衡器 EQ、智能播放列表、iTunes XML 导入、音频可视化、桌面歌词浮窗、外部歌词抓取

---

## 3. 技术栈

### 3.1 客户端

| 层 | 选择 | 备注 |
|---|---|---|
| 桌面壳 | **Tauri 2.x** | Rust 内核，体积小（DMG < 20MB），未来支持 iOS/Android |
| 前端框架 | **React 19 + TypeScript（strict）** | |
| 构建 | **Vite** | Tauri 默认 |
| 样式 | **Tailwind CSS v4** | |
| UI primitives | **Radix UI + 自写 Apple Music 风组件** | 不用 shadcn 默认皮肤，做差异化 |
| 动画 | **Framer Motion** + 少量 CSS | |
| 状态管理 | **Zustand** | UI 状态 |
| 服务端状态 | **TanStack Query** | 缓存/失效，Phase 2 切 HTTP API 不动 UI |
| 路由 | **React Router v7** | |
| i18n | **i18next + react-i18next** | |
| 图标 | **Lucide** + 必要时 SF Symbols | |

### 3.2 音频与数据

| 关注点 | macOS 原生版 | Web Demo |
|---|---|---|
| 解码 / 播放引擎 | **Rust 侧 `symphonia` + `cpal`**（Tauri 命令暴露给前端） | Web Audio API / `<audio>` fallback |
| 元数据读取 | **Rust 侧 `lofty`**（ID3 / FLAC Vorbis / MP4 Atom） | 浏览器侧轻量解析；必要时只支持基础标签 |
| 文件系统访问 | Tauri dialog + macOS security-scoped bookmark | File System API（不支持时降级为单文件导入） |
| 文件系统监听 | Rust 侧 **`notify`** crate | 不做持久监听，用户手动重新导入 |
| 本地数据库 | **SQLite** via `rusqlite`（WAL 模式） | IndexedDB（仅 Demo 缓存） |
| 全文搜索 | **SQLite FTS5** | JS 内存搜索 / IndexedDB 索引 |
| 封面取色 | JS 侧 **`extract-colors`** | 同 macOS 前端实现 |

Phase 1 的产品验收以 macOS 原生版为准。Web Demo 只复用 React UI 与部分 repository 接口，不承诺 Gapless、系统媒体集成、文件夹监听或大曲库性能。

### 3.3 工程

| 项 | 选择 |
|---|---|
| License | **MIT** |
| 仓库结构 | 单仓库，`src/`（前端）+ `src-tauri/`（Rust） |
| 包管理 | **pnpm** |
| Lint / Format | ESLint + Prettier + cargo fmt + cargo clippy |
| 测试 | Vitest（前端单元）+ cargo test（Rust）+ Playwright（仅冒烟级，可推迟） |
| CI | GitHub Actions（PR 检查 + tag 发布） |
| 文档语言 | 注释 / Issue / PR 中文为主，README 双语 |

---

## 4. 系统架构

### 4.1 顶层架构

macOS 原生版是单进程双语言：Tauri 2.x 应用 = WebView（前端） + Rust 核心，通过 Tauri IPC（commands + events）通信。

```
┌─────────────────────────────────────────────────────┐
│  Tauri 2.x App                                       │
│   ┌──────────────────┐    Tauri IPC    ┌──────────┐ │
│   │  WebView (React) │ ◄──Commands───► │ Rust 核心 │ │
│   │                  │ ◄──Events──────  │          │ │
│   └──────────────────┘                  └──────────┘ │
└─────────────────────────────────────────────────────┘
                                 │
                                 ▼
                  app_data_dir() / Application Support
                    - SQLite db (WAL)
                    - cache/covers/<hash>.jpg
                    - settings
```

### 4.2 前后端职责

| 层 | 职责 | 不做 |
|---|---|---|
| **React 前端** | UI 渲染、路由、交互、动画、毛玻璃、UI 状态、TanStack Query 缓存 | 不直接读音频文件、不操作 SQLite、不解码音频 |
| **Rust 核心** | 音频解码 & 播放、库扫描、元数据、SQLite 读写、文件监听、Now Playing 系统集成 | 不渲染 UI、不做颜色提取、不管 i18n |
| **Tauri IPC** | 命令调用 + 事件流 | 不做大数据流（PCM 不走 IPC） |

### 4.3 关键架构决策

1. **音频流不过 IPC**：macOS 原生版中，解码后 PCM 由 Rust 通过 `cpal` 直接写入系统音频设备；前端只订阅"位置/状态变化"事件。这样可以控制队列、预解码、Gapless 与系统媒体集成。Web Demo 使用浏览器音频能力，体验降级。
2. **库索引在 Rust 后台线程**：扫描 IO+CPU 双密集，必须后台 worker，前端用进度事件订阅。
3. **SQLite + WAL**：让"扫描写入"和"前端读取"互不阻塞。
4. **数据存放**：统一通过 Tauri `app_data_dir()` 获取。非沙盒构建通常落在 `~/Library/Application Support/<bundle-id>/`；Mac App Store 沙盒构建落在容器内的 Application Support。
5. **路由层级浅**：所有页面在主窗口切换；Now Playing 是 overlay（从 Mini Player 上拉），不是单独路由。
6. **`repositories/` 解耦层**：UI 只依赖 repository 接口。macOS adapter 调 Tauri 命令；Web Demo adapter 调浏览器 API；Phase 2 可切到 HTTP API。

---

## 5. 模块拆分

### 5.1 前端（`src/`）

```
src/
├── pages/            # Library / Albums / AlbumDetail / Artists / ArtistDetail
│                     # / Playlists / PlaylistDetail / Search / Settings
├── components/
│   ├── layout/       # AppShell / Sidebar / TopBar
│   ├── player/       # MiniPlayer / NowPlayingScreen / Queue
│   ├── library/      # AlbumCard / TrackRow / ArtistTile / CoverArt
│   ├── ui/           # Button / Slider / ContextMenu / Tooltip
│   └── effects/      # GlassPanel / VibrancyView / ColorTintBackground
├── stores/           # Zustand：playerStore / uiStore / prefsStore
├── hooks/            # usePlayer / useScanProgress / useColorTint
├── repositories/     # trackRepo / albumRepo / artistRepo / playlistRepo / playerRepo
├── lib/              # 通用工具
├── i18n/             # zh.json（v1.0 必需）/ en.json（v1.1）
└── styles/           # Tailwind config + 全局 CSS
```

### 5.2 Rust 核心（`src-tauri/src/`）

```
src-tauri/src/
├── main.rs
├── commands/
│   ├── player.rs       # play / pause / seek / next / prev / set_queue
│   ├── library.rs      # scan_folder / get_albums / get_tracks / search
│   ├── playlist.rs     # CRUD + reorder
│   └── prefs.rs
├── player/
│   ├── engine.rs       # symphonia 解码 + cpal 输出
│   ├── queue.rs        # 队列 / 历史 / 随机 / 循环
│   ├── gapless.rs      # 双缓冲
│   └── state.rs        # 状态机
├── library/
│   ├── scanner.rs      # 递归遍历
│   ├── watcher.rs      # notify
│   └── indexer.rs      # 增量索引
├── metadata/
│   ├── reader.rs       # lofty
│   └── art.rs          # 封面提取 + 缓存
├── db/
│   ├── schema.rs       # 迁移
│   ├── tracks.rs / albums.rs / artists.rs / playlists.rs
├── system/
│   └── now_playing.rs  # MPNowPlayingInfoCenter 桥接
└── error.rs            # 统一错误类型
```

---

## 6. 数据模型（SQLite）

### 6.1 表结构

```sql
-- 艺人
CREATE TABLE artists (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  added_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

-- 专辑
CREATE TABLE albums (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  album_artist_id INTEGER NOT NULL REFERENCES artists(id),
  year INTEGER,
  cover_path TEXT,
  added_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  UNIQUE(name, album_artist_id)
);

-- 曲目（核心表）
CREATE TABLE tracks (
  id INTEGER PRIMARY KEY,
  file_path TEXT NOT NULL UNIQUE,
  file_size INTEGER NOT NULL,
  file_modified_at INTEGER NOT NULL,
  hash TEXT,                              -- xxhash 首尾 64KB
  title TEXT NOT NULL,
  album_id INTEGER REFERENCES albums(id),
  primary_artist_id INTEGER REFERENCES artists(id),
  album_artist_id INTEGER REFERENCES artists(id),
  track_no INTEGER,
  disc_no INTEGER,
  year INTEGER,
  genre TEXT,
  duration_ms INTEGER NOT NULL,
  bitrate INTEGER,
  sample_rate INTEGER,
  channels INTEGER,
  codec TEXT,
  is_favorite INTEGER NOT NULL DEFAULT 0,
  play_count INTEGER NOT NULL DEFAULT 0,
  last_played_at INTEGER,
  last_seen_at INTEGER NOT NULL,           -- 最近一次扫描仍然存在
  missing_at INTEGER,                      -- 文件缺失时软删除，不破坏播放列表/历史
  added_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX idx_tracks_album ON tracks(album_id);
CREATE INDEX idx_tracks_primary_artist ON tracks(primary_artist_id);
CREATE INDEX idx_tracks_title ON tracks(title);
CREATE INDEX idx_tracks_missing ON tracks(missing_at);

-- 曲目-艺人多对多（含 role：main / featured / composer 等）
CREATE TABLE track_artists (
  track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  artist_id INTEGER NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
  role TEXT NOT NULL DEFAULT 'main',
  position INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (track_id, artist_id, role)
);

-- 用户播放列表
CREATE TABLE playlists (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  cover_path TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE playlist_tracks (
  playlist_id INTEGER NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
  track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  position INTEGER NOT NULL,
  added_at INTEGER NOT NULL,
  PRIMARY KEY (playlist_id, track_id, position)
);
CREATE INDEX idx_playlist_tracks ON playlist_tracks(playlist_id, position);

-- 播放历史
CREATE TABLE play_history (
  id INTEGER PRIMARY KEY,
  track_id INTEGER NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
  played_at INTEGER NOT NULL,
  duration_played_ms INTEGER NOT NULL,
  completed INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_play_history_played_at ON play_history(played_at DESC);

-- 监听文件夹
CREATE TABLE scan_folders (
  id INTEGER PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  bookmark_data BLOB,                     -- macOS 沙盒持久访问用 security-scoped bookmark
  added_at INTEGER NOT NULL,
  last_scanned_at INTEGER
);

-- 杂项 KV
CREATE TABLE app_state (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at INTEGER NOT NULL
);

-- 全文搜索
CREATE VIEW tracks_search_view AS
SELECT
  tracks.id AS id,
  tracks.title AS title,
  albums.name AS album_name,
  artists.name AS artist_name
FROM tracks
LEFT JOIN albums ON albums.id = tracks.album_id
LEFT JOIN artists ON artists.id = tracks.primary_artist_id
WHERE tracks.missing_at IS NULL;

CREATE VIRTUAL TABLE tracks_fts USING fts5(
  title, album_name, artist_name,
  content='tracks_search_view',
  content_rowid='id',
  tokenize='unicode61 remove_diacritics 2'
);

-- 迁移时建立 tracks / albums / artists 相关 triggers 保持 tracks_fts 与 tracks_search_view 一致；
-- 大规模扫描或批量元数据修正后可执行 INSERT INTO tracks_fts(tracks_fts) VALUES('rebuild') 重建索引。

-- 迁移记录
CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY,
  applied_at INTEGER NOT NULL
);
```

### 6.2 关键约束

- `tracks.file_path` UNIQUE：业务主键
- `(albums.name, album_artist_id)` UNIQUE：去重规则；缺失专辑艺人统一指向内置 `Unknown Artist`，避免 SQLite 中 `NULL` 破坏唯一性
- `tracks.hash`：用于文件被移动 / 重命名时识别同一首歌，保留收藏与播放次数
- `tracks.missing_at`：文件消失时软删除；播放列表、收藏、播放历史不被级联破坏
- `tracks.primary_artist_id`：主艺人缓存字段，用于列表/搜索排序；完整艺人关系以 `track_artists` 为准
- `play_history.completed`：>= 95% 时长视为听完，决定 `play_count` 累加

### 6.3 迁移策略

`db/migrations/V1__init.sql`、`V2__xxx.sql` 顺序执行；启动时按 `schema_migrations` 表已应用版本号增量执行。

---

## 7. 数据流

### 7.1 添加音乐文件夹

1. 前端调 `add_folder(path)`
2. Rust 写 `scan_folders` 表；macOS 沙盒构建同时保存 security-scoped bookmark，起后台扫描任务
3. 每解析 50 首 emit `scan_progress` 事件 → 前端进度条
4. 解析完写 `tracks/albums/artists/track_artists`，提取嵌入封面到 `cache/covers/<hash>.jpg`
5. 扫描完成后更新 `tracks_fts`（增量 triggers 或 rebuild）并 emit `scan_done` → 前端 TanStack Query 失效缓存 → UI 刷新

### 7.2 播放一首歌

1. 用户双击 → 前端调 `player_play(track_id, queue, index)`
2. Rust 取 `file_path`，symphonia 解码，cpal 播放
3. 100ms 定时器 emit `playback_progress` 事件
4. 剩 1 秒预解码下一首 → Gapless 切换 → emit `track_changed`
5. 完整播放写入 `play_history`，`tracks.play_count++`

### 7.3 文件夹监听（增量更新）

1. notify crate 检测到变化 → 防抖 2 秒
2. 对比 `file_path + hash + mtime`：新文件 INSERT / 修改 UPDATE / 消失文件设置 `missing_at`
3. emit `library_changed` → 前端刷新

### 7.4 搜索

1. 输入框 debounce 200ms → `search(query)`
2. Rust 查 `tracks_fts`，只返回 `missing_at IS NULL` 的曲目 top 50（按相关度）
3. 前端按"歌曲 / 专辑 / 艺人"分组渲染

---

## 8. 错误处理

| 严重度 | 表现 | 例子 | 处理 |
|---|---|---|---|
| 轻微 | log + 静默 | 单首歌封面解析失败 / 元数据缺失 | 占位封面 / 文件名兜底 |
| 中等 | toast 提示 | 播放失败、文件夹无权限 | 顶部 toast，提供"重试 / 跳过" |
| 严重 | 模态阻塞 | SQLite 损坏 / 迁移失败 | 模态弹窗，提供"备份 db / 重置库 / 提交问题" |

统一错误类型 `src-tauri/src/error.rs::AppError`，序列化经 IPC 传到前端，由全局 ErrorBoundary + TanStack Query 统一展示。

---

## 9. 测试策略

| 层 | 工具 | 覆盖 | 目标 |
|---|---|---|---|
| Rust 单元 | `cargo test` | 元数据、队列、SQL 查询 | 核心 80%+ |
| Rust 集成 | `cargo test --test integration` | DB 迁移、扫描+查询全链路 | 关键路径 100% |
| 前端单元 | Vitest | repositories / hooks / stores | 70%+ |
| 前端组件 | Vitest + RTL | Sidebar / MiniPlayer / TrackRow | 关键组件 100% |
| E2E | Playwright + Tauri WebDriver（可选，推迟） | 核心 5 流冒烟 | — |
| 手测 | Markdown checklist | 空库 / 大库（10k 首）/ 边扫边播 / 锁屏后台播放等 | 100% |

不做：100% 覆盖率追求、快照测试、视觉回归测试。

---

## 10. 项目结构

详见 §5。补充：

```
.github/
├── workflows/
│   ├── ci.yml         # PR：lint + test + build (debug)
│   ├── release.yml    # tag：build DMG + Web，发 Release
│   └── pages.yml      # 部署 Web 到 GitHub Pages
├── ISSUE_TEMPLATE/
│   ├── bug_report.md
│   ├── feature_request.md
│   └── question.md
└── PULL_REQUEST_TEMPLATE.md
```

---

## 11. 工程规范

| 项 | 规则 |
|---|---|
| 分支 | Trunk-based：`main` + `feat/*` / `fix/*` 短分支 |
| 保护 | `main` 必须 PR + CI 全绿 |
| 提交格式 | Conventional Commits（`feat:`, `fix:`, `refactor:` 等） |
| 版本号 | SemVer：v0.x 开发期 → v1.0.0 首发 |
| 发版 | `git tag vX.Y.Z && git push --tags` 触发 release.yml |
| 注释 | 仅写"为什么"，不写"做什么" |
| TS | strict 模式，函数式组件 + 自定义 hooks |
| Rust | 模块按领域切（player / library / metadata / db / system） |

---

## 12. CI / CD

### 12.1 `ci.yml`（PR 触发）

- macos-latest，pnpm + cargo 缓存
- 并行 jobs：`lint-frontend` / `lint-rust` / `test-frontend` / `test-rust` / `build`（debug）
- 预期 3–5 分钟

### 12.2 `release.yml`（tag 触发）

Phase 1 发布拆成三条路径，避免把 App Store、直接分发和 Web Demo 混在同一个 job：

1. **直接下载版（Developer ID）**
   - `tauri build --target universal-apple-darwin`（Intel + ARM 通用包）
   - 产出 `.dmg` + `.app.tar.gz` + `.sig`
   - 使用 Developer ID Application 证书签名并 notarize
   - 创建 GitHub Release（draft），上传产物

2. **Mac App Store 版（Apple Distribution）**
   - Phase 1 末期再接入，手动触发
   - 独立 entitlements：App Sandbox、User Selected Files / Music Folder、security-scoped bookmarks
   - 可用 fastlane 或 Xcode 工具链上传 App Store Connect

3. **Web Demo**
   - `vite build` → 部署到 GitHub Pages / `gh-pages` 分支
   - 明确标注为 Demo，不展示原生版不可用的功能入口

---

## 13. README 结构

中英双语，包含：

- Hero 截图 / 录屏 GIF
- 项目简介 + 标语
- 特性列表
- 下载安装（DMG / Web 体验链接）
- 从源码构建（macOS 13+ / Xcode CLT / Node ≥ 20 / pnpm 9+ / Rust 1.79+）
- Roadmap（v1.0 / v2.0 / v3.0）
- 贡献指南（CONTRIBUTING.md 链接）
- License (MIT)

---

## 14. 风险与开放问题

| 风险 | 描述 | 缓解 |
|---|---|---|
| Mac App Store 沙盒 | 沙盒下需要用 NSOpenPanel 取得文件夹权限，之后用 security-scoped bookmark 持久化 | 在文件选择对话框层处理；写在 Rust 侧 macOS-specific 模块 |
| 代码签名 / 公证 / App Store | 直接分发与 Mac App Store 使用不同证书、entitlements 与上传流程 | release job 拆分为 Developer ID、Mac App Store、Web Demo 三条路径 |
| Tauri 媒体 API 成熟度 | macOS Now Playing 系统集成需 objc 桥接，社区 crate 不一定齐全 | 必要时手写 FFI（`objc2` crate） |
| FLAC / ALAC 解码兼容性 | symphonia 对部分 FLAC 变体可能不支持 | 手测覆盖常见编码组合，问题 fallback 用 ffmpeg-rs（Phase 1.x） |
| FTS5 一致性 | tracks 与 FTS 索引可能因扫描/删除不同步 | 使用 external content + triggers；大规模扫描后支持 rebuild |
| Web 端文件与音频能力 | Web 端不能用 cpal、notify、SQLite，浏览器文件夹能力也有兼容差异 | Web Demo 独立 adapter，隐藏不支持的功能入口，必要时降级为单文件导入 |
| Tauri 2.x 文档 / 生态 | 2.x 仍在快速迭代 | 锁版本，关注 changelog；CI 矩阵不上 nightly |

---

## 15. 后续步骤

1. ✅ 设计文档首版完成
2. ✅ 首轮审阅修订（Web 边界、发布链路、SQLite/FTS、软删除）
3. ⏭️ 用户最终确认
4. ⏭️ 调用 `superpowers:writing-plans` 出可执行的分阶段实施计划
5. ⏭️ 按 plan 进入 `executing-plans` / `subagent-driven-development`

---

_本设计基于 `调研文档/Apple Music 风格音乐播放器分析报告.pdf` 调研，核心技术选型在调研建议基础上根据"桌面优先 + 个人作品集起步"做了调整：移动端 Flutter 推迟至 Phase 3，桌面采用 Tauri + React，并通过 repository adapter 区分 macOS 原生版与 Web Demo。_
