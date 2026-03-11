use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

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

/// Novel category information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelCategory {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// Novel chapter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelChapter {
    pub id: String,
    pub book_id: String,
    pub chapter_number: u32,
    pub title: String,
    pub content: String,
    pub word_count: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Novel book metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelBook {
    pub id: String,
    pub title: String,
    pub author: String,
    pub description: Option<String>,
    pub category_id: String,
    pub status: BookStatus,
    pub cover_url: Option<String>,
    pub total_chapters: u32,
    pub word_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Chapter preview for import operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterPreview {
    pub chapter_number: u32,
    pub title: String,
    pub word_count: u32,
}

/// Novel import preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub title: String,
    pub author: String,
    pub category: String,
    pub chapters: Vec<ChapterPreview>,
    pub total_chapters: u32,
    pub total_words: u64,
}

/// Novel bookmark for readers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NovelBookmark {
    pub id: String,
    pub book_id: String,
    pub chapter_id: String,
    pub user_id: String,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
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
    fn test_novel_category_serialization() {
        let category = NovelCategory {
            id: "cat1".to_string(),
            name: "Fantasy".to_string(),
            description: Some("Fantasy novels".to_string()),
        };
        let json = serde_json::to_string(&category).unwrap();
        assert!(json.contains("Fantasy"));
    }

    #[test]
    fn test_novel_book_serialization() {
        let book = NovelBook {
            id: "book1".to_string(),
            title: "The Great Novel".to_string(),
            author: "Author Name".to_string(),
            description: Some("A great novel".to_string()),
            category_id: "cat1".to_string(),
            status: BookStatus::Completed,
            cover_url: Some("https://example.com/cover.jpg".to_string()),
            total_chapters: 100,
            word_count: 500000,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&book).unwrap();
        assert!(json.contains("The Great Novel"));
        assert!(json.contains("completed"));
    }

    #[test]
    fn test_import_preview_creation() {
        let preview = ImportPreview {
            title: "Test Novel".to_string(),
            author: "Test Author".to_string(),
            category: "Fantasy".to_string(),
            chapters: vec![
                ChapterPreview {
                    chapter_number: 1,
                    title: "Chapter 1".to_string(),
                    word_count: 5000,
                },
                ChapterPreview {
                    chapter_number: 2,
                    title: "Chapter 2".to_string(),
                    word_count: 5500,
                },
            ],
            total_chapters: 2,
            total_words: 10500,
        };
        assert_eq!(preview.total_chapters, 2);
        assert_eq!(preview.total_words, 10500);
    }

    #[test]
    fn test_novel_bookmark_serialization() {
        let bookmark = NovelBookmark {
            id: "bm1".to_string(),
            book_id: "book1".to_string(),
            chapter_id: "ch1".to_string(),
            user_id: "user1".to_string(),
            note: Some("Great part!".to_string()),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&bookmark).unwrap();
        assert!(json.contains("book1"));
    }
}
