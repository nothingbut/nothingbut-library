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

### AI 助手功能（2026-03-29）
- **自然语言交互**: 使用 Ollama + qwen2.5:7b-instruct 模型
- **Function Calling**: 支持工具调用实现库导航和内容控制
- **可用工具**:
  - 库管理: 列出库、获取当前库、切换库
  - 书籍操作: 搜索书籍、打开阅读器
  - 音乐操作: 搜索歌曲、播放音乐
- **浮动 UI**: 右下角浮动按钮，随时唤起
- **详细文档**: 参见 `AI_ASSISTANT_GUIDE.md`

## 测试

- Rust: 在 `src-tauri/` 目录运行 `cargo test`
- TypeScript: 运行 `bun run check` 进行类型检查
- 测试使用 `#[cfg(test)]` 与实现文件内联

## 多库架构实现进度

**更新时间:** 2026-03-27 17:15

### ✅ Phase 1: Novel 模块前端（已完成）

**完成内容:**
1. ✅ 修改 `src/lib/services/api.ts`
   - `listBooks(libraryId)` - 添加 libraryId 参数
   - `importNovel(libraryId, ...)` - 添加 libraryId 参数（第一个参数）
   - `deleteBook(libraryId, ...)` - 添加 libraryId 参数

2. ✅ 修改 `src/lib/components/ImportDialog.svelte`
   - 添加 `libraryId` prop
   - `checkDuplicates()` - 传递 libraryId 给 listBooks
   - `handleImport()` - 传递 libraryId 给 importNovel

3. ✅ 修改 `src/routes/novel/+page.svelte`
   - `handleBookSelect()` - 添加库检查，传递 libraryId 给 listBooks
   - `openImportDialog()` - 添加库检查
   - `<ImportDialog>` - 传递 currentLibrary.id

4. ✅ 修改后端 `src-tauri/src/modules/novel/commands.rs`
   - `delete_book()` - 添加 libraryId 参数

5. ✅ **Bug 修复** - `src-tauri/src/core/library.rs`
   - 修复 `get_current_library()` 函数的类型转换问题
   - 问题：`library_config.value` 字段在数据库中是 TEXT 类型，代码尝试直接获取为 i64
   - 解决：先尝试获取 i64，失败则尝试获取 String 并解析为 i64
   - 这修复了"请先选择一个库"的错误提示

**验证:**
- ✅ Rust 编译成功（`cargo build`）
- ✅ 数据库类型转换修复（TEXT → i64）
- ⏳ UI 功能测试待完成

6. ✅ **Bug 修复** - CategoryTree 组件库参数传递
   - 修复 `src/lib/components/CategoryTree.svelte`
   - 添加 `libraryId` prop
   - `listBooks(libraryId)` - 传递 libraryId 参数
   - `deleteBook(libraryId, ...)` - 传递 libraryId 参数
   - 修复 `src/routes/novel/+page.svelte` 传递 libraryId 给 CategoryTree
   - 这修复了左侧栏"Failed to load data"的错误

7. ✅ **分类树库级隔离** - `novel_categories` 表添加 library_id
   - 创建迁移文件 `0011_add_library_id_to_categories.sql`
   - 给 novel_categories 表添加 library_id 字段和 UNIQUE 约束
   - 修改 `src-tauri/src/modules/novel/models.rs` - NovelCategory 添加 library_id 字段
   - 修改 `src-tauri/src/modules/novel/database.rs`:
     - `insert_category(library_id, ...)` - 添加 library_id 参数
     - `list_categories(library_id)` - 添加 library_id 过滤
   - 修改 `src-tauri/src/modules/novel/commands.rs`:
     - `create_category(libraryId, ...)` - 添加 libraryId 参数
     - `list_categories(libraryId)` - 添加 libraryId 参数
     - `seed_categories(libraryId, ...)` - 添加 libraryId 参数
   - 修改 `src-tauri/src/modules/novel/seed.rs`:
     - `seed_categories_from_config(library_id, ...)` - 添加 library_id 参数
     - 更新所有测试
   - 修改 `src/lib/services/api.ts`:
     - `createCategory(libraryId, ...)` - 添加 libraryId 参数
     - `listCategories(libraryId)` - 添加 libraryId 参数
   - 修改 `src/lib/types.ts` - Category 接口添加 library_id 字段
   - 修改 `src/lib/components/CategoryTree.svelte` - 传递 libraryId 给 listCategories
   - 修改 `src/lib/components/ImportDialog.svelte` - 传递 libraryId 给分类相关操作
   - 修改 `src/routes/novel/+page.svelte` - 传递 libraryId 给 listCategories

**已知问题:**
- ⚠️ **Music 模块未实施多库支持** - 这是 Phase 3 的工作
  - 数据库表有 library_id 字段
  - 但 Rust 模型、命令、查询都未实施
  - 导入的音乐无法按库隔离
  - 需要完整 Phase 3 实施（预计 3-4 小时）

### ✅ Phase 2: EPUB 模块（已完成）

**更新时间:** 2026-03-27

**完成内容:**
1. ✅ 修改 `src-tauri/src/modules/epub/database.rs`
   - `create_book(library_id, ...)` - 添加 library_id 参数
   - `list_books(library_id)` - 添加 WHERE library_id = ? 过滤
   - `search_books(library_id, ...)` - 添加 library_id 过滤

2. ✅ 修改 `src-tauri/src/modules/epub/commands.rs`
   - `import_epub(library_id, ...)` - 添加 library_id 参数
   - `batch_import_epub(library_id, ...)` - 添加 library_id 参数
   - `list_epub_books_with_details(library_id)` - 添加 library_id 参数
   - `search_epub_books(library_id, ...)` - 添加 library_id 参数

3. ✅ 修改 `src/lib/services/epub.ts`
   - `importEpub(libraryId, ...)` - 添加 libraryId 参数
   - `batchImportEpub(libraryId, ...)` - 添加 libraryId 参数
   - `listBooksWithDetails(libraryId)` - 添加 libraryId 参数

4. ✅ 修改 `src/lib/components/epub/EpubLibrary.svelte`
   - 添加库状态管理（libraries, currentLibrary）
   - 集成 getCurrentLibrary 和 listLibraries
   - 添加库选择器 UI 和创建库对话框
   - 添加 handleLibrarySwitch 和 handleCreateLibrary 函数
   - 传递 libraryId 给所有 API 调用

5. ✅ 修改 `src/lib/components/epub/EpubImportDialog.svelte`
   - 添加 `libraryId` prop
   - 修改 importEpub 和 batchImportEpub 调用传递 libraryId

6. ✅ 修改 `src/lib/components/LibraryGrid.svelte`
   - 添加 `libraryId` prop
   - 修改 loadBooks 调用传递 libraryId

**验证:**
- ✅ TypeScript 类型检查通过（0 errors）
- ⏳ UI 功能测试待完成

### ✅ Phase 3: Music 模块（已完成）

**更新时间:** 2026-03-27

**完成内容:**
1. ✅ 修改 `src-tauri/src/modules/music/commands.rs`
   - `scan_music_folder(library_id, ...)` - 添加 library_id 参数
   - `get_all_tracks(library_id)` - 添加 library_id 参数
   - `get_tracks_by_artist(library_id, ...)` - 添加 library_id 参数
   - `get_tracks_by_album(library_id, ...)` - 添加 library_id 参数
   - `get_tracks_by_genre(library_id, ...)` - 添加 library_id 参数
   - `search_tracks(library_id, ...)` - 添加 library_id 参数
   - `get_all_artists(library_id)` - 添加 library_id 参数
   - `get_all_albums(library_id)` - 添加 library_id 参数
   - `process_audio_file(library_id, ...)` - 添加 library_id 参数并传递给数据库函数

2. ✅ 修改 `src/lib/services/music.ts`
   - `scanMusicFolder(libraryId, ...)` - 添加 libraryId 参数
   - `getAllTracks(libraryId)` - 添加 libraryId 参数
   - `getTracksByArtist(libraryId, ...)` - 添加 libraryId 参数
   - `getTracksByAlbum(libraryId, ...)` - 添加 libraryId 参数
   - `getTracksByGenre(libraryId, ...)` - 添加 libraryId 参数
   - `searchTracks(libraryId, ...)` - 添加 libraryId 参数
   - `getAllArtists(libraryId)` - 添加 libraryId 参数
   - `getAllAlbums(libraryId)` - 添加 libraryId 参数

3. ✅ 修改 `src/routes/music/+page.svelte`
   - 添加库检查逻辑（loadLibrary 和 scanFolder）
   - 传递 libraryId 给所有 API 调用
   - 库选择器和创建库对话框已存在

**验证:**
- ✅ Rust 编译成功（`cargo build`）
- ✅ TypeScript 类型检查通过（0 errors）

### 📊 总体进度

- ✅ 数据库迁移（0010, 0011, 0012）
- ✅ Novel 模块后端
- ✅ Novel 模块前端
- ✅ EPUB 模块后端
- ✅ EPUB 模块前端
- ✅ Music 模块后端
- ✅ Music 模块前端
- ✅ 迁移修复（触发器和列名）
- ✅ 集成测试 ← **全部完成！**

**完成时间:** 2026-03-27
- Phase 3 实施: 约 30 分钟
- 迁移修复: 约 45 分钟
- 集成测试: 约 15 分钟
**总用时:** 约 90 分钟

### 🎯 验证结果

**数据库结构:** ✅ 所有表都有 library_id 列和正确的约束
**触发器:** ✅ 6 个触发器全部创建成功
**迁移历史:** ✅ 12 个迁移全部成功应用
**应用启动:** ✅ 无错误启动
**默认库:** ✅ 自动创建 3 个默认库（novel, epub, music）

### 📝 相关文档

- `PHASE3_MUSIC_COMPLETE.md` - Music 模块实施详情
- `MIGRATION_FIXES_SUMMARY.md` - 迁移修复总结和最佳实践
- `test-multi-library.sh` - 集成测试脚本

### ✨ 多库架构特性

1. **库级数据隔离** - 每个模块可以有多个独立的库
2. **自动统计** - 触发器自动维护 track_count, album_count 等统计
3. **级联删除** - 删除库时自动清理所有相关数据
4. **唯一性约束** - 艺术家、专辑、播放列表在库级别保证唯一
