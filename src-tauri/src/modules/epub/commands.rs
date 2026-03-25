use crate::errors::AppResult;
use crate::modules::epub::database::EpubDatabase;
use crate::modules::epub::models::{
    EpubBook, EpubBookWithDetails, ImportProgress, ImportResult, SearchQuery,
};
use crate::modules::epub::parser::EpubParser;
use crate::modules::epub::storage::EpubStorage;
use chrono::Utc;
use sqlx::SqlitePool;
use std::path::Path;
use tauri::{Emitter, State, Window};

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

    // Step 4: 创建数据库记录（先创建以获取 book_id，file_path 临时为空）
    let db = EpubDatabase::new(pool.inner().clone());

    let book = EpubBook {
        id: 0, // 将被数据库自动分配
        title: metadata.title.unwrap_or_else(|| "Unknown Title".to_string()),
        sort_title: None,
        isbn: metadata.isbn,
        publisher: metadata.publisher,
        pubdate: metadata.pubdate,
        language: metadata.language,
        series: None,
        series_index: None,
        rating: None,
        file_path: String::new(), // 临时为空，后续更新
        file_size: std::fs::metadata(source_path)?.len() as i64,
        cover_path: None,
        description: metadata.description,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    };

    let book_id = db.create_book(&book).await?;

    // Step 5: 复制 EPUB 文件到书库
    let storage = EpubStorage::new(&workspace_path);
    storage.ensure_epub_root()?;
    let dest_path = storage.copy_epub_file(source_path, book_id)?;

    // Step 6: 保存封面图片
    if let Ok(Some(cover_data)) = parser.extract_cover() {
        storage.save_cover(&cover_data, book_id)?;
    }

    // Step 7: 更新数据库中的文件路径和封面路径
    let file_path_str = dest_path.to_string_lossy().to_string();
    let cover_path_str = if storage.cover_path(book_id).exists() {
        Some(storage.cover_path(book_id).to_string_lossy().to_string())
    } else {
        None
    };

    let updated_book = EpubBook {
        id: book_id,
        title: book.title,
        sort_title: book.sort_title,
        isbn: book.isbn,
        publisher: book.publisher,
        pubdate: book.pubdate,
        language: book.language,
        series: book.series,
        series_index: book.series_index,
        rating: book.rating,
        file_path: file_path_str,
        file_size: book.file_size,
        cover_path: cover_path_str,
        description: book.description,
        created_at: book.created_at,
        updated_at: Utc::now().to_rfc3339(),
    };

    db.update_book(book_id, &updated_book).await?;

    // Step 8: 设置作者关联
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

        let _ = window.emit("epub-import-progress", &progress);

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
