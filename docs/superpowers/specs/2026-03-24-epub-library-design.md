# EPUB 书库功能设计文档

**日期**: 2026-03-24
**状态**: 设计完成，待实施
**预计工期**: 6-8 周（分 4 个阶段）

---

## 目录

1. [项目概述](#项目概述)
2. [需求分析](#需求分析)
3. [技术方案](#技术方案)
4. [架构设计](#架构设计)
5. [数据模型](#数据模型)
6. [核心功能](#核心功能)
7. [用户界面](#用户界面)
8. [AI 功能集成](#ai-功能集成)
9. [实施计划](#实施计划)
10. [验收标准](#验收标准)

---

## 项目概述

### 目标

为 NothingBut Library 添加完整的 EPUB 电子书管理功能，参考 Calibre 但专注于 EPUB 格式，提供从导入、管理、编辑到阅读的完整体验。

### 核心价值

- **专注 EPUB**: 仅支持 EPUB 格式，简化功能，提升体验
- **完整元数据**: Calibre 级别的元数据管理，包括自定义字段
- **增强阅读**: 字体、主题、书签、高亮等现代阅读体验
- **AI 增强**: 复用现有 AI 功能，提供智能摘要和语义搜索
- **独立模块**: 与现有 Novel 模块完全分离，互不影响

### 与现有功能的关系

- **Novel 模块**: TXT 小说管理，保持独立
- **EPUB 模块**: EPUB 电子书管理，新增功能
- **AI 模块**: 共享使用，为两个模块提供 AI 能力

---

## 需求分析

### 功能需求

#### 1. 书库管理
- ✅ 建立独立的 EPUB 书库
- ✅ 导入 EPUB 文件（单个/批量）
- ✅ 元数据编辑（Calibre 级别）
- ✅ 删除书籍
- ✅ 搜索和过滤

#### 2. 元数据管理
- ✅ **基础元数据**: 标题、作者、出版社、ISBN、封面、出版日期、语言、系列、评分、简介
- ✅ **多作者支持**: 一本书可以有多个作者
- ✅ **标签系统**: 多标签支持
- ✅ **自定义字段**: 用户可创建任意类型的自定义字段（文本、日期、数字、评分、是/否、枚举等）

#### 3. 阅读功能
- ✅ EPUB 阅读器
- ✅ 目录导航
- ✅ 翻页功能
- ✅ 字体设置（大小、字体）
- ✅ 主题切换（白天、护眼、夜间）
- ✅ 书签管理
- ✅ 高亮功能
- ✅ 阅读进度保存（章节 + 百分比）

#### 4. 界面展示
- ✅ **三种视图**: 网格视图、列表视图、详细列表视图
- ✅ **侧边栏编辑**: 选中书籍后在右侧展开详情和编辑

#### 5. 搜索功能
- ✅ **多字段搜索**: 标题、作者、出版社、ISBN、标签、系列名
- ✅ **组合过滤**: 支持多条件组合搜索
- ✅ **语义搜索**: AI 驱动的自然语言搜索（Phase 4）

#### 6. AI 功能（Phase 4）
- ✅ AI 对话助手（讨论书籍内容）
- ✅ 智能摘要（章节/全书）
- ✅ 语义搜索（向量检索）
- ✅ 关键词提取（智能标签建议）

### 非功能需求

#### 性能
- 导入速度: 单本书 < 5 秒（包含元数据提取和封面生成）
- 搜索响应: < 500ms（百本规模）
- 阅读器加载: < 2 秒

#### 可用性
- 界面直观，无需学习成本
- 支持拖拽导入
- 实时搜索提示

#### 可扩展性
- 支持数千本书籍
- 模块化设计，易于扩展新功能

---

## 技术方案

### 技术栈

#### 后端
- **语言**: Rust
- **EPUB 解析**: `epub` crate
- **图片处理**: `image` crate
- **数据库**: SQLite + sqlx

#### 前端
- **框架**: SvelteKit 2 + Svelte 5
- **EPUB 渲染**: epub.js（行业标准）
- **样式**: Tailwind CSS 4

#### AI 集成
- **复用现有模块**: `modules/ai/`
- **向量存储**: 纯 Rust 实现（已有）

### 关键技术决策

#### 1. EPUB 解析库选择
**决策**: 使用 `epub` crate

**理由**:
- Rust 原生，性能好
- 支持 EPUB 2/3 标准
- 活跃维护
- API 简单清晰

#### 2. 阅读器选择
**决策**: 使用 epub.js

**理由**:
- 业界标准，功能完善
- 支持 CFI 精确定位
- 响应式布局
- 主题和字体自定义
- 书签、高亮 API 完整

**重要**: 默认尊重出版商样式，仅对正文应用用户设置

#### 3. 存储结构
**决策**: 数据库式存储（方案 D）

```
workspace/epub/books/
  1/
    book.epub
    cover.jpg          (600x800)
    cover_thumb.jpg    (200x267)
    metadata.json      (备份)
  2/
    ...
```

**理由**:
- 每本书独立文件夹，结构清晰
- 支持多种资源（封面、附件等）
- 符合项目现有模式
- 适合大规模书库

#### 4. AI 集成方式
**决策**: 完全复用现有 `modules/ai/`

**理由**:
- 避免重复开发
- 统一体验
- 代码复用率高

---

## 架构设计

### 模块结构

```
src-tauri/src/modules/epub/
├── mod.rs              # 模块入口
├── models.rs           # 数据模型
├── commands.rs         # Tauri 命令
├── database.rs         # 数据库操作
├── parser.rs           # EPUB 解析器
├── metadata.rs         # 元数据提取和管理
├── storage.rs          # 文件存储管理
└── custom_fields.rs    # 自定义字段系统
```

```
src/lib/components/epub/
├── EpubLibrary.svelte       # 书库主界面
├── BookGrid.svelte          # 网格视图
├── BookList.svelte          # 列表视图
├── BookDetailList.svelte    # 详细列表视图
├── BookSidebar.svelte       # 侧边栏编辑
├── MetadataEditor.svelte    # 元数据编辑器
├── CustomFieldEditor.svelte # 自定义字段编辑
├── BookImport.svelte        # 导入界面
├── SearchBar.svelte         # 搜索和过滤
└── reader/
    ├── EpubReader.svelte    # 阅读器主组件
    ├── ReaderToolbar.svelte # 工具栏
    └── ReaderSettings.svelte # 阅读设置
```

### 系统集成

```
┌─────────────────────────────────────────┐
│          NothingBut Library             │
├─────────────────────────────────────────┤
│                                         │
│  ┌──────────┐  ┌──────────┐            │
│  │  Novel   │  │  EPUB    │            │
│  │  Module  │  │  Module  │            │
│  │ (TXT)    │  │ (EPUB)   │            │
│  └────┬─────┘  └────┬─────┘            │
│       │             │                   │
│       └──────┬──────┘                   │
│              │                          │
│       ┌──────▼──────┐                   │
│       │  AI Module  │                   │
│       │  (Shared)   │                   │
│       └─────────────┘                   │
│              │                          │
│       ┌──────▼──────┐                   │
│       │   Ollama    │                   │
│       └─────────────┘                   │
└─────────────────────────────────────────┘
```

### 数据流

#### 导入流程
```
用户选择 EPUB 文件
    ↓
验证文件格式
    ↓
解析元数据（标题、作者、出版社等）
    ↓
提取封面图片
    ↓
创建数据库记录
    ↓
创建存储文件夹 (books/[id]/)
    ↓
复制 EPUB 文件
    ↓
生成封面缩略图（600x800 + 200x267）
    ↓
保存元数据 JSON（备份）
    ↓
更新数据库路径
    ↓
完成导入
```

#### 阅读流程
```
用户打开书籍
    ↓
加载 EPUB 文件
    ↓
初始化 epub.js
    ↓
加载阅读进度（章节 + 位置）
    ↓
加载目录
    ↓
加载书签和高亮
    ↓
应用阅读设置（字体、主题）
    ↓
显示内容
    ↓
监听翻页事件 → 保存进度
    ↓
监听文本选择 → 高亮功能
```

---

## 数据模型

### 核心实体

#### EpubBook（书籍）
```rust
pub struct EpubBook {
    pub id: i64,
    pub title: String,
    pub sort_title: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub pubdate: Option<String>,
    pub language: Option<String>,
    pub series: Option<String>,
    pub series_index: Option<f32>,
    pub rating: Option<i32>,           // 1-5
    pub file_path: String,
    pub file_size: i64,
    pub cover_path: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

#### Author（作者）
```rust
pub struct Author {
    pub id: i64,
    pub name: String,
    pub sort_name: Option<String>,
    pub created_at: String,
}
```

#### Tag（标签）
```rust
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}
```

#### CustomField（自定义字段定义）
```rust
pub struct CustomField {
    pub id: i64,
    pub name: String,
    pub label: String,
    pub datatype: CustomFieldType,
    pub is_multiple: bool,
    pub display_order: i32,
    pub created_at: String,
}

pub enum CustomFieldType {
    Text,           // 文本
    Series,         // 系列
    Enumeration,    // 枚举
    Number,         // 数字
    Rating,         // 评分
    Date,           // 日期
    Bool,           // 是/否
    Comments,       // 长文本
}
```

#### ReadingProgress（阅读进度）
```rust
pub struct ReadingProgress {
    pub book_id: i64,
    pub chapter_href: String,
    pub progress_percent: f32,
    pub updated_at: String,
}
```

#### Bookmark & Highlight（书签和高亮）
```rust
pub struct Bookmark {
    pub id: i64,
    pub book_id: i64,
    pub chapter_href: String,
    pub cfi: String,                // EPUB CFI
    pub note: Option<String>,
    pub created_at: String,
}

pub struct Highlight {
    pub id: i64,
    pub book_id: i64,
    pub chapter_href: String,
    pub cfi_range: String,
    pub text: String,
    pub color: String,
    pub note: Option<String>,
    pub created_at: String,
}
```

### 数据库 Schema

详见 `migrations/0007_epub.sql`:
- `epub_books` - 书籍主表
- `epub_authors` - 作者表
- `epub_book_authors` - 书籍-作者关联（多对多）
- `epub_tags` - 标签表
- `epub_book_tags` - 书籍-标签关联（多对多）
- `epub_custom_fields` - 自定义字段定义
- `epub_custom_field_values` - 自定义字段值
- `epub_reading_progress` - 阅读进度
- `epub_bookmarks` - 书签
- `epub_highlights` - 高亮

### ER 图

```
epub_books (1) ──< (N) epub_book_authors >── (N) epub_authors
    │
    ├──< (N) epub_book_tags >── (N) epub_tags
    │
    ├──< (N) epub_custom_field_values >── (N) epub_custom_fields
    │
    ├── (1) epub_reading_progress
    │
    ├──< (N) epub_bookmarks
    │
    └──< (N) epub_highlights
```

---

## 核心功能

### 1. EPUB 解析

#### 元数据提取
```rust
pub struct EpubMetadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub publisher: Option<String>,
    pub pubdate: Option<String>,
    pub language: Option<String>,
    pub isbn: Option<String>,
    pub description: Option<String>,
}
```

#### 封面提取
- 大图: 600x800 (阅读器显示)
- 缩略图: 200x267 (列表显示)
- 格式: JPEG
- 质量: 85%

#### 目录提取
```rust
pub struct EpubChapter {
    pub href: String,      // 章节路径
    pub title: String,
    pub level: i32,        // 层级
    pub order_index: i32,
}
```

### 2. 导入功能

#### 单个导入
```rust
#[tauri::command]
pub async fn import_epub(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    source_file_path: String,
) -> AppResult<i64>
```

**流程**:
1. 验证文件格式
2. 解析元数据
3. 提取封面
4. 创建数据库记录
5. 创建存储目录
6. 复制 EPUB 文件
7. 生成封面缩略图
8. 保存元数据 JSON
9. 返回书籍 ID

#### 批量导入
```rust
#[tauri::command]
pub async fn batch_import_epub(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    file_paths: Vec<String>,
    window: Window,
) -> AppResult<Vec<ImportResult>>
```

**特性**:
- 支持拖拽多文件
- 实时进度反馈（Tauri Events）
- 错误不中断，继续处理
- 返回成功/失败列表

### 3. 元数据编辑

#### 基础元数据
```rust
#[tauri::command]
pub async fn update_book_metadata(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    metadata: UpdateMetadataRequest,
) -> AppResult<()>
```

#### 作者管理
```rust
#[tauri::command]
pub async fn set_book_authors(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    authors: Vec<AuthorInput>,
) -> AppResult<()>
```

#### 标签管理
```rust
#[tauri::command]
pub async fn set_book_tags(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    tag_names: Vec<String>,
) -> AppResult<()>
```

#### 自定义字段
```rust
#[tauri::command]
pub async fn set_custom_field_value(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    field_id: i64,
    value: String,
) -> AppResult<()>

#[tauri::command]
pub async fn create_custom_field(
    pool: State<'_, SqlitePool>,
    field: CustomFieldInput,
) -> AppResult<i64>
```

### 4. 搜索功能

#### 多字段搜索
```rust
pub struct SearchQuery {
    pub keyword: Option<String>,        // 通用关键词
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub series: Option<String>,
    pub tags: Option<Vec<String>>,
    pub rating_min: Option<i32>,
    pub rating_max: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}
```

**搜索逻辑**:
- 关键词搜索: 标题 OR 作者 OR 出版社
- 精确字段搜索: 各字段独立条件
- 标签过滤: 多标签 OR 关系
- 评分范围: >= min AND <= max
- 排序支持: 标题、作者、出版日期、评分、创建时间
- 分页支持: LIMIT + OFFSET

### 5. 阅读器功能

#### 核心 API
```javascript
// 初始化
const book = ePub(epubPath);
const rendition = book.renderTo(container, { width, height });

// 显示内容
await rendition.display(location);

// 翻页
rendition.next();
rendition.prev();

// 跳转
rendition.display(chapterHref);

// 主题设置
rendition.themes.fontSize('18px');
rendition.themes.font('serif');
rendition.themes.default({ body: { background, color } });

// 书签和高亮
rendition.annotations.add('highlight', cfiRange, {}, null, 'hl', { fill, opacity });
```

#### 阅读设置
```javascript
{
  fontSize: 18,                 // 12-32
  fontFamily: 'serif',          // serif, sans-serif, monospace
  theme: 'light',               // light, sepia, dark
  lineHeight: 1.6,              // 1.2-2.0
  respectOriginalStyles: true   // 是否尊重出版商样式
}
```

#### 进度保存
```rust
#[tauri::command]
pub async fn save_reading_progress(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    chapter_href: String,
    progress_percent: f32,
) -> AppResult<()>
```

---

## 用户界面

### 界面 Mockup

设计效果图已保存在：
- `docs/superpowers/mockups/epub-library-ui.html` - 主界面
- `docs/superpowers/mockups/epub-metadata-editor.html` - 元数据编辑器

### 主界面布局

```
┌─────────────────────────────────────────────────────────────┐
│  EPUB 书库        [📥 导入书籍]  [⊞ ☰ ≡ 视图切换]           │
├─────────────────────────────────────────────────────────────┤
│  🔍 搜索框                                [高级搜索]          │
├─────────────────────────────────────┬───────────────────────┤
│                                     │                       │
│                                     │   侧边栏详情/编辑      │
│           书籍展示区                 │                       │
│       (网格/列表/详细列表)            │   - 封面              │
│                                     │   - 元数据            │
│                                     │   - 编辑表单          │
│                                     │   - [开始阅读]        │
│                                     │   - [删除]            │
│                                     │                       │
└─────────────────────────────────────┴───────────────────────┘
```

### 三种视图

#### 1. 网格视图
- 封面为主，卡片式布局
- 显示: 封面、标题、作者、系列、评分
- 适合: 浏览封面、快速选择

#### 2. 列表视图
- 横向卡片，信息更多
- 显示: 小封面、标题、作者、系列、评分、标签
- 适合: 快速扫描信息

#### 3. 详细列表视图
- 表格式，信息最全
- 显示: 缩略图、标题、作者、系列、出版社、评分、标签、添加日期
- 适合: 管理大量书籍、批量操作

### 侧边栏

#### 查看模式
- 大封面
- 完整元数据展示
- 标签列表
- 简介
- 自定义字段
- [开始阅读] 按钮

#### 编辑模式
- 封面上传
- 表单输入（所有字段）
- 作者列表编辑（可添加/删除）
- 标签编辑器（可视化添加/删除）
- 评分选择器（星级）
- 自定义字段编辑
- [保存] [取消] 按钮

---

## AI 功能集成

### 复用现有模块

EPUB 模块实现 `AIEnhanced` trait，调用 `modules/ai/` 的功能。

### 功能集成

#### 1. 智能摘要
```rust
async fn generate_summary(&self, item_id: &str) -> AppResult<String> {
    let book = self.get_book(item_id).await?;
    let content = self.extract_text_content(&book).await?;
    ai::summarize::generate_summary(&content, SummaryLength::Medium).await
}
```

#### 2. 关键词提取
```rust
async fn extract_keywords(&self, item_id: &str) -> AppResult<Vec<String>> {
    let book = self.get_book(item_id).await?;
    let content = self.extract_text_content(&book).await?;
    ai::extract_keywords(&content).await
}
```

#### 3. 语义搜索
```rust
async fn semantic_search(&self, query: &str, top_k: usize) -> AppResult<Vec<LibraryItem>> {
    ai::search::semantic_search(query, "epub", top_k).await
}
```

### 文本提取

```rust
impl EpubParser {
    /// 提取全文
    pub fn extract_full_text(&mut self) -> AppResult<String> {
        // 遍历所有章节，清理 HTML 标签，拼接文本
    }

    /// 提取摘要文本（前 N 字符）
    pub fn extract_sample_text(&mut self, max_chars: usize) -> AppResult<String> {
        // 限制长度避免 token 超限
    }
}
```

### 前端 AI 入口

在侧边栏添加 AI 功能按钮：
- 🤖 生成摘要
- 🏷️ 智能标签（关键词提取）
- 💬 AI 对话

### 向量索引

```rust
// 为书籍建立向量索引
#[tauri::command]
pub async fn index_epub_book(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    book_id: i64,
) -> AppResult<()>

// 批量索引
#[tauri::command]
pub async fn batch_index_epub_books(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    window: Window,
) -> AppResult<()>
```

---

## 实施计划

### Phase 1: 核心书库管理（2-3周）

#### Week 1: 后端基础
- [ ] 数据库迁移文件 (`0007_epub.sql`)
- [ ] EPUB 解析器 (`parser.rs`)
  - 元数据提取
  - 封面提取
  - 目录提取
  - 验证功能
- [ ] 存储管理 (`storage.rs`)
  - 文件夹创建
  - EPUB 复制
  - 封面缩略图生成
- [ ] 基础数据库操作 (`database.rs`)
  - CRUD 书籍
  - 作者管理
  - 标签管理
- [ ] Tauri 命令 (`commands.rs`)
  - `import_epub`
  - `batch_import_epub`
  - `get_epub_book`
  - `list_epub_books`
  - `delete_epub_book`

#### Week 2: 前端界面
- [ ] 书库主界面 (`EpubLibrary.svelte`)
- [ ] 三种视图组件
  - `BookGrid.svelte`
  - `BookList.svelte`
  - `BookDetailList.svelte`
- [ ] 搜索栏 (`SearchBar.svelte`)
  - 基础搜索
  - 高级搜索面板
- [ ] 侧边栏详情 (`BookSidebar.svelte`)
  - 查看模式
- [ ] 导入功能
  - 文件选择对话框
  - 拖拽导入
  - 进度显示

#### Week 3: 元数据编辑
- [ ] 侧边栏编辑模式 (`MetadataEditor.svelte`)
- [ ] 作者列表编辑
  - 添加/删除作者
  - 作者排序
- [ ] 标签编辑器
  - 可视化添加/删除
  - 标签输入提示
- [ ] 评分选择器
- [ ] 封面上传功能
- [ ] 基础自定义字段支持
- [ ] API 集成
  - `update_book_metadata`
  - `set_book_authors`
  - `set_book_tags`
  - `update_book_cover`

**阶段交付物**: 可导入、管理、编辑 EPUB 书籍的完整书库系统（不含阅读器）

---

### Phase 2: 阅读器功能（2周）

#### Week 4: 基础阅读
- [ ] epub.js 集成
  - 依赖安装: `bun add epubjs`
  - 基础渲染器设置
- [ ] 阅读器组件 (`EpubReader.svelte`)
  - 加载 EPUB 文件
  - 初始化渲染
  - 显示内容
- [ ] 目录导航
  - 目录侧边栏
  - 章节跳转
- [ ] 翻页功能
  - 上一页/下一页按钮
  - 键盘导航
  - 触摸手势（如适用）
- [ ] 阅读进度
  - 后端 API: `save_reading_progress`, `get_reading_progress`
  - 自动保存（翻页时）
  - 恢复上次位置

#### Week 5: 增强功能
- [ ] 阅读设置 (`ReaderSettings.svelte`)
  - 字体大小滑块
  - 字体选择
  - 主题切换（白天/护眼/夜间）
  - 行高调整
  - 尊重原始样式开关
- [ ] 工具栏 (`ReaderToolbar.svelte`)
  - 翻页按钮
  - 目录按钮
  - 设置按钮
  - 书签按钮
- [ ] 书签功能
  - 添加书签
  - 书签列表
  - 跳转到书签
  - 删除书签
  - 后端 API: `add_bookmark`, `get_bookmarks`, `delete_bookmark`
- [ ] 高亮功能
  - 文本选择监听
  - 高亮颜色选择
  - 添加高亮
  - 高亮列表
  - 删除高亮
  - 高亮笔记
  - 后端 API: `add_highlight`, `get_highlights`, `delete_highlight`, `update_highlight_note`
- [ ] 设置持久化
  - 保存到 LocalStorage
  - 应用启动时恢复

**阶段交付物**: 完整的 EPUB 阅读体验，支持个性化设置、书签、高亮

---

### Phase 3: 高级元数据（1-2周）

#### Week 6-7
- [ ] 自定义字段系统 (`custom_fields.rs`)
  - 字段定义 CRUD
  - 支持多种数据类型
  - 字段排序
- [ ] 自定义字段编辑器 (`CustomFieldEditor.svelte`)
  - 创建字段定义
  - 字段类型选择
  - 字段值编辑（根据类型）
- [ ] 系列管理优化
  - 系列列表视图
  - 系列内排序
  - 系列封面
- [ ] 多作者排序
  - 作者顺序拖拽
  - 主要作者标记
- [ ] 高级搜索
  - 自定义字段搜索
  - 复杂组合条件
  - 搜索结果排序选项
  - 保存搜索条件（可选）

**阶段交付物**: Calibre 级别的完整元数据管理系统

---

### Phase 4: AI 集成（1周）

#### Week 8
- [ ] AI 对话集成
  - 设置 EPUB 书籍上下文
  - 复用 `AIAssistant.svelte`
- [ ] 智能摘要
  - 后端 API: `epub_generate_summary`
  - 文本提取优化
  - 摘要显示对话框
- [ ] 语义搜索
  - 向量索引构建
  - 后端 API: `index_epub_book`, `batch_index_epub_books`
  - 搜索模式切换（文本/语义）
  - 语义搜索 UI
- [ ] 关键词提取
  - 后端 API: `epub_extract_keywords`
  - 智能标签建议
  - 一键应用标签
- [ ] AI 功能入口
  - 侧边栏 AI 按钮
  - 功能说明和引导

**阶段交付物**: AI 增强的 EPUB 书库，智能摘要、对话、搜索完整可用

---

## 验收标准

### Phase 1 验收

#### 功能验收
- [ ] 可导入单个 EPUB 文件
- [ ] 可批量导入多个 EPUB 文件
- [ ] 导入时正确提取元数据（标题、作者、出版社、ISBN等）
- [ ] 导入时正确提取和显示封面
- [ ] 三种视图（网格、列表、详细列表）正常切换和显示
- [ ] 可搜索书籍（标题、作者）
- [ ] 侧边栏显示完整书籍详情
- [ ] 可编辑书籍元数据（标题、作者、出版社等）
- [ ] 可添加/删除作者
- [ ] 可添加/删除标签
- [ ] 可上传/更换封面
- [ ] 可删除书籍（文件和数据库记录同步删除）

#### 界面验收
- [ ] 界面与 Mockup 效果一致或更好
- [ ] 选中书籍时侧边栏正确展开
- [ ] 编辑模式和查看模式切换流畅
- [ ] 表单验证正确（必填项提示）

#### 性能验收
- [ ] 单本书导入 < 5 秒
- [ ] 批量导入 10 本书 < 1 分钟
- [ ] 搜索响应 < 500ms（100 本规模）
- [ ] 视图切换无卡顿

### Phase 2 验收

#### 功能验收
- [ ] 可打开任意 EPUB 书籍
- [ ] 目录正确显示
- [ ] 点击目录可跳转到对应章节
- [ ] 翻页功能正常（上一页/下一页）
- [ ] 阅读进度自动保存
- [ ] 重新打开书籍时恢复上次阅读位置
- [ ] 可调整字体大小（12-32px）
- [ ] 可切换字体（衬线/无衬线/等宽）
- [ ] 可切换主题（白天/护眼/夜间）
- [ ] 可调整行高
- [ ] 可添加书签
- [ ] 书签列表正确显示
- [ ] 可跳转到书签位置
- [ ] 可删除书签
- [ ] 可高亮选中文本
- [ ] 高亮正确显示在文本上
- [ ] 可为高亮添加笔记
- [ ] 可删除高亮

#### 界面验收
- [ ] 阅读器界面简洁美观
- [ ] 工具栏不遮挡内容
- [ ] 设置面板操作直观
- [ ] 主题切换视觉效果正确

#### 性能验收
- [ ] 阅读器加载 < 2 秒
- [ ] 翻页响应 < 200ms
- [ ] 主题切换即时生效

### Phase 3 验收

#### 功能验收
- [ ] 可创建自定义字段（所有数据类型）
- [ ] 自定义字段正确显示和编辑
- [ ] 系列管理功能正常
- [ ] 多作者排序功能正常
- [ ] 高级搜索支持自定义字段
- [ ] 自定义字段可用于排序和过滤

#### 数据验收
- [ ] 自定义字段数据正确存储
- [ ] 字段删除后关联数据正确处理
- [ ] 不同数据类型的验证正确

### Phase 4 验收

#### 功能验收
- [ ] 可为书籍生成摘要
- [ ] 摘要内容相关且准确
- [ ] 可提取关键词作为标签建议
- [ ] 关键词提取准确率 > 80%
- [ ] 可使用语义搜索查找书籍
- [ ] 语义搜索结果相关度高
- [ ] 可与 AI 对话讨论书籍内容
- [ ] AI 对话理解书籍上下文

#### 性能验收
- [ ] 摘要生成 < 10 秒
- [ ] 关键词提取 < 5 秒
- [ ] 语义搜索 < 2 秒（100 本规模）
- [ ] 向量索引构建 < 10 秒/本

### 整体验收

#### 代码质量
- [ ] 所有 Rust 代码通过 `cargo clippy`
- [ ] 所有 TypeScript 代码通过类型检查
- [ ] 关键功能有单元测试
- [ ] 代码遵循项目规范（CLAUDE.md）

#### 文档
- [ ] 功能使用文档完整
- [ ] API 文档清晰
- [ ] 数据库 Schema 文档完整

#### 用户体验
- [ ] 界面直观，无需学习成本
- [ ] 错误提示友好且可操作
- [ ] 加载状态有明确反馈
- [ ] 操作流程顺畅无阻塞

---

## 风险和应对

### 技术风险

#### 1. EPUB 格式兼容性
**风险**: 不同出版商的 EPUB 文件格式可能有差异，导致解析失败

**应对**:
- 使用成熟的 `epub` crate
- 添加格式验证和错误处理
- 记录失败案例，逐步优化

#### 2. 阅读器性能
**风险**: 大型 EPUB 文件可能导致阅读器加载慢或卡顿

**应对**:
- epub.js 已优化大文件处理
- 按需加载章节
- 图片懒加载

#### 3. AI 功能依赖
**风险**: Ollama 服务不可用时 AI 功能无法使用

**应对**:
- AI 功能为增强功能，不影响核心使用
- 服务状态检测和友好提示
- 离线模式正常使用基础功能

### 进度风险

#### 1. 功能范围扩大
**风险**: 开发过程中发现新需求，导致进度延期

**应对**:
- 严格遵循分阶段计划
- 新需求记录到 backlog
- 每个阶段交付后再考虑扩展

#### 2. 技术难点耗时
**风险**: 某些技术点（如自定义字段系统）可能比预期复杂

**应对**:
- 预留 buffer 时间
- 及时沟通，调整优先级
- 必要时简化实现

---

## 未来扩展

### 短期（可选）
- 批量编辑元数据
- 导出书籍列表（CSV/JSON）
- 书籍评论和笔记系统
- 阅读统计（阅读时长、进度）

### 中期
- EPUB 文件格式转换
- 云同步（阅读进度、书签、高亮）
- 移动端支持
- 插件系统

### 长期
- 社区功能（书评、推荐）
- 多设备协同阅读
- 在线书城集成

---

## 附录

### 参考资料

- [EPUB 3.3 Specification](https://www.w3.org/TR/epub-33/)
- [epub.js Documentation](https://github.com/futurepress/epub.js)
- [Calibre User Manual](https://manual.calibre-ebook.com/)
- [Rust epub crate](https://docs.rs/epub/latest/epub/)

### 设计文件

- `docs/superpowers/mockups/epub-library-ui.html` - 主界面 Mockup
- `docs/superpowers/mockups/epub-metadata-editor.html` - 元数据编辑器 Mockup

---

**文档版本**: 1.0
**最后更新**: 2026-03-24
**作者**: Claude Sonnet 4.5
**状态**: ✅ 设计完成，待审核
