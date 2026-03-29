use crate::errors::AppResult;
use super::{assistant, database, models::*, vector, OllamaClient, DEFAULT_CHAT_MODEL, DEFAULT_EMBEDDING_MODEL, DEFAULT_OLLAMA_URL};
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

// ==================== AI 助手命令 ====================

/// AI 助手对话（简化版本，不依赖原生 function calling）
#[tauri::command]
pub async fn assistant_chat(
    pool: State<'_, SqlitePool>,
    user_message: String,
    conversation_history: Option<Vec<ChatMessage>>,
) -> AppResult<AssistantResponse> {
    // 获取信号量许可
    let _permit = super::AI_REQUEST_SEMAPHORE.acquire().await
        .map_err(|e| crate::errors::AppError::Module(format!("Failed to acquire semaphore: {}", e)))?;

    // 构建消息历史
    let mut messages = conversation_history.unwrap_or_else(|| {
        vec![ChatMessage {
            role: MessageRole::System,
            content: r#"你是一个智能助手，帮助用户管理图书馆和音乐库。

可用命令：
1. "列出所有库" - 显示所有库
2. "切换到[库名]" - 切换到指定的库
3. "搜索[书名/作者]" - 搜索书籍
4. "播放[歌名/歌手]" - 搜索并播放音乐

请用简洁友好的中文回复，并在需要时直接执行相应操作。"#.to_string(),
        }]
    });

    // 添加用户消息
    messages.push(ChatMessage {
        role: MessageRole::User,
        content: user_message.clone(),
    });

    // 调用 Ollama（普通聊天）
    let client = OllamaClient::new(DEFAULT_OLLAMA_URL.to_string());

    // 先检测用户意图并执行相应工具
    let tool_result = detect_and_execute_tool(&pool, &user_message).await;

    if let Some((tool_name, result)) = tool_result {
        // 有工具执行结果，添加到上下文
        messages.push(ChatMessage {
            role: MessageRole::System,
            content: format!("工具 {} 执行结果:\n{}", tool_name, result),
        });
    }

    // 生成AI响应
    let response = client.chat(messages.clone(), DEFAULT_CHAT_MODEL).await?;

    // 保存助手响应到历史
    messages.push(ChatMessage {
        role: MessageRole::Assistant,
        content: response.clone(),
    });

    Ok(AssistantResponse {
        message: response,
        tool_calls: None,
        tool_results: None,
        conversation_history: messages,
    })
}

/// 检测用户意图并执行相应工具（简单的关键词匹配）
async fn detect_and_execute_tool(
    pool: &SqlitePool,
    user_message: &str,
) -> Option<(String, String)> {
    let msg_lower = user_message.to_lowercase();

    // 列出所有库
    if msg_lower.contains("列出") && (msg_lower.contains("库") || msg_lower.contains("library")) {
        match assistant::execute_tool(pool, "list_libraries", serde_json::json!({})).await {
            Ok(result) => return Some(("list_libraries".to_string(), result)),
            Err(e) => return Some(("list_libraries".to_string(), format!("错误: {}", e))),
        }
    }

    // 获取当前库
    if msg_lower.contains("当前") && msg_lower.contains("库") {
        match assistant::execute_tool(pool, "get_current_library", serde_json::json!({})).await {
            Ok(result) => return Some(("get_current_library".to_string(), result)),
            Err(e) => return Some(("get_current_library".to_string(), format!("错误: {}", e))),
        }
    }

    // 切换库 - 必须包含"库"字，避免误触发
    if (msg_lower.contains("切换") || msg_lower.contains("打开")) && msg_lower.contains("库") {
        // 尝试多种模式提取库名
        let library_name = if let Some(idx) = user_message.find("切换到") {
            user_message[idx + "切换到".len()..].trim().to_string()
        } else if let Some(idx) = user_message.find("打开") {
            user_message[idx + "打开".len()..].trim().to_string()
        } else if let Some(idx) = user_message.find("切换") {
            // 处理 "切换XX库" 的情况
            user_message[idx + "切换".len()..].trim().to_string()
        } else if let Some(idx) = user_message.find("到") {
            user_message[idx + "到".len()..].trim().to_string()
        } else {
            String::new()
        };

        let library_name = library_name.trim().to_string();

        if !library_name.is_empty() {
            eprintln!("🔍 检测到切换库命令，提取的库名: '{}'", library_name);

            let args = serde_json::json!({
                "library_name": library_name
            });
            match assistant::execute_tool(pool, "switch_library", args).await {
                Ok(result) => {
                    eprintln!("✅ 切换库成功: {}", result);
                    return Some(("switch_library".to_string(), result));
                },
                Err(e) => {
                    eprintln!("❌ 切换库失败: {}", e);
                    return Some(("switch_library".to_string(), format!("错误: {}", e)));
                }
            }
        } else {
            eprintln!("⚠️ 提取的库名为空");
        }
    }

    // 搜索书籍
    if (msg_lower.contains("搜索") || msg_lower.contains("找")) &&
       (msg_lower.contains("书") || msg_lower.contains("小说") || msg_lower.contains("epub")) {
        // 提取搜索关键词
        let mut query = user_message.to_string();

        // 移除命令词
        for word in &["搜索", "找", "找一下", "看看", "有没有", "小说", "书籍", "的书"] {
            query = query.replace(word, "");
        }

        let query = query.trim().to_string();

        if !query.is_empty() {
            let args = serde_json::json!({
                "query": query
            });
            match assistant::execute_tool(pool, "search_books", args).await {
                Ok(result) => return Some(("search_books".to_string(), result)),
                Err(e) => return Some(("search_books".to_string(), format!("错误: {}", e))),
            }
        }
    }

    // 播放音乐
    if (msg_lower.contains("播放") || msg_lower.contains("听")) &&
       (msg_lower.contains("歌") || msg_lower.contains("音乐")) {
        let mut query = user_message.to_string();

        // 移除命令词
        for word in &["播放", "听", "听一下", "放", "的歌", "的音乐", "音乐", "歌曲"] {
            query = query.replace(word, "");
        }

        let query = query.trim().to_string();

        if !query.is_empty() {
            let args = serde_json::json!({
                "query": query
            });
            match assistant::execute_tool(pool, "search_tracks", args).await {
                Ok(result) => return Some(("search_tracks".to_string(), result)),
                Err(e) => return Some(("search_tracks".to_string(), format!("错误: {}", e))),
            }
        }
    }

    None
}

/// AI 助手响应
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssistantResponse {
    pub message: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_results: Option<Vec<ToolResult>>,
    pub conversation_history: Vec<ChatMessage>,
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
