-- Novel module migration
-- Version: 0002
-- Description: Create tables for novel management (categories, books, chapters, bookmarks, stats)

-- Novel categories table (hierarchical structure)
CREATE TABLE IF NOT EXISTS novel_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    parent_id INTEGER,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (parent_id) REFERENCES novel_categories(id) ON DELETE CASCADE
);

-- Novel books table
CREATE TABLE IF NOT EXISTS novel_books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    author TEXT,
    description TEXT,
    cover_path TEXT,
    category_id INTEGER,
    book_dir TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL DEFAULT 0,
    word_count INTEGER NOT NULL DEFAULT 0,
    chapter_count INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'ongoing' CHECK(status IN ('ongoing', 'completed', 'abandoned')),
    reading_progress REAL NOT NULL DEFAULT 0.0,
    last_read_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (category_id) REFERENCES novel_categories(id) ON DELETE SET NULL
);

-- Novel chapters table
CREATE TABLE IF NOT EXISTS novel_chapters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    file_path TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    word_count INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (book_id) REFERENCES novel_books(id) ON DELETE CASCADE
);

-- Novel bookmarks table
CREATE TABLE IF NOT EXISTS novel_bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL,
    chapter_id INTEGER,
    position INTEGER NOT NULL DEFAULT 0,
    note TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (book_id) REFERENCES novel_books(id) ON DELETE CASCADE,
    FOREIGN KEY (chapter_id) REFERENCES novel_chapters(id) ON DELETE SET NULL
);

-- Novel reading statistics table
CREATE TABLE IF NOT EXISTS novel_reading_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL,
    date TEXT NOT NULL,
    reading_duration INTEGER NOT NULL DEFAULT 0,
    words_read INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (book_id) REFERENCES novel_books(id) ON DELETE CASCADE,
    UNIQUE(book_id, date)
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_categories_parent
    ON novel_categories(parent_id);

CREATE INDEX IF NOT EXISTS idx_categories_sort
    ON novel_categories(sort_order);

CREATE INDEX IF NOT EXISTS idx_books_category
    ON novel_books(category_id);

CREATE INDEX IF NOT EXISTS idx_books_status
    ON novel_books(status);

CREATE INDEX IF NOT EXISTS idx_books_last_read
    ON novel_books(last_read_at DESC);

CREATE INDEX IF NOT EXISTS idx_chapters_book
    ON novel_chapters(book_id, sort_order);

CREATE INDEX IF NOT EXISTS idx_bookmarks_book
    ON novel_bookmarks(book_id);

CREATE INDEX IF NOT EXISTS idx_bookmarks_chapter
    ON novel_bookmarks(chapter_id);

CREATE INDEX IF NOT EXISTS idx_reading_stats_book_date
    ON novel_reading_stats(book_id, date DESC);
