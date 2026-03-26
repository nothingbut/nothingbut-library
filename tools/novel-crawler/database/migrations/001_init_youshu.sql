-- Migration 001: Initialize main database schema
-- This migration creates the initial tables for the youshu.db database

-- Books table: Main book index
CREATE TABLE IF NOT EXISTS books (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT,
    description TEXT,
    tags TEXT,
    cover_path TEXT,
    source_site TEXT,
    source_url TEXT,
    update_status TEXT,
    crawled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Crawl status table: Track crawl progress
CREATE TABLE IF NOT EXISTS crawl_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    last_valid_id INTEGER NOT NULL,
    last_crawl_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    total_books INTEGER DEFAULT 0,
    failed_ids TEXT,
    crawl_type TEXT,
    duration_seconds INTEGER,
    success_count INTEGER DEFAULT 0,
    failure_count INTEGER DEFAULT 0
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_author ON books(author);
CREATE INDEX IF NOT EXISTS idx_source ON books(source_site);
CREATE INDEX IF NOT EXISTS idx_status ON books(update_status);
CREATE INDEX IF NOT EXISTS idx_title ON books(title);

-- Insert initial crawl status
INSERT INTO crawl_status (last_valid_id, total_books, failed_ids, crawl_type)
VALUES (0, 0, '[]', 'initial');
