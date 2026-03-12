use std::path::Path;
use tauri_plugin_sql::{Migration, MigrationKind};

/// Get all database migrations in order
pub fn get_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "create core tables",
            sql: include_str!("../migrations/0001_core.sql"),
            kind: MigrationKind::Up,
        },
        Migration {
            version: 2,
            description: "create novel module tables",
            sql: include_str!("../migrations/0002_novel.sql"),
            kind: MigrationKind::Up,
        },
    ]
}

/// Initialize workspace database at given path
pub async fn init_workspace_db(workspace_path: &Path) -> crate::AppResult<()> {
    if !workspace_path.exists() {
        return Err(crate::AppError::Database(format!(
            "Workspace path does not exist: {}",
            workspace_path.display()
        )));
    }

    // Database initialization is handled by tauri-plugin-sql
    // This function validates the workspace path
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_migrations() {
        let migrations = get_migrations();
        assert_eq!(migrations.len(), 2);

        assert_eq!(migrations[0].version, 1);
        assert_eq!(migrations[0].description, "create core tables");
        assert!(migrations[0].sql.contains("app_workspaces"));
        assert!(migrations[0].sql.contains("library_config"));

        assert_eq!(migrations[1].version, 2);
        assert_eq!(migrations[1].description, "create novel module tables");
        assert!(migrations[1].sql.contains("novel_categories"));
        assert!(migrations[1].sql.contains("novel_books"));
        assert!(migrations[1].sql.contains("novel_chapters"));
    }

    #[tokio::test]
    async fn test_init_workspace_db_success() {
        let temp_dir = TempDir::new().unwrap();
        let result = init_workspace_db(temp_dir.path()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_workspace_db_nonexistent_path() {
        let nonexistent = Path::new("/nonexistent/path");
        let result = init_workspace_db(nonexistent).await;
        assert!(result.is_err());

        if let Err(crate::AppError::Database(msg)) = result {
            assert!(msg.contains("does not exist"));
        } else {
            panic!("Expected Database error");
        }
    }
}
