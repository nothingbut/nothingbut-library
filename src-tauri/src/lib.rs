pub mod core;
pub mod database;
pub mod errors;
pub mod modules;

pub use errors::{AppError, AppResult};
pub use modules::{BookStatus, NovelBook, NovelBookmark, NovelCategory, NovelChapter, ChapterPreview, ImportPreview};

use sqlx::sqlite::SqlitePoolOptions;
use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:library.db", database::get_migrations())
                .build(),
        )
        .setup(|app| {
            let app_handle = app.handle();
            tauri::async_runtime::block_on(async move {
                // Initialize SQLite connection pool
                // In development mode, use local directory for easier access
                #[cfg(debug_assertions)]
                let db_path = {
                    // Use project root directory (parent of src-tauri)
                    let path = std::env::current_dir()
                        .expect("Failed to get current directory")
                        .parent()
                        .expect("Failed to get parent directory")
                        .join("library.db");
                    println!("[DEV] Using database path: {:?}", path);
                    path
                };

                #[cfg(not(debug_assertions))]
                let db_path = {
                    let path = app_handle
                        .path()
                        .app_data_dir()
                        .expect("Failed to get app data dir")
                        .join("library.db");
                    println!("[PROD] Using database path: {:?}", path);
                    path
                };

                // Ensure parent directory exists
                if let Some(parent) = db_path.parent() {
                    println!("Creating directory if needed: {:?}", parent);
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create directory {:?}: {}", parent, e))
                        .expect("Directory creation failed");
                }

                // Connect to database with create mode
                let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
                println!("Connecting to database: {}", db_url);

                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&db_url)
                    .await
                    .map_err(|e| format!("Failed to connect to database at {:?}: {}", db_path, e))
                    .expect("Database connection failed");

                println!("Database connected successfully");

                // Run migrations
                println!("Running database migrations...");
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .map_err(|e| format!("Failed to run migrations: {}", e))
                    .expect("Migration failed");

                println!("Migrations completed successfully");

                // Store pool in app state
                app_handle.manage(pool);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            modules::novel::commands::preview_import,
            modules::novel::commands::import_novel,
            modules::novel::commands::list_books,
            modules::novel::commands::list_chapters,
            modules::novel::commands::get_chapter_content,
            modules::novel::commands::create_category,
            modules::novel::commands::list_categories,
            modules::novel::commands::seed_categories,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
