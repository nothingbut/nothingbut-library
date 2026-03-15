use crate::errors::AppResult;
use super::{database, models::*, vector, OllamaClient, DEFAULT_CHAT_MODEL, DEFAULT_EMBEDDING_MODEL, DEFAULT_OLLAMA_URL};
use sqlx::{SqlitePool, Row};
use tauri::{AppHandle, Emitter, State};
use futures::StreamExt;

/// 检查 Ollama 服务状态
#[tauri::command]
pub async fn check_ollama_status() -> AppResult<bool> {
    let client = OllamaClient::new(DEFAULT_OLLAMA_URL.to_string());
    client.health_check().await
}

/// 测试 Ollama 生成功能
#[tauri::command]
pub async fn test_ollama_generate(prompt: String, model: Option<String>) -> AppResult<String> {
    let client = OllamaClient::new(DEFAULT_OLLAMA_URL.to_string());
    let model = model.unwrap_or_else(|| DEFAULT_CHAT_MODEL.to_string());
    client.generate(&prompt, &model).await
}

/// 创建对话
#[tauri::command]
pub async fn create_conversation(
    pool: State<'_, SqlitePool>,
    title: String,
    context_type: String,
    context_id: Option<i64>,
) -> AppResult<i64> {
    database::create_conversation(&pool, &title, &context_type, context_id).await
}

/// 发送消息并获取 AI 响应（非流式版本，保留用于测试）
#[tauri::command]
pub async fn send_message(
    pool: State<'_, SqlitePool>,
    conversation_id: i64,
    message: String,
) -> AppResult<String> {
    // 保存用户消息
    database::add_message(&pool, conversation_id, "user", &message).await?;

    // 获取对话历史（最近10条）
    let history = database::get_conversation_messages(&pool, conversation_id, Some(10)).await?;
    let messages: Vec<ChatMessage> = history.iter().map(|m| m.to_chat_message()).collect();

    // 调用 Ollama
    let client = OllamaClient::new(DEFAULT_OLLAMA_URL.to_string());
    let response = client.chat(messages, DEFAULT_CHAT_MODEL).await?;

    // 保存 AI 响应
    database::add_message(&pool, conversation_id, "assistant", &response).await?;

    Ok(response)
}

/// 发送消息并获取 AI 响应（流式版本）
#[tauri::command]
pub async fn send_message_stream(
    app: AppHandle,
    pool: State<'_, SqlitePool>,
    conversation_id: i64,
    message: String,
) -> AppResult<()> {
    // 获取信号量许可（限制并发）
    let _permit = super::AI_REQUEST_SEMAPHORE.acquire().await
        .map_err(|e| crate::errors::AppError::Module(format!("Failed to acquire semaphore: {}", e)))?;

    // 保存用户消息
    database::add_message(&pool, conversation_id, "user", &message).await?;

    // 获取对话历史（最近10条）
    let history = database::get_conversation_messages(&pool, conversation_id, Some(10)).await?;
    let messages: Vec<ChatMessage> = history.iter().map(|m| m.to_chat_message()).collect();

    // 调用 Ollama 流式接口
    let client = OllamaClient::new(DEFAULT_OLLAMA_URL.to_string());
    let mut stream = client.chat_stream(messages, DEFAULT_CHAT_MODEL).await?;

    // 累积完整响应用于保存
    let mut full_response = String::new();

    // 逐个发送流式片段
    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                if !chunk.is_empty() {
                    full_response.push_str(&chunk);

                    // 发送事件到前端
                    app.emit("ai-message-chunk", serde_json::json!({
                        "conversation_id": conversation_id,
                        "chunk": chunk
                    })).ok();
                }
            }
            Err(e) => {
                // 发送错误事件
                app.emit("ai-message-error", serde_json::json!({
                    "conversation_id": conversation_id,
                    "error": e.to_string()
                })).ok();
                return Err(e);
            }
        }
    }

    // 发送完成事件
    app.emit("ai-message-done", serde_json::json!({
        "conversation_id": conversation_id
    })).ok();

    // 保存完整响应到数据库
    database::add_message(&pool, conversation_id, "assistant", &full_response).await?;

    Ok(())
}

/// 获取对话历史
#[tauri::command]
pub async fn get_conversation_history(
    pool: State<'_, SqlitePool>,
    conversation_id: i64,
    limit: Option<i32>,
) -> AppResult<Vec<StoredMessage>> {
    database::get_conversation_messages(&pool, conversation_id, limit).await
}

/// 生成章节摘要
#[tauri::command]
pub async fn summarize_chapter(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    chapter_id: i64,
    length: String,
) -> AppResult<String> {
    // 检查缓存
    if let Some(cached) = database::get_summary(&pool, "chapter", chapter_id, &length).await? {
        return Ok(cached);
    }

    // 获取信号量许可（限制并发）
    let _permit = super::AI_REQUEST_SEMAPHORE.acquire().await
        .map_err(|e| crate::errors::AppError::Module(format!("Failed to acquire semaphore: {}", e)))?;

    // 读取章节内容
    let content = crate::modules::novel::commands::get_chapter_content(pool.clone(), workspace_path, chapter_id).await?;

    // 生成摘要
    let summary_length = match length.as_str() {
        "short" => SummaryLength::Short,
        "medium" => SummaryLength::Medium,
        "long" => SummaryLength::Long,
        _ => SummaryLength::Medium,
    };

    let client = OllamaClient::new(DEFAULT_OLLAMA_URL.to_string());
    let summary = super::summarize::generate_summary(&client, &content, summary_length, DEFAULT_CHAT_MODEL).await?;

    // 保存缓存
    database::save_summary(&pool, "chapter", chapter_id, &summary, &length, DEFAULT_CHAT_MODEL).await?;

    Ok(summary)
}

/// 为单个章节建立向量索引
#[tauri::command]
pub async fn index_chapter(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    chapter_id: i64,
) -> AppResult<()> {
    // 获取信号量许可
    let _permit = super::AI_REQUEST_SEMAPHORE.acquire().await
        .map_err(|e| crate::errors::AppError::Module(format!("Failed to acquire semaphore: {}", e)))?;

    // 获取章节内容
    let content = crate::modules::novel::commands::get_chapter_content(pool.clone(), workspace_path, chapter_id).await?;
    let content_hash = vector::compute_content_hash(&content);

    // 检查是否需要重新索引
    if vector::is_chapter_indexed(&pool, chapter_id, &content_hash).await? {
        return Ok(()); // 已索引且内容未变
    }

    // 生成嵌入向量
    let client = OllamaClient::new(DEFAULT_OLLAMA_URL.to_string());
    let embedding = client.embeddings(&content, DEFAULT_EMBEDDING_MODEL).await?;

    // 保存到数据库
    vector::save_chapter_embedding(&pool, chapter_id, &embedding, &content_hash).await?;

    Ok(())
}

/// 批量索引整本书的所有章节
#[tauri::command]
pub async fn index_book(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    book_id: i64,
) -> AppResult<Vec<i64>> {
    // 获取所有章节 ID
    let rows = sqlx::query(
        r#"
        SELECT id
        FROM novel_chapters
        WHERE book_id = ?
        ORDER BY chapter_number
        "#
    )
    .bind(book_id)
    .fetch_all(&*pool)
    .await?;

    let chapter_ids: Vec<i64> = rows.iter().map(|row| row.get("id")).collect();
    let mut indexed_ids = Vec::new();

    // 逐个索引章节
    for chapter_id in chapter_ids {
        match index_chapter(pool.clone(), workspace_path.clone(), chapter_id).await {
            Ok(_) => {
                indexed_ids.push(chapter_id);
            }
            Err(e) => {
                eprintln!("Failed to index chapter {}: {}", chapter_id, e);
            }
        }
    }

    Ok(indexed_ids)
}

/// 语义搜索
#[tauri::command]
pub async fn semantic_search(
    pool: State<'_, SqlitePool>,
    query: String,
    book_id: Option<i64>,
    limit: Option<i32>,
    min_similarity: Option<f32>,
) -> AppResult<Vec<SearchResult>> {
    let limit = limit.unwrap_or(10) as usize;
    let min_similarity = min_similarity.unwrap_or(0.7);

    // 获取信号量许可
    let _permit = super::AI_REQUEST_SEMAPHORE.acquire().await
        .map_err(|e| crate::errors::AppError::Module(format!("Failed to acquire semaphore: {}", e)))?;

    // 生成查询向量
    let client = OllamaClient::new(DEFAULT_OLLAMA_URL.to_string());
    let query_embedding = client.embeddings(&query, DEFAULT_EMBEDDING_MODEL).await?;

    // 搜索相似章节
    let similar_chapters = vector::search_similar_chapters(
        &pool,
        &query_embedding,
        book_id,
        limit,
        min_similarity,
    ).await?;

    // 获取章节详细信息
    let mut results = Vec::new();
    for (chapter_id, similarity) in similar_chapters {
        // 查询章节和书籍信息
        let row = sqlx::query(
            r#"
            SELECT
                nc.id as chapter_id,
                nc.title as chapter_title,
                nc.chapter_number,
                nb.id as book_id,
                nb.title as book_title
            FROM novel_chapters nc
            INNER JOIN novel_books nb ON nc.book_id = nb.id
            WHERE nc.id = ?
            "#
        )
        .bind(chapter_id)
        .fetch_optional(&*pool)
        .await?;

        if let Some(row) = row {
            // 获取章节预览（前200字）
            let preview = match get_chapter_preview(&pool, chapter_id).await {
                Ok(p) => p,
                Err(_) => String::from("无预览"),
            };

            results.push(SearchResult {
                chapter_id: row.get("chapter_id"),
                chapter_title: row.get("chapter_title"),
                chapter_number: row.get("chapter_number"),
                book_id: row.get("book_id"),
                book_title: row.get("book_title"),
                similarity,
                preview,
            });
        }
    }

    Ok(results)
}

/// 获取章节预览（前200字）
async fn get_chapter_preview(pool: &SqlitePool, chapter_id: i64) -> AppResult<String> {
    let row = sqlx::query(
        r#"
        SELECT content
        FROM novel_chapters
        WHERE id = ?
        "#
    )
    .bind(chapter_id)
    .fetch_one(pool)
    .await?;

    let content: String = row.get("content");
    let preview: String = content.chars().take(200).collect();
    Ok(preview)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_check_ollama_status() {
        let status = check_ollama_status().await.unwrap();
        assert!(status);
    }

    #[tokio::test]
    #[ignore]
    async fn test_test_ollama_generate() {
        let response = test_ollama_generate(
            "Say hello in Chinese".to_string(),
            Some("qwen2.5:7b".to_string()),
        )
        .await
        .unwrap();

        println!("Response: {}", response);
        assert!(!response.is_empty());
    }
}
