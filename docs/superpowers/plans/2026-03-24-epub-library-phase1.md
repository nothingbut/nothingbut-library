# EPUB 书库 Phase 1 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现 EPUB 书库的核心管理功能，包括导入、元数据管理、三种视图展示、搜索和编辑

**Architecture:** 遵循项目现有模块化架构，创建独立的 `epub` 模块。后端使用 Rust + SQLite，前端使用 SvelteKit + Svelte 5。EPUB 解析使用 `epub` crate，封面处理使用 `image` crate。

**Tech Stack:**
- Backend: Rust, epub crate, image crate, sqlx, Tauri 2
- Frontend: SvelteKit 2, Svelte 5, TypeScript, Tailwind CSS 4
- Database: SQLite

**Phase 1 交付物**: 可导入、管理、编辑 EPUB 书籍的完整书库系统（不含阅读器）

---

## Week 1: 后端基础

### Task 1: 数据库迁移文件

**Files:**
- Create: `src-tauri/migrations/0007_epub.sql`

- [ ] **Step 1: 创建迁移文件骨架**

```sql
-- EPUB 书库数据库 Schema
-- 创建时间: 2026-03-24

-- ==================== 书籍主表 ====================
CREATE TABLE epub_books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    sort_title TEXT,
    isbn TEXT,
    publisher TEXT,
    pubdate TEXT,
    language TEXT,
    series TEXT,
    series_index REAL,
    rating INTEGER CHECK(rating >= 0 AND rating <= 5),
    file_path TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL,
    cover_path TEXT,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_epub_books_title ON epub_books(title);
CREATE INDEX idx_epub_books_series ON epub_books(series);
CREATE INDEX idx_epub_books_isbn ON epub_books(isbn);
```

- [ ] **Step 2: 添加作者相关表**

```sql
-- ==================== 作者表 ====================
CREATE TABLE epub_authors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    sort_name TEXT,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_epub_authors_name ON epub_authors(name);

-- 书籍-作者关联表（多对多）
CREATE TABLE epub_book_authors (
    book_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    author_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (book_id, author_id),
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES epub_authors(id) ON DELETE CASCADE
);
```

- [ ] **Step 3: 添加标签相关表**

```sql
-- ==================== 标签表 ====================
CREATE TABLE epub_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

-- 书籍-标签关联表（多对多）
CREATE TABLE epub_book_tags (
    book_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (book_id, tag_id),
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES epub_tags(id) ON DELETE CASCADE
);
```

- [ ] **Step 4: 添加自定义字段相关表**

```sql
-- ==================== 自定义字段 ====================
CREATE TABLE epub_custom_fields (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    datatype TEXT NOT NULL,
    is_multiple INTEGER NOT NULL DEFAULT 0,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE epub_custom_field_values (
    book_id INTEGER NOT NULL,
    field_id INTEGER NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (book_id, field_id),
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE,
    FOREIGN KEY (field_id) REFERENCES epub_custom_fields(id) ON DELETE CASCADE
);
```

- [ ] **Step 5: 添加阅读相关表**

```sql
-- ==================== 阅读进度 ====================
CREATE TABLE epub_reading_progress (
    book_id INTEGER PRIMARY KEY,
    chapter_href TEXT NOT NULL,
    progress_percent REAL NOT NULL DEFAULT 0.0,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE
);

-- ==================== 书签 ====================
CREATE TABLE epub_bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL,
    chapter_href TEXT NOT NULL,
    cfi TEXT NOT NULL,
    note TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE
);

CREATE INDEX idx_epub_bookmarks_book ON epub_bookmarks(book_id);

-- ==================== 高亮 ====================
CREATE TABLE epub_highlights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL,
    chapter_href TEXT NOT NULL,
    cfi_range TEXT NOT NULL,
    text TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#ffeb3b',
    note TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE
);

CREATE INDEX idx_epub_highlights_book ON epub_highlights(book_id);
```

- [ ] **Step 6: 测试迁移执行**

Run: `cd src-tauri && cargo sqlx migrate run`
Expected: 成功创建所有表

- [ ] **Step 7: 提交迁移文件**

```bash
git add src-tauri/migrations/0007_epub.sql
git commit -m "feat(epub): add database migration for EPUB library

- Create epub_books table with metadata fields
- Create epub_authors and epub_tags with many-to-many relations
- Create epub_custom_fields for Calibre-style metadata
- Create reading progress, bookmarks, and highlights tables

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: EPUB 模块数据模型

**Files:**
- Create: `src-tauri/src/modules/epub/mod.rs`
- Create: `src-tauri/src/modules/epub/models.rs`

- [ ] **Step 1: 创建模块入口文件**

```rust
// src-tauri/src/modules/epub/mod.rs
pub mod models;
pub mod database;
pub mod parser;
pub mod metadata;
pub mod storage;
pub mod commands;
pub mod custom_fields;

pub use models::*;
```

- [ ] **Step 2: 创建数据模型文件骨架**

```rust
// src-tauri/src/modules/epub/models.rs
use serde::{Deserialize, Serialize};

/// EPUB 书籍
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub rating: Option<i32>,
    pub file_path: String,
    pub file_size: i64,
    pub cover_path: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
```

- [ ] **Step 3: 添加作者和标签模型**

```rust
/// 作者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub sort_name: Option<String>,
    pub created_at: String,
}

/// 标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

/// 书籍-作者关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookAuthor {
    pub book_id: i64,
    pub author_id: i64,
    pub author_order: i32,
}

/// 书籍-标签关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookTag {
    pub book_id: i64,
    pub tag_id: i64,
}
```

- [ ] **Step 4: 添加自定义字段模型**

```rust
/// 自定义字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub id: i64,
    pub name: String,
    pub label: String,
    pub datatype: CustomFieldType,
    pub is_multiple: bool,
    pub display_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CustomFieldType {
    Text,
    Series,
    Enumeration,
    Number,
    Rating,
    Date,
    Bool,
    Comments,
}

/// 自定义字段值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldValue {
    pub book_id: i64,
    pub field_id: i64,
    pub value: String,
}
```

- [ ] **Step 5: 添加阅读相关模型**

```rust
/// 阅读进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub book_id: i64,
    pub chapter_href: String,
    pub progress_percent: f32,
    pub updated_at: String,
}

/// 书签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub book_id: i64,
    pub chapter_href: String,
    pub cfi: String,
    pub note: Option<String>,
    pub created_at: String,
}

/// 高亮
#[derive(Debug, Clone, Serialize, Deserialize)]
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

- [ ] **Step 6: 添加辅助模型**

```rust
/// EPUB 元数据（解析用）
#[derive(Debug, Clone)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub publisher: Option<String>,
    pub pubdate: Option<String>,
    pub language: Option<String>,
    pub isbn: Option<String>,
    pub description: Option<String>,
}

/// EPUB 章节信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubChapter {
    pub href: String,
    pub title: String,
    pub level: i32,
    pub order_index: i32,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ImportResult {
    Success { book_id: i64 },
    Failed { file_path: String, error: String },
}

/// 导入进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProgress {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
}
```

- [ ] **Step 7: 编译检查**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 8: 提交模型定义**

```bash
git add src-tauri/src/modules/epub/
git commit -m "feat(epub): add data models for EPUB library

- EpubBook, Author, Tag models
- CustomField system models
- Reading progress, bookmark, highlight models
- Helper models for import and parsing

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: EPUB 解析器

**Files:**
- Create: `src-tauri/src/modules/epub/parser.rs`
- Modify: `src-tauri/Cargo.toml` (add epub dependency)

- [ ] **Step 1: 添加 epub 依赖**

```toml
# src-tauri/Cargo.toml
[dependencies]
epub = "2.0"
```

- [ ] **Step 2: 创建解析器骨架**

```rust
// src-tauri/src/modules/epub/parser.rs
use crate::errors::{AppError, AppResult};
use crate::modules::epub::models::{EpubChapter, EpubMetadata};
use epub::doc::EpubDoc;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct EpubParser {
    doc: EpubDoc<BufReader<File>>,
}

impl EpubParser {
    /// 打开 EPUB 文件
    pub fn open(path: &Path) -> AppResult<Self> {
        let doc = EpubDoc::new(path)
            .map_err(|e| AppError::InvalidInput(format!("Failed to open EPUB: {}", e)))?;
        Ok(Self { doc })
    }
}
```

- [ ] **Step 3: 实现元数据提取**

```rust
impl EpubParser {
    /// 提取元数据
    pub fn extract_metadata(&mut self) -> AppResult<EpubMetadata> {
        let metadata = EpubMetadata {
            title: self.doc.mdata("title"),
            authors: self.extract_authors(),
            publisher: self.doc.mdata("publisher"),
            pubdate: self.doc.mdata("date"),
            language: self.doc.mdata("language"),
            isbn: self.extract_isbn(),
            description: self.doc.mdata("description"),
        };
        Ok(metadata)
    }

    fn extract_authors(&mut self) -> Vec<String> {
        let mut authors = Vec::new();

        // EPUB 可能有多个作者
        if let Some(author) = self.doc.mdata("creator") {
            authors.push(author);
        }

        // 尝试其他作者字段
        let mut i = 1;
        while let Some(author) = self.doc.mdata(&format!("creator_{}", i)) {
            authors.push(author);
            i += 1;
        }

        authors
    }

    fn extract_isbn(&mut self) -> Option<String> {
        // 尝试多种 ISBN 字段
        self.doc.mdata("isbn")
            .or_else(|| self.doc.mdata("identifier"))
            .or_else(|| self.doc.mdata("ISBN"))
    }
}
```

- [ ] **Step 4: 实现封面提取**

```rust
impl EpubParser {
    /// 提取封面图片
    pub fn extract_cover(&mut self) -> AppResult<Option<Vec<u8>>> {
        match self.doc.get_cover() {
            Some(cover_data) => Ok(Some(cover_data)),
            None => Ok(None),
        }
    }

    /// 获取封面 MIME 类型
    pub fn get_cover_mime(&mut self) -> Option<String> {
        self.doc.get_cover_mime()
    }
}
```

- [ ] **Step 5: 实现目录提取**

```rust
impl EpubParser {
    /// 提取目录（TOC）
    pub fn extract_toc(&mut self) -> AppResult<Vec<EpubChapter>> {
        let chapters: Vec<EpubChapter> = self
            .doc
            .toc
            .iter()
            .enumerate()
            .map(|(i, item)| EpubChapter {
                href: item.content.clone(),
                title: item.label.clone(),
                level: item.level,
                order_index: i as i32,
            })
            .collect();

        Ok(chapters)
    }
}
```

- [ ] **Step 6: 实现验证功能**

```rust
impl EpubParser {
    /// 验证 EPUB 文件完整性
    pub fn validate(&mut self) -> AppResult<bool> {
        // 检查是否能读取资源
        if self.doc.resources.is_empty() {
            return Err(AppError::InvalidInput(
                "EPUB file has no resources".to_string(),
            ));
        }

        // 检查是否有内容
        if self.doc.get_num_pages() == 0 {
            return Err(AppError::InvalidInput(
                "EPUB file has no pages".to_string(),
            ));
        }

        Ok(true)
    }
}
```

- [ ] **Step 7: 编译检查**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 8: 提交解析器**

```bash
git add src-tauri/Cargo.toml src-tauri/src/modules/epub/parser.rs
git commit -m "feat(epub): add EPUB parser with metadata extraction

- Use epub crate for EPUB parsing
- Extract metadata (title, authors, publisher, etc.)
- Extract cover image
- Extract table of contents
- Validate EPUB file integrity

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: 存储管理

**Files:**
- Create: `src-tauri/src/modules/epub/storage.rs`
- Modify: `src-tauri/Cargo.toml` (add image dependency)

- [ ] **Step 1: 添加 image 依赖**

```toml
# src-tauri/Cargo.toml
[dependencies]
image = "0.24"
```

- [ ] **Step 2: 创建存储管理器骨架**

```rust
// src-tauri/src/modules/epub/storage.rs
use crate::errors::{AppError, AppResult};
use image::ImageFormat;
use std::fs;
use std::path::{Path, PathBuf};

pub struct EpubStorage {
    workspace_path: PathBuf,
}

impl EpubStorage {
    pub fn new(workspace_path: impl AsRef<Path>) -> Self {
        Self {
            workspace_path: workspace_path.as_ref().to_path_buf(),
        }
    }

    /// 获取 EPUB 书库根目录
    pub fn epub_root(&self) -> PathBuf {
        self.workspace_path.join("epub")
    }

    /// 获取书籍目录
    pub fn book_dir(&self, book_id: i64) -> PathBuf {
        self.epub_root().join("books").join(book_id.to_string())
    }
}
```

- [ ] **Step 3: 实现目录创建**

```rust
impl EpubStorage {
    /// 创建书籍存储目录
    pub fn create_book_dir(&self, book_id: i64) -> AppResult<PathBuf> {
        let book_dir = self.book_dir(book_id);

        fs::create_dir_all(&book_dir)
            .map_err(|e| AppError::Io(format!("Failed to create book directory: {}", e)))?;

        Ok(book_dir)
    }

    /// 确保 EPUB 根目录存在
    pub fn ensure_epub_root(&self) -> AppResult<()> {
        let epub_root = self.epub_root();

        if !epub_root.exists() {
            fs::create_dir_all(&epub_root)
                .map_err(|e| AppError::Io(format!("Failed to create epub directory: {}", e)))?;
        }

        Ok(())
    }
}
```

- [ ] **Step 4: 实现文件复制**

```rust
impl EpubStorage {
    /// 复制 EPUB 文件到书库
    pub fn copy_epub_file(
        &self,
        source_path: &Path,
        book_id: i64,
    ) -> AppResult<PathBuf> {
        let book_dir = self.create_book_dir(book_id)?;
        let dest_path = book_dir.join("book.epub");

        fs::copy(source_path, &dest_path)
            .map_err(|e| AppError::Io(format!("Failed to copy EPUB file: {}", e)))?;

        Ok(dest_path)
    }

    /// 获取 EPUB 文件路径
    pub fn epub_file_path(&self, book_id: i64) -> PathBuf {
        self.book_dir(book_id).join("book.epub")
    }
}
```

- [ ] **Step 5: 实现封面保存**

```rust
impl EpubStorage {
    /// 保存封面图片（生成多个尺寸）
    pub fn save_cover(
        &self,
        cover_data: &[u8],
        book_id: i64,
    ) -> AppResult<()> {
        let book_dir = self.book_dir(book_id);

        // 加载图片
        let img = image::load_from_memory(cover_data)
            .map_err(|e| AppError::InvalidInput(format!("Failed to load cover image: {}", e)))?;

        // 生成大图（600x800）
        let large = img.resize(600, 800, image::imageops::FilterType::Lanczos3);
        let large_path = book_dir.join("cover.jpg");
        large
            .save_with_format(&large_path, ImageFormat::Jpeg)
            .map_err(|e| AppError::Io(format!("Failed to save large cover: {}", e)))?;

        // 生成缩略图（200x267）
        let thumb = img.resize(200, 267, image::imageops::FilterType::Lanczos3);
        let thumb_path = book_dir.join("cover_thumb.jpg");
        thumb
            .save_with_format(&thumb_path, ImageFormat::Jpeg)
            .map_err(|e| AppError::Io(format!("Failed to save cover thumbnail: {}", e)))?;

        Ok(())
    }

    /// 获取封面路径
    pub fn cover_path(&self, book_id: i64) -> PathBuf {
        self.book_dir(book_id).join("cover.jpg")
    }

    /// 获取封面缩略图路径
    pub fn cover_thumb_path(&self, book_id: i64) -> PathBuf {
        self.book_dir(book_id).join("cover_thumb.jpg")
    }
}
```

- [ ] **Step 6: 实现元数据备份**

```rust
impl EpubStorage {
    /// 保存元数据 JSON（备份）
    pub fn save_metadata_json(
        &self,
        book_id: i64,
        metadata: &serde_json::Value,
    ) -> AppResult<()> {
        let book_dir = self.book_dir(book_id);
        let metadata_path = book_dir.join("metadata.json");

        let json_str = serde_json::to_string_pretty(metadata)
            .map_err(|e| AppError::Serialization(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(&metadata_path, json_str)
            .map_err(|e| AppError::Io(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }
}
```

- [ ] **Step 7: 实现删除功能**

```rust
impl EpubStorage {
    /// 删除书籍所有文件
    pub fn delete_book(&self, book_id: i64) -> AppResult<()> {
        let book_dir = self.book_dir(book_id);

        if book_dir.exists() {
            fs::remove_dir_all(&book_dir)
                .map_err(|e| AppError::Io(format!("Failed to delete book directory: {}", e)))?;
        }

        Ok(())
    }
}
```

- [ ] **Step 8: 编译检查**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 9: 提交存储管理器**

```bash
git add src-tauri/Cargo.toml src-tauri/src/modules/epub/storage.rs
git commit -m "feat(epub): add storage manager for EPUB files

- Create book directories structure
- Copy EPUB files to library
- Generate cover images (600x800 + 200x267)
- Save metadata JSON backup
- Delete book files

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## ⏸️ Checkpoint: Week 1 进度检查

此时应该完成：
- ✅ 数据库迁移文件
- ✅ 数据模型定义
- ✅ EPUB 解析器
- ✅ 存储管理器

下一步：基础数据库操作和 Tauri 命令（由于篇幅限制，后续任务将在独立文件中继续）

---

## 继续实施说明

由于完整的 Phase 1 计划非常详细（预计超过 5000 行），建议采用以下方式继续：

1. **分周拆分**:
   - Week 1 计划（当前文件）
   - Week 2 计划（前端界面）
   - Week 3 计划（元数据编辑）

2. **渐进式实施**:
   - 完成 Week 1 后，根据实际进展调整 Week 2 计划
   - 每个里程碑后进行代码审查和验收测试

3. **参考设计文档**:
   - 详细的 API 设计：`docs/superpowers/specs/2026-03-24-epub-library-design.md`
   - UI Mockup：`docs/superpowers/mockups/epub-*.html`

---

## Week 2: 前端界面

### Task 5: 数据库操作层

**Files:**
- Create: `src-tauri/src/modules/epub/database.rs`

- [ ] **Step 1: 创建数据库操作骨架**

```rust
// src-tauri/src/modules/epub/database.rs
use crate::errors::{AppError, AppResult};
use crate::modules::epub::models::*;
use sqlx::{SqlitePool, Row};

pub struct EpubDatabase {
    pool: SqlitePool,
}

impl EpubDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
```

- [ ] **Step 2: 实现书籍 CRUD**

```rust
impl EpubDatabase {
    /// 创建书籍记录
    pub async fn create_book(&self, book: &EpubBook) -> AppResult<i64> {
        let result = sqlx::query(
            r#"
            INSERT INTO epub_books (
                title, sort_title, isbn, publisher, pubdate, language,
                series, series_index, rating, file_path, file_size,
                cover_path, description, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&book.title)
        .bind(&book.sort_title)
        .bind(&book.isbn)
        .bind(&book.publisher)
        .bind(&book.pubdate)
        .bind(&book.language)
        .bind(&book.series)
        .bind(book.series_index)
        .bind(book.rating)
        .bind(&book.file_path)
        .bind(book.file_size)
        .bind(&book.cover_path)
        .bind(&book.description)
        .bind(&book.created_at)
        .bind(&book.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create book: {}", e)))?;

        Ok(result.last_insert_rowid())
    }

    /// 获取书籍详情
    pub async fn get_book(&self, book_id: i64) -> AppResult<Option<EpubBook>> {
        let book = sqlx::query_as::<_, EpubBook>(
            "SELECT * FROM epub_books WHERE id = ?"
        )
        .bind(book_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get book: {}", e)))?;

        Ok(book)
    }

    /// 列出所有书籍
    pub async fn list_books(&self) -> AppResult<Vec<EpubBook>> {
        let books = sqlx::query_as::<_, EpubBook>(
            "SELECT * FROM epub_books ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to list books: {}", e)))?;

        Ok(books)
    }

    /// 更新书籍元数据
    pub async fn update_book(&self, book_id: i64, book: &EpubBook) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE epub_books
            SET title = ?, sort_title = ?, isbn = ?, publisher = ?,
                pubdate = ?, language = ?, series = ?, series_index = ?,
                rating = ?, description = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&book.title)
        .bind(&book.sort_title)
        .bind(&book.isbn)
        .bind(&book.publisher)
        .bind(&book.pubdate)
        .bind(&book.language)
        .bind(&book.series)
        .bind(book.series_index)
        .bind(book.rating)
        .bind(&book.description)
        .bind(&book.updated_at)
        .bind(book_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to update book: {}", e)))?;

        Ok(())
    }

    /// 删除书籍
    pub async fn delete_book(&self, book_id: i64) -> AppResult<()> {
        sqlx::query("DELETE FROM epub_books WHERE id = ?")
            .bind(book_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to delete book: {}", e)))?;

        Ok(())
    }
}
```

- [ ] **Step 3: 实现作者管理**

```rust
impl EpubDatabase {
    /// 获取或创建作者
    pub async fn get_or_create_author(&self, name: &str, sort_name: Option<&str>) -> AppResult<i64> {
        // 先尝试查找
        if let Some(author) = sqlx::query_as::<_, Author>(
            "SELECT * FROM epub_authors WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query author: {}", e)))?
        {
            return Ok(author.id);
        }

        // 不存在则创建
        let now = chrono::Utc::now().to_rfc3339();
        let result = sqlx::query(
            "INSERT INTO epub_authors (name, sort_name, created_at) VALUES (?, ?, ?)"
        )
        .bind(name)
        .bind(sort_name)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create author: {}", e)))?;

        Ok(result.last_insert_rowid())
    }

    /// 设置书籍作者（替换现有）
    pub async fn set_book_authors(
        &self,
        book_id: i64,
        authors: Vec<(String, Option<String>, i32)>, // (name, sort_name, order)
    ) -> AppResult<()> {
        // 删除现有关联
        sqlx::query("DELETE FROM epub_book_authors WHERE book_id = ?")
            .bind(book_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to delete book authors: {}", e)))?;

        // 添加新关联
        for (name, sort_name, order) in authors {
            let author_id = self.get_or_create_author(&name, sort_name.as_deref()).await?;

            sqlx::query(
                "INSERT INTO epub_book_authors (book_id, author_id, author_order) VALUES (?, ?, ?)"
            )
            .bind(book_id)
            .bind(author_id)
            .bind(order)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to add book author: {}", e)))?;
        }

        Ok(())
    }

    /// 获取书籍作者列表
    pub async fn get_book_authors(&self, book_id: i64) -> AppResult<Vec<Author>> {
        let authors = sqlx::query_as::<_, Author>(
            r#"
            SELECT a.* FROM epub_authors a
            JOIN epub_book_authors ba ON a.id = ba.author_id
            WHERE ba.book_id = ?
            ORDER BY ba.author_order
            "#
        )
        .bind(book_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get book authors: {}", e)))?;

        Ok(authors)
    }
}
```

- [ ] **Step 4: 实现标签管理**

```rust
impl EpubDatabase {
    /// 获取或创建标签
    pub async fn get_or_create_tag(&self, name: &str) -> AppResult<i64> {
        // 先尝试查找
        if let Some(tag) = sqlx::query_as::<_, Tag>(
            "SELECT * FROM epub_tags WHERE name = ?"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to query tag: {}", e)))?
        {
            return Ok(tag.id);
        }

        // 不存在则创建
        let now = chrono::Utc::now().to_rfc3339();
        let result = sqlx::query(
            "INSERT INTO epub_tags (name, created_at) VALUES (?, ?)"
        )
        .bind(name)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to create tag: {}", e)))?;

        Ok(result.last_insert_rowid())
    }

    /// 设置书籍标签（替换现有）
    pub async fn set_book_tags(&self, book_id: i64, tag_names: Vec<String>) -> AppResult<()> {
        // 删除现有关联
        sqlx::query("DELETE FROM epub_book_tags WHERE book_id = ?")
            .bind(book_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to delete book tags: {}", e)))?;

        // 添加新关联
        for name in tag_names {
            let tag_id = self.get_or_create_tag(&name).await?;

            sqlx::query(
                "INSERT INTO epub_book_tags (book_id, tag_id) VALUES (?, ?)"
            )
            .bind(book_id)
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to add book tag: {}", e)))?;
        }

        Ok(())
    }

    /// 获取书籍标签列表
    pub async fn get_book_tags(&self, book_id: i64) -> AppResult<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>(
            r#"
            SELECT t.* FROM epub_tags t
            JOIN epub_book_tags bt ON t.id = bt.tag_id
            WHERE bt.book_id = ?
            ORDER BY t.name
            "#
        )
        .bind(book_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to get book tags: {}", e)))?;

        Ok(tags)
    }
}
```

- [ ] **Step 5: 实现搜索功能**

```rust
impl EpubDatabase {
    /// 搜索书籍
    pub async fn search_books(&self, query: &SearchQuery) -> AppResult<Vec<EpubBook>> {
        let mut sql = String::from("SELECT DISTINCT b.* FROM epub_books b");
        let mut conditions = Vec::new();
        let mut bindings = Vec::new();

        // 关键词搜索（标题、作者、出版社）
        if let Some(keyword) = &query.keyword {
            sql.push_str(" LEFT JOIN epub_book_authors ba ON b.id = ba.book_id");
            sql.push_str(" LEFT JOIN epub_authors a ON ba.author_id = a.id");
            conditions.push("(b.title LIKE ? OR a.name LIKE ? OR b.publisher LIKE ?)");
            let pattern = format!("%{}%", keyword);
            bindings.push(pattern.clone());
            bindings.push(pattern.clone());
            bindings.push(pattern);
        }

        // 标题搜索
        if let Some(title) = &query.title {
            conditions.push("b.title LIKE ?");
            bindings.push(format!("%{}%", title));
        }

        // 系列搜索
        if let Some(series) = &query.series {
            conditions.push("b.series LIKE ?");
            bindings.push(format!("%{}%", series));
        }

        // 评分范围
        if let Some(min) = query.rating_min {
            conditions.push("b.rating >= ?");
            bindings.push(min.to_string());
        }
        if let Some(max) = query.rating_max {
            conditions.push("b.rating <= ?");
            bindings.push(max.to_string());
        }

        // 组装 WHERE 子句
        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        // 排序
        let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("DESC");
        sql.push_str(&format!(" ORDER BY b.{} {}", sort_by, sort_order));

        // 分页
        if let Some(limit) = query.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
            if let Some(offset) = query.offset {
                sql.push_str(&format!(" OFFSET {}", offset));
            }
        }

        // TODO: 动态绑定参数（简化示例）
        let books = sqlx::query_as::<_, EpubBook>(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::Database(format!("Failed to search books: {}", e)))?;

        Ok(books)
    }
}
```

- [ ] **Step 6: 编译检查**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 7: 提交数据库操作层**

```bash
git add src-tauri/src/modules/epub/database.rs
git commit -m "feat(epub): add database operations layer

- Book CRUD operations
- Author management with auto-creation
- Tag management with auto-creation
- Search functionality with multiple filters

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Tauri 命令处理器

**Files:**
- Create: `src-tauri/src/modules/epub/commands.rs`
- Modify: `src-tauri/src/lib.rs` (注册命令)

- [ ] **Step 1: 创建命令处理器骨架**

```rust
// src-tauri/src/modules/epub/commands.rs
use crate::errors::AppResult;
use crate::modules::epub::{database::EpubDatabase, models::*, parser::EpubParser, storage::EpubStorage};
use sqlx::SqlitePool;
use tauri::{AppHandle, State, Window};
use std::path::Path;

/// 导入单个 EPUB 文件
#[tauri::command]
pub async fn import_epub(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    source_file_path: String,
) -> AppResult<i64> {
    // 实现见下一步
    todo!()
}
```

- [ ] **Step 2: 实现导入功能**

```rust
/// 导入单个 EPUB 文件
#[tauri::command]
pub async fn import_epub(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    source_file_path: String,
) -> AppResult<i64> {
    let db = EpubDatabase::new(pool.inner().clone());
    let storage = EpubStorage::new(&workspace_path);
    storage.ensure_epub_root()?;

    // 1. 解析 EPUB
    let source_path = Path::new(&source_file_path);
    let mut parser = EpubParser::open(source_path)?;
    parser.validate()?;

    // 2. 提取元数据
    let metadata = parser.extract_metadata()?;
    let cover_data = parser.extract_cover()?;

    // 3. 获取文件信息
    let file_size = std::fs::metadata(source_path)
        .map_err(|e| crate::errors::AppError::Io(format!("Failed to get file size: {}", e)))?
        .len() as i64;

    // 4. 创建数据库记录（临时路径）
    let now = chrono::Utc::now().to_rfc3339();
    let book = EpubBook {
        id: 0,
        title: metadata.title.unwrap_or_else(|| "Unknown".to_string()),
        sort_title: None,
        isbn: metadata.isbn,
        publisher: metadata.publisher,
        pubdate: metadata.pubdate,
        language: metadata.language,
        series: None,
        series_index: None,
        rating: None,
        file_path: String::new(), // 临时
        file_size,
        cover_path: None,
        description: metadata.description,
        created_at: now.clone(),
        updated_at: now,
    };

    let book_id = db.create_book(&book).await?;

    // 5. 复制 EPUB 文件
    let epub_path = storage.copy_epub_file(source_path, book_id)?;

    // 6. 保存封面
    let cover_path = if let Some(cover_bytes) = cover_data {
        storage.save_cover(&cover_bytes, book_id)?;
        Some(storage.cover_path(book_id).to_string_lossy().to_string())
    } else {
        None
    };

    // 7. 更新数据库路径
    let mut updated_book = book;
    updated_book.id = book_id;
    updated_book.file_path = epub_path.to_string_lossy().to_string();
    updated_book.cover_path = cover_path;
    db.update_book(book_id, &updated_book).await?;

    // 8. 设置作者
    if !metadata.authors.is_empty() {
        let authors: Vec<(String, Option<String>, i32)> = metadata
            .authors
            .into_iter()
            .enumerate()
            .map(|(i, name)| (name, None, i as i32))
            .collect();
        db.set_book_authors(book_id, authors).await?;
    }

    Ok(book_id)
}
```

- [ ] **Step 3: 实现批量导入**

```rust
/// 批量导入 EPUB 文件
#[tauri::command]
pub async fn batch_import_epub(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    file_paths: Vec<String>,
    window: Window,
) -> AppResult<Vec<ImportResult>> {
    let mut results = Vec::new();

    for (index, file_path) in file_paths.iter().enumerate() {
        // 发送进度事件
        let _ = window.emit(
            "epub-import-progress",
            ImportProgress {
                current: index + 1,
                total: file_paths.len(),
                file_name: Path::new(file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string(),
            },
        );

        // 导入单个文件
        match import_epub(pool.clone(), workspace_path.clone(), file_path.clone()).await {
            Ok(book_id) => results.push(ImportResult::Success { book_id }),
            Err(e) => results.push(ImportResult::Failed {
                file_path: file_path.clone(),
                error: e.to_string(),
            }),
        }
    }

    Ok(results)
}
```

- [ ] **Step 4: 实现书籍查询命令**

```rust
/// 获取书籍详情（包含作者和标签）
#[tauri::command]
pub async fn get_epub_book(
    pool: State<'_, SqlitePool>,
    book_id: i64,
) -> AppResult<Option<EpubBookWithDetails>> {
    let db = EpubDatabase::new(pool.inner().clone());

    let book = db.get_book(book_id).await?;
    if book.is_none() {
        return Ok(None);
    }

    let book = book.unwrap();
    let authors = db.get_book_authors(book_id).await?;
    let tags = db.get_book_tags(book_id).await?;

    Ok(Some(EpubBookWithDetails {
        book,
        authors,
        tags,
    }))
}

/// 列出所有书籍
#[tauri::command]
pub async fn list_epub_books(pool: State<'_, SqlitePool>) -> AppResult<Vec<EpubBook>> {
    let db = EpubDatabase::new(pool.inner().clone());
    db.list_books().await
}

/// 搜索书籍
#[tauri::command]
pub async fn search_epub_books(
    pool: State<'_, SqlitePool>,
    query: SearchQuery,
) -> AppResult<Vec<EpubBook>> {
    let db = EpubDatabase::new(pool.inner().clone());
    db.search_books(&query).await
}
```

- [ ] **Step 5: 实现删除命令**

```rust
/// 删除书籍
#[tauri::command]
pub async fn delete_epub_book(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    book_id: i64,
) -> AppResult<()> {
    let db = EpubDatabase::new(pool.inner().clone());
    let storage = EpubStorage::new(&workspace_path);

    // 删除文件
    storage.delete_book(book_id)?;

    // 删除数据库记录（级联删除关联数据）
    db.delete_book(book_id).await?;

    Ok(())
}
```

- [ ] **Step 6: 注册 Tauri 命令**

```rust
// src-tauri/src/lib.rs
mod modules {
    pub mod epub;
    // ... 其他模块
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ... 其他配置
        .invoke_handler(tauri::generate_handler![
            // ... 现有命令
            modules::epub::commands::import_epub,
            modules::epub::commands::batch_import_epub,
            modules::epub::commands::get_epub_book,
            modules::epub::commands::list_epub_books,
            modules::epub::commands::search_epub_books,
            modules::epub::commands::delete_epub_book,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 7: 编译检查**

Run: `cd src-tauri && cargo check`
Expected: 编译通过

- [ ] **Step 8: 测试命令**

Run: `cd src-tauri && cargo test`
Expected: 所有测试通过

- [ ] **Step 9: 提交命令处理器**

```bash
git add src-tauri/src/modules/epub/commands.rs src-tauri/src/lib.rs
git commit -m "feat(epub): add Tauri command handlers

- import_epub: Single file import
- batch_import_epub: Batch import with progress events
- get_epub_book: Get book with authors and tags
- list_epub_books: List all books
- search_epub_books: Search with filters
- delete_epub_book: Delete book and files

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## ⏸️ Checkpoint: Week 1 完成

此时应该完成：
- ✅ 数据库迁移文件
- ✅ 数据模型定义
- ✅ EPUB 解析器
- ✅ 存储管理器
- ✅ 数据库操作层
- ✅ Tauri 命令处理器

**验收测试**:
```bash
# 编译检查
cd src-tauri && cargo check

# 运行测试
cargo test

# 运行迁移
cargo sqlx migrate run

# 启动应用（验证命令注册）
cd .. && bun run tauri:dev
```

---

## Week 2: 前端界面

### Task 7: TypeScript 类型定义

**Files:**
- Create: `src/lib/types/epub.ts`
- Modify: `src/lib/types.ts` (导出)

- [ ] **Step 1: 创建 EPUB 类型文件**

```typescript
// src/lib/types/epub.ts

export interface EpubBook {
  id: number;
  title: string;
  sort_title: string | null;
  isbn: string | null;
  publisher: string | null;
  pubdate: string | null;
  language: string | null;
  series: string | null;
  series_index: number | null;
  rating: number | null;
  file_path: string;
  file_size: number;
  cover_path: string | null;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export interface Author {
  id: number;
  name: string;
  sort_name: string | null;
  created_at: string;
}

export interface Tag {
  id: number;
  name: string;
  created_at: string;
}

export interface EpubBookWithDetails {
  book: EpubBook;
  authors: Author[];
  tags: Tag[];
}

export interface SearchQuery {
  keyword?: string;
  title?: string;
  author?: string;
  publisher?: string;
  isbn?: string;
  series?: string;
  tags?: string[];
  rating_min?: number;
  rating_max?: number;
  sort_by?: string;
  sort_order?: 'ASC' | 'DESC';
  limit?: number;
  offset?: number;
}

export interface ImportResult {
  type: 'success' | 'failed';
  book_id?: number;
  file_path?: string;
  error?: string;
}

export interface ImportProgress {
  current: number;
  total: number;
  file_name: string;
}

export type ViewMode = 'grid' | 'list' | 'detail';
```

- [ ] **Step 2: 导出类型**

```typescript
// src/lib/types.ts
export * from './types/epub';
// ... 其他类型导出
```

- [ ] **Step 3: 提交类型定义**

```bash
git add src/lib/types/epub.ts src/lib/types.ts
git commit -m "feat(epub): add TypeScript type definitions

- EpubBook, Author, Tag interfaces
- SearchQuery and import-related types
- ViewMode enum

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 8: API 服务层

**Files:**
- Create: `src/lib/services/epub.ts`

- [ ] **Step 1: 创建 API 服务**

```typescript
// src/lib/services/epub.ts
import { invoke } from '@tauri-apps/api/core';
import type {
  EpubBook,
  EpubBookWithDetails,
  SearchQuery,
  ImportResult,
} from '$lib/types/epub';

export class EpubService {
  /**
   * 导入单个 EPUB 文件
   */
  static async importEpub(
    workspacePath: string,
    sourceFilePath: string
  ): Promise<number> {
    return await invoke<number>('import_epub', {
      workspacePath,
      sourceFilePath,
    });
  }

  /**
   * 批量导入 EPUB 文件
   */
  static async batchImportEpub(
    workspacePath: string,
    filePaths: string[]
  ): Promise<ImportResult[]> {
    return await invoke<ImportResult[]>('batch_import_epub', {
      workspacePath,
      filePaths,
    });
  }

  /**
   * 获取书籍详情
   */
  static async getBook(bookId: number): Promise<EpubBookWithDetails | null> {
    return await invoke<EpubBookWithDetails | null>('get_epub_book', {
      bookId,
    });
  }

  /**
   * 列出所有书籍
   */
  static async listBooks(): Promise<EpubBook[]> {
    return await invoke<EpubBook[]>('list_epub_books');
  }

  /**
   * 搜索书籍
   */
  static async searchBooks(query: SearchQuery): Promise<EpubBook[]> {
    return await invoke<EpubBook[]>('search_epub_books', { query });
  }

  /**
   * 删除书籍
   */
  static async deleteBook(
    workspacePath: string,
    bookId: number
  ): Promise<void> {
    await invoke('delete_epub_book', { workspacePath, bookId });
  }
}
```

- [ ] **Step 2: 提交 API 服务**

```bash
git add src/lib/services/epub.ts
git commit -m "feat(epub): add API service layer

- Wrapper for all Tauri EPUB commands
- Type-safe API calls
- Clean service interface

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 9: 书库主界面

**Files:**
- Create: `src/lib/components/epub/EpubLibrary.svelte`
- Create: `src/routes/epub/+page.svelte`

- [ ] **Step 1: 创建书库主组件**

```svelte
<!-- src/lib/components/epub/EpubLibrary.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { EpubService } from '$lib/services/epub';
  import type { EpubBook, ViewMode } from '$lib/types/epub';

  import BookGrid from './BookGrid.svelte';
  import BookList from './BookList.svelte';
  import BookDetailList from './BookDetailList.svelte';
  import BookSidebar from './BookSidebar.svelte';
  import SearchBar from './SearchBar.svelte';

  let books: EpubBook[] = $state([]);
  let selectedBook: EpubBook | null = $state(null);
  let viewMode: ViewMode = $state('grid');
  let loading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    await loadBooks();
  });

  async function loadBooks() {
    try {
      loading = true;
      error = null;
      books = await EpubService.listBooks();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load books';
    } finally {
      loading = false;
    }
  }

  function handleBookSelect(book: EpubBook) {
    selectedBook = book;
  }

  function handleBookDeselect() {
    selectedBook = null;
  }

  async function handleBookDeleted() {
    selectedBook = null;
    await loadBooks();
  }

  async function handleSearch(query: any) {
    try {
      loading = true;
      error = null;
      books = await EpubService.searchBooks(query);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Search failed';
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex h-screen flex-col bg-gray-50">
  <!-- 标题栏 -->
  <header class="border-b bg-white px-6 py-4">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-gray-900">EPUB 书库</h1>

      <div class="flex items-center gap-4">
        <!-- 导入按钮 -->
        <button
          class="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
          onclick={() => {/* TODO: 打开导入对话框 */}}
        >
          📥 导入书籍
        </button>

        <!-- 视图切换 -->
        <div class="flex rounded-lg border">
          <button
            class="px-3 py-2 {viewMode === 'grid' ? 'bg-blue-50' : ''}"
            onclick={() => (viewMode = 'grid')}
            title="网格视图"
          >
            ⊞
          </button>
          <button
            class="border-x px-3 py-2 {viewMode === 'list' ? 'bg-blue-50' : ''}"
            onclick={() => (viewMode = 'list')}
            title="列表视图"
          >
            ☰
          </button>
          <button
            class="px-3 py-2 {viewMode === 'detail' ? 'bg-blue-50' : ''}"
            onclick={() => (viewMode = 'detail')}
            title="详细列表视图"
          >
            ≡
          </button>
        </div>
      </div>
    </div>

    <!-- 搜索栏 -->
    <div class="mt-4">
      <SearchBar onSearch={handleSearch} />
    </div>
  </header>

  <!-- 主内容区 -->
  <div class="flex flex-1 overflow-hidden">
    <!-- 书籍展示区 -->
    <div class="flex-1 overflow-y-auto p-6">
      {#if loading}
        <div class="flex h-full items-center justify-center">
          <div class="text-gray-500">加载中...</div>
        </div>
      {:else if error}
        <div class="flex h-full items-center justify-center">
          <div class="text-red-500">{error}</div>
        </div>
      {:else if books.length === 0}
        <div class="flex h-full items-center justify-center">
          <div class="text-center text-gray-500">
            <p class="text-lg">暂无书籍</p>
            <p class="mt-2 text-sm">点击"导入书籍"开始添加</p>
          </div>
        </div>
      {:else}
        {#if viewMode === 'grid'}
          <BookGrid {books} onSelect={handleBookSelect} />
        {:else if viewMode === 'list'}
          <BookList {books} onSelect={handleBookSelect} />
        {:else}
          <BookDetailList {books} onSelect={handleBookSelect} />
        {/if}
      {/if}
    </div>

    <!-- 侧边栏 -->
    {#if selectedBook}
      <BookSidebar
        book={selectedBook}
        onClose={handleBookDeselect}
        onDeleted={handleBookDeleted}
      />
    {/if}
  </div>
</div>
```

- [ ] **Step 2: 创建路由页面**

```svelte
<!-- src/routes/epub/+page.svelte -->
<script lang="ts">
  import EpubLibrary from '$lib/components/epub/EpubLibrary.svelte';
</script>

<EpubLibrary />
```

- [ ] **Step 3: 提交主界面**

```bash
git add src/lib/components/epub/EpubLibrary.svelte src/routes/epub/+page.svelte
git commit -m "feat(epub): add library main interface

- Main layout with header, content, and sidebar
- View mode switcher (grid/list/detail)
- Book loading and error handling
- Search integration

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 10: 网格视图组件

**Files:**
- Create: `src/lib/components/epub/BookGrid.svelte`

- [ ] **Step 1: 创建网格视图组件**

```svelte
<!-- src/lib/components/epub/BookGrid.svelte -->
<script lang="ts">
  import type { EpubBook } from '$lib/types/epub';
  import { convertFileSrc } from '@tauri-apps/api/core';

  interface Props {
    books: EpubBook[];
    onSelect: (book: EpubBook) => void;
  }

  let { books, onSelect }: Props = $props();

  function getCoverUrl(book: EpubBook): string {
    if (book.cover_path) {
      return convertFileSrc(book.cover_path);
    }
    return '/placeholder-cover.jpg'; // TODO: 添加默认封面
  }

  function formatAuthors(book: EpubBook): string {
    // TODO: 从详情中获取作者列表
    return '作者信息加载中...';
  }

  function getRatingStars(rating: number | null): string {
    if (!rating) return '';
    return '★'.repeat(rating) + '☆'.repeat(5 - rating);
  }
</script>

<div class="grid grid-cols-2 gap-6 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
  {#each books as book (book.id)}
    <button
      class="group cursor-pointer overflow-hidden rounded-lg bg-white shadow transition hover:shadow-lg"
      onclick={() => onSelect(book)}
    >
      <!-- 封面 -->
      <div class="aspect-[2/3] overflow-hidden bg-gray-200">
        <img
          src={getCoverUrl(book)}
          alt={book.title}
          class="h-full w-full object-cover transition group-hover:scale-105"
        />
      </div>

      <!-- 信息 -->
      <div class="p-3">
        <!-- 标题 -->
        <h3 class="line-clamp-2 font-semibold text-gray-900" title={book.title}>
          {book.title}
        </h3>

        <!-- 作者 -->
        <p class="mt-1 line-clamp-1 text-sm text-gray-600">
          {formatAuthors(book)}
        </p>

        <!-- 系列 -->
        {#if book.series}
          <p class="mt-1 line-clamp-1 text-xs text-gray-500">
            {book.series}
            {#if book.series_index}#{book.series_index}{/if}
          </p>
        {/if}

        <!-- 评分 -->
        {#if book.rating}
          <div class="mt-2 text-sm text-yellow-500">
            {getRatingStars(book.rating)}
          </div>
        {/if}
      </div>
    </button>
  {/each}
</div>
```

- [ ] **Step 2: 添加默认封面**

创建或下载一个默认封面图片并放置在 `static/placeholder-cover.jpg`

- [ ] **Step 3: 提交网格视图**

```bash
git add src/lib/components/epub/BookGrid.svelte
git commit -m "feat(epub): add grid view component

- Card-based layout with cover images
- Display title, authors, series, rating
- Hover effects and responsive grid

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 11: 列表视图和详细列表视图

**Files:**
- Create: `src/lib/components/epub/BookList.svelte`
- Create: `src/lib/components/epub/BookDetailList.svelte`

- [ ] **Step 1: 创建列表视图组件**

```svelte
<!-- src/lib/components/epub/BookList.svelte -->
<script lang="ts">
  import type { EpubBook } from '$lib/types/epub';
  import { convertFileSrc } from '@tauri-apps/api/core';

  interface Props {
    books: EpubBook[];
    onSelect: (book: EpubBook) => void;
  }

  let { books, onSelect }: Props = $props();

  function getCoverUrl(book: EpubBook): string {
    if (book.cover_path) {
      return convertFileSrc(book.cover_path);
    }
    return '/placeholder-cover.jpg';
  }

  function getRatingStars(rating: number | null): string {
    if (!rating) return '';
    return '★'.repeat(rating) + '☆'.repeat(5 - rating);
  }
</script>

<div class="space-y-4">
  {#each books as book (book.id)}
    <button
      class="group flex w-full items-center gap-4 rounded-lg bg-white p-4 shadow transition hover:shadow-lg"
      onclick={() => onSelect(book)}
    >
      <!-- 封面缩略图 -->
      <div class="h-32 w-24 flex-shrink-0 overflow-hidden rounded bg-gray-200">
        <img
          src={getCoverUrl(book)}
          alt={book.title}
          class="h-full w-full object-cover"
        />
      </div>

      <!-- 信息 -->
      <div class="flex-1 text-left">
        <!-- 标题 -->
        <h3 class="text-lg font-semibold text-gray-900">
          {book.title}
        </h3>

        <!-- 作者 -->
        <p class="mt-1 text-sm text-gray-600">
          作者信息加载中...
        </p>

        <!-- 系列 -->
        {#if book.series}
          <p class="mt-1 text-sm text-gray-500">
            系列: {book.series}
            {#if book.series_index}(#{book.series_index}){/if}
          </p>
        {/if}

        <!-- 标签 -->
        <div class="mt-2 flex flex-wrap gap-2">
          <!-- TODO: 显示标签 -->
        </div>

        <!-- 评分 -->
        {#if book.rating}
          <div class="mt-2 text-sm text-yellow-500">
            {getRatingStars(book.rating)}
          </div>
        {/if}
      </div>

      <!-- 元数据（右侧） -->
      <div class="flex-shrink-0 text-right text-sm text-gray-500">
        {#if book.publisher}
          <div>{book.publisher}</div>
        {/if}
        {#if book.pubdate}
          <div class="mt-1">{book.pubdate}</div>
        {/if}
      </div>
    </button>
  {/each}
</div>
```

- [ ] **Step 2: 创建详细列表视图组件**

```svelte
<!-- src/lib/components/epub/BookDetailList.svelte -->
<script lang="ts">
  import type { EpubBook } from '$lib/types/epub';
  import { convertFileSrc } from '@tauri-apps/api/core';

  interface Props {
    books: EpubBook[];
    onSelect: (book: EpubBook) => void;
  }

  let { books, onSelect }: Props = $props();

  function getCoverUrl(book: EpubBook): string {
    if (book.cover_path) {
      return convertFileSrc(book.cover_path);
    }
    return '/placeholder-cover.jpg';
  }

  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('zh-CN');
  }

  function getRatingStars(rating: number | null): string {
    if (!rating) return '-';
    return '★'.repeat(rating) + '☆'.repeat(5 - rating);
  }
</script>

<div class="overflow-x-auto rounded-lg bg-white shadow">
  <table class="w-full">
    <thead class="bg-gray-50">
      <tr>
        <th class="px-4 py-3 text-left text-sm font-semibold text-gray-900">封面</th>
        <th class="px-4 py-3 text-left text-sm font-semibold text-gray-900">标题</th>
        <th class="px-4 py-3 text-left text-sm font-semibold text-gray-900">作者</th>
        <th class="px-4 py-3 text-left text-sm font-semibold text-gray-900">系列</th>
        <th class="px-4 py-3 text-left text-sm font-semibold text-gray-900">出版社</th>
        <th class="px-4 py-3 text-left text-sm font-semibold text-gray-900">评分</th>
        <th class="px-4 py-3 text-left text-sm font-semibold text-gray-900">标签</th>
        <th class="px-4 py-3 text-left text-sm font-semibold text-gray-900">添加日期</th>
        <th class="px-4 py-3 text-left text-sm font-semibold text-gray-900">大小</th>
      </tr>
    </thead>
    <tbody class="divide-y divide-gray-200">
      {#each books as book (book.id)}
        <tr
          class="cursor-pointer transition hover:bg-gray-50"
          onclick={() => onSelect(book)}
        >
          <!-- 封面 -->
          <td class="px-4 py-3">
            <div class="h-16 w-12 overflow-hidden rounded bg-gray-200">
              <img
                src={getCoverUrl(book)}
                alt={book.title}
                class="h-full w-full object-cover"
              />
            </div>
          </td>

          <!-- 标题 -->
          <td class="px-4 py-3">
            <div class="max-w-xs truncate font-medium text-gray-900" title={book.title}>
              {book.title}
            </div>
          </td>

          <!-- 作者 -->
          <td class="px-4 py-3">
            <div class="max-w-xs truncate text-sm text-gray-600">
              作者加载中...
            </div>
          </td>

          <!-- 系列 -->
          <td class="px-4 py-3">
            {#if book.series}
              <div class="max-w-xs truncate text-sm text-gray-600">
                {book.series}
                {#if book.series_index}#{book.series_index}{/if}
              </div>
            {:else}
              <span class="text-gray-400">-</span>
            {/if}
          </td>

          <!-- 出版社 -->
          <td class="px-4 py-3">
            <div class="max-w-xs truncate text-sm text-gray-600">
              {book.publisher || '-'}
            </div>
          </td>

          <!-- 评分 -->
          <td class="px-4 py-3">
            <div class="text-sm text-yellow-500">
              {getRatingStars(book.rating)}
            </div>
          </td>

          <!-- 标签 -->
          <td class="px-4 py-3">
            <div class="flex flex-wrap gap-1">
              <!-- TODO: 显示标签 -->
              <span class="text-sm text-gray-400">-</span>
            </div>
          </td>

          <!-- 添加日期 -->
          <td class="px-4 py-3">
            <div class="text-sm text-gray-600">
              {formatDate(book.created_at)}
            </div>
          </td>

          <!-- 文件大小 -->
          <td class="px-4 py-3">
            <div class="text-sm text-gray-600">
              {formatFileSize(book.file_size)}
            </div>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
```

- [ ] **Step 3: 提交列表视图组件**

```bash
git add src/lib/components/epub/BookList.svelte src/lib/components/epub/BookDetailList.svelte
git commit -m "feat(epub): add list and detail list view components

- BookList: horizontal card layout
- BookDetailList: table layout with all metadata
- Responsive and sortable design

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 12: 搜索栏组件

**Files:**
- Create: `src/lib/components/epub/SearchBar.svelte`

- [ ] **Step 1: 创建搜索栏组件**

```svelte
<!-- src/lib/components/epub/SearchBar.svelte -->
<script lang="ts">
  import type { SearchQuery } from '$lib/types/epub';

  interface Props {
    onSearch: (query: SearchQuery) => void;
  }

  let { onSearch }: Props = $props();

  let keyword = $state('');
  let showAdvanced = $state(false);

  // 高级搜索字段
  let advancedQuery = $state<SearchQuery>({
    title: '',
    author: '',
    publisher: '',
    series: '',
    rating_min: undefined,
    rating_max: undefined,
  });

  function handleBasicSearch() {
    if (keyword.trim()) {
      onSearch({ keyword: keyword.trim() });
    } else {
      onSearch({});
    }
  }

  function handleAdvancedSearch() {
    const query: SearchQuery = {};

    if (advancedQuery.title) query.title = advancedQuery.title;
    if (advancedQuery.author) query.author = advancedQuery.author;
    if (advancedQuery.publisher) query.publisher = advancedQuery.publisher;
    if (advancedQuery.series) query.series = advancedQuery.series;
    if (advancedQuery.rating_min !== undefined) query.rating_min = advancedQuery.rating_min;
    if (advancedQuery.rating_max !== undefined) query.rating_max = advancedQuery.rating_max;

    onSearch(query);
    showAdvanced = false;
  }

  function handleClearAdvanced() {
    advancedQuery = {
      title: '',
      author: '',
      publisher: '',
      series: '',
      rating_min: undefined,
      rating_max: undefined,
    };
    onSearch({});
  }
</script>

<div class="relative">
  <!-- 基础搜索栏 -->
  <div class="flex gap-2">
    <div class="relative flex-1">
      <input
        type="text"
        bind:value={keyword}
        placeholder="搜索书名、作者、出版社..."
        class="w-full rounded-lg border border-gray-300 px-4 py-2 pr-10 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
        onkeydown={(e) => e.key === 'Enter' && handleBasicSearch()}
      />
      <button
        class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
        onclick={handleBasicSearch}
      >
        🔍
      </button>
    </div>

    <button
      class="rounded-lg border border-gray-300 px-4 py-2 hover:bg-gray-50"
      onclick={() => (showAdvanced = !showAdvanced)}
    >
      高级搜索
    </button>
  </div>

  <!-- 高级搜索面板 -->
  {#if showAdvanced}
    <div class="absolute left-0 right-0 top-full z-10 mt-2 rounded-lg border bg-white p-6 shadow-lg">
      <h3 class="mb-4 text-lg font-semibold">高级搜索</h3>

      <div class="grid grid-cols-2 gap-4">
        <!-- 标题 -->
        <div>
          <label class="mb-1 block text-sm font-medium text-gray-700">标题</label>
          <input
            type="text"
            bind:value={advancedQuery.title}
            placeholder="输入标题关键词"
            class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <!-- 作者 -->
        <div>
          <label class="mb-1 block text-sm font-medium text-gray-700">作者</label>
          <input
            type="text"
            bind:value={advancedQuery.author}
            placeholder="输入作者名称"
            class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <!-- 出版社 -->
        <div>
          <label class="mb-1 block text-sm font-medium text-gray-700">出版社</label>
          <input
            type="text"
            bind:value={advancedQuery.publisher}
            placeholder="输入出版社名称"
            class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <!-- 系列 -->
        <div>
          <label class="mb-1 block text-sm font-medium text-gray-700">系列</label>
          <input
            type="text"
            bind:value={advancedQuery.series}
            placeholder="输入系列名称"
            class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          />
        </div>

        <!-- 评分范围 -->
        <div>
          <label class="mb-1 block text-sm font-medium text-gray-700">最低评分</label>
          <select
            bind:value={advancedQuery.rating_min}
            class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            <option value={undefined}>不限</option>
            <option value={1}>1 星</option>
            <option value={2}>2 星</option>
            <option value={3}>3 星</option>
            <option value={4}>4 星</option>
            <option value={5}>5 星</option>
          </select>
        </div>

        <div>
          <label class="mb-1 block text-sm font-medium text-gray-700">最高评分</label>
          <select
            bind:value={advancedQuery.rating_max}
            class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          >
            <option value={undefined}>不限</option>
            <option value={1}>1 星</option>
            <option value={2}>2 星</option>
            <option value={3}>3 星</option>
            <option value={4}>4 星</option>
            <option value={5}>5 星</option>
          </select>
        </div>
      </div>

      <!-- 按钮 -->
      <div class="mt-6 flex justify-end gap-2">
        <button
          class="rounded-lg border border-gray-300 px-4 py-2 hover:bg-gray-50"
          onclick={handleClearAdvanced}
        >
          清除
        </button>
        <button
          class="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
          onclick={handleAdvancedSearch}
        >
          搜索
        </button>
      </div>
    </div>
  {/if}
</div>
```

- [ ] **Step 2: 提交搜索栏组件**

```bash
git add src/lib/components/epub/SearchBar.svelte
git commit -m "feat(epub): add search bar component

- Basic keyword search
- Advanced search panel with multiple fields
- Rating range filter
- Clear and search actions

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 13: 侧边栏详情组件

**Files:**
- Create: `src/lib/components/epub/BookSidebar.svelte`

- [ ] **Step 1: 创建侧边栏组件（查看模式）**

```svelte
<!-- src/lib/components/epub/BookSidebar.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { EpubService } from '$lib/services/epub';
  import type { EpubBook, EpubBookWithDetails } from '$lib/types/epub';

  interface Props {
    book: EpubBook;
    onClose: () => void;
    onDeleted: () => void;
  }

  let { book, onClose, onDeleted }: Props = $props();

  let bookDetails: EpubBookWithDetails | null = $state(null);
  let editMode = $state(false);
  let loading = $state(true);

  onMount(async () => {
    await loadBookDetails();
  });

  async function loadBookDetails() {
    try {
      loading = true;
      const details = await EpubService.getBook(book.id);
      if (details) {
        bookDetails = details;
      }
    } catch (e) {
      console.error('Failed to load book details:', e);
    } finally {
      loading = false;
    }
  }

  function getCoverUrl(): string {
    if (book.cover_path) {
      return convertFileSrc(book.cover_path);
    }
    return '/placeholder-cover.jpg';
  }

  function getRatingStars(rating: number | null): string {
    if (!rating) return '未评分';
    return '★'.repeat(rating) + '☆'.repeat(5 - rating);
  }

  async function handleDelete() {
    if (!confirm(`确定要删除《${book.title}》吗？此操作不可恢复。`)) {
      return;
    }

    try {
      // TODO: 获取 workspace_path
      const workspacePath = ''; // 从 store 或 context 获取
      await EpubService.deleteBook(workspacePath, book.id);
      onDeleted();
    } catch (e) {
      alert(`删除失败: ${e}`);
    }
  }

  function handleStartReading() {
    // TODO: 打开阅读器
    console.log('Start reading:', book.id);
  }
</script>

<div class="flex h-full w-96 flex-col border-l bg-white shadow-lg">
  <!-- 标题栏 -->
  <div class="flex items-center justify-between border-b px-4 py-3">
    <h2 class="text-lg font-semibold">书籍详情</h2>
    <button
      class="text-gray-400 hover:text-gray-600"
      onclick={onClose}
    >
      ✕
    </button>
  </div>

  <!-- 内容区 -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if loading}
      <div class="flex h-full items-center justify-center">
        <div class="text-gray-500">加载中...</div>
      </div>
    {:else if bookDetails}
      {#if !editMode}
        <!-- 查看模式 -->
        <div class="space-y-6">
          <!-- 封面 -->
          <div class="flex justify-center">
            <img
              src={getCoverUrl()}
              alt={book.title}
              class="max-h-80 rounded-lg shadow-md"
            />
          </div>

          <!-- 标题 -->
          <div>
            <h3 class="text-xl font-bold text-gray-900">{book.title}</h3>
            {#if book.sort_title}
              <p class="mt-1 text-sm text-gray-500">排序标题: {book.sort_title}</p>
            {/if}
          </div>

          <!-- 作者 -->
          {#if bookDetails.authors.length > 0}
            <div>
              <label class="text-sm font-medium text-gray-700">作者</label>
              <div class="mt-1">
                {#each bookDetails.authors as author, i}
                  <span class="text-gray-900">
                    {author.name}{i < bookDetails.authors.length - 1 ? ', ' : ''}
                  </span>
                {/each}
              </div>
            </div>
          {/if}

          <!-- 系列 -->
          {#if book.series}
            <div>
              <label class="text-sm font-medium text-gray-700">系列</label>
              <div class="mt-1 text-gray-900">
                {book.series}
                {#if book.series_index}(第 {book.series_index} 册){/if}
              </div>
            </div>
          {/if}

          <!-- 出版信息 -->
          {#if book.publisher || book.pubdate}
            <div>
              <label class="text-sm font-medium text-gray-700">出版信息</label>
              <div class="mt-1 text-gray-900">
                {#if book.publisher}{book.publisher}{/if}
                {#if book.publisher && book.pubdate}, {/if}
                {#if book.pubdate}{book.pubdate}{/if}
              </div>
            </div>
          {/if}

          <!-- ISBN -->
          {#if book.isbn}
            <div>
              <label class="text-sm font-medium text-gray-700">ISBN</label>
              <div class="mt-1 text-gray-900">{book.isbn}</div>
            </div>
          {/if}

          <!-- 语言 -->
          {#if book.language}
            <div>
              <label class="text-sm font-medium text-gray-700">语言</label>
              <div class="mt-1 text-gray-900">{book.language}</div>
            </div>
          {/if}

          <!-- 评分 -->
          <div>
            <label class="text-sm font-medium text-gray-700">评分</label>
            <div class="mt-1 text-yellow-500">
              {getRatingStars(book.rating)}
            </div>
          </div>

          <!-- 标签 -->
          {#if bookDetails.tags.length > 0}
            <div>
              <label class="text-sm font-medium text-gray-700">标签</label>
              <div class="mt-2 flex flex-wrap gap-2">
                {#each bookDetails.tags as tag}
                  <span class="rounded-full bg-blue-100 px-3 py-1 text-sm text-blue-700">
                    {tag.name}
                  </span>
                {/each}
              </div>
            </div>
          {/if}

          <!-- 简介 -->
          {#if book.description}
            <div>
              <label class="text-sm font-medium text-gray-700">简介</label>
              <div class="mt-1 text-sm text-gray-900 whitespace-pre-wrap">
                {book.description}
              </div>
            </div>
          {/if}

          <!-- 文件信息 -->
          <div class="border-t pt-4 text-sm text-gray-500">
            <div>文件大小: {(book.file_size / (1024 * 1024)).toFixed(2)} MB</div>
            <div class="mt-1">添加时间: {new Date(book.created_at).toLocaleString('zh-CN')}</div>
          </div>
        </div>
      {:else}
        <!-- 编辑模式 - TODO: Week 3 实现 -->
        <div class="text-center text-gray-500">编辑模式（Week 3 实现）</div>
      {/if}
    {/if}
  </div>

  <!-- 底部按钮 -->
  <div class="border-t px-6 py-4">
    <div class="flex gap-2">
      <button
        class="flex-1 rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
        onclick={handleStartReading}
      >
        开始阅读
      </button>
      {#if !editMode}
        <button
          class="rounded-lg border border-gray-300 px-4 py-2 hover:bg-gray-50"
          onclick={() => (editMode = true)}
        >
          编辑
        </button>
      {/if}
    </div>

    {#if !editMode}
      <button
        class="mt-2 w-full rounded-lg border border-red-300 px-4 py-2 text-red-600 hover:bg-red-50"
        onclick={handleDelete}
      >
        删除书籍
      </button>
    {/if}
  </div>
</div>
```

- [ ] **Step 2: 提交侧边栏组件**

```bash
git add src/lib/components/epub/BookSidebar.svelte
git commit -m "feat(epub): add book sidebar component (view mode)

- Display full book details with cover
- Show authors, tags, series, rating
- Start reading button
- Edit and delete actions
- Responsive layout

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## ⏸️ Checkpoint: Week 2 完成

此时应该完成：
- ✅ TypeScript 类型定义
- ✅ API 服务层
- ✅ 书库主界面
- ✅ 网格视图
- ✅ 列表视图和详细列表视图
- ✅ 搜索栏
- ✅ 侧边栏详情（查看模式）

**验收测试**:
```bash
# 类型检查
bun run check

# 启动开发服务器
bun run dev

# 测试功能:
# 1. 访问 /epub 路由
# 2. 切换三种视图模式
# 3. 搜索书籍
# 4. 点击书籍查看详情
# 5. 关闭侧边栏
```

---

## Week 3: 元数据编辑

### Task 14: 元数据编辑器组件

**Files:**
- Create: `src/lib/components/epub/MetadataEditor.svelte`
- Modify: `src/lib/components/epub/BookSidebar.svelte` (集成编辑器)

- [ ] **Step 1: 创建元数据编辑器组件**

```svelte
<!-- src/lib/components/epub/MetadataEditor.svelte -->
<script lang="ts">
  import type { EpubBook, Author, Tag } from '$lib/types/epub';

  interface Props {
    book: EpubBook;
    authors: Author[];
    tags: Tag[];
    onSave: (updatedData: any) => Promise<void>;
    onCancel: () => void;
  }

  let { book, authors, tags, onSave, onCancel }: Props = $props();

  // 编辑状态
  let editedBook = $state({ ...book });
  let editedAuthors = $state([...authors.map(a => a.name)]);
  let editedTags = $state([...tags.map(t => t.name)]);
  let saving = $state(false);

  // 临时输入
  let newAuthor = $state('');
  let newTag = $state('');

  function addAuthor() {
    if (newAuthor.trim() && !editedAuthors.includes(newAuthor.trim())) {
      editedAuthors = [...editedAuthors, newAuthor.trim()];
      newAuthor = '';
    }
  }

  function removeAuthor(index: number) {
    editedAuthors = editedAuthors.filter((_, i) => i !== index);
  }

  function addTag() {
    if (newTag.trim() && !editedTags.includes(newTag.trim())) {
      editedTags = [...editedTags, newTag.trim()];
      newTag = '';
    }
  }

  function removeTag(index: number) {
    editedTags = editedTags.filter((_, i) => i !== index);
  }

  async function handleSave() {
    try {
      saving = true;
      await onSave({
        book: editedBook,
        authors: editedAuthors,
        tags: editedTags,
      });
    } catch (e) {
      alert(`保存失败: ${e}`);
    } finally {
      saving = false;
    }
  }
</script>

<div class="space-y-4">
  <!-- 标题 -->
  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">
      标题 <span class="text-red-500">*</span>
    </label>
    <input
      type="text"
      bind:value={editedBook.title}
      placeholder="输入书名"
      class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
      required
    />
  </div>

  <!-- 排序标题 -->
  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">排序标题</label>
    <input
      type="text"
      bind:value={editedBook.sort_title}
      placeholder="用于排序的标题（可选）"
      class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
  </div>

  <!-- 作者 -->
  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">作者</label>
    <div class="space-y-2">
      {#each editedAuthors as author, i}
        <div class="flex items-center gap-2">
          <span class="flex-1 rounded border border-gray-300 bg-gray-50 px-3 py-2">
            {author}
          </span>
          <button
            class="text-red-500 hover:text-red-700"
            onclick={() => removeAuthor(i)}
          >
            ✕
          </button>
        </div>
      {/each}

      <div class="flex gap-2">
        <input
          type="text"
          bind:value={newAuthor}
          placeholder="添加作者"
          class="flex-1 rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          onkeydown={(e) => e.key === 'Enter' && addAuthor()}
        />
        <button
          class="rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
          onclick={addAuthor}
        >
          添加
        </button>
      </div>
    </div>
  </div>

  <!-- 系列 -->
  <div class="grid grid-cols-2 gap-4">
    <div>
      <label class="mb-1 block text-sm font-medium text-gray-700">系列</label>
      <input
        type="text"
        bind:value={editedBook.series}
        placeholder="系列名称"
        class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
      />
    </div>
    <div>
      <label class="mb-1 block text-sm font-medium text-gray-700">系列序号</label>
      <input
        type="number"
        bind:value={editedBook.series_index}
        placeholder="1"
        step="0.1"
        class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
      />
    </div>
  </div>

  <!-- 出版信息 -->
  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">出版社</label>
    <input
      type="text"
      bind:value={editedBook.publisher}
      placeholder="出版社名称"
      class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
  </div>

  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">出版日期</label>
    <input
      type="date"
      bind:value={editedBook.pubdate}
      class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
  </div>

  <!-- ISBN -->
  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">ISBN</label>
    <input
      type="text"
      bind:value={editedBook.isbn}
      placeholder="ISBN 号码"
      class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
    />
  </div>

  <!-- 语言 -->
  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">语言</label>
    <select
      bind:value={editedBook.language}
      class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
    >
      <option value="">请选择</option>
      <option value="zh">中文</option>
      <option value="en">English</option>
      <option value="ja">日本語</option>
      <option value="fr">Français</option>
      <option value="de">Deutsch</option>
      <option value="es">Español</option>
    </select>
  </div>

  <!-- 评分 -->
  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">评分</label>
    <div class="flex gap-2">
      {#each [1, 2, 3, 4, 5] as star}
        <button
          class="text-2xl {editedBook.rating && editedBook.rating >= star ? 'text-yellow-500' : 'text-gray-300'} hover:text-yellow-500"
          onclick={() => (editedBook.rating = star)}
        >
          ★
        </button>
      {/each}
      {#if editedBook.rating}
        <button
          class="ml-2 text-sm text-gray-500 hover:text-gray-700"
          onclick={() => (editedBook.rating = null)}
        >
          清除
        </button>
      {/if}
    </div>
  </div>

  <!-- 标签 -->
  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">标签</label>
    <div class="space-y-2">
      <div class="flex flex-wrap gap-2">
        {#each editedTags as tag, i}
          <span class="flex items-center gap-1 rounded-full bg-blue-100 px-3 py-1 text-sm text-blue-700">
            {tag}
            <button
              class="hover:text-blue-900"
              onclick={() => removeTag(i)}
            >
              ✕
            </button>
          </span>
        {/each}
      </div>

      <div class="flex gap-2">
        <input
          type="text"
          bind:value={newTag}
          placeholder="添加标签"
          class="flex-1 rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
          onkeydown={(e) => e.key === 'Enter' && addTag()}
        />
        <button
          class="rounded bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
          onclick={addTag}
        >
          添加
        </button>
      </div>
    </div>
  </div>

  <!-- 简介 -->
  <div>
    <label class="mb-1 block text-sm font-medium text-gray-700">简介</label>
    <textarea
      bind:value={editedBook.description}
      placeholder="书籍简介"
      rows="6"
      class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
    ></textarea>
  </div>

  <!-- 按钮 -->
  <div class="flex gap-2 pt-4">
    <button
      class="flex-1 rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:bg-gray-400"
      onclick={handleSave}
      disabled={saving || !editedBook.title.trim()}
    >
      {saving ? '保存中...' : '保存'}
    </button>
    <button
      class="rounded-lg border border-gray-300 px-4 py-2 hover:bg-gray-50"
      onclick={onCancel}
      disabled={saving}
    >
      取消
    </button>
  </div>
</div>
```

- [ ] **Step 2: 集成到侧边栏**

```svelte
<!-- 修改 src/lib/components/epub/BookSidebar.svelte -->
<script lang="ts">
  // ... 现有代码
  import MetadataEditor from './MetadataEditor.svelte';

  async function handleSaveMetadata(updatedData: any) {
    try {
      // TODO: 调用保存 API
      console.log('Save metadata:', updatedData);

      // 刷新数据
      await loadBookDetails();
      editMode = false;
    } catch (e) {
      throw e;
    }
  }
</script>

<!-- 在编辑模式部分替换为: -->
{#if editMode && bookDetails}
  <MetadataEditor
    book={bookDetails.book}
    authors={bookDetails.authors}
    tags={bookDetails.tags}
    onSave={handleSaveMetadata}
    onCancel={() => (editMode = false)}
  />
{/if}
```

- [ ] **Step 3: 提交元数据编辑器**

```bash
git add src/lib/components/epub/MetadataEditor.svelte src/lib/components/epub/BookSidebar.svelte
git commit -m "feat(epub): add metadata editor component

- Edit all book metadata fields
- Add/remove authors and tags
- Star rating selector
- Form validation
- Save and cancel actions

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 15: 后端元数据更新 API

**Files:**
- Modify: `src-tauri/src/modules/epub/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 添加元数据更新命令**

```rust
// src-tauri/src/modules/epub/commands.rs

#[derive(Debug, Deserialize)]
pub struct UpdateMetadataRequest {
    pub title: String,
    pub sort_title: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub pubdate: Option<String>,
    pub language: Option<String>,
    pub series: Option<String>,
    pub series_index: Option<f32>,
    pub rating: Option<i32>,
    pub description: Option<String>,
}

/// 更新书籍元数据
#[tauri::command]
pub async fn update_epub_metadata(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    metadata: UpdateMetadataRequest,
) -> AppResult<()> {
    let db = EpubDatabase::new(pool.inner().clone());

    let now = chrono::Utc::now().to_rfc3339();
    let mut book = db
        .get_book(book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Book {} not found", book_id)))?;

    // 更新字段
    book.title = metadata.title;
    book.sort_title = metadata.sort_title;
    book.isbn = metadata.isbn;
    book.publisher = metadata.publisher;
    book.pubdate = metadata.pubdate;
    book.language = metadata.language;
    book.series = metadata.series;
    book.series_index = metadata.series_index;
    book.rating = metadata.rating;
    book.description = metadata.description;
    book.updated_at = now;

    db.update_book(book_id, &book).await?;

    Ok(())
}

/// 设置书籍作者
#[tauri::command]
pub async fn set_epub_book_authors(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    author_names: Vec<String>,
) -> AppResult<()> {
    let db = EpubDatabase::new(pool.inner().clone());

    let authors: Vec<(String, Option<String>, i32)> = author_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| (name, None, i as i32))
        .collect();

    db.set_book_authors(book_id, authors).await?;

    Ok(())
}

/// 设置书籍标签
#[tauri::command]
pub async fn set_epub_book_tags(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    tag_names: Vec<String>,
) -> AppResult<()> {
    let db = EpubDatabase::new(pool.inner().clone());
    db.set_book_tags(book_id, tag_names).await?;
    Ok(())
}
```

- [ ] **Step 2: 注册新命令**

```rust
// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    modules::epub::commands::update_epub_metadata,
    modules::epub::commands::set_epub_book_authors,
    modules::epub::commands::set_epub_book_tags,
])
```

- [ ] **Step 3: 更新前端 API 服务**

```typescript
// src/lib/services/epub.ts

/**
 * 更新书籍元数据
 */
static async updateMetadata(
  bookId: number,
  metadata: Partial<EpubBook>
): Promise<void> {
  await invoke('update_epub_metadata', { bookId, metadata });
}

/**
 * 设置书籍作者
 */
static async setAuthors(
  bookId: number,
  authorNames: string[]
): Promise<void> {
  await invoke('set_epub_book_authors', { bookId, authorNames });
}

/**
 * 设置书籍标签
 */
static async setTags(bookId: number, tagNames: string[]): Promise<void> {
  await invoke('set_epub_book_tags', { bookId, tagNames });
}
```

- [ ] **Step 4: 连接前端保存逻辑**

```svelte
<!-- src/lib/components/epub/BookSidebar.svelte -->
<script lang="ts">
  async function handleSaveMetadata(updatedData: any) {
    try {
      // 更新书籍元数据
      await EpubService.updateMetadata(book.id, updatedData.book);

      // 更新作者
      await EpubService.setAuthors(book.id, updatedData.authors);

      // 更新标签
      await EpubService.setTags(book.id, updatedData.tags);

      // 刷新数据
      await loadBookDetails();
      editMode = false;
    } catch (e) {
      throw e;
    }
  }
</script>
```

- [ ] **Step 5: 编译和测试**

```bash
# 编译检查
cd src-tauri && cargo check

# 类型检查
cd .. && bun run check

# 启动应用测试
bun run tauri:dev
```

- [ ] **Step 6: 提交元数据更新 API**

```bash
git add src-tauri/src/modules/epub/commands.rs src-tauri/src/lib.rs src/lib/services/epub.ts src/lib/components/epub/BookSidebar.svelte
git commit -m "feat(epub): add metadata update APIs

Backend:
- update_epub_metadata: Update book metadata
- set_epub_book_authors: Set book authors
- set_epub_book_tags: Set book tags

Frontend:
- Connect editor to APIs
- Save all metadata changes

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 16: 封面上传功能

**Files:**
- Modify: `src-tauri/src/modules/epub/commands.rs`
- Modify: `src/lib/services/epub.ts`
- Modify: `src/lib/components/epub/MetadataEditor.svelte`

- [ ] **Step 1: 后端封面更新命令**

```rust
// src-tauri/src/modules/epub/commands.rs

/// 更新书籍封面
#[tauri::command]
pub async fn update_epub_cover(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    book_id: i64,
    cover_file_path: String,
) -> AppResult<()> {
    let db = EpubDatabase::new(pool.inner().clone());
    let storage = EpubStorage::new(&workspace_path);

    // 读取新封面文件
    let cover_data = std::fs::read(&cover_file_path)
        .map_err(|e| AppError::Io(format!("Failed to read cover file: {}", e)))?;

    // 保存封面（覆盖旧封面）
    storage.save_cover(&cover_data, book_id)?;

    // 更新数据库路径
    let cover_path = storage.cover_path(book_id).to_string_lossy().to_string();
    let mut book = db
        .get_book(book_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Book {} not found", book_id)))?;

    book.cover_path = Some(cover_path);
    book.updated_at = chrono::Utc::now().to_rfc3339();
    db.update_book(book_id, &book).await?;

    Ok(())
}
```

- [ ] **Step 2: 注册封面更新命令**

```rust
// src-tauri/src/lib.rs
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    modules::epub::commands::update_epub_cover,
])
```

- [ ] **Step 3: 前端 API 服务**

```typescript
// src/lib/services/epub.ts

/**
 * 更新书籍封面
 */
static async updateCover(
  workspacePath: string,
  bookId: number,
  coverFilePath: string
): Promise<void> {
  await invoke('update_epub_cover', { workspacePath, bookId, coverFilePath });
}
```

- [ ] **Step 4: 添加封面上传 UI**

```svelte
<!-- src/lib/components/epub/MetadataEditor.svelte -->
<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';

  let coverFile = $state<string | null>(null);

  async function handleCoverUpload() {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'Images',
        extensions: ['jpg', 'jpeg', 'png', 'webp']
      }]
    });

    if (selected) {
      coverFile = selected;
    }
  }
</script>

<!-- 在元数据表单开头添加: -->
<div>
  <label class="mb-1 block text-sm font-medium text-gray-700">封面</label>
  <div class="flex items-center gap-4">
    <img
      src={getCoverUrl()}
      alt="Current cover"
      class="h-32 w-24 rounded object-cover"
    />
    <div class="flex-1">
      <button
        type="button"
        class="rounded bg-gray-600 px-4 py-2 text-white hover:bg-gray-700"
        onclick={handleCoverUpload}
      >
        选择新封面
      </button>
      {#if coverFile}
        <p class="mt-2 text-sm text-green-600">
          已选择新封面
        </p>
      {/if}
    </div>
  </div>
</div>
```

- [ ] **Step 5: 保存时上传封面**

```svelte
<!-- src/lib/components/epub/MetadataEditor.svelte -->
<script lang="ts">
  async function handleSave() {
    try {
      saving = true;

      // 如果有新封面，先上传
      if (coverFile) {
        // TODO: 获取 workspace_path
        const workspacePath = '';
        await EpubService.updateCover(workspacePath, book.id, coverFile);
      }

      // 保存元数据
      await onSave({
        book: editedBook,
        authors: editedAuthors,
        tags: editedTags,
      });
    } catch (e) {
      alert(`保存失败: ${e}`);
    } finally {
      saving = false;
    }
  }
</script>
```

- [ ] **Step 6: 测试封面上传**

```bash
# 启动应用
bun run tauri:dev

# 测试步骤:
# 1. 编辑书籍
# 2. 点击"选择新封面"
# 3. 选择图片文件
# 4. 保存
# 5. 验证封面已更新
```

- [ ] **Step 7: 提交封面上传功能**

```bash
git add src-tauri/src/modules/epub/commands.rs src-tauri/src/lib.rs src/lib/services/epub.ts src/lib/components/epub/MetadataEditor.svelte
git commit -m "feat(epub): add cover upload functionality

- Backend API for cover update
- File dialog for cover selection
- Cover preview in editor
- Replace existing cover images

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## ⏸️ Checkpoint: Week 3 完成

此时应该完成：
- ✅ 元数据编辑器组件
- ✅ 后端元数据更新 API
- ✅ 封面上传功能

**验收测试**:
```bash
# 完整功能测试
bun run tauri:dev

# 测试清单:
# 1. 编辑书籍标题、作者、出版社等
# 2. 添加/删除作者
# 3. 添加/删除标签
# 4. 修改评分
# 5. 上传新封面
# 6. 保存所有更改
# 7. 刷新后验证数据持久化
```

---

## ⏸️ Phase 1 完整验收

### 功能验收清单

#### 导入功能
- [ ] 可以选择单个 EPUB 文件导入
- [ ] 可以选择多个 EPUB 文件批量导入
- [ ] 导入时正确提取元数据（标题、作者等）
- [ ] 导入时正确提取和生成封面缩略图
- [ ] 导入进度实时显示
- [ ] 导入失败时显示友好错误信息

#### 视图功能
- [ ] 网格视图正常显示（封面为主）
- [ ] 列表视图正常显示（横向卡片）
- [ ] 详细列表视图正常显示（表格）
- [ ] 三种视图可以流畅切换
- [ ] 每种视图都显示正确的信息

#### 搜索功能
- [ ] 基础搜索可以搜索标题、作者、出版社
- [ ] 高级搜索面板可以打开/关闭
- [ ] 高级搜索支持多字段过滤
- [ ] 搜索结果正确显示
- [ ] 可以清除搜索条件

#### 详情和编辑
- [ ] 点击书籍后侧边栏正确展开
- [ ] 详情模式显示完整信息
- [ ] 可以切换到编辑模式
- [ ] 可以编辑所有元数据字段
- [ ] 可以添加/删除作者
- [ ] 可以添加/删除标签
- [ ] 可以修改评分
- [ ] 可以上传新封面
- [ ] 保存后数据正确更新

#### 删除功能
- [ ] 可以删除书籍
- [ ] 删除前有确认提示
- [ ] 删除后文件和数据库记录同步删除
- [ ] 删除后界面正确更新

### 性能验收
- [ ] 单本书导入 < 5 秒
- [ ] 批量导入 10 本书 < 1 分钟
- [ ] 搜索响应 < 500ms（100 本规模）
- [ ] 视图切换无明显卡顿
- [ ] 侧边栏展开/关闭流畅

### 代码质量
- [ ] 所有 Rust 代码通过 `cargo clippy`
- [ ] 所有 TypeScript 代码通过类型检查
- [ ] 代码符合项目规范

### 用户体验
- [ ] 界面美观，布局合理
- [ ] 操作流程顺畅
- [ ] 错误提示友好
- [ ] 加载状态有明确反馈

---

## Phase 1 交付物

1. **后端功能**
   - EPUB 解析器（元数据、封面、目录）
   - 存储管理器（文件组织、封面生成）
   - 数据库操作层（CRUD、搜索）
   - Tauri 命令处理器（完整 API）

2. **前端功能**
   - 书库主界面（三种视图）
   - 搜索和过滤
   - 书籍详情展示
   - 元数据编辑器
   - 封面上传

3. **文档**
   - API 文档
   - 数据库 Schema 文档
   - 用户使用指南

---

## 下一步: Phase 2

Phase 2 将实现阅读器功能，包括：
- EPUB 阅读器（epub.js 集成）
- 目录导航和翻页
- 阅读设置（字体、主题）
- 书签和高亮功能
- 阅读进度保存

**预计工期**: 2 周

---

**Phase 1 计划完成**
**文档版本**: 1.0
**最后更新**: 2026-03-24
**状态**: ✅ 完整计划已完成
