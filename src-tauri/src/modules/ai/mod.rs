pub mod commands;
pub mod database;
pub mod models;
pub mod ollama;
pub mod summarize;
pub mod vector;

pub use models::*;
pub use ollama::OllamaClient;

use tokio::sync::Semaphore;
use std::sync::LazyLock;

// 默认模型配置
pub const DEFAULT_CHAT_MODEL: &str = "qwen2.5:7b";
pub const DEFAULT_EMBEDDING_MODEL: &str = "nomic-embed-text";
pub const DEFAULT_OLLAMA_URL: &str = "http://localhost:11434";

// 并发控制：最多 3 个并发 AI 请求
pub static AI_REQUEST_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(3));
