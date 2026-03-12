-- Add source_site field to novel_books table
-- Version: 0004
-- Description: Add source site information for books

ALTER TABLE novel_books ADD COLUMN source_site TEXT;
