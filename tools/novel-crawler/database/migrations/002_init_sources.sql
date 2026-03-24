-- Migration 002: Initialize source site database schema
-- This migration creates tables for source site databases (qidian.db, zongheng.db, etc.)

-- Book details table: Extended information from source sites
CREATE TABLE IF NOT EXISTS book_details (
    book_id INTEGER PRIMARY KEY,
    youshu_id INTEGER,
    title TEXT NOT NULL,
    author TEXT,
    description TEXT,
    category TEXT,
    sub_category TEXT,
    tags TEXT,
    cover_url TEXT,
    cover_path TEXT,
    word_count INTEGER,
    chapter_count INTEGER,
    status TEXT,
    rating REAL,
    view_count INTEGER,
    favorite_count INTEGER,
    crawled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (youshu_id) REFERENCES books(id)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_details_youshu ON book_details(youshu_id);
CREATE INDEX IF NOT EXISTS idx_details_author ON book_details(author);
CREATE INDEX IF NOT EXISTS idx_details_category ON book_details(category);
CREATE INDEX IF NOT EXISTS idx_details_rating ON book_details(rating);
