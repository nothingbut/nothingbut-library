use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub default_workspace: Option<String>,
    pub recent_workspaces: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_workspace: None,
            recent_workspaces: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(config.default_workspace.is_none());
        assert!(config.recent_workspaces.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig {
            default_workspace: Some("/path/to/workspace".to_string()),
            recent_workspaces: vec![
                "/path/to/workspace1".to_string(),
                "/path/to/workspace2".to_string(),
            ],
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.default_workspace, Some("/path/to/workspace".to_string()));
        assert_eq!(deserialized.recent_workspaces.len(), 2);
    }
}
