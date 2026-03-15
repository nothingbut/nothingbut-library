use crate::errors::AppResult;
use super::models::*;
use sqlx::{SqlitePool, Row};

/// 创建对话
pub async fn create_conversation(
    pool: &SqlitePool,
    title: &str,
    context_type: &str,
    context_id: Option<i64>,
) -> AppResult<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO ai_conversations (title, context_type, context_id)
        VALUES (?, ?, ?)
        "#
    )
    .bind(title)
    .bind(context_type)
    .bind(context_id)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// 添加消息
pub async fn add_message(
    pool: &SqlitePool,
    conversation_id: i64,
    role: &str,
    content: &str,
) -> AppResult<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO ai_messages (conversation_id, role, content)
        VALUES (?, ?, ?)
        "#
    )
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// 获取对话历史
pub async fn get_conversation_messages(
    pool: &SqlitePool,
    conversation_id: i64,
    limit: Option<i32>,
) -> AppResult<Vec<StoredMessage>> {
    let limit = limit.unwrap_or(50);

    let rows = sqlx::query(
        r#"
        SELECT id, conversation_id, role, content, timestamp
        FROM ai_messages
        WHERE conversation_id = ?
        ORDER BY timestamp ASC
        LIMIT ?
        "#
    )
    .bind(conversation_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let messages = rows
        .into_iter()
        .map(|row| StoredMessage {
            id: row.get("id"),
            conversation_id: row.get("conversation_id"),
            role: row.get("role"),
            content: row.get("content"),
            timestamp: row.get("timestamp"),
        })
        .collect();

    Ok(messages)
}

/// 保存或更新摘要
pub async fn save_summary(
    pool: &SqlitePool,
    target_type: &str,
    target_id: i64,
    summary: &str,
    length: &str,
    model: &str,
) -> AppResult<i64> {
    let result = sqlx::query(
        r#"
        INSERT INTO ai_summaries (target_type, target_id, summary, length, model)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(target_type, target_id, length)
        DO UPDATE SET summary = ?, model = ?, created_at = datetime('now')
        "#
    )
    .bind(target_type)
    .bind(target_id)
    .bind(summary)
    .bind(length)
    .bind(model)
    .bind(summary)
    .bind(model)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
}

/// 获取摘要
pub async fn get_summary(
    pool: &SqlitePool,
    target_type: &str,
    target_id: i64,
    length: &str,
) -> AppResult<Option<String>> {
    let row = sqlx::query(
        r#"
        SELECT summary
        FROM ai_summaries
        WHERE target_type = ? AND target_id = ? AND length = ?
        "#
    )
    .bind(target_type)
    .bind(target_id)
    .bind(length)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.get("summary")))
}
