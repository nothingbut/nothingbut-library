CREATE TABLE IF NOT EXISTS weread_accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    vid TEXT UNIQUE NOT NULL,
    username TEXT NOT NULL,
    avatar_url TEXT,
    cookies_json TEXT NOT NULL,
    cookie_expires_at INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS weread_books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,
    book_id TEXT NOT NULL,
    title TEXT NOT NULL,
    author TEXT,
    cover_url TEXT,
    intro TEXT,
    category TEXT,
    word_count INTEGER DEFAULT 0,
    chapter_count INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL,
    UNIQUE(library_id, book_id),
    FOREIGN KEY (library_id) REFERENCES libraries(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS weread_chapters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,
    book_id INTEGER NOT NULL,
    chapter_uid TEXT NOT NULL,
    title TEXT NOT NULL,
    level INTEGER DEFAULT 1,
    word_count INTEGER DEFAULT 0,
    sort_order INTEGER NOT NULL,
    content_md TEXT,
    extracted_at INTEGER,
    UNIQUE(library_id, book_id, chapter_uid),
    FOREIGN KEY (book_id) REFERENCES weread_books(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS weread_downloads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL,
    book_id INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    progress_current INTEGER DEFAULT 0,
    progress_total INTEGER DEFAULT 0,
    output_path TEXT,
    error_message TEXT,
    started_at INTEGER,
    completed_at INTEGER,
    created_at INTEGER NOT NULL,
    UNIQUE(library_id, book_id),
    FOREIGN KEY (book_id) REFERENCES weread_books(id) ON DELETE CASCADE
);

-- Expand module_type CHECK constraint to include 'weread'
CREATE TABLE libraries_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    module_type TEXT NOT NULL CHECK(module_type IN ('novel', 'epub', 'music', 'note', 'bilibili', 'weread')),
    storage_path TEXT NOT NULL,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_accessed_at INTEGER,
    UNIQUE(module_type, name)
);

INSERT INTO libraries_new SELECT * FROM libraries;
DROP TABLE libraries;
ALTER TABLE libraries_new RENAME TO libraries;

CREATE INDEX IF NOT EXISTS idx_libraries_module_type ON libraries(module_type);
CREATE INDEX IF NOT EXISTS idx_libraries_last_accessed ON libraries(last_accessed_at DESC);

INSERT OR IGNORE INTO libraries (name, module_type, is_default, storage_path, description, created_at)
VALUES ('微信读书', 'weread', 1, 'weread', '微信读书导出', strftime('%s', 'now'));
