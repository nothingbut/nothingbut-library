-- Core tables migration
-- Version: 0001
-- Description: Create core application tables for workspaces and library configuration

-- App workspaces table
CREATE TABLE IF NOT EXISTS app_workspaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    module_type TEXT NOT NULL CHECK(module_type IN ('novel', 'knowledge', 'mixed')),
    last_opened_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Library configuration table (key-value store)
CREATE TABLE IF NOT EXISTS library_config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_workspaces_last_opened
    ON app_workspaces(last_opened_at DESC);

CREATE INDEX IF NOT EXISTS idx_workspaces_module_type
    ON app_workspaces(module_type);
