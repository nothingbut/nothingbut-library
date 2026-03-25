use crate::errors::AppResult;
use crate::modules::epub::database::EpubDatabase;
use crate::modules::epub::models::{
    EpubBook, EpubBookWithDetails, ImportProgress, ImportResult, SearchQuery,
};
use crate::modules::epub::parser::EpubParser;
use crate::modules::epub::storage::EpubStorage;
use chrono::Utc;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::path::Path;
use tauri::{Emitter, State, Window};

/// 元数据更新请求
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

/// 导入单个 EPUB 文件
#[tauri::command]
pub async fn import_epub(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    source_file_path: String,
) -> AppResult<i64> {
    // Step 1: 解析 EPUB
    let source_path = Path::new(&source_file_path);
    let parser = EpubParser::open(source_path)?;

    // Step 2: 验证文件
    parser.validate()?;

    // Step 3: 提取元数据
    let metadata = parser.extract_metadata()?;
    let file_size = std::fs::metadata(source_path)?.len() as i64;

    // Step 4: 使用临时 ID 进行文件操作（使用时间戳作为临时 ID）
    let storage = EpubStorage::new(&workspace_path);
    storage.ensure_epub_root()?;

    let temp_id = Utc::now().timestamp();
    let dest_path = storage.copy_epub_file(source_path, temp_id)?;

    // Step 5: 保存封面（如果有）
    let cover_path_option = match parser.extract_cover() {
        Ok(Some(cover_data)) => {
            storage.save_cover(&cover_data, temp_id)?;
            Some(storage.cover_path(temp_id).to_string_lossy().to_string())
        }
        Ok(None) => None,
        Err(e) => {
            eprintln!("Warning: Failed to extract cover: {}", e);
            None
        }
    };

    // Step 6: 创建数据库记录（现在有完整路径）
    let db = EpubDatabase::new(pool.inner().clone());
    let now = Utc::now().to_rfc3339();

    let book = EpubBook {
        id: 0, // 将被数据库自动分配
        title: metadata.title.unwrap_or_else(|| "Unknown Title".to_string()),
        sort_title: None,
        isbn: metadata.isbn.clone(),
        publisher: metadata.publisher.clone(),
        pubdate: metadata.pubdate.clone(),
        language: metadata.language.clone(),
        series: None,
        series_index: None,
        rating: None,
        file_path: dest_path.to_string_lossy().to_string(),
        file_size,
        cover_path: cover_path_option.clone(),
        description: metadata.description.clone(),
        created_at: now.clone(),
        updated_at: now,
    };

    let book_id = db.create_book(&book).await?;

    // Step 7: 重命名文件（从 temp_id 到 book_id）
    let final_book_dir = storage.book_dir(book_id);
    let temp_book_dir = storage.book_dir(temp_id);
    std::fs::rename(&temp_book_dir, &final_book_dir)?;

    // Step 8: 更新数据库路径
    let final_file_path = storage.epub_file_path(book_id).to_string_lossy().to_string();
    let final_cover_path = if cover_path_option.is_some() {
        Some(storage.cover_path(book_id).to_string_lossy().to_string())
    } else {
        None
    };

    let updated_book = book.with_storage_paths(final_file_path, final_cover_path);
    db.update_book(book_id, &updated_book).await?;

    // Step 9: 设置作者关联
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

/// 批量导入 EPUB 文件
#[tauri::command]
pub async fn batch_import_epub(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    file_paths: Vec<String>,
    window: Window,
) -> AppResult<Vec<ImportResult>> {
    let total = file_paths.len();
    let mut results = Vec::new();

    for (index, file_path) in file_paths.iter().enumerate() {
        let current = index + 1;

        // 发送进度事件
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let progress = ImportProgress {
            current,
            total,
            file_name: file_name.clone(),
        };

        if let Err(e) = window.emit("epub-import-progress", &progress) {
            eprintln!("Warning: Failed to emit progress event: {}", e);
        }

        // 尝试导入
        match import_epub(pool.clone(), workspace_path.clone(), file_path.clone()).await {
            Ok(book_id) => {
                results.push(ImportResult::Success { book_id });
            }
            Err(e) => {
                results.push(ImportResult::Failed {
                    file_path: file_path.clone(),
                    error: e.to_string(),
                });
            }
        }
    }

    Ok(results)
}

/// 获取书籍详情（包含作者和标签）
#[tauri::command]
pub async fn get_epub_book(
    pool: State<'_, SqlitePool>,
    book_id: i64,
) -> AppResult<Option<EpubBookWithDetails>> {
    let db = EpubDatabase::new(pool.inner().clone());

    // 获取书籍
    let book = match db.get_book(book_id).await? {
        Some(book) => book,
        None => return Ok(None),
    };

    // 获取作者
    let authors = db.get_book_authors(book_id).await?;

    // 获取标签
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

/// 删除书籍
#[tauri::command]
pub async fn delete_epub_book(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    book_id: i64,
) -> AppResult<()> {
    let storage = EpubStorage::new(&workspace_path);
    let db = EpubDatabase::new(pool.inner().clone());

    // Step 1: 删除文件
    storage.delete_book(book_id)?;

    // Step 2: 删除数据库记录（级联删除关联）
    db.delete_book(book_id).await?;

    Ok(())
}

/// 更新书籍元数据
#[tauri::command]
pub async fn update_epub_metadata(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    metadata: UpdateMetadataRequest,
) -> AppResult<()> {
    let db = EpubDatabase::new(pool.inner().clone());

    // Step 1: 获取现有书籍
    let existing_book = db.get_book(book_id).await?;
    let mut book = existing_book.ok_or_else(|| {
        crate::AppError::NotFound(format!("Book {} not found", book_id))
    })?;

    // Step 2: 更新字段
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

    // Step 3: 保存到数据库
    db.update_book(book_id, &book).await?;

    Ok(())
}

/// 设置书籍的作者列表
#[tauri::command]
pub async fn set_epub_book_authors(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    author_names: Vec<String>,
) -> AppResult<()> {
    let db = EpubDatabase::new(pool.inner().clone());

    // 验证书籍存在
    let _book = db.get_book(book_id).await?;
    if _book.is_none() {
        return Err(crate::AppError::NotFound(format!("Book {} not found", book_id)));
    }

    // 转换作者名称为 (String, Option<String>, i32) 元组，带上顺序
    let authors: Vec<(String, Option<String>, i32)> = author_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| (name, None, i as i32))
        .collect();

    // 设置作者关联
    db.set_book_authors(book_id, authors).await?;

    Ok(())
}

/// 设置书籍的标签列表
#[tauri::command]
pub async fn set_epub_book_tags(
    pool: State<'_, SqlitePool>,
    book_id: i64,
    tag_names: Vec<String>,
) -> AppResult<()> {
    let db = EpubDatabase::new(pool.inner().clone());

    // 验证书籍存在
    let _book = db.get_book(book_id).await?;
    if _book.is_none() {
        return Err(crate::AppError::NotFound(format!("Book {} not found", book_id)));
    }

    // 设置标签关联
    db.set_book_tags(book_id, tag_names).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use tempfile::TempDir;

    async fn setup_test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();

        // 运行迁移
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS epub_books (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                sort_title TEXT,
                isbn TEXT,
                publisher TEXT,
                pubdate TEXT,
                language TEXT,
                series TEXT,
                series_index REAL,
                rating INTEGER,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                cover_path TEXT,
                description TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS epub_authors (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                sort_name TEXT,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS epub_book_authors (
                book_id INTEGER NOT NULL,
                author_id INTEGER NOT NULL,
                author_order INTEGER NOT NULL,
                PRIMARY KEY (book_id, author_id),
                FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE,
                FOREIGN KEY (author_id) REFERENCES epub_authors(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS epub_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS epub_book_tags (
                book_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (book_id, tag_id),
                FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES epub_tags(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_list_epub_books_empty() {
        let pool = setup_test_pool().await;
        let db = EpubDatabase::new(pool);

        let books = db.list_books().await.unwrap();
        assert_eq!(books.len(), 0);
    }

    #[tokio::test]
    async fn test_get_epub_book_not_found() {
        let pool = setup_test_pool().await;
        let db = EpubDatabase::new(pool);

        let result = db.get_book(999).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_search_epub_books_empty_query() {
        let pool = setup_test_pool().await;
        let db = EpubDatabase::new(pool);

        let query = SearchQuery {
            keyword: None,
            title: None,
            author: None,
            publisher: None,
            isbn: None,
            series: None,
            tags: None,
            rating_min: None,
            rating_max: None,
            sort_by: None,
            sort_order: None,
            limit: None,
            offset: None,
        };

        let books = db.search_books(&query).await.unwrap();
        assert_eq!(books.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_epub_book_not_found() {
        let pool = setup_test_pool().await;
        let db = EpubDatabase::new(pool);

        let result = db.delete_book(999).await;

        // 应该返回错误（书籍不存在）
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_import_epub_invalid_path() {
        // 测试无效路径解析
        let result = EpubParser::open(Path::new("/nonexistent/file.epub"));

        // 应该返回错误（文件不存在）
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_storage_delete_nonexistent_book() {
        let temp_dir = TempDir::new().unwrap();
        let storage = EpubStorage::new(temp_dir.path());

        // 删除不存在的书籍应该成功（幂等）
        let result = storage.delete_book(999);
        assert!(result.is_ok());
    }
}
