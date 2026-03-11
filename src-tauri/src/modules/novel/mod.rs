pub mod commands;
pub mod database;
pub mod models;
pub mod parser;
pub mod storage;

pub use models::{
    BookStatus, NovelBook, NovelBookmark, NovelCategory, NovelChapter, ChapterPreview,
    ImportPreview,
};
pub use parser::{TxtParser, Chapter, DetectedEncoding};
pub use storage::{BookMetadata, count_words, create_book_dir, save_chapter, save_metadata, load_metadata};
pub use commands::{
    preview_import, import_novel, list_books, list_chapters,
    create_category, list_categories,
};
