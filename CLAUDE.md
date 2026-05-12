# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

基于 Tauri 2 的桌面应用，用于管理个人媒体库（小说、EPUB 电子书、音乐、B 站音频），集成本地 AI 功能。前端使用 SvelteKit（SPA 模式，不支持 SSR），后端使用 Rust，数据库使用 SQLite。

## 技术栈

- **前端**: SvelteKit 2.x + Svelte 5.x, TypeScript, Tailwind CSS 4.x
- **后端**: Rust with Tauri 2.x, tokio 异步运行时, sqlx 数据库操作
- **数据库**: SQLite（sqlx 迁移，编译时查询验证）
- **包管理器**: bun
- **AI**: Ollama（qwen2.5:7b-instruct 用于对话，nomic-embed-text 用于向量嵌入）

## 开发命令

```bash
# 完整 Tauri 开发（Rust 编译 + 前端）
bun run tauri:dev

# 仅前端（Vite 开发服务器 localhost:1420）
bun run dev

# TypeScript 类型检查
bun run check

# Rust 测试（必须在 src-tauri/ 目录下运行）
cd src-tauri && cargo test
cd src-tauri && cargo test <测试名>

# Rust 代码检查
cd src-tauri && cargo clippy

# 生产构建
bun run tauri:build
```

## 架构设计

### 前后端通信

所有前后端通信通过 Tauri 的 `invoke()` 系统：

1. TypeScript 服务函数（`src/lib/services/`）调用 `invoke('command_name', { args })`
2. Rust 的 `#[tauri::command]` 处理器（`src-tauri/src/modules/*/commands.rs`）接收调用
3. 命令在 `src-tauri/src/lib.rs` 的 `invoke_handler![]` 中注册

**参数命名规则**: Rust 命令使用 `snake_case`，Tauri 自动转换为 `camelCase`。TypeScript 的 `invoke()` 调用必须使用 `camelCase` 参数名。

### 后端模块系统

```
src-tauri/src/
├── core/                  # 核心基础设施
│   ├── library.rs         # 多库 CRUD（Library 实体，每个模块的当前库）
│   ├── commands.rs        # 库管理的 Tauri 命令
│   ├── traits.rs          # LibraryModule, Searchable, Categorizable, AIEnhanced trait
│   ├── models.rs          # Workspace, Category, LibraryItem
│   └── workspace.rs       # 工作空间路径管理
├── modules/
│   ├── novel/             # TXT 小说阅读器（导入、解析、章节检测、编码检测）
│   ├── epub/              # EPUB 阅读器（导入、元数据提取、封面处理）
│   ├── music/             # 音乐播放器（文件夹扫描、元数据、rodio 播放）
│   ├── bilibili/          # B 站音频下载（QR 登录、WBI 签名、yt-dlp 下载）
│   └── ai/                # Ollama 集成（对话、摘要、向量搜索、函数调用）
├── database.rs            # 迁移初始化
├── errors.rs              # AppError 枚举（thiserror 派生，序列化为带标签的 JSON）
└── lib.rs                 # Tauri 应用设置、命令注册、数据库连接池初始化
```

每个内容模块（novel/epub/music/bilibili）遵循相同结构：
- `commands.rs` — `#[tauri::command]` 处理器
- `database.rs` — sqlx 查询
- `models.rs` — 领域结构体（Serialize/Deserialize）
- `mod.rs` — 模块导出

bilibili 模块额外包含：
- `api.rs` — B 站 HTTP API 封装（QR 登录、UP 主信息、视频列表、Cookie 刷新）
- `wbi.rs` — WBI 签名算法（置换表混淆 + MD5）
- `cookie.rs` — Cookie 管理（Netscape 格式转换、登录 URL 解析）
- `downloader.rs` — yt-dlp 进程管理（下载执行、进度解析、取消支持）

### 前端结构

```
src/
├── routes/
│   ├── +page.svelte           # 首页/仪表板
│   ├── novel/+page.svelte     # 小说库
│   ├── epub/+page.svelte      # EPUB 库
│   ├── music/+page.svelte     # 音乐库
│   ├── bilibili/+page.svelte  # B 站音频下载
│   └── reader/
│       ├── [bookId]/          # 小说阅读器
│       └── epub/[bookId]/     # EPUB 阅读器
├── lib/
│   ├── services/              # Tauri invoke 封装（每个模块一个 + library.ts）
│   ├── components/            # 共享 + 模块专用组件（epub/, ai/）
│   ├── stores/                # Svelte stores（novel.ts, workspace.ts）
│   ├── types/                 # TypeScript 类型（library.ts, epub.ts, music.ts, bilibili.ts）
│   ├── types.ts               # 小说类型 + 重新导出
│   └── data/                  # 静态数据（分类、来源站点）
```

### 多库架构

每个内容模块都支持多个库，关键模式：

- `libraries` 表存储所有库，使用 `module_type` 区分类型（novel/epub/music/bilibili）
- `library_config` 表追踪每个模块类型的当前活跃库
- 所有内容表（books, tracks, categories）都有 `library_id` 外键
- 前端所有 API 调用的第一个参数传递 `libraryId`
- 数据库触发器自动维护聚合统计（track_count, album_count）
- 级联删除：删除库时自动清理所有子数据
- 首次运行时自动创建三个默认库

### 数据库

- **开发环境**: `./library.db` 在项目根目录（从 `src-tauri/..` 解析）
- **生产环境**: 应用数据目录（`$APPDATA/com.nothingbut.library/`）
- 迁移文件在 `src-tauri/migrations/`（14 个文件，`0001_core.sql` 到 `0014_bilibili_settings.sql`）
- 启动时通过 `sqlx::migrate!()` 自动运行
- 连接池：最大 5 个连接
- URL 格式：`sqlite:<path>?mode=rwc`

### 错误处理

Rust 错误使用 `AppError` 枚举（`errors.rs`），通过 `thiserror` 派生并以带标签的 JSON 序列化。所有命令处理器返回 `Result<T, AppError>`。前端收到的错误格式为 `{ type: "...", message: "..." }`。

## 关键约定

- SPA 模式强制：`+layout.ts` 中 `export const ssr = false`，`adapter-static` 配合 `fallback: "index.html"`
- Vite 开发服务器：`localhost:1420`（严格端口），HMR 端口 `1421`
- Tauri asset protocol 启用 `$APPDATA/**`，用于访问本地文件（封面等）
- 小说 TXT 解析器通过 `encoding_rs` 自动检测编码（GB18030, UTF-8, GBK）
- 音乐播放使用 `rodio`，全局单例播放器（在 setup 中初始化）
- AI 助手使用 Ollama 函数调用来导航库和控制播放
- B 站模块依赖外部工具 `yt-dlp` 进行音频下载，Cookie 通过 Netscape 格式文件传递
- B 站 API 需要 WBI 签名（`wbi.rs`），密钥缓存 30 分钟自动刷新
- bilibili_accounts 表全局共享（不关联 library_id），uploaders/videos/downloads 关联 library_id

## 添加新模块

1. 创建 `src-tauri/src/modules/<模块名>/`，包含 `mod.rs`, `models.rs`, `commands.rs`, `database.rs`
2. 在 `src-tauri/migrations/` 添加迁移文件，内容表须包含 `library_id` 列
3. 在 `src-tauri/src/lib.rs` 的 `invoke_handler![]` 注册命令
4. 在 `src-tauri/src/modules/mod.rs` 添加 `pub mod <模块名>;`
5. 在 `src/lib/types/<模块名>.ts` 创建 TypeScript 类型
6. 在 `src/lib/services/<模块名>.ts` 创建 invoke 封装
7. 在 `src/routes/<模块名>/+page.svelte` 创建路由页面
