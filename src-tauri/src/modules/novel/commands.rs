use std::fs::File;
use std::path::Path;
use sqlx::SqlitePool;
use tauri::State;
use crate::errors::AppResult;
use super::database;
use super::models::{BookStatus, ChapterPreview, ImportPreview, NovelBook, NovelCategory, NovelChapter};
use super::parser::TxtParser;
use super::scraper;
use super::storage::{count_words, create_book_dir, save_chapter, save_metadata, BookMetadata};

/// Preview import - show first 3 chapters with auto-extracted metadata
#[tauri::command]
pub async fn preview_import(
    #[allow(non_snake_case)]
    filePath: String,
    title: String,
    category: String,
) -> AppResult<ImportPreview> {
    let parser = TxtParser::new();
    let file = File::open(&filePath)
        .map_err(|e| crate::AppError::Io(format!("Failed to open file: {}", e)))?;

    let chapters = parser.parse(file)?;

    // Re-read file to extract metadata
    let file2 = File::open(&filePath)
        .map_err(|e| crate::AppError::Io(format!("Failed to open file: {}", e)))?;
    let content = parser.read_file(file2)?;
    let metadata = parser.extract_metadata(&content);

    // Calculate preview (first 3 chapters)
    let preview_chapters: Vec<ChapterPreview> = chapters
        .iter()
        .take(3)
        .enumerate()
        .map(|(idx, ch)| {
            let wc = count_words(&ch.content);
            ChapterPreview {
                chapter_number: (idx + 1) as u32,
                title: ch.title.clone(),
                preview: ch.preview.clone(),
                word_count: wc as u32,
            }
        })
        .collect();

    let total_chapters = chapters.len() as u32;
    let total_words = chapters
        .iter()
        .map(|ch| count_words(&ch.content))
        .sum::<usize>() as u64;

    Ok(ImportPreview {
        title,
        author: metadata.author,
        description: metadata.description,
        category,
        chapters: preview_chapters,
        total_chapters,
        total_words,
    })
}

/// Complete import flow: parse → save files → insert to DB
#[tauri::command]
pub async fn import_novel(
    pool: State<'_, SqlitePool>,
    #[allow(non_snake_case)]
    workspacePath: String,
    #[allow(non_snake_case)]
    filePath: String,
    title: String,
    author: Option<String>,
    description: Option<String>,
    #[allow(non_snake_case)]
    categoryId: Option<i64>,
    #[allow(non_snake_case)]
    sourceSite: Option<String>,
) -> AppResult<i64> {
    let workspace = Path::new(&workspacePath);

    // Parse the file
    let parser = TxtParser::new();
    let file = File::open(&filePath)
        .map_err(|e| crate::AppError::Io(format!("Failed to open file: {}", e)))?;
    let chapters = parser.parse(file)?;

    if chapters.is_empty() {
        return Err(crate::AppError::Validation(
            "No chapters found in file".to_string(),
        ));
    }

    // Calculate totals
    let chapter_count = chapters.len() as i32;
    let total_words: i64 = chapters
        .iter()
        .map(|ch| count_words(&ch.content) as i64)
        .sum();

    // Get file size
    let file_metadata = std::fs::metadata(&filePath)
        .map_err(|e| crate::AppError::Io(format!("Failed to read file metadata: {}", e)))?;
    let file_size = file_metadata.len() as i64;

    // Insert book into database first to get the ID
    let book_id = database::insert_book(
        &pool,
        &title,
        author.as_deref(),
        description.as_deref(),
        None, // cover_path
        categoryId,
        sourceSite.as_deref(),
        &format!("books/book-{}", 0), // Temporary, will be updated
        file_size,
        total_words,
        chapter_count,
        BookStatus::Ongoing,
    )
    .await?;

    // Update book_dir with actual book_id
    let book_dir_path = format!("books/book-{}", book_id);
    sqlx::query("UPDATE novel_books SET book_dir = ? WHERE id = ?")
        .bind(&book_dir_path)
        .bind(book_id)
        .execute(pool.inner())
        .await
        .map_err(|e| crate::AppError::Database(format!("Failed to update book_dir: {}", e)))?;

    // Create book directory
    let book_dir = create_book_dir(workspace, book_id)?;

    // Save chapters to files and insert into database
    for (idx, chapter) in chapters.iter().enumerate() {
        let file_path = save_chapter(&book_dir, idx + 1, chapter)?;
        let word_count = count_words(&chapter.content) as i64;

        // Store full relative path from workspace (books/book-{id}/chapters/...)
        let full_file_path = format!("{}/{}", book_dir_path, file_path);

        database::insert_chapter(
            &pool,
            book_id,
            &chapter.title,
            &chapter.preview,
            &full_file_path,
            (idx + 1) as i32,
            word_count,
        )
        .await?;
    }

    // Save metadata file
    let metadata = BookMetadata {
        title: title.clone(),
        author: author.clone(),
        description: description.clone(),
        chapter_count: chapters.len(),
        total_words: total_words as usize,
        created_at: chrono::Utc::now().to_rfc3339(),
    };
    save_metadata(&book_dir, &metadata)?;

    Ok(book_id)
}

/// List all books
#[tauri::command]
pub async fn list_books(pool: State<'_, SqlitePool>) -> AppResult<Vec<NovelBook>> {
    database::list_books(&pool).await
}

/// List chapters by book_id
#[tauri::command]
pub async fn list_chapters(
    pool: State<'_, SqlitePool>,
    #[allow(non_snake_case)]
    bookId: i64,
) -> AppResult<Vec<NovelChapter>> {
    database::list_chapters(&pool, bookId).await
}

/// Create a new category
#[tauri::command]
pub async fn create_category(
    pool: State<'_, SqlitePool>,
    name: String,
    #[allow(non_snake_case)]
    parentId: Option<i64>,
    #[allow(non_snake_case)]
    sortOrder: i32,
) -> AppResult<i64> {
    database::insert_category(&pool, &name, parentId, sortOrder).await
}

/// List all categories
#[tauri::command]
pub async fn list_categories(pool: State<'_, SqlitePool>) -> AppResult<Vec<NovelCategory>> {
    database::list_categories(&pool).await
}

/// Get chapter content by reading from file
#[tauri::command]
pub async fn get_chapter_content(
    pool: State<'_, SqlitePool>,
    #[allow(non_snake_case)]
    workspacePath: String,
    #[allow(non_snake_case)]
    chapterId: i64,
) -> AppResult<String> {
    println!("[get_chapter_content] workspace: {}, chapterId: {}", workspacePath, chapterId);

    let workspace = Path::new(&workspacePath);

    // Get chapter info from database
    let chapter = database::get_chapter(&pool, chapterId).await?
        .ok_or_else(|| crate::AppError::Validation(format!("Chapter {} not found", chapterId)))?;

    println!("[get_chapter_content] chapter.file_path: {}", chapter.file_path);

    // Read content from file
    let file_path = workspace.join(&chapter.file_path);
    println!("[get_chapter_content] full file path: {:?}", file_path);

    if !file_path.exists() {
        return Err(crate::AppError::Io(format!("Chapter file does not exist: {:?}", file_path)));
    }

    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| crate::AppError::Io(format!("Failed to read chapter file {:?}: {}", file_path, e)))?;

    println!("[get_chapter_content] content length: {}", content.len());
    Ok(content)
}

/// Seed categories from bsconfig.json
#[tauri::command]
pub async fn seed_categories(
    pool: State<'_, SqlitePool>,
    #[allow(non_snake_case)]
    configPath: String,
) -> AppResult<usize> {
    let path = Path::new(&configPath);
    super::seed::seed_categories_from_config(&pool, path).await
}

/// Fetch book metadata from source website
#[tauri::command]
pub async fn fetch_book_metadata(
    #[allow(non_snake_case)]
    workspacePath: String,
    #[allow(non_snake_case)]
    bookId: Option<i64>,
    #[allow(non_snake_case)]
    sourceSite: String,
    title: String,
    author: Option<String>,
) -> AppResult<serde_json::Value> {
    let workspace = Path::new(&workspacePath);

    // Scrape metadata from website
    let metadata = scraper::scrape_metadata(
        &sourceSite,
        &title,
        author.as_deref(),
    )
    .await?;

    let mut result = serde_json::json!({
        "description": metadata.description,
        "author": metadata.author,
        "category": metadata.category,
    });

    // Download cover if available and bookId is provided
    if let (Some(ref cover_url), Some(id)) = (metadata.cover_url.as_ref(), bookId) {
        // Create or get book directory
        let book_dir = workspace.join("books").join(format!("book-{}", id));

        // Download and save cover
        match scraper::download_cover(cover_url, &book_dir).await {
            Ok(cover_path) => {
                result["coverPath"] = serde_json::json!(cover_path);
            }
            Err(e) => {
                println!("Warning: Failed to download cover: {}", e);
                // Continue without cover
            }
        }
    } else if let Some(cover_url) = metadata.cover_url {
        // Return cover URL for preview (before book is created)
        result["coverUrl"] = serde_json::json!(cover_url);
    }

    Ok(result)
}

/// Delete a book and all its data
#[tauri::command]
pub async fn delete_book(
    pool: State<'_, SqlitePool>,
    #[allow(non_snake_case)]
    workspacePath: String,
    #[allow(non_snake_case)]
    bookId: i64,
) -> AppResult<()> {
    let workspace = Path::new(&workspacePath);

    // Get book info first
    let books = database::list_books(&pool).await?;
    let book = books.iter().find(|b| b.id == bookId)
        .ok_or_else(|| crate::AppError::NotFound(format!("Book {} not found", bookId)))?;

    // Delete book directory
    let book_dir = workspace.join(&book.book_dir);
    if book_dir.exists() {
        std::fs::remove_dir_all(&book_dir)
            .map_err(|e| crate::AppError::Io(format!("Failed to delete book directory: {}", e)))?;
    }

    // Delete from database (cascades to chapters)
    database::delete_book(&pool, bookId).await?;

    Ok(())
}
