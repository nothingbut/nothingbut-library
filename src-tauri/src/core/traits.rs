use crate::core::models::{Category, LibraryItem};
use crate::errors::AppResult;
use async_trait::async_trait;

#[async_trait]
pub trait LibraryModule: Send + Sync {
    async fn import(&self, file_path: &str) -> AppResult<LibraryItem>;
    async fn get_by_id(&self, id: &str) -> AppResult<LibraryItem>;
    async fn list(&self, category_id: Option<&str>) -> AppResult<Vec<LibraryItem>>;
    async fn delete(&self, id: &str) -> AppResult<()>;
    async fn update_metadata(&self, id: &str, metadata: serde_json::Value) -> AppResult<LibraryItem>;
}

#[async_trait]
pub trait Searchable: Send + Sync {
    async fn search(&self, query: &str) -> AppResult<Vec<LibraryItem>>;
    async fn search_advanced(&self, filters: serde_json::Value) -> AppResult<Vec<LibraryItem>>;
}

#[async_trait]
pub trait Categorizable: Send + Sync {
    async fn create_category(&self, name: &str, parent_id: Option<&str>) -> AppResult<Category>;
    async fn get_categories(&self) -> AppResult<Vec<Category>>;
    async fn move_to_category(&self, item_id: &str, category_id: Option<&str>) -> AppResult<()>;
}

#[async_trait]
pub trait AIEnhanced: Send + Sync {
    async fn generate_summary(&self, item_id: &str) -> AppResult<String>;
    async fn extract_keywords(&self, item_id: &str) -> AppResult<Vec<String>>;
    async fn semantic_search(&self, query: &str, top_k: usize) -> AppResult<Vec<LibraryItem>>;
}
