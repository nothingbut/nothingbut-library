use chrono::Utc;
use sqlx::SqlitePool;

use super::models::{WereadAccount, WereadBook, WereadChapter, WereadDownload};

// --- Account ---

pub async fn upsert_account(
    pool: &SqlitePool,
    vid: &str,
    username: &str,
    avatar_url: Option<&str>,
    cookies_json: &str,
) -> Result<WereadAccount, String> {
    let now = Utc::now().timestamp();
    let cookie_expires_at = now + 7 * 24 * 3600;

    sqlx::query(
        r#"INSERT INTO weread_accounts (vid, username, avatar_url, cookies_json, cookie_expires_at, is_active, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, 1, ?, ?)
           ON CONFLICT(vid) DO UPDATE SET
             username = excluded.username,
             avatar_url = excluded.avatar_url,
             cookies_json = excluded.cookies_json,
             cookie_expires_at = excluded.cookie_expires_at,
             is_active = 1,
             updated_at = excluded.updated_at"#,
    )
    .bind(vid)
    .bind(username)
    .bind(avatar_url)
    .bind(cookies_json)
    .bind(cookie_expires_at)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| format!("保存账号失败: {}", e))?;

    get_active_account(pool).await?.ok_or("保存后无法读取账号".to_string())
}

pub async fn get_active_account(pool: &SqlitePool) -> Result<Option<WereadAccount>, String> {
    sqlx::query_as::<_, WereadAccount>(
        "SELECT * FROM weread_accounts WHERE is_active = 1 ORDER BY updated_at DESC LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("查询账号失败: {}", e))
}

pub async fn deactivate_account(pool: &SqlitePool, vid: &str) -> Result<(), String> {
    sqlx::query("UPDATE weread_accounts SET is_active = 0 WHERE vid = ?")
        .bind(vid)
        .execute(pool)
        .await
        .map_err(|e| format!("注销账号失败: {}", e))?;
    Ok(())
}

// --- Books ---

pub async fn upsert_book(
    pool: &SqlitePool,
    library_id: i64,
    book_id: &str,
    title: &str,
    author: Option<&str>,
    cover_url: Option<&str>,
    intro: Option<&str>,
    category: Option<&str>,
    word_count: i64,
    chapter_count: i64,
) -> Result<(), String> {
    let now = Utc::now().timestamp();

    sqlx::query(
        r#"INSERT INTO weread_books (library_id, book_id, title, author, cover_url, intro, category, word_count, chapter_count, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT(library_id, book_id) DO UPDATE SET
             title = excluded.title,
             author = excluded.author,
             cover_url = excluded.cover_url,
             intro = excluded.intro,
             category = excluded.category,
             word_count = excluded.word_count,
             chapter_count = excluded.chapter_count"#,
    )
    .bind(library_id)
    .bind(book_id)
    .bind(title)
    .bind(author)
    .bind(cover_url)
    .bind(intro)
    .bind(category)
    .bind(word_count)
    .bind(chapter_count)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| format!("保存书籍失败: {}", e))?;

    Ok(())
}

pub async fn list_books(
    pool: &SqlitePool,
    library_id: i64,
) -> Result<Vec<WereadBook>, String> {
    sqlx::query_as::<_, WereadBook>(
        "SELECT * FROM weread_books WHERE library_id = ? ORDER BY created_at DESC",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("查询书籍列表失败: {}", e))
}

pub async fn search_books(
    pool: &SqlitePool,
    library_id: i64,
    query: &str,
) -> Result<Vec<WereadBook>, String> {
    let pattern = format!("%{}%", query);
    sqlx::query_as::<_, WereadBook>(
        "SELECT * FROM weread_books WHERE library_id = ? AND (title LIKE ? OR author LIKE ?) ORDER BY created_at DESC",
    )
    .bind(library_id)
    .bind(&pattern)
    .bind(&pattern)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("搜索书籍失败: {}", e))
}

pub async fn get_book_by_id(
    pool: &SqlitePool,
    book_id: i64,
) -> Result<Option<WereadBook>, String> {
    sqlx::query_as::<_, WereadBook>("SELECT * FROM weread_books WHERE id = ?")
        .bind(book_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| format!("查询书籍失败: {}", e))
}

// --- Chapters ---

pub async fn upsert_chapter(
    pool: &SqlitePool,
    library_id: i64,
    book_id: i64,
    chapter_uid: &str,
    title: &str,
    level: i64,
    word_count: i64,
    sort_order: i64,
) -> Result<(), String> {
    sqlx::query(
        r#"INSERT INTO weread_chapters (library_id, book_id, chapter_uid, title, level, word_count, sort_order)
           VALUES (?, ?, ?, ?, ?, ?, ?)
           ON CONFLICT(library_id, book_id, chapter_uid) DO UPDATE SET
             title = excluded.title,
             level = excluded.level,
             word_count = excluded.word_count,
             sort_order = excluded.sort_order"#,
    )
    .bind(library_id)
    .bind(book_id)
    .bind(chapter_uid)
    .bind(title)
    .bind(level)
    .bind(word_count)
    .bind(sort_order)
    .execute(pool)
    .await
    .map_err(|e| format!("保存章节失败: {}", e))?;

    Ok(())
}

pub async fn update_chapter_content(
    pool: &SqlitePool,
    library_id: i64,
    book_id: i64,
    chapter_uid: &str,
    content_md: &str,
) -> Result<(), String> {
    let now = Utc::now().timestamp();
    sqlx::query(
        "UPDATE weread_chapters SET content_md = ?, extracted_at = ? WHERE library_id = ? AND book_id = ? AND chapter_uid = ?",
    )
    .bind(content_md)
    .bind(now)
    .bind(library_id)
    .bind(book_id)
    .bind(chapter_uid)
    .execute(pool)
    .await
    .map_err(|e| format!("更新章节内容失败: {}", e))?;

    Ok(())
}

pub async fn list_chapters(
    pool: &SqlitePool,
    library_id: i64,
    book_id: i64,
) -> Result<Vec<WereadChapter>, String> {
    sqlx::query_as::<_, WereadChapter>(
        "SELECT * FROM weread_chapters WHERE library_id = ? AND book_id = ? ORDER BY sort_order ASC",
    )
    .bind(library_id)
    .bind(book_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("查询章节列表失败: {}", e))
}

// --- Downloads ---

pub async fn upsert_download(
    pool: &SqlitePool,
    library_id: i64,
    book_id: i64,
) -> Result<i64, String> {
    let now = Utc::now().timestamp();

    let result = sqlx::query(
        r#"INSERT INTO weread_downloads (library_id, book_id, status, created_at)
           VALUES (?, ?, 'pending', ?)
           ON CONFLICT(library_id, book_id) DO UPDATE SET
             status = 'pending',
             progress_current = 0,
             error_message = NULL,
             started_at = NULL,
             completed_at = NULL"#,
    )
    .bind(library_id)
    .bind(book_id)
    .bind(now)
    .execute(pool)
    .await
    .map_err(|e| format!("创建下载记录失败: {}", e))?;

    Ok(result.last_insert_rowid())
}

pub async fn update_download_progress(
    pool: &SqlitePool,
    library_id: i64,
    book_id: i64,
    current: i64,
    total: i64,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE weread_downloads SET status = 'downloading', progress_current = ?, progress_total = ? WHERE library_id = ? AND book_id = ?",
    )
    .bind(current)
    .bind(total)
    .bind(library_id)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(|e| format!("更新下载进度失败: {}", e))?;

    Ok(())
}

pub async fn complete_download(
    pool: &SqlitePool,
    library_id: i64,
    book_id: i64,
    output_path: &str,
) -> Result<(), String> {
    let now = Utc::now().timestamp();
    sqlx::query(
        "UPDATE weread_downloads SET status = 'completed', output_path = ?, completed_at = ? WHERE library_id = ? AND book_id = ?",
    )
    .bind(output_path)
    .bind(now)
    .bind(library_id)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(|e| format!("完成下载失败: {}", e))?;

    Ok(())
}

pub async fn fail_download(
    pool: &SqlitePool,
    library_id: i64,
    book_id: i64,
    error: &str,
) -> Result<(), String> {
    let now = Utc::now().timestamp();
    sqlx::query(
        "UPDATE weread_downloads SET status = 'failed', error_message = ?, completed_at = ? WHERE library_id = ? AND book_id = ?",
    )
    .bind(error)
    .bind(now)
    .bind(library_id)
    .bind(book_id)
    .execute(pool)
    .await
    .map_err(|e| format!("标记下载失败: {}", e))?;

    Ok(())
}

pub async fn list_downloads(
    pool: &SqlitePool,
    library_id: i64,
) -> Result<Vec<WereadDownload>, String> {
    sqlx::query_as::<_, WereadDownload>(
        "SELECT * FROM weread_downloads WHERE library_id = ? ORDER BY created_at DESC",
    )
    .bind(library_id)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("查询下载列表失败: {}", e))
}
