-- EPUB module migration
-- Version: 0007
-- Description: Create tables for EPUB e-book management (books, authors, tags, custom fields, reading progress)
-- Date: 2026-03-24

-- EPUB books table (main books catalog)
CREATE TABLE IF NOT EXISTS epub_books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    sort_title TEXT,
    isbn TEXT,
    publisher TEXT,
    pubdate TEXT,
    language TEXT,
    series TEXT,
    series_index REAL,
    rating INTEGER CHECK(rating >= 0 AND rating <= 5),
    file_path TEXT NOT NULL UNIQUE,
    file_size INTEGER NOT NULL,
    cover_path TEXT,
    description TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- EPUB authors table
CREATE TABLE IF NOT EXISTS epub_authors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    sort_name TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- EPUB book-author relationship (many-to-many)
CREATE TABLE IF NOT EXISTS epub_book_authors (
    book_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    author_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (book_id, author_id),
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE,
    FOREIGN KEY (author_id) REFERENCES epub_authors(id) ON DELETE CASCADE
);

-- EPUB tags table
CREATE TABLE IF NOT EXISTS epub_tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- EPUB book-tag relationship (many-to-many)
CREATE TABLE IF NOT EXISTS epub_book_tags (
    book_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (book_id, tag_id),
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES epub_tags(id) ON DELETE CASCADE
);

-- EPUB custom fields definition
CREATE TABLE IF NOT EXISTS epub_custom_fields (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    label TEXT NOT NULL,
    datatype TEXT NOT NULL CHECK(datatype IN ('text', 'series', 'enumeration', 'number', 'rating', 'date', 'bool', 'comments')),
    is_multiple INTEGER NOT NULL DEFAULT 0 CHECK(is_multiple IN (0, 1)),
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- EPUB custom field values (book-specific custom metadata)
CREATE TABLE IF NOT EXISTS epub_custom_field_values (
    book_id INTEGER NOT NULL,
    field_id INTEGER NOT NULL,
    value TEXT NOT NULL,
    PRIMARY KEY (book_id, field_id),
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE,
    FOREIGN KEY (field_id) REFERENCES epub_custom_fields(id) ON DELETE CASCADE
);

-- EPUB reading progress tracking
CREATE TABLE IF NOT EXISTS epub_reading_progress (
    book_id INTEGER PRIMARY KEY,
    chapter_href TEXT NOT NULL,
    progress_percent REAL NOT NULL DEFAULT 0.0 CHECK(progress_percent >= 0.0 AND progress_percent <= 100.0),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE
);

-- EPUB bookmarks (user-placed bookmarks)
CREATE TABLE IF NOT EXISTS epub_bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL,
    chapter_href TEXT NOT NULL,
    cfi TEXT NOT NULL,
    note TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE
);

-- EPUB highlights (highlighted text passages)
CREATE TABLE IF NOT EXISTS epub_highlights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER NOT NULL,
    chapter_href TEXT NOT NULL,
    cfi_range TEXT NOT NULL,
    text TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#ffeb3b',
    note TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (book_id) REFERENCES epub_books(id) ON DELETE CASCADE
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_epub_books_title
    ON epub_books(title);

CREATE INDEX IF NOT EXISTS idx_epub_books_series
    ON epub_books(series);

CREATE INDEX IF NOT EXISTS idx_epub_books_isbn
    ON epub_books(isbn);

CREATE INDEX IF NOT EXISTS idx_epub_authors_name
    ON epub_authors(name);

CREATE INDEX IF NOT EXISTS idx_epub_book_authors_book
    ON epub_book_authors(book_id);

CREATE INDEX IF NOT EXISTS idx_epub_book_authors_author
    ON epub_book_authors(author_id);

CREATE INDEX IF NOT EXISTS idx_epub_book_tags_book
    ON epub_book_tags(book_id);

CREATE INDEX IF NOT EXISTS idx_epub_book_tags_tag
    ON epub_book_tags(tag_id);

CREATE INDEX IF NOT EXISTS idx_epub_bookmarks_book
    ON epub_bookmarks(book_id);

CREATE INDEX IF NOT EXISTS idx_epub_highlights_book
    ON epub_highlights(book_id);

CREATE INDEX IF NOT EXISTS idx_epub_custom_field_values_field
    ON epub_custom_field_values(field_id);
