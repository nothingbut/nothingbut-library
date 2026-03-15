# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

基于 Tauri 2 的桌面应用程序，用于管理和阅读小说。前端使用 SvelteKit（SPA 模式，非 SSR），后端使用 Rust，数据库使用 SQLite。

## 技术栈

- **前端**: SvelteKit 2.x + Svelte 5.x, TypeScript, Tailwind CSS 4.x
- **后端**: Rust with Tauri 2.x, tokio 异步运行时, sqlx 数据库操作
- **数据库**: SQLite (sqlx migrations)
- **包管理器**: bun

## 开发命令

```bash
# 前端开发（仅 Vite 开发服务器）
bun run dev

# Tauri 完整开发（包含 Rust 编译 + 前端）
bun run tauri:dev

# 类型检查
bun run check
bun run check:watch

# 生产构建
bun run build          # 仅前端
bun run tauri:build    # 完整 Tauri 应用

# Rust 相关
cd src-tauri
cargo test             # 运行所有测试
cargo test <测试名>     # 运行特定测试
cargo build            # 构建 Rust 后端
cargo clippy           # 代码检查
```

## 架构设计

### 前端架构（SvelteKit SPA）

- **SPA 模式**: 使用 `adapter-static` + `fallback: "index.html"`，因为 Tauri 不支持 SSR
- **路由**: 基于文件的路由系统，位于 `src/routes/`
- **状态管理**: Svelte stores，位于 `src/lib/stores/`
- **组件**: 可复用组件在 `src/lib/components/`
- **API 层**: 通过 `src/lib/services/api.ts` 使用 Tauri 的 `invoke()` 与后端通信

### 后端架构（Rust/Tauri）

后端采用模块化架构：

```
src-tauri/src/
├── core/              # 核心基础设施
│   ├── config.rs      # 应用配置
│   ├── models.rs      # 共享模型（Workspace, Category, LibraryItem）
│   ├── traits.rs      # 共享 trait（LibraryModule, Searchable, AIEnhanced, Categorizable）
│   └── workspace.rs   # 工作空间管理系统
├── modules/           # 功能模块
│   └── novel/         # 小说阅读模块
│       ├── commands.rs    # Tauri 命令处理器
│       ├── database.rs    # 数据库操作
│       ├── models.rs      # 小说相关模型
│       ├── parser.rs      # TXT 文件解析器（自动编码检测）
│       ├── scraper.rs     # 网页爬虫功能
│       ├── storage.rs     # 文件系统操作
│       └── seed.rs        # 数据库种子数据
├── database.rs        # 数据库迁移和初始化
└── errors.rs          # 错误类型定义
```

#### 关键架构模式

1. **工作空间系统**: 应用支持多个工作空间，每个工作空间有独立的配置和模块选择。工作空间是位于可配置路径的隔离数据容器。

2. **基于模块的设计**: 功能按模块组织（当前有 `novel` 模块）。每个模块包含：
   - 命令处理器（暴露给前端）
   - 数据库操作
   - 领域模型
   - 业务逻辑

3. **基于 Trait 的可扩展性**: 核心 trait 定义标准能力：
   - `LibraryModule`: 所有模块的基础接口
   - `Searchable`: 全文搜索能力
   - `Categorizable`: 分类层级支持
   - `AIEnhanced`: AI 辅助功能

4. **数据库策略**:
   - 开发环境: 数据库存储在项目根目录（`./library.db`）
   - 生产环境: 数据库存储在应用数据目录
   - 迁移文件在 `src-tauri/migrations/`，启动时自动执行

5. **测试**: Rust 测试与模块代码放在一起，使用 `#[cfg(test)]` 和 `#[test]` 属性。没有单独的 `tests/` 目录。

### 前后端通信

- 使用 Tauri 的 `invoke()` 系统调用命令
- 命令在 `src-tauri/src/lib.rs` 中通过 `invoke_handler![]` 注册
- TypeScript 类型（`src/lib/types.ts`）应与 Rust 模型匹配

## 重要配置细节

### Vite 配置
- 开发服务器: `localhost:1420`
- HMR 端口: `localhost:1421`
- 严格端口模式（端口不可用时失败）
- 忽略 `src-tauri/` 目录的文件监听

### 数据库
- SQLite + sqlx（编译时查询验证）
- 应用启动时自动运行迁移
- 连接池: 最大 5 个连接
- 数据库 URL 格式: `sqlite:<path>?mode=rwc`

### Tauri 插件
- `tauri-plugin-dialog`: 文件/文件夹选择对话框
- `tauri-plugin-sql`: SQLite 数据库访问
- `tauri-plugin-opener`: 打开外部链接/文件

## 模块开发指南

添加新模块时：

1. 在 `src-tauri/src/modules/<模块名>/` 创建模块目录
2. 实现必需文件: `mod.rs`, `models.rs`, `commands.rs`, `database.rs`
3. 在 `src-tauri/src/lib.rs` 中注册命令
4. 添加数据库迁移到 `src-tauri/migrations/`
5. 在 `src/lib/types.ts` 中创建对应的 TypeScript 类型
6. 在 `src/lib/services/api.ts` 中实现 API 封装

## 特殊功能

### 小说模块功能
- **TXT 解析器**: 使用 `encoding_rs` 自动检测编码（GB18030, UTF-8, GBK）
- **章节检测**: 基于正则表达式的章节标题检测
- **网页爬虫**: 使用 `reqwest`、`scraper` 和 `headless_chrome` 从小说网站抓取元数据
- **文件存储**: 书籍存储在工作空间中，包含元数据 JSON 和独立的章节文件
- **阅读进度**: 追踪最后阅读位置和章节

## 测试

- Rust: 在 `src-tauri/` 目录运行 `cargo test`
- TypeScript: 运行 `bun run check` 进行类型检查
- 测试使用 `#[cfg(test)]` 与实现文件内联
