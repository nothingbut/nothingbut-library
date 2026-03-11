use std::fs::File;
use std::path::Path;
use sqlx::SqlitePool;
use tauri::State;
use crate::errors::AppResult;
use super::database;
use super::models::{BookStatus, ChapterPreview, ImportPreview, NovelBook, NovelCategory, NovelChapter};
use super::parser::TxtParser;
use super::storage::{count_words, create_book_dir, save_chapter, save_metadata, BookMetadata};

/// Preview import - show first 3 chapters
#[tauri::command]
pub async fn preview_import(
    file_path: String,
    title: String,
    author: String,
    category: String,
) -> AppResult<ImportPreview> {
    let parser = TxtParser::new();
    let file = File::open(&file_path)
        .map_err(|e| crate::AppError::Io(format!("Failed to open file: {}", e)))?;

    let chapters = parser.parse(file)?;

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
        author,
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
    workspace_path: String,
    file_path: String,
    title: String,
    author: Option<String>,
    description: Option<String>,
    category_id: Option<i64>,
) -> AppResult<i64> {
    let workspace = Path::new(&workspace_path);

    // Parse the file
    let parser = TxtParser::new();
    let file = File::open(&file_path)
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
    let file_metadata = std::fs::metadata(&file_path)
        .map_err(|e| crate::AppError::Io(format!("Failed to read file metadata: {}", e)))?;
    let file_size = file_metadata.len() as i64;

    // Insert book into database first to get the ID
    let book_id = database::insert_book(
        &pool,
        &title,
        author.as_deref(),
        description.as_deref(),
        None, // cover_path
        category_id,
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

        database::insert_chapter(
            &pool,
            book_id,
            &chapter.title,
            &file_path,
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
    book_id: i64,
) -> AppResult<Vec<NovelChapter>> {
    database::list_chapters(&pool, book_id).await
}

/// Create a new category
#[tauri::command]
pub async fn create_category(
    pool: State<'_, SqlitePool>,
    name: String,
    parent_id: Option<i64>,
    sort_order: i32,
) -> AppResult<i64> {
    database::insert_category(&pool, &name, parent_id, sort_order).await
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
    workspace_path: String,
    chapter_id: i64,
) -> AppResult<String> {
    let workspace = Path::new(&workspace_path);

    // Get chapter info from database
    let chapter = database::get_chapter(&pool, chapter_id).await?
        .ok_or_else(|| crate::AppError::Validation(format!("Chapter {} not found", chapter_id)))?;

    // Read content from file
    let file_path = workspace.join(&chapter.file_path);
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| crate::AppError::Io(format!("Failed to read chapter file: {}", e)))?;

    Ok(content)
}
