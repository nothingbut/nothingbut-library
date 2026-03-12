#[cfg(test)]
mod tests {
    use crate::modules::novel::{commands, database, models::BookStatus, storage};
    use sqlx::sqlite::SqlitePoolOptions;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    async fn setup_test_db() -> sqlx::SqlitePool {
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

    fn create_test_novel_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let content = r#"第一章 开端

这是第一章的内容。
一些文字。

第二章 发展

这是第二章的内容。
更多的文字。

第三章 高潮

这是第三章的内容。
最后的文字。
"#;

        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[tokio::test]
    async fn test_preview_import_basic() {
        let test_file = create_test_novel_file();

        let result = commands::preview_import(
            test_file.path().to_str().unwrap().to_string(),
            "测试小说".to_string(),
            "玄幻".to_string(),
        )
        .await;

        assert!(result.is_ok());
        let preview = result.unwrap();

        assert_eq!(preview.title, "测试小说");
        assert_eq!(preview.category, "玄幻");
        assert_eq!(preview.total_chapters, 3);
        assert_eq!(preview.chapters.len(), 3); // Shows first 3 chapters
        assert_eq!(preview.chapters[0].chapter_number, 1);
        assert_eq!(preview.chapters[0].title, "第一章 开端");
        assert!(!preview.chapters[0].preview.is_empty()); // Has preview
        assert!(preview.total_words > 0);
    }

    #[tokio::test]
    async fn test_preview_import_file_not_found() {
        let result = commands::preview_import(
            "/nonexistent/file.txt".to_string(),
            "Test".to_string(),
            "Category".to_string(),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_import_novel_complete_flow() {
        let pool = setup_test_db().await;
        let test_file = create_test_novel_file();
        let workspace = TempDir::new().unwrap();

        // Create a category first
        let category_id = database::insert_category(&pool, "玄幻", None, 0)
            .await
            .unwrap();

        // Manually implement the import flow (testing the underlying logic)
        let parser = super::super::parser::TxtParser::new();
        let file = std::fs::File::open(test_file.path()).unwrap();
        let chapters = parser.parse(file).unwrap();

        let chapter_count = chapters.len() as i32;
        let total_words: i64 = chapters
            .iter()
            .map(|ch| storage::count_words(&ch.content) as i64)
            .sum();

        let file_metadata = std::fs::metadata(test_file.path()).unwrap();
        let file_size = file_metadata.len() as i64;

        // Insert book
        let book_id = database::insert_book(
            &pool,
            "测试小说",
            Some("测试作者"),
            Some("这是一个测试小说"),
            None,
            Some(category_id),
            None,
            "books/book-1",
            file_size,
            total_words,
            chapter_count,
            BookStatus::Ongoing,
        )
        .await
        .unwrap();

        // Update book_dir
        sqlx::query("UPDATE novel_books SET book_dir = ? WHERE id = ?")
            .bind(format!("books/book-{}", book_id))
            .bind(book_id)
            .execute(&pool)
            .await
            .unwrap();

        // Create book directory
        let book_dir = storage::create_book_dir(workspace.path(), book_id).unwrap();

        // Save chapters
        for (idx, chapter) in chapters.iter().enumerate() {
            let file_path = storage::save_chapter(&book_dir, idx + 1, chapter).unwrap();
            let word_count = storage::count_words(&chapter.content) as i64;

            // Store full relative path from workspace
            let full_file_path = format!("books/book-{}/{}", book_id, file_path);

            database::insert_chapter(
                &pool,
                book_id,
                &chapter.title,
                &chapter.preview,
                &full_file_path,
                (idx + 1) as i32,
                word_count,
            )
            .await
            .unwrap();
        }

        // Save metadata
        let metadata = storage::BookMetadata {
            title: "测试小说".to_string(),
            author: Some("测试作者".to_string()),
            description: Some("这是一个测试小说".to_string()),
            chapter_count: chapters.len(),
            total_words: total_words as usize,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        storage::save_metadata(&book_dir, &metadata).unwrap();

        // Verify book was inserted
        let books = database::list_books(&pool).await.unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].title, "测试小说");
        assert_eq!(books[0].author, Some("测试作者".to_string()));
        assert_eq!(books[0].chapter_count, 3);
        assert!(books[0].word_count > 0);

        // Verify chapters were inserted
        let chapters_from_db = database::list_chapters(&pool, book_id).await.unwrap();
        assert_eq!(chapters_from_db.len(), 3);
        assert_eq!(chapters_from_db[0].title, "第一章 开端");
        assert_eq!(chapters_from_db[1].title, "第二章 发展");
        assert_eq!(chapters_from_db[2].title, "第三章 高潮");

        // Verify files were created
        let book_dir_path = workspace.path().join(format!("books/book-{}", book_id));
        assert!(book_dir_path.exists());
        assert!(book_dir_path.join("chapters").exists());
        assert!(book_dir_path.join("metadata.json").exists());

        // Verify chapter files exist
        assert!(book_dir_path
            .join("chapters")
            .join("chapter-0001.txt")
            .exists());
        assert!(book_dir_path
            .join("chapters")
            .join("chapter-0002.txt")
            .exists());
        assert!(book_dir_path
            .join("chapters")
            .join("chapter-0003.txt")
            .exists());
    }

    #[tokio::test]
    async fn test_import_without_category() {
        let pool = setup_test_db().await;
        let test_file = create_test_novel_file();

        // Parse file
        let parser = super::super::parser::TxtParser::new();
        let file = std::fs::File::open(test_file.path()).unwrap();
        let chapters = parser.parse(file).unwrap();

        // Import without category
        let _book_id = database::insert_book(
            &pool,
            "无分类小说",
            None,
            None,
            None,
            None,
            None,
            "books/book-1",
            0,
            0,
            chapters.len() as i32,
            BookStatus::Ongoing,
        )
        .await
        .unwrap();

        let books = database::list_books(&pool).await.unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].title, "无分类小说");
        assert_eq!(books[0].category_id, None);
    }

    #[tokio::test]
    async fn test_database_operations() {
        let pool = setup_test_db().await;

        // Insert test data
        let book_id = database::insert_book(
            &pool,
            "测试书籍",
            Some("作者"),
            Some("描述"),
            None,
            None,
            None,
            "books/book-1",
            1024,
            5000,
            2,
            BookStatus::Ongoing,
        )
        .await
        .unwrap();

        database::insert_chapter(
            &pool,
            book_id,
            "第一章",
            "Preview...",
            "books/book-1/chapters/chapter-0001.txt",
            1,
            2500,
        )
        .await
        .unwrap();

        database::insert_chapter(
            &pool,
            book_id,
            "第二章",
            "Preview...",
            "books/book-1/chapters/chapter-0002.txt",
            2,
            2500,
        )
        .await
        .unwrap();

        // Test list_books
        let books = database::list_books(&pool).await.unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].title, "测试书籍");

        // Test list_chapters
        let chapters = database::list_chapters(&pool, book_id).await.unwrap();
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第一章");
        assert_eq!(chapters[1].title, "第二章");
    }

    #[tokio::test]
    async fn test_get_chapter_file() {
        let pool = setup_test_db().await;
        let workspace = TempDir::new().unwrap();

        // Create a book and chapter
        let book_id = database::insert_book(
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
            1,
            BookStatus::Ongoing,
        )
        .await
        .unwrap();

        let chapter_id = database::insert_chapter(
            &pool,
            book_id,
            "Test Chapter",
            "Test preview",
            "books/book-1/chapters/chapter-0001.txt",
            1,
            100,
        )
        .await
        .unwrap();

        // Create the chapter file
        let chapter_path = workspace
            .path()
            .join("books/book-1/chapters/chapter-0001.txt");
        std::fs::create_dir_all(chapter_path.parent().unwrap()).unwrap();
        std::fs::write(&chapter_path, "This is the chapter content.").unwrap();

        // Get chapter info
        let chapter = database::get_chapter(&pool, chapter_id)
            .await
            .unwrap()
            .unwrap();

        // Read content from file
        let file_path = workspace.path().join(&chapter.file_path);
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "This is the chapter content.");
    }

    #[tokio::test]
    async fn test_category_operations() {
        let pool = setup_test_db().await;

        // Create main category
        let parent_id = database::insert_category(&pool, "玄幻", None, 0)
            .await
            .unwrap();

        // Create subcategory
        let sub_id = database::insert_category(&pool, "东方玄幻", Some(parent_id), 0)
            .await
            .unwrap();

        assert!(sub_id > parent_id);

        // List categories
        let categories = database::list_categories(&pool).await.unwrap();
        assert_eq!(categories.len(), 2);
        assert_eq!(categories[0].name, "玄幻");
        assert_eq!(categories[0].parent_id, None);
        assert_eq!(categories[1].name, "东方玄幻");
        assert_eq!(categories[1].parent_id, Some(parent_id));
    }
}
