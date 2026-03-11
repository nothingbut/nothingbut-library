use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::errors::{AppError, AppResult};
use super::parser::Chapter;

/// Book metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BookMetadata {
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub chapter_count: usize,
    pub total_words: usize,
    pub created_at: String,
}

/// Count words in content (Chinese characters + English words)
pub fn count_words(content: &str) -> usize {
    let mut count = 0;

    // Count Chinese characters
    for ch in content.chars() {
        if ('\u{4E00}'..='\u{9FFF}').contains(&ch) {
            count += 1;
        }
    }

    // Count English words (simple word splitting by whitespace)
    let english_words = content
        .split_whitespace()
        .filter(|word| word.chars().any(|c| c.is_ascii_alphabetic()))
        .count();

    count + english_words
}

/// Create book directory
pub fn create_book_dir(workspace_path: &Path, book_id: i64) -> AppResult<PathBuf> {
    let book_dir = workspace_path.join("books").join(format!("book-{}", book_id));
    fs::create_dir_all(&book_dir)?;
    fs::create_dir_all(book_dir.join("chapters"))?;
    Ok(book_dir)
}

/// Save chapter file
pub fn save_chapter(
    book_dir: &Path,
    chapter_index: usize,
    chapter: &Chapter,
) -> AppResult<String> {
    let chapter_file = format!("chapter-{:04}.txt", chapter_index);
    let chapter_path = book_dir.join("chapters").join(&chapter_file);

    fs::write(&chapter_path, &chapter.content)?;

    Ok(format!("chapters/{}", chapter_file))
}

/// Save metadata
pub fn save_metadata(
    book_dir: &Path,
    metadata: &BookMetadata,
) -> AppResult<()> {
    let metadata_path = book_dir.join("metadata.json");
    let json = serde_json::to_string_pretty(metadata)?;
    fs::write(metadata_path, json)?;
    Ok(())
}

/// Load metadata
pub fn load_metadata(book_dir: &Path) -> AppResult<BookMetadata> {
    let metadata_path = book_dir.join("metadata.json");

    if !metadata_path.exists() {
        return Err(AppError::NotFound(format!(
            "Metadata not found: {}",
            metadata_path.display()
        )));
    }

    let json = fs::read_to_string(metadata_path)?;
    let metadata = serde_json::from_str(&json)?;
    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use chrono::Utc;

    #[test]
    fn test_count_words() {
        assert_eq!(count_words("章节内容"), 4);
        assert_eq!(count_words("Hello world"), 2);
        assert_eq!(count_words("混合 mixed 内容"), 5);
        assert_eq!(count_words(""), 0);
    }

    #[test]
    fn test_create_book_dir() {
        let temp_dir = TempDir::new().unwrap();
        let book_dir = create_book_dir(temp_dir.path(), 1).unwrap();

        assert!(book_dir.exists());
        assert!(book_dir.join("chapters").exists());
    }

    #[test]
    fn test_save_and_load_chapter() {
        let temp_dir = TempDir::new().unwrap();
        let book_dir = create_book_dir(temp_dir.path(), 1).unwrap();

        let chapter = Chapter {
            title: "第一章".to_string(),
            content: "章节内容".to_string(),
            start_position: 0,
        };

        let path = save_chapter(&book_dir, 1, &chapter).unwrap();
        assert_eq!(path, "chapters/chapter-0001.txt");

        let saved_content = fs::read_to_string(book_dir.join(&path)).unwrap();
        assert_eq!(saved_content, "章节内容");
    }

    #[test]
    fn test_save_and_load_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let book_dir = create_book_dir(temp_dir.path(), 1).unwrap();

        let metadata = BookMetadata {
            title: "测试书籍".to_string(),
            author: Some("测试作者".to_string()),
            description: None,
            chapter_count: 10,
            total_words: 50000,
            created_at: Utc::now().to_rfc3339(),
        };

        save_metadata(&book_dir, &metadata).unwrap();
        let loaded = load_metadata(&book_dir).unwrap();

        assert_eq!(loaded.title, "测试书籍");
        assert_eq!(loaded.chapter_count, 10);
    }

    #[test]
    fn test_load_metadata_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let book_dir = create_book_dir(temp_dir.path(), 1).unwrap();

        let result = load_metadata(&book_dir);
        assert!(result.is_err());

        match result {
            Err(AppError::NotFound(msg)) => {
                assert!(msg.contains("Metadata not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }
}
