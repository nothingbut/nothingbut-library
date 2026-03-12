-- Add preview column to novel_chapters
-- Version: 0003
-- Description: Add preview field to store first line of chapter content

ALTER TABLE novel_chapters ADD COLUMN preview TEXT DEFAULT '';
