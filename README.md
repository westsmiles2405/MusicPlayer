# MusicPlayer

Apple Music 风格的桌面音乐播放器，技术栈 Tauri 2.x + React 19 + Rust。

## 功能特性

- **本地音乐扫描** — 支持 mp3/m4a/flac/wav/aac，递归扫描 + 元数据提取 + 封面缓存
- **资料库浏览** — 最近添加 / 歌曲 / 专辑 / 艺人，React Query 缓存
- **播放控制** — 播放/暂停/上下首/进度/音量，Ringbuf 背压 + Gapless 无缝播放
- **播放队列** — 确定性顺序/随机/循环/单曲循环
- **收藏系统** — 喜欢的歌曲标记 + 收藏页面
- **最近播放** — 播放历史记录
- **用户播放列表** — CRUD + 拖拽排序
- **macOS 系统集成** — Now Playing 媒体栏 / 控制中心 / 媒体键
- **毛玻璃 UI** — backdrop-filter 实现的 Apple Music 风格界面

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面壳 | Tauri 2.x |
| 前端 | React 19 + TypeScript + Vite |
| 样式 | Tailwind CSS v4 + 自写 Apple Music 风组件 |
| 动画 | Framer Motion |
| 状态 | Zustand + TanStack Query |
| 音频 | Rust symphonia + cpal |
| 数据库 | SQLite (rusqlite) |
| 元数据 | Rust lofty |

## 安装

### 前置要求

- Node.js >= 20
- pnpm 9+
- Rust 1.79+
- Xcode Command Line Tools (macOS)

### 安装依赖

```bash
pnpm install
```

## 开发

### 桌面应用 (Tauri)

```bash
source ~/.cargo/env
pnpm tauri dev
```

### 仅 Web 端

```bash
pnpm dev
```

访问 http://localhost:1420/

## 构建

### 开发构建

```bash
pnpm tauri build --debug --bundles app
```

### 生产构建

```bash
pnpm tauri build
```

### 通用二进制 (Intel + ARM)

```bash
pnpm tauri build --target universal-apple-darwin
```

## 测试

### 前端测试

```bash
pnpm test
```

### Rust 测试

```bash
CARGO_TARGET_DIR=/tmp/musicplayer-target cargo test --manifest-path src-tauri/Cargo.toml
```

### Lint

```bash
pnpm lint
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

## 项目结构

```
MusicPlayer/
├── src/                        # React 前端
│   ├── pages/                  # 页面组件
│   ├── components/             # UI 组件
│   ├── stores/                 # Zustand 状态
│   ├── hooks/                  # 自定义 Hooks
│   └── repositories/           # Tauri 命令封装
├── src-tauri/                  # Rust 核心
│   ├── src/commands/           # Tauri 命令
│   ├── src/player/             # 音频引擎
│   ├── src/library/            # 库扫描
│   ├── src/db/                 # SQLite 数据库
│   └── migrations/             # 数据库迁移
└── docs/                       # 设计文档
```

## 架构原则

- 音频流不过 IPC — Rust 直接输出 PCM 到音频设备
- 库扫描在后台线程 — 前端通过事件订阅进度
- Repository 解耦层 — 数据查询统一走 `src/repositories/`
- SQLite WAL 模式 — 扫描写入和前端读取互不阻塞

## 版本历史

- **v0.8.0** — UI 升级 + 收藏 + 封面 + 播放修复
- **v0.7.0** — Now Playing 精修
- **v0.6.0** — 全局搜索 + 收藏 + 最近播放
- **v0.5.0** — 资料库浏览 + 播放列表
- **v0.4.0** — 音频引擎 + Gapless 播放
- **v0.3.0** — 库扫描器

## License

MIT
