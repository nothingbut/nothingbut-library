use crate::core::models::{ModuleType, Workspace, WorkspaceConfig};
use crate::errors::{AppError, AppResult};
use std::fs;
use std::path::{Path, PathBuf};

pub fn create_workspace(name: &str, base_path: &Path, modules: Vec<ModuleType>) -> AppResult<Workspace> {
    let workspace_path = base_path.join(name);

    if workspace_path.exists() {
        return Err(AppError::Workspace(format!("Workspace '{}' already exists", name)));
    }

    fs::create_dir_all(&workspace_path)?;
    fs::create_dir_all(workspace_path.join("data"))?;
    fs::create_dir_all(workspace_path.join("config"))?;

    let config = WorkspaceConfig {
        name: name.to_string(),
        modules,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let config_path = workspace_path.join("config").join("workspace.json");
    let config_json = serde_json::to_string_pretty(&config)?;
    fs::write(&config_path, config_json)?;

    Ok(Workspace {
        name: name.to_string(),
        path: workspace_path,
        created_at: config.created_at,
    })
}

pub fn load_workspace_config(workspace_path: &Path) -> AppResult<WorkspaceConfig> {
    let config_path = workspace_path.join("config").join("workspace.json");

    if !config_path.exists() {
        return Err(AppError::NotFound(format!(
            "Workspace config not found at {:?}",
            config_path
        )));
    }

    let config_content = fs::read_to_string(config_path)?;
    let config: WorkspaceConfig = serde_json::from_str(&config_content)?;

    Ok(config)
}

pub fn get_database_path(workspace_path: &Path) -> PathBuf {
    workspace_path.join("data").join("library.db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_create_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let modules = vec![ModuleType::Novel, ModuleType::Music];

        let workspace = create_workspace("test_workspace", temp_dir.path(), modules.clone()).unwrap();

        assert_eq!(workspace.name, "test_workspace");
        assert!(workspace.path.exists());
        assert!(workspace.path.join("data").exists());
        assert!(workspace.path.join("config").exists());
        assert!(workspace.path.join("config").join("workspace.json").exists());
    }

    #[test]
    fn test_create_duplicate_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let modules = vec![ModuleType::Novel];

        create_workspace("duplicate", temp_dir.path(), modules.clone()).unwrap();
        let result = create_workspace("duplicate", temp_dir.path(), modules);

        assert!(result.is_err());
        match result {
            Err(AppError::Workspace(msg)) => assert!(msg.contains("already exists")),
            _ => panic!("Expected Workspace error"),
        }
    }

    #[test]
    fn test_load_workspace_config() {
        let temp_dir = TempDir::new().unwrap();
        let modules = vec![ModuleType::Novel, ModuleType::Ebook];

        let workspace = create_workspace("config_test", temp_dir.path(), modules).unwrap();
        let config = load_workspace_config(&workspace.path).unwrap();

        assert_eq!(config.name, "config_test");
        assert_eq!(config.modules.len(), 2);
        assert_eq!(config.modules[0], ModuleType::Novel);
        assert_eq!(config.modules[1], ModuleType::Ebook);
    }

    #[test]
    fn test_load_nonexistent_config() {
        let temp_dir = TempDir::new().unwrap();
        let result = load_workspace_config(temp_dir.path());

        assert!(result.is_err());
        match result {
            Err(AppError::NotFound(_)) => (),
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_get_database_path() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().join("test_workspace");

        let db_path = get_database_path(&workspace_path);

        assert_eq!(db_path, workspace_path.join("data").join("library.db"));
    }

    #[test]
    fn test_workspace_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let modules = vec![ModuleType::Note];

        let workspace = create_workspace("structure_test", temp_dir.path(), modules).unwrap();

        assert!(workspace.path.join("data").is_dir());
        assert!(workspace.path.join("config").is_dir());

        let config_file = workspace.path.join("config").join("workspace.json");
        assert!(config_file.is_file());

        let content = fs::read_to_string(config_file).unwrap();
        assert!(content.contains("structure_test"));
        assert!(content.contains("Note"));
    }
}
