use serde::Deserialize;
use sqlx::SqlitePool;
use std::path::Path;
use crate::errors::{AppError, AppResult};

/// Category structure from bsconfig.json
#[derive(Debug, Deserialize)]
struct CategoryData {
    category: String,
    subcategories: Vec<String>,
}

/// Root structure for bsconfig.json
#[derive(Debug, Deserialize)]
struct BsConfig {
    #[serde(rename = "attr.tagsJson")]
    tags_json: Vec<CategoryData>,
}

/// Seed categories from bsconfig.json into the database
///
/// This function reads category data from bsconfig.json and inserts them into the database.
/// It creates a two-level hierarchy: main categories (parent_id = NULL) and subcategories.
///
/// Returns the number of categories inserted (main + subcategories).
pub async fn seed_categories_from_config(
    pool: &SqlitePool,
    config_path: &Path,
) -> AppResult<usize> {
    // Read and parse bsconfig.json
    let content = std::fs::read_to_string(config_path)
        .map_err(|e| AppError::Io(format!("Failed to read config file: {}", e)))?;

    let config: BsConfig = serde_json::from_str(&content)
        .map_err(|e| AppError::Json(format!("Failed to parse config file: {}", e)))?;

    // Check if categories already exist
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM novel_categories")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(format!("Failed to check existing categories: {}", e)))?;

    if count > 0 {
        return Ok(0); // Categories already seeded
    }

    let mut inserted_count = 0;

    // Insert categories with their subcategories
    for (sort_order, cat_data) in config.tags_json.into_iter().enumerate() {
        // Insert main category
        let parent_id = super::database::insert_category(
            pool,
            &cat_data.category,
            None,
            sort_order as i32,
        ).await?;

        inserted_count += 1;

        // Insert subcategories
        for (sub_sort_order, subcat) in cat_data.subcategories.into_iter().enumerate() {
            super::database::insert_category(
                pool,
                &subcat,
                Some(parent_id),
                sub_sort_order as i32,
            ).await?;

            inserted_count += 1;
        }
    }

    Ok(inserted_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::io::Write;
    use tempfile::NamedTempFile;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Create categories table
        sqlx::query(
            r#"
            CREATE TABLE novel_categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                parent_id INTEGER,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    fn create_test_config() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        let config_content = r#"{
            "attr.tagsJson": [
                {
                    "category": "玄幻",
                    "subcategories": ["东方玄幻", "异世大陆"]
                },
                {
                    "category": "科幻",
                    "subcategories": ["未来世界", "星际文明"]
                }
            ]
        }"#;

        file.write_all(config_content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[tokio::test]
    async fn test_seed_categories_from_config() {
        let pool = setup_test_db().await;
        let config_file = create_test_config();

        let count = seed_categories_from_config(&pool, config_file.path())
            .await
            .unwrap();

        // Should insert 2 main categories + 4 subcategories = 6 total
        assert_eq!(count, 6);

        // Verify categories were inserted correctly
        let categories = super::super::database::list_categories(&pool)
            .await
            .unwrap();

        assert_eq!(categories.len(), 6);

        // Check main categories
        let main_cats: Vec<_> = categories
            .iter()
            .filter(|c| c.parent_id.is_none())
            .collect();
        assert_eq!(main_cats.len(), 2);
        assert_eq!(main_cats[0].name, "玄幻");
        assert_eq!(main_cats[1].name, "科幻");

        // Check subcategories
        let sub_cats: Vec<_> = categories
            .iter()
            .filter(|c| c.parent_id.is_some())
            .collect();
        assert_eq!(sub_cats.len(), 4);
    }

    #[tokio::test]
    async fn test_seed_categories_idempotent() {
        let pool = setup_test_db().await;
        let config_file = create_test_config();

        // First seeding
        let count1 = seed_categories_from_config(&pool, config_file.path())
            .await
            .unwrap();
        assert_eq!(count1, 6);

        // Second seeding should return 0 (no duplicates)
        let count2 = seed_categories_from_config(&pool, config_file.path())
            .await
            .unwrap();
        assert_eq!(count2, 0);

        // Should still have only 6 categories
        let categories = super::super::database::list_categories(&pool)
            .await
            .unwrap();
        assert_eq!(categories.len(), 6);
    }

    #[tokio::test]
    async fn test_seed_categories_invalid_path() {
        let pool = setup_test_db().await;
        let result = seed_categories_from_config(&pool, Path::new("/nonexistent/path.json"))
            .await;

        assert!(result.is_err());
        match result {
            Err(AppError::Io(_)) => {} // Expected
            _ => panic!("Expected Io error"),
        }
    }

    #[tokio::test]
    async fn test_seed_categories_invalid_json() {
        let pool = setup_test_db().await;

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"{ invalid json }").unwrap();
        file.flush().unwrap();

        let result = seed_categories_from_config(&pool, file.path()).await;

        assert!(result.is_err());
        match result {
            Err(AppError::Json(_)) => {} // Expected
            _ => panic!("Expected Json error"),
        }
    }
}
