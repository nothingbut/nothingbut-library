use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModuleType {
    Novel,
    Music,
    Ebook,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub name: String,
    pub path: PathBuf,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    pub modules: Vec<ModuleType>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItem {
    pub id: String,
    pub title: String,
    pub module_type: ModuleType,
    pub file_path: PathBuf,
    pub category_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub module_type: ModuleType,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_module_type_serialization() {
        let module_type = ModuleType::Novel;
        let json = serde_json::to_string(&module_type).unwrap();
        assert_eq!(json, "\"Novel\"");

        let deserialized: ModuleType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ModuleType::Novel);
    }

    #[test]
    fn test_workspace_creation() {
        let workspace = Workspace {
            name: "Test Workspace".to_string(),
            path: PathBuf::from("/test/path"),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(workspace.name, "Test Workspace");
        assert_eq!(workspace.path, PathBuf::from("/test/path"));
    }

    #[test]
    fn test_workspace_config_serialization() {
        let config = WorkspaceConfig {
            name: "Test Config".to_string(),
            modules: vec![ModuleType::Novel, ModuleType::Music],
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WorkspaceConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, "Test Config");
        assert_eq!(deserialized.modules.len(), 2);
    }

    #[test]
    fn test_library_item_with_metadata() {
        let item = LibraryItem {
            id: "123".to_string(),
            title: "Test Novel".to_string(),
            module_type: ModuleType::Novel,
            file_path: PathBuf::from("/path/to/novel.txt"),
            category_id: Some("cat1".to_string()),
            metadata: json!({"author": "Test Author", "pages": 300}),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(item.title, "Test Novel");
        assert_eq!(item.metadata["author"], "Test Author");
        assert_eq!(item.metadata["pages"], 300);
    }

    #[test]
    fn test_category_hierarchy() {
        let parent = Category {
            id: "parent1".to_string(),
            name: "Parent Category".to_string(),
            parent_id: None,
            module_type: ModuleType::Novel,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let child = Category {
            id: "child1".to_string(),
            name: "Child Category".to_string(),
            parent_id: Some("parent1".to_string()),
            module_type: ModuleType::Novel,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        assert!(parent.parent_id.is_none());
        assert_eq!(child.parent_id, Some("parent1".to_string()));
    }

    #[test]
    fn test_library_item_serialization() {
        let item = LibraryItem {
            id: "test-id".to_string(),
            title: "Test".to_string(),
            module_type: ModuleType::Ebook,
            file_path: PathBuf::from("/test.epub"),
            category_id: None,
            metadata: json!({}),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&item).unwrap();
        let deserialized: LibraryItem = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "test-id");
        assert_eq!(deserialized.module_type, ModuleType::Ebook);
    }
}
