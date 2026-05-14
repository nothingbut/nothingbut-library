pub mod ai;
pub mod bilibili;
pub mod epub;
pub mod music;
pub mod novel;
pub mod weread;

pub use novel::{BookStatus, NovelBook, NovelBookmark, NovelCategory, NovelChapter, ChapterPreview, ImportPreview};
