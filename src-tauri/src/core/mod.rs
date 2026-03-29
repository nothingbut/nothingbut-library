pub mod commands;
pub mod config;
pub mod library;
pub mod models;
pub mod traits;
pub mod workspace;

pub use config::AppConfig;
pub use library::{Library, CreateLibraryRequest};
pub use models::{Category, LibraryItem, ModuleType, Workspace, WorkspaceConfig};
pub use traits::{AIEnhanced, Categorizable, LibraryModule, Searchable};
pub use workspace::{create_workspace, get_database_path, load_workspace_config};
