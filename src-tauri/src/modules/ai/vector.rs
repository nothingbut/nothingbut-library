use crate::errors::AppResult;
use super::OllamaClient;
use sqlx::{SqlitePool, Row};
use std::collections::HashMap;

/// 计算两个向量的余弦相似度
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a * norm_b)
}

/// 序列化向量为 BLOB
pub fn serialize_embedding(embedding: &[f32]) -> Vec<u8> {
    embedding
        .iter()
        .flat_map(|&f| f.to_le_bytes())
        .collect()
}

/// 反序列化 BLOB 为向量
pub fn deserialize_embedding(blob: &[u8]) -> Vec<f32> {
    blob.chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
}

/// 计算文本内容的哈希值
pub fn compute_content_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// 保存章节嵌入向量
pub async fn save_chapter_embedding(
    pool: &SqlitePool,
    chapter_id: i64,
    embedding: &[f32],
    content_hash: &str,
) -> AppResult<()> {
    let blob = serialize_embedding(embedding);

    sqlx::query(
        r#"
        INSERT INTO chapter_embeddings (chapter_id, embedding, content_hash)
        VALUES (?, ?, ?)
        ON CONFLICT(chapter_id)
        DO UPDATE SET embedding = ?, content_hash = ?, created_at = datetime('now')
        "#
    )
    .bind(chapter_id)
    .bind(&blob)
    .bind(content_hash)
    .bind(&blob)
    .bind(content_hash)
    .execute(pool)
    .await?;

    Ok(())
}

/// 检查章节是否已索引且内容未变
pub async fn is_chapter_indexed(
    pool: &SqlitePool,
    chapter_id: i64,
    content_hash: &str,
) -> AppResult<bool> {
    let row = sqlx::query(
        r#"
        SELECT content_hash
        FROM chapter_embeddings
        WHERE chapter_id = ?
        "#
    )
    .bind(chapter_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| r.get::<String, _>("content_hash") == content_hash).unwrap_or(false))
}

/// 获取所有已索引的章节向量
pub async fn get_all_embeddings(
    pool: &SqlitePool,
    book_id: Option<i64>,
) -> AppResult<Vec<(i64, Vec<f32>)>> {
    let query = if let Some(book_id) = book_id {
        format!(
            r#"
            SELECT ce.chapter_id, ce.embedding
            FROM chapter_embeddings ce
            INNER JOIN novel_chapters nc ON ce.chapter_id = nc.id
            WHERE nc.book_id = {}
            "#,
            book_id
        )
    } else {
        r#"
        SELECT chapter_id, embedding
        FROM chapter_embeddings
        "#.to_string()
    };

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await?;

    let embeddings = rows
        .into_iter()
        .map(|row| {
            let chapter_id: i64 = row.get("chapter_id");
            let blob: Vec<u8> = row.get("embedding");
            let embedding = deserialize_embedding(&blob);
            (chapter_id, embedding)
        })
        .collect();

    Ok(embeddings)
}

/// 语义搜索：根据查询向量找到最相似的章节
pub async fn search_similar_chapters(
    pool: &SqlitePool,
    query_embedding: &[f32],
    book_id: Option<i64>,
    limit: usize,
    min_similarity: f32,
) -> AppResult<Vec<(i64, f32)>> {
    // 获取所有候选向量
    let embeddings = get_all_embeddings(pool, book_id).await?;

    // 计算相似度并排序
    let mut similarities: Vec<(i64, f32)> = embeddings
        .iter()
        .map(|(chapter_id, embedding)| {
            let similarity = cosine_similarity(query_embedding, embedding);
            (*chapter_id, similarity)
        })
        .filter(|(_, similarity)| *similarity >= min_similarity)
        .collect();

    // 按相似度降序排序
    similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // 取前 N 个
    similarities.truncate(limit);

    Ok(similarities)
}

/// 批量索引书籍的所有章节
pub async fn index_book_chapters(
    pool: &SqlitePool,
    client: &OllamaClient,
    book_id: i64,
    get_chapter_content: impl Fn(i64) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<String>> + Send>>,
) -> AppResult<HashMap<i64, bool>> {
    // 获取书籍的所有章节
    let rows = sqlx::query(
        r#"
        SELECT id
        FROM novel_chapters
        WHERE book_id = ?
        ORDER BY chapter_number
        "#
    )
    .bind(book_id)
    .fetch_all(pool)
    .await?;

    let chapter_ids: Vec<i64> = rows.iter().map(|row| row.get("id")).collect();

    let mut results = HashMap::new();

    // 逐个索引章节（使用信号量控制并发）
    for chapter_id in chapter_ids {
        match index_single_chapter(pool, client, chapter_id, &get_chapter_content).await {
            Ok(_) => {
                results.insert(chapter_id, true);
            }
            Err(e) => {
                eprintln!("Failed to index chapter {}: {}", chapter_id, e);
                results.insert(chapter_id, false);
            }
        }
    }

    Ok(results)
}

/// 索引单个章节
async fn index_single_chapter(
    pool: &SqlitePool,
    client: &OllamaClient,
    chapter_id: i64,
    get_chapter_content: &impl Fn(i64) -> std::pin::Pin<Box<dyn std::future::Future<Output = AppResult<String>> + Send>>,
) -> AppResult<()> {
    // 获取章节内容
    let content = get_chapter_content(chapter_id).await?;
    let content_hash = compute_content_hash(&content);

    // 检查是否需要重新索引
    if is_chapter_indexed(pool, chapter_id, &content_hash).await? {
        return Ok(()); // 已索引且内容未变，跳过
    }

    // 生成嵌入向量
    let embedding = client.embeddings(&content, super::DEFAULT_EMBEDDING_MODEL).await?;

    // 保存到数据库
    save_chapter_embedding(pool, chapter_id, &embedding, &content_hash).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0, 0.0];
        let d = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&c, &d).abs() < 0.001);

        let e = vec![1.0, 1.0, 0.0];
        let f = vec![1.0, 1.0, 0.0];
        assert!((cosine_similarity(&e, &f) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_serialize_deserialize() {
        let original = vec![1.5, -2.3, 0.0, 100.5];
        let serialized = serialize_embedding(&original);
        let deserialized = deserialize_embedding(&serialized);

        assert_eq!(original.len(), deserialized.len());
        for (a, b) in original.iter().zip(deserialized.iter()) {
            assert!((a - b).abs() < 0.0001);
        }
    }

    #[test]
    fn test_content_hash() {
        let content1 = "Hello, world!";
        let content2 = "Hello, world!";
        let content3 = "Hello, World!";

        assert_eq!(compute_content_hash(content1), compute_content_hash(content2));
        assert_ne!(compute_content_hash(content1), compute_content_hash(content3));
    }
}
