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
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:library.db", database::get_migrations())
                .build(),
        )
        .setup(|app| {
            let app_handle = app.handle();
            tauri::async_runtime::block_on(async move {
                // Initialize SQLite connection pool
                let db_path = app_handle
                    .path()
                    .app_data_dir()
                    .expect("Failed to get app data dir")
                    .join("library.db");

                // Ensure parent directory exists
                if let Some(parent) = db_path.parent() {
                    std::fs::create_dir_all(parent)
                        .expect("Failed to create app data directory");
                }

                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&format!("sqlite:{}", db_path.display()))
                    .await
                    .expect("Failed to connect to database");

                // Run migrations
                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .expect("Failed to run migrations");

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
            modules::novel::commands::create_category,
            modules::novel::commands::list_categories,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
