use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WereadAccount {
    pub id: i64,
    pub vid: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub cookies_json: String,
    pub cookie_expires_at: Option<i64>,
    pub is_active: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WereadBook {
    pub id: i64,
    pub library_id: i64,
    pub book_id: String,
    pub title: String,
    pub author: Option<String>,
    pub cover_url: Option<String>,
    pub intro: Option<String>,
    pub category: Option<String>,
    pub word_count: i64,
    pub chapter_count: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WereadChapter {
    pub id: i64,
    pub library_id: i64,
    pub book_id: i64,
    pub chapter_uid: String,
    pub title: String,
    pub level: i64,
    pub word_count: i64,
    pub sort_order: i64,
    pub content_md: Option<String>,
    pub extracted_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WereadDownload {
    pub id: i64,
    pub library_id: i64,
    pub book_id: i64,
    pub status: String,
    pub progress_current: i64,
    pub progress_total: i64,
    pub output_path: Option<String>,
    pub error_message: Option<String>,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginStatus {
    pub logged_in: bool,
    pub account: Option<WereadAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProgress {
    pub book_id: i64,
    pub title: String,
    pub current: i64,
    pub total: i64,
    pub status: String,
}
