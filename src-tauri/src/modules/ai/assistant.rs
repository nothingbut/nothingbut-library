use crate::errors::AppResult;
use super::models::*;
use sqlx::{SqlitePool, Row};
use serde_json::json;

/// 获取所有可用的工具定义
pub fn get_available_tools() -> Vec<Tool> {
    vec![
        // 库管理工具
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "list_libraries".to_string(),
                description: "列出所有可用的库。返回库的 ID、名称和类型。".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "get_current_library".to_string(),
                description: "获取当前选中的库信息。".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "switch_library".to_string(),
                description: "切换到指定的库。需要提供库的 ID 或名称。".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "library_id": {
                            "type": "integer",
                            "description": "要切换到的库 ID"
                        },
                        "library_name": {
                            "type": "string",
                            "description": "要切换到的库名称（如果不知道 ID）"
                        }
                    },
                    "required": []
                }),
            },
        },
        // 小说/书籍工具
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "search_books".to_string(),
                description: "在当前库中搜索书籍。支持按标题、作者搜索。".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "搜索关键词（书名或作者名）"
                        },
                        "library_id": {
                            "type": "integer",
                            "description": "库 ID（可选，默认使用当前库）"
                        }
                    },
                    "required": ["query"]
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "open_book".to_string(),
                description: "打开指定的书籍进入阅读模式。需要提供书籍 ID。".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "book_id": {
                            "type": "integer",
                            "description": "要打开的书籍 ID"
                        },
                        "book_type": {
                            "type": "string",
                            "description": "书籍类型：novel 或 epub",
                            "enum": ["novel", "epub"]
                        }
                    },
                    "required": ["book_id", "book_type"]
                }),
            },
        },
        // 音乐工具
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "search_tracks".to_string(),
                description: "在音乐库中搜索歌曲。支持按标题、艺术家、专辑搜索。".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "搜索关键词（歌曲名、艺术家或专辑）"
                        },
                        "library_id": {
                            "type": "integer",
                            "description": "库 ID（可选，默认使用当前库）"
                        }
                    },
                    "required": ["query"]
                }),
            },
        },
        Tool {
            tool_type: "function".to_string(),
            function: ToolFunction {
                name: "play_track".to_string(),
                description: "播放指定的歌曲。需要提供歌曲 ID。".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "track_id": {
                            "type": "integer",
                            "description": "要播放的歌曲 ID"
                        }
                    },
                    "required": ["track_id"]
                }),
            },
        },
    ]
}

/// 执行工具调用
pub async fn execute_tool(
    pool: &SqlitePool,
    tool_name: &str,
    arguments: serde_json::Value,
) -> AppResult<String> {
    match tool_name {
        "list_libraries" => list_libraries_impl(pool).await,
        "get_current_library" => get_current_library_impl(pool).await,
        "switch_library" => switch_library_impl(pool, arguments).await,
        "search_books" => search_books_impl(pool, arguments).await,
        "open_book" => open_book_impl(arguments).await,
        "search_tracks" => search_tracks_impl(pool, arguments).await,
        "play_track" => play_track_impl(arguments).await,
        _ => Ok(format!("未知工具: {}", tool_name)),
    }
}

// ==================== 工具实现 ====================

/// 列出所有库
async fn list_libraries_impl(pool: &SqlitePool) -> AppResult<String> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, library_type, path
        FROM libraries
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    let libraries: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.get::<i64, _>("id"),
                "name": row.get::<String, _>("name"),
                "type": row.get::<String, _>("library_type"),
                "path": row.get::<String, _>("path"),
            })
        })
        .collect();

    Ok(serde_json::to_string_pretty(&libraries)?)
}

/// 获取当前库
async fn get_current_library_impl(pool: &SqlitePool) -> AppResult<String> {
    let row = sqlx::query(
        r#"
        SELECT value FROM library_config WHERE key = 'current_library_id'
        "#
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        // 尝试解析库 ID
        let library_id: i64 = row.try_get::<i64, _>("value")
            .or_else(|_| {
                let value_str: String = row.get("value");
                value_str.parse::<i64>().map_err(|e| sqlx::Error::Decode(Box::new(e)))
            })?;

        // 获取库详情
        let lib_row = sqlx::query(
            r#"
            SELECT id, name, library_type, path
            FROM libraries
            WHERE id = ?
            "#
        )
        .bind(library_id)
        .fetch_one(pool)
        .await?;

        let library = json!({
            "id": lib_row.get::<i64, _>("id"),
            "name": lib_row.get::<String, _>("name"),
            "type": lib_row.get::<String, _>("library_type"),
            "path": lib_row.get::<String, _>("path"),
        });

        Ok(serde_json::to_string_pretty(&library)?)
    } else {
        Ok("未设置当前库".to_string())
    }
}

/// 切换库
async fn switch_library_impl(pool: &SqlitePool, arguments: serde_json::Value) -> AppResult<String> {
    // 优先使用 library_id
    if let Some(library_id) = arguments.get("library_id").and_then(|v| v.as_i64()) {
        // 获取库信息
        let row = sqlx::query(
            "SELECT name, module_type FROM libraries WHERE id = ?"
        )
        .bind(library_id)
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            let name: String = row.get("name");
            let module_type: String = row.get("module_type");

            // 更新当前库
            sqlx::query(
                r#"
                INSERT INTO library_config (key, value)
                VALUES ('current_library_id', ?)
                ON CONFLICT(key) DO UPDATE SET value = excluded.value
                "#
            )
            .bind(library_id.to_string())
            .execute(pool)
            .await?;

            // 返回导航指令
            return Ok(json!({
                "action": "navigate",
                "route": format!("/{}", module_type),
                "message": format!("已切换到库: {} (ID: {})", name, library_id),
                "library_id": library_id,
                "library_name": name,
                "library_type": module_type
            }).to_string());
        } else {
            return Ok(format!("库 ID {} 不存在", library_id));
        }
    }

    // 使用 library_name
    if let Some(library_name) = arguments.get("library_name").and_then(|v| v.as_str()) {
        // 移除用户输入中的特殊字符以提高匹配率
        let clean_input = library_name
            .replace("（", "")
            .replace("）", "")
            .replace("(", "")
            .replace(")", "")
            .replace("【", "")
            .replace("】", "")
            .replace("[", "")
            .replace("]", "")
            .replace(" ", "");

        // 查找库（先尝试精确匹配，再尝试模糊匹配）
        let rows = sqlx::query(
            "SELECT id, name FROM libraries"
        )
        .fetch_all(pool)
        .await?;

        // 遍历所有库，找到最佳匹配
        let mut best_match: Option<(i64, String)> = None;

        for row in rows {
            let lib_id: i64 = row.get("id");
            let lib_name: String = row.get("name");

            // 清理库名用于比较
            let clean_lib_name = lib_name
                .replace("（", "")
                .replace("）", "")
                .replace("(", "")
                .replace(")", "")
                .replace("【", "")
                .replace("】", "")
                .replace("[", "")
                .replace("]", "")
                .replace(" ", "");

            // 检查是否匹配
            if clean_lib_name.contains(&clean_input) || clean_input.contains(&clean_lib_name) {
                best_match = Some((lib_id, lib_name));
                break; // 找到第一个匹配就返回
            }
        }

        if let Some((library_id, name)) = best_match {
            // 获取库的模块类型
            let module_type: String = sqlx::query_scalar(
                "SELECT module_type FROM libraries WHERE id = ?"
            )
            .bind(library_id)
            .fetch_one(pool)
            .await?;

            // 更新当前库
            sqlx::query(
                r#"
                INSERT INTO library_config (key, value)
                VALUES ('current_library_id', ?)
                ON CONFLICT(key) DO UPDATE SET value = excluded.value
                "#
            )
            .bind(library_id.to_string())
            .execute(pool)
            .await?;

            // 返回导航指令
            return Ok(json!({
                "action": "navigate",
                "route": format!("/{}", module_type),
                "message": format!("已切换到库: {} (ID: {})", name, library_id),
                "library_id": library_id,
                "library_name": name,
                "library_type": module_type
            }).to_string());
        } else {
            return Ok(format!("未找到名称包含 '{}' 的库", library_name));
        }
    }

    Ok("请提供 library_id 或 library_name 参数".to_string())
}

/// 搜索书籍
async fn search_books_impl(pool: &SqlitePool, arguments: serde_json::Value) -> AppResult<String> {
    let query = arguments.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::errors::AppError::Module("缺少 query 参数".to_string()))?;

    // 获取 library_id（可选）
    let library_id = if let Some(lib_id) = arguments.get("library_id").and_then(|v| v.as_i64()) {
        lib_id
    } else {
        // 使用当前库
        let row = sqlx::query(
            "SELECT value FROM library_config WHERE key = 'current_library_id'"
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            row.try_get::<i64, _>("value")
                .or_else(|_| {
                    let value_str: String = row.get("value");
                    value_str.parse::<i64>().map_err(|e| sqlx::Error::Decode(Box::new(e)))
                })?
        } else {
            return Ok("未设置当前库".to_string());
        }
    };

    // 搜索小说
    let novel_rows = sqlx::query(
        r#"
        SELECT id, title, author
        FROM novel_books
        WHERE library_id = ? AND (title LIKE ? OR author LIKE ?)
        LIMIT 10
        "#
    )
    .bind(library_id)
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;

    // 搜索 EPUB
    let epub_rows = sqlx::query(
        r#"
        SELECT id, title, creator as author
        FROM epub_books
        WHERE library_id = ? AND (title LIKE ? OR creator LIKE ?)
        LIMIT 10
        "#
    )
    .bind(library_id)
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();

    for row in novel_rows {
        results.push(json!({
            "id": row.get::<i64, _>("id"),
            "title": row.get::<String, _>("title"),
            "author": row.get::<Option<String>, _>("author"),
            "type": "novel"
        }));
    }

    for row in epub_rows {
        results.push(json!({
            "id": row.get::<i64, _>("id"),
            "title": row.get::<String, _>("title"),
            "author": row.get::<Option<String>, _>("author"),
            "type": "epub"
        }));
    }

    if results.is_empty() {
        Ok(format!("未找到包含 '{}' 的书籍", query))
    } else {
        Ok(serde_json::to_string_pretty(&results)?)
    }
}

/// 打开书籍
async fn open_book_impl(arguments: serde_json::Value) -> AppResult<String> {
    let book_id = arguments.get("book_id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| crate::errors::AppError::Module("缺少 book_id 参数".to_string()))?;

    let book_type = arguments.get("book_type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::errors::AppError::Module("缺少 book_type 参数".to_string()))?;

    // 返回导航指令（前端会处理）
    Ok(json!({
        "action": "navigate",
        "route": format!("/reader/{}", book_id),
        "book_id": book_id,
        "book_type": book_type
    }).to_string())
}

/// 搜索歌曲
async fn search_tracks_impl(pool: &SqlitePool, arguments: serde_json::Value) -> AppResult<String> {
    let query = arguments.get("query")
        .and_then(|v| v.as_str())
        .ok_or_else(|| crate::errors::AppError::Module("缺少 query 参数".to_string()))?;

    // 获取 library_id（可选）
    let library_id = if let Some(lib_id) = arguments.get("library_id").and_then(|v| v.as_i64()) {
        lib_id
    } else {
        // 使用当前库
        let row = sqlx::query(
            "SELECT value FROM library_config WHERE key = 'current_library_id'"
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            row.try_get::<i64, _>("value")
                .or_else(|_| {
                    let value_str: String = row.get("value");
                    value_str.parse::<i64>().map_err(|e| sqlx::Error::Decode(Box::new(e)))
                })?
        } else {
            return Ok("未设置当前库".to_string());
        }
    };

    // 搜索歌曲
    let rows = sqlx::query(
        r#"
        SELECT t.id, t.title, a.name as artist, al.title as album
        FROM music_tracks t
        LEFT JOIN music_artists a ON t.artist_id = a.id
        LEFT JOIN music_albums al ON t.album_id = al.id
        WHERE t.library_id = ? AND (
            t.title LIKE ? OR
            a.name LIKE ? OR
            al.title LIKE ?
        )
        LIMIT 10
        "#
    )
    .bind(library_id)
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .bind(format!("%{}%", query))
    .fetch_all(pool)
    .await?;

    let results: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            json!({
                "id": row.get::<i64, _>("id"),
                "title": row.get::<String, _>("title"),
                "artist": row.get::<Option<String>, _>("artist"),
                "album": row.get::<Option<String>, _>("album"),
            })
        })
        .collect();

    if results.is_empty() {
        Ok(format!("未找到包含 '{}' 的歌曲", query))
    } else {
        Ok(serde_json::to_string_pretty(&results)?)
    }
}

/// 播放歌曲
async fn play_track_impl(arguments: serde_json::Value) -> AppResult<String> {
    let track_id = arguments.get("track_id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| crate::errors::AppError::Module("缺少 track_id 参数".to_string()))?;

    // 返回播放指令（前端会处理）
    Ok(json!({
        "action": "play_music",
        "track_id": track_id
    }).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_tools() {
        let tools = get_available_tools();
        assert!(!tools.is_empty());
        assert!(tools.iter().any(|t| t.function.name == "list_libraries"));
        assert!(tools.iter().any(|t| t.function.name == "search_books"));
        assert!(tools.iter().any(|t| t.function.name == "search_tracks"));
    }
}
