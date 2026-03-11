use serde::{Deserialize, Serialize};

/// Novel book publication status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookStatus {
    /// Book has been completed
    Completed,
    /// Book is currently being serialized
    Ongoing,
    /// Book has been abandoned
    Abandoned,
}

impl std::fmt::Display for BookStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BookStatus::Completed => write!(f, "completed"),
            BookStatus::Ongoing => write!(f, "ongoing"),
            BookStatus::Abandoned => write!(f, "abandoned"),
        }
    }
}

impl BookStatus {
    /// Parse from database string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "completed" => Some(BookStatus::Completed),
            "ongoing" => Some(BookStatus::Ongoing),
            "abandoned" => Some(BookStatus::Abandoned),
            _ => None,
        }
    }
}

/// Novel category information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelCategory {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
    pub created_at: String,
}

/// Novel chapter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelChapter {
    pub id: i64,
    pub book_id: i64,
    pub title: String,
    pub file_path: String,
    pub sort_order: i32,
    pub word_count: i64,
    pub created_at: String,
}

/// Novel book metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelBook {
    pub id: i64,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub cover_path: Option<String>,
    pub category_id: Option<i64>,
    pub book_dir: String,
    pub file_size: i64,
    pub word_count: i64,
    pub chapter_count: i32,
    pub status: BookStatus,
    pub reading_progress: f64,
    pub last_read_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Chapter preview for import operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterPreview {
    pub chapter_number: u32,
    pub title: String,
    pub preview: String, // First line preview (up to 20 chars)
    pub word_count: u32,
}

/// Novel import preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub title: String,
    pub author: Option<String>, // Auto-extracted if found
    pub description: Option<String>, // Auto-extracted if found
    pub category: String,
    pub chapters: Vec<ChapterPreview>,
    pub total_chapters: u32,
    pub total_words: u64,
}

/// Novel bookmark for readers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelBookmark {
    pub id: i64,
    pub book_id: i64,
    pub chapter_id: Option<i64>,
    pub position: i64,
    pub note: Option<String>,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_status_display() {
        assert_eq!(BookStatus::Completed.to_string(), "completed");
        assert_eq!(BookStatus::Ongoing.to_string(), "ongoing");
        assert_eq!(BookStatus::Abandoned.to_string(), "abandoned");
    }

    #[test]
    fn test_book_status_serialization() {
        let status = BookStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"completed\"");
    }

    #[test]
    fn test_book_status_deserialization() {
        let json = r#""ongoing""#;
        let status: BookStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status, BookStatus::Ongoing);
    }

    #[test]
    fn test_book_status_from_str() {
        assert_eq!(BookStatus::from_str("completed"), Some(BookStatus::Completed));
        assert_eq!(BookStatus::from_str("ongoing"), Some(BookStatus::Ongoing));
        assert_eq!(BookStatus::from_str("abandoned"), Some(BookStatus::Abandoned));
        assert_eq!(BookStatus::from_str("invalid"), None);
    }

    #[test]
    fn test_novel_category_serialization() {
        let category = NovelCategory {
            id: 1,
            name: "Fantasy".to_string(),
            parent_id: None,
            sort_order: 0,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&category).unwrap();
        assert!(json.contains("Fantasy"));
    }

    #[test]
    fn test_novel_book_serialization() {
        let book = NovelBook {
            id: 1,
            title: "The Great Novel".to_string(),
            author: Some("Author Name".to_string()),
            description: Some("A great novel".to_string()),
            cover_path: Some("covers/book-1.jpg".to_string()),
            category_id: Some(1),
            book_dir: "books/book-1".to_string(),
            file_size: 1024000,
            word_count: 500000,
            chapter_count: 100,
            status: BookStatus::Completed,
            reading_progress: 0.0,
            last_read_at: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&book).unwrap();
        assert!(json.contains("The Great Novel"));
        assert!(json.contains("completed"));
    }

    #[test]
    fn test_import_preview_creation() {
        let preview = ImportPreview {
            title: "Test Novel".to_string(),
            author: Some("Test Author".to_string()),
            description: Some("Test description".to_string()),
            category: "Fantasy".to_string(),
            chapters: vec![
                ChapterPreview {
                    chapter_number: 1,
                    title: "Chapter 1".to_string(),
                    preview: "This is the first...".to_string(),
                    word_count: 5000,
                },
                ChapterPreview {
                    chapter_number: 2,
                    title: "Chapter 2".to_string(),
                    preview: "This is the second...".to_string(),
                    word_count: 5500,
                },
            ],
            total_chapters: 2,
            total_words: 10500,
        };
        assert_eq!(preview.total_chapters, 2);
        assert_eq!(preview.total_words, 10500);
        assert_eq!(preview.author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_novel_bookmark_serialization() {
        let bookmark = NovelBookmark {
            id: 1,
            book_id: 1,
            chapter_id: Some(1),
            position: 1000,
            note: Some("Great part!".to_string()),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&bookmark).unwrap();
        assert!(json.contains("1000"));
    }
}
