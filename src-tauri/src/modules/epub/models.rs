use serde::{Deserialize, Serialize};

/// EPUB 书籍
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubBook {
    pub id: i64,
    pub title: String,
    pub sort_title: Option<String>,
    pub isbn: Option<String>,
    pub publisher: Option<String>,
    pub pubdate: Option<String>,
    pub language: Option<String>,
    pub series: Option<String>,
    pub series_index: Option<f32>,
    pub rating: Option<i32>,
    pub file_path: String,
    pub file_size: i64,
    pub cover_path: Option<String>,
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 作者
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub sort_name: Option<String>,
    pub created_at: String,
}

/// 标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

/// 书籍-作者关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookAuthor {
    pub book_id: i64,
    pub author_id: i64,
    pub author_order: i32,
}

/// 书籍-标签关联
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookTag {
    pub book_id: i64,
    pub tag_id: i64,
}

/// 自定义字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    pub id: i64,
    pub name: String,
    pub label: String,
    pub datatype: CustomFieldType,
    pub is_multiple: bool,
    pub display_order: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CustomFieldType {
    Text,
    Series,
    Enumeration,
    Number,
    Rating,
    Date,
    Bool,
    Comments,
}

/// 自定义字段值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFieldValue {
    pub book_id: i64,
    pub field_id: i64,
    pub value: String,
}

/// 阅读进度
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingProgress {
    pub book_id: i64,
    pub chapter_href: String,
    pub progress_percent: f32,
    pub updated_at: String,
}

/// 书签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i64,
    pub book_id: i64,
    pub chapter_href: String,
    pub cfi: String,
    pub note: Option<String>,
    pub created_at: String,
}

/// 高亮
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub id: i64,
    pub book_id: i64,
    pub chapter_href: String,
    pub cfi_range: String,
    pub text: String,
    pub color: String,
    pub note: Option<String>,
    pub created_at: String,
}

/// EPUB 元数据（解析用）
#[derive(Debug, Clone)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub publisher: Option<String>,
    pub pubdate: Option<String>,
    pub language: Option<String>,
    pub isbn: Option<String>,
    pub description: Option<String>,
}

/// EPUB 章节信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubChapter {
    pub href: String,
    pub title: String,
    pub level: i32,
    pub order_index: i32,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ImportResult {
    Success { book_id: i64 },
    Failed { file_path: String, error: String },
}

/// 导入进度事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProgress {
    pub current: usize,
    pub total: usize,
    pub file_name: String,
}

/// 搜索查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub keyword: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub series: Option<String>,
    pub tags: Option<Vec<String>>,
    pub rating_min: Option<i32>,
    pub rating_max: Option<i32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// 书籍详情（包含作者和标签）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubBookWithDetails {
    pub book: EpubBook,
    pub authors: Vec<Author>,
    pub tags: Vec<Tag>,
}
