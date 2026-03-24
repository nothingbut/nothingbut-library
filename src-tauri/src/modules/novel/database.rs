use sqlx::{Row, SqlitePool};
use chrono::Utc;
use crate::errors::{AppError, AppResult};
use super::models::{BookStatus, NovelBook, NovelCategory, NovelChapter};

/// Insert a book into the database
pub async fn insert_book(
    pool: &SqlitePool,
    title: &str,
    author: Option<&str>,
    description: Option<&str>,
    cover_path: Option<&str>,
    category_id: Option<i64>,
    source_site: Option<&str>,
    book_dir: &str,
    file_size: i64,
    word_count: i64,
    chapter_count: i32,
    status: BookStatus,
) -> AppResult<i64> {
    let now = Utc::now().timestamp();
    let status_str = status.to_string();

    let result = sqlx::query(
        r#"
        INSERT INTO novel_books (
            title, author, description, cover_path, category_id, source_site,
            book_dir, file_size, word_count, chapter_count, status,
            created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(title)
    .bind(author)
    .bind(description)
    .bind(cover_path)
    .bind(category_id)
    .bind(source_site)
    .bind(book_dir)
    .bind(file_size)
    .bind(word_count)
    .bind(chapter_count)
    .bind(&status_str)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to insert book: {}", e)))?;

    Ok(result.last_insert_rowid())
}

/// List all books
pub async fn list_books(pool: &SqlitePool) -> AppResult<Vec<NovelBook>> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, title, author, description, cover_path, category_id, source_site,
            book_dir, file_size, word_count, chapter_count, status,
            reading_progress, last_read_at, created_at, updated_at
        FROM novel_books
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to list books: {}", e)))?;

    let books = rows
        .into_iter()
        .map(|row| {
            let status_str: String = row.get("status");
            let status = BookStatus::from_str(&status_str)
                .unwrap_or(BookStatus::Ongoing);

            let created_timestamp: i64 = row.get("created_at");
            let updated_timestamp: i64 = row.get("updated_at");
            let last_read_timestamp: Option<i64> = row.get("last_read_at");

            let created_at = chrono::DateTime::from_timestamp(created_timestamp, 0)
                .unwrap()
                .to_rfc3339();
            let updated_at = chrono::DateTime::from_timestamp(updated_timestamp, 0)
                .unwrap()
                .to_rfc3339();
            let last_read_at = last_read_timestamp.and_then(|ts| {
                chrono::DateTime::from_timestamp(ts, 0)
                    .map(|dt| dt.to_rfc3339())
            });

            NovelBook {
                id: row.get("id"),
                title: row.get("title"),
                author: row.get("author"),
                description: row.get("description"),
                cover_path: row.get("cover_path"),
                category_id: row.get("category_id"),
                source_site: row.get("source_site"),
                book_dir: row.get("book_dir"),
                file_size: row.get("file_size"),
                word_count: row.get("word_count"),
                chapter_count: row.get("chapter_count"),
                status,
                reading_progress: row.get("reading_progress"),
                last_read_at,
                created_at,
                updated_at,
            }
        })
        .collect();

    Ok(books)
}

/// Insert a chapter into the database
pub async fn insert_chapter(
    pool: &SqlitePool,
    book_id: i64,
    title: &str,
    preview: &str,
    file_path: &str,
    sort_order: i32,
    word_count: i64,
) -> AppResult<i64> {
    let now = Utc::now().timestamp();

    let result = sqlx::query(
        r#"
        INSERT INTO novel_chapters (
            book_id, title, preview, file_path, sort_order, word_count, created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(book_id)
    .bind(title)
    .bind(preview)
    .bind(file_path)
    .bind(sort_order)
    .bind(word_count)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to insert chapter: {}", e)))?;

    Ok(result.last_insert_rowid())
}

/// Get a single chapter by ID
pub async fn get_chapter(pool: &SqlitePool, chapter_id: i64) -> AppResult<Option<NovelChapter>> {
    let row = sqlx::query(
        r#"
        SELECT id, book_id, title, preview, file_path, sort_order, word_count, created_at
        FROM novel_chapters
        WHERE id = ?
        "#,
    )
    .bind(chapter_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to get chapter: {}", e)))?;

    Ok(row.map(|row| {
        let created_timestamp: i64 = row.get("created_at");
        let created_at = chrono::DateTime::from_timestamp(created_timestamp, 0)
            .unwrap()
            .to_rfc3339();

        NovelChapter {
            id: row.get("id"),
            book_id: row.get("book_id"),
            title: row.get("title"),
            preview: row.get("preview"),
            file_path: row.get("file_path"),
            sort_order: row.get("sort_order"),
            word_count: row.get("word_count"),
            created_at,
        }
    }))
}

/// List chapters by book_id
pub async fn list_chapters(pool: &SqlitePool, book_id: i64) -> AppResult<Vec<NovelChapter>> {
    let rows = sqlx::query(
        r#"
        SELECT id, book_id, title, preview, file_path, sort_order, word_count, created_at
        FROM novel_chapters
        WHERE book_id = ?
        ORDER BY sort_order ASC
        "#,
    )
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to list chapters: {}", e)))?;

    let chapters = rows
        .into_iter()
        .map(|row| {
            let created_timestamp: i64 = row.get("created_at");
            let created_at = chrono::DateTime::from_timestamp(created_timestamp, 0)
                .unwrap()
                .to_rfc3339();

            NovelChapter {
                id: row.get("id"),
                book_id: row.get("book_id"),
                title: row.get("title"),
                file_path: row.get("file_path"),
                preview: row.get("preview"),
                sort_order: row.get("sort_order"),
                word_count: row.get("word_count"),
                created_at,
            }
        })
        .collect();

    Ok(chapters)
}

/// Insert a category into the database
pub async fn insert_category(
    pool: &SqlitePool,
    name: &str,
    parent_id: Option<i64>,
    sort_order: i32,
) -> AppResult<i64> {
    let now = Utc::now().timestamp();

    let result = sqlx::query(
        r#"
        INSERT INTO novel_categories (name, parent_id, sort_order, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(name)
    .bind(parent_id)
    .bind(sort_order)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to insert category: {}", e)))?;

    Ok(result.last_insert_rowid())
}

/// List all categories
pub async fn list_categories(pool: &SqlitePool) -> AppResult<Vec<NovelCategory>> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, parent_id, sort_order, created_at
        FROM novel_categories
        ORDER BY sort_order ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(format!("Failed to list categories: {}", e)))?;

    let categories = rows
        .into_iter()
        .map(|row| {
            let created_timestamp: i64 = row.get("created_at");
            let created_at = chrono::DateTime::from_timestamp(created_timestamp, 0)
                .unwrap()
                .to_rfc3339();

            NovelCategory {
                id: row.get("id"),
                name: row.get("name"),
                parent_id: row.get("parent_id"),
                sort_order: row.get("sort_order"),
                created_at,
            }
        })
        .collect();

    Ok(categories)
}

/// Delete a book
pub async fn delete_book(pool: &SqlitePool, book_id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM novel_books WHERE id = ?")
        .bind(book_id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to delete book: {}", e)))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE novel_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                parent_id INTEGER,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE novel_books (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                author TEXT,
                description TEXT,
                cover_path TEXT,
                category_id INTEGER,
                source_site TEXT,
                book_dir TEXT NOT NULL UNIQUE,
                file_size INTEGER NOT NULL DEFAULT 0,
                word_count INTEGER NOT NULL DEFAULT 0,
                chapter_count INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'ongoing',
                reading_progress REAL NOT NULL DEFAULT 0.0,
                last_read_at INTEGER,
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
            CREATE TABLE novel_chapters (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                book_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                preview TEXT NOT NULL,
                file_path TEXT NOT NULL,
                sort_order INTEGER NOT NULL,
                word_count INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_insert_and_list_books() {
        let pool = setup_test_db().await;

        let book_id = insert_book(
            &pool,
            "Test Book",
            Some("Test Author"),
            Some("Test description"),
            None,
            None,
            None,
            "books/book-1",
            1024000,
            50000,
            10,
            BookStatus::Ongoing,
        )
        .await
        .unwrap();

        assert_eq!(book_id, 1);

        let books = list_books(&pool).await.unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].title, "Test Book");
        assert_eq!(books[0].author, Some("Test Author".to_string()));
        assert_eq!(books[0].word_count, 50000);
        assert_eq!(books[0].chapter_count, 10);
        assert_eq!(books[0].status, BookStatus::Ongoing);
    }

    #[tokio::test]
    async fn test_insert_and_list_chapters() {
        let pool = setup_test_db().await;

        let book_id = insert_book(
            &pool,
            "Test Book",
            None,
            None,
            None,
            None,
            None,
            "books/book-1",
            0,
            0,
            0,
            BookStatus::Ongoing,
        )
        .await
        .unwrap();

        let chapter_id1 = insert_chapter(
            &pool,
            book_id,
            "Chapter 1",
            "Chapter 1 preview",
            "books/book-1/chapters/chapter-0001.txt",
            1,
            5000,
        )
        .await
        .unwrap();

        let chapter_id2 = insert_chapter(
            &pool,
            book_id,
            "Chapter 2",
            "Chapter 2 preview",
            "books/book-1/chapters/chapter-0002.txt",
            2,
            5500,
        )
        .await
        .unwrap();

        assert_eq!(chapter_id1, 1);
        assert_eq!(chapter_id2, 2);

        let chapters = list_chapters(&pool, book_id).await.unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter 1");
        assert_eq!(chapters[0].word_count, 5000);
        assert_eq!(chapters[0].sort_order, 1);
        assert_eq!(chapters[1].title, "Chapter 2");
        assert_eq!(chapters[1].word_count, 5500);
        assert_eq!(chapters[1].sort_order, 2);
    }

    #[tokio::test]
    async fn test_insert_and_list_categories() {
        let pool = setup_test_db().await;

        let cat_id1 = insert_category(&pool, "Fantasy", None, 0)
            .await
            .unwrap();
        let cat_id2 = insert_category(&pool, "Sci-Fi", None, 1)
            .await
            .unwrap();

        assert_eq!(cat_id1, 1);
        assert_eq!(cat_id2, 2);

        let categories = list_categories(&pool).await.unwrap();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].name, "Fantasy");
        assert_eq!(categories[0].sort_order, 0);
        assert_eq!(categories[1].name, "Sci-Fi");
        assert_eq!(categories[1].sort_order, 1);
    }
}
