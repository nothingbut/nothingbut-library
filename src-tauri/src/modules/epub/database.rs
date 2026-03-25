use crate::errors::{AppError, AppResult};
use crate::modules::epub::models::{Author, EpubBook, SearchQuery, Tag};
use chrono::Utc;
use sqlx::{Row, SqlitePool};

/// EPUB 数据库操作层
pub struct EpubDatabase {
    pool: SqlitePool,
}

impl EpubDatabase {
    /// 创建新的数据库操作实例
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ==================== 书籍 CRUD 操作 ====================

    /// 创建新书籍
    pub async fn create_book(&self, book: &EpubBook) -> AppResult<i64> {
        let timestamp = Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            INSERT INTO epub_books (
                title, sort_title, isbn, publisher, pubdate, language,
                series, series_index, rating, file_path, file_size,
                cover_path, description, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&book.title)
        .bind(&book.sort_title)
        .bind(&book.isbn)
        .bind(&book.publisher)
        .bind(&book.pubdate)
        .bind(&book.language)
        .bind(&book.series)
        .bind(book.series_index)
        .bind(book.rating)
        .bind(&book.file_path)
        .bind(book.file_size)
        .bind(&book.cover_path)
        .bind(&book.description)
        .bind(timestamp)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 获取单个书籍
    pub async fn get_book(&self, book_id: i64) -> AppResult<Option<EpubBook>> {
        let row = sqlx::query(
            r#"
            SELECT id, title, sort_title, isbn, publisher, pubdate, language,
                   series, series_index, rating, file_path, file_size,
                   cover_path, description, created_at, updated_at
            FROM epub_books
            WHERE id = ?
            "#,
        )
        .bind(book_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(Self::row_to_book(row)?)),
            None => Ok(None),
        }
    }

    /// 列出所有书籍
    pub async fn list_books(&self) -> AppResult<Vec<EpubBook>> {
        let rows = sqlx::query(
            r#"
            SELECT id, title, sort_title, isbn, publisher, pubdate, language,
                   series, series_index, rating, file_path, file_size,
                   cover_path, description, created_at, updated_at
            FROM epub_books
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut books = Vec::new();
        for row in rows {
            books.push(Self::row_to_book(row)?);
        }

        Ok(books)
    }

    /// 更新书籍信息
    pub async fn update_book(&self, book_id: i64, book: &EpubBook) -> AppResult<()> {
        let timestamp = Utc::now().timestamp();

        let result = sqlx::query(
            r#"
            UPDATE epub_books
            SET title = ?, sort_title = ?, isbn = ?, publisher = ?, pubdate = ?,
                language = ?, series = ?, series_index = ?, rating = ?,
                file_path = ?, file_size = ?, cover_path = ?, description = ?,
                updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&book.title)
        .bind(&book.sort_title)
        .bind(&book.isbn)
        .bind(&book.publisher)
        .bind(&book.pubdate)
        .bind(&book.language)
        .bind(&book.series)
        .bind(book.series_index)
        .bind(book.rating)
        .bind(&book.file_path)
        .bind(book.file_size)
        .bind(&book.cover_path)
        .bind(&book.description)
        .bind(timestamp)
        .bind(book_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Book {} not found", book_id)));
        }

        Ok(())
    }

    /// 删除书籍
    pub async fn delete_book(&self, book_id: i64) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM epub_books WHERE id = ?")
            .bind(book_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Book {} not found", book_id)));
        }

        Ok(())
    }

    // ==================== 作者管理 ====================

    /// 获取或创建作者（根据名称）
    pub async fn get_or_create_author(
        &self,
        name: &str,
        sort_name: Option<&str>,
    ) -> AppResult<i64> {
        // 首先尝试查找现有作者
        let existing = sqlx::query("SELECT id FROM epub_authors WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = existing {
            return Ok(row.get("id"));
        }

        // 如果不存在，则创建新作者
        let timestamp = Utc::now().timestamp();
        let result = sqlx::query(
            "INSERT INTO epub_authors (name, sort_name, created_at) VALUES (?, ?, ?)",
        )
        .bind(name)
        .bind(sort_name)
        .bind(timestamp)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// 设置书籍的作者列表（替换现有关联）
    pub async fn set_book_authors(
        &self,
        book_id: i64,
        authors: Vec<(String, Option<String>, i32)>,
    ) -> AppResult<()> {
        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 删除现有关联
        sqlx::query("DELETE FROM epub_book_authors WHERE book_id = ?")
            .bind(book_id)
            .execute(&mut *tx)
            .await?;

        // 添加新关联（内联 get_or_create 逻辑以保持事务原子性）
        for (name, sort_name, order) in authors.iter() {
            // 先尝试查找作者（在事务中）
            let author_id = match sqlx::query_scalar::<_, i64>(
                "SELECT id FROM epub_authors WHERE name = ?"
            )
            .bind(name)
            .fetch_optional(&mut *tx)
            .await? {
                Some(id) => id,
                None => {
                    // 不存在则创建（在事务中）
                    let timestamp = Utc::now().timestamp();
                    sqlx::query(
                        "INSERT INTO epub_authors (name, sort_name, created_at) VALUES (?, ?, ?)"
                    )
                    .bind(name)
                    .bind(sort_name)
                    .bind(timestamp)
                    .execute(&mut *tx)
                    .await?
                    .last_insert_rowid()
                }
            };

            sqlx::query(
                "INSERT INTO epub_book_authors (book_id, author_id, author_order) VALUES (?, ?, ?)",
            )
            .bind(book_id)
            .bind(author_id)
            .bind(order)
            .execute(&mut *tx)
            .await?;
        }

        // 提交事务
        tx.commit().await?;

        Ok(())
    }

    /// 获取书籍的作者列表
    pub async fn get_book_authors(&self, book_id: i64) -> AppResult<Vec<Author>> {
        let rows = sqlx::query(
            r#"
            SELECT a.id, a.name, a.sort_name, a.created_at
            FROM epub_authors a
            INNER JOIN epub_book_authors ba ON a.id = ba.author_id
            WHERE ba.book_id = ?
            ORDER BY ba.author_order
            "#,
        )
        .bind(book_id)
        .fetch_all(&self.pool)
        .await?;

        let mut authors = Vec::new();
        for row in rows {
            authors.push(Self::row_to_author(row)?);
        }

        Ok(authors)
    }

    // ==================== 标签管理 ====================

    /// 获取或创建标签（根据名称）
    pub async fn get_or_create_tag(&self, name: &str) -> AppResult<i64> {
        // 首先尝试查找现有标签
        let existing = sqlx::query("SELECT id FROM epub_tags WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = existing {
            return Ok(row.get("id"));
        }

        // 如果不存在，则创建新标签
        let timestamp = Utc::now().timestamp();
        let result = sqlx::query("INSERT INTO epub_tags (name, created_at) VALUES (?, ?)")
            .bind(name)
            .bind(timestamp)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// 设置书籍的标签列表（替换现有关联）
    pub async fn set_book_tags(&self, book_id: i64, tag_names: Vec<String>) -> AppResult<()> {
        // 开始事务
        let mut tx = self.pool.begin().await?;

        // 删除现有关联
        sqlx::query("DELETE FROM epub_book_tags WHERE book_id = ?")
            .bind(book_id)
            .execute(&mut *tx)
            .await?;

        // 添加新关联（内联 get_or_create 逻辑以保持事务原子性）
        for name in tag_names {
            // 先尝试查找标签（在事务中）
            let tag_id = match sqlx::query_scalar::<_, i64>(
                "SELECT id FROM epub_tags WHERE name = ?"
            )
            .bind(&name)
            .fetch_optional(&mut *tx)
            .await? {
                Some(id) => id,
                None => {
                    // 不存在则创建（在事务中）
                    let timestamp = Utc::now().timestamp();
                    sqlx::query(
                        "INSERT INTO epub_tags (name, created_at) VALUES (?, ?)"
                    )
                    .bind(&name)
                    .bind(timestamp)
                    .execute(&mut *tx)
                    .await?
                    .last_insert_rowid()
                }
            };

            sqlx::query("INSERT INTO epub_book_tags (book_id, tag_id) VALUES (?, ?)")
                .bind(book_id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
        }

        // 提交事务
        tx.commit().await?;

        Ok(())
    }

    /// 获取书籍的标签列表
    pub async fn get_book_tags(&self, book_id: i64) -> AppResult<Vec<Tag>> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.name, t.created_at
            FROM epub_tags t
            INNER JOIN epub_book_tags bt ON t.id = bt.tag_id
            WHERE bt.book_id = ?
            ORDER BY t.name
            "#,
        )
        .bind(book_id)
        .fetch_all(&self.pool)
        .await?;

        let mut tags = Vec::new();
        for row in rows {
            tags.push(Self::row_to_tag(row)?);
        }

        Ok(tags)
    }

    // ==================== 搜索功能 ====================

    /// 搜索书籍（支持多条件过滤和排序）
    pub async fn search_books(&self, query: &SearchQuery) -> AppResult<Vec<EpubBook>> {
        // 构建动态 SQL 查询
        let mut sql = String::from(
            r#"
            SELECT DISTINCT b.id, b.title, b.sort_title, b.isbn, b.publisher, b.pubdate,
                   b.language, b.series, b.series_index, b.rating, b.file_path,
                   b.file_size, b.cover_path, b.description, b.created_at, b.updated_at
            FROM epub_books b
            "#,
        );

        let mut conditions = Vec::new();
        let mut has_author_filter = false;
        let mut has_tag_filter = false;

        // 添加 JOIN 如果需要作者过滤
        if query.author.is_some() {
            sql.push_str(
                " LEFT JOIN epub_book_authors ba ON b.id = ba.book_id
                  LEFT JOIN epub_authors a ON ba.author_id = a.id ",
            );
            has_author_filter = true;
        }

        // 添加 JOIN 如果需要标签过滤
        if query.tags.is_some() {
            sql.push_str(
                " LEFT JOIN epub_book_tags bt ON b.id = bt.book_id
                  LEFT JOIN epub_tags t ON bt.tag_id = t.id ",
            );
            has_tag_filter = true;
        }

        // 构建 WHERE 条件
        if query.keyword.is_some() {
            conditions.push(format!(
                "(b.title LIKE '%' || ? || '%' OR b.description LIKE '%' || ? || '%')"
            ));
        }
        if query.title.is_some() {
            conditions.push("b.title LIKE '%' || ? || '%'".to_string());
        }
        if has_author_filter {
            conditions.push("a.name LIKE '%' || ? || '%'".to_string());
        }
        if query.publisher.is_some() {
            conditions.push("b.publisher LIKE '%' || ? || '%'".to_string());
        }
        if query.isbn.is_some() {
            conditions.push("b.isbn = ?".to_string());
        }
        if query.series.is_some() {
            conditions.push("b.series LIKE '%' || ? || '%'".to_string());
        }
        if has_tag_filter {
            if let Some(ref tags) = query.tags {
                if !tags.is_empty() {
                    let placeholders = tags.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                    conditions.push(format!("t.name IN ({})", placeholders));
                }
            }
        }
        if query.rating_min.is_some() {
            conditions.push("b.rating >= ?".to_string());
        }
        if query.rating_max.is_some() {
            conditions.push("b.rating <= ?".to_string());
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        // 添加排序
        let sort_by = query.sort_by.as_deref().unwrap_or("created_at");
        let sort_order = query.sort_order.as_deref().unwrap_or("DESC");

        // 验证排序字段（防止 SQL 注入）
        let valid_sort_fields =
            ["title", "created_at", "updated_at", "rating", "pubdate", "series"];
        let sort_field = if valid_sort_fields.contains(&sort_by) {
            sort_by
        } else {
            "created_at"
        };

        // 验证排序方向
        let sort_direction = if sort_order.to_uppercase() == "ASC" {
            "ASC"
        } else {
            "DESC"
        };

        sql.push_str(&format!(" ORDER BY b.{} {}", sort_field, sort_direction));

        // 添加分页（使用参数化查询防止 SQL 注入）
        if query.limit.is_some() {
            sql.push_str(" LIMIT ?");
        }
        if query.offset.is_some() {
            sql.push_str(" OFFSET ?");
        }

        // 绑定参数
        let mut query_builder = sqlx::query(&sql);

        // 按照条件添加的顺序绑定参数
        if let Some(ref keyword) = query.keyword {
            query_builder = query_builder.bind(keyword).bind(keyword); // title 和 description 各一次
        }
        if let Some(ref title) = query.title {
            query_builder = query_builder.bind(title);
        }
        if let Some(ref author) = query.author {
            query_builder = query_builder.bind(author);
        }
        if let Some(ref publisher) = query.publisher {
            query_builder = query_builder.bind(publisher);
        }
        if let Some(ref isbn) = query.isbn {
            query_builder = query_builder.bind(isbn);
        }
        if let Some(ref series) = query.series {
            query_builder = query_builder.bind(series);
        }
        if let Some(ref tags) = query.tags {
            if !tags.is_empty() {
                // 绑定所有标签
                for tag in tags {
                    query_builder = query_builder.bind(tag);
                }
            }
        }
        if let Some(rating_min) = query.rating_min {
            query_builder = query_builder.bind(rating_min);
        }
        if let Some(rating_max) = query.rating_max {
            query_builder = query_builder.bind(rating_max);
        }
        if let Some(limit) = query.limit {
            query_builder = query_builder.bind(limit);
        }
        if let Some(offset) = query.offset {
            query_builder = query_builder.bind(offset);
        }

        // 执行查询
        let rows = query_builder.fetch_all(&self.pool).await?;

        let mut books = Vec::new();
        for row in rows {
            books.push(Self::row_to_book(row)?);
        }

        Ok(books)
    }

    // ==================== 辅助方法 ====================

    /// 将数据库行转换为 EpubBook
    fn row_to_book(row: sqlx::sqlite::SqliteRow) -> AppResult<EpubBook> {
        let created_at: i64 = row.try_get("created_at")?;
        let updated_at: i64 = row.try_get("updated_at")?;

        Ok(EpubBook {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            sort_title: row.try_get("sort_title")?,
            isbn: row.try_get("isbn")?,
            publisher: row.try_get("publisher")?,
            pubdate: row.try_get("pubdate")?,
            language: row.try_get("language")?,
            series: row.try_get("series")?,
            series_index: row.try_get("series_index")?,
            rating: row.try_get("rating")?,
            file_path: row.try_get("file_path")?,
            file_size: row.try_get("file_size")?,
            cover_path: row.try_get("cover_path")?,
            description: row.try_get("description")?,
            created_at: chrono::DateTime::from_timestamp(created_at, 0)
                .unwrap_or_default()
                .to_rfc3339(),
            updated_at: chrono::DateTime::from_timestamp(updated_at, 0)
                .unwrap_or_default()
                .to_rfc3339(),
        })
    }

    /// 将数据库行转换为 Author
    fn row_to_author(row: sqlx::sqlite::SqliteRow) -> AppResult<Author> {
        let created_at: i64 = row.try_get("created_at")?;

        Ok(Author {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            sort_name: row.try_get("sort_name")?,
            created_at: chrono::DateTime::from_timestamp(created_at, 0)
                .unwrap_or_default()
                .to_rfc3339(),
        })
    }

    /// 将数据库行转换为 Tag
    fn row_to_tag(row: sqlx::sqlite::SqliteRow) -> AppResult<Tag> {
        let created_at: i64 = row.try_get("created_at")?;

        Ok(Tag {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            created_at: chrono::DateTime::from_timestamp(created_at, 0)
                .unwrap_or_default()
                .to_rfc3339(),
        })
    }
}
