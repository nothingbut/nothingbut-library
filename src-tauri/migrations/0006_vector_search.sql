-- 章节向量索引表
CREATE TABLE IF NOT EXISTS chapter_embeddings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chapter_id INTEGER NOT NULL UNIQUE,
    embedding BLOB NOT NULL, -- 序列化的 Vec<f32> (768 维)
    content_hash TEXT NOT NULL, -- 内容哈希，用于判断是否需要重新索引
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (chapter_id) REFERENCES novel_chapters(id) ON DELETE CASCADE
);

-- 创建索引加速查询
CREATE INDEX IF NOT EXISTS idx_chapter_embeddings_chapter_id ON chapter_embeddings(chapter_id);
CREATE INDEX IF NOT EXISTS idx_chapter_embeddings_content_hash ON chapter_embeddings(content_hash);
