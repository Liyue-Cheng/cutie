-- Migration: add shutdown ritual settings (singleton)
--
-- Purpose:
-- - Persist customizable ritual title (configured from frontend editor panel)
-- - Keep steps/progress schema unchanged

PRAGMA foreign_keys = ON;

-- =========================
-- shutdown_ritual_settings (singleton)
-- =========================
CREATE TABLE IF NOT EXISTS shutdown_ritual_settings (
    id TEXT PRIMARY KEY NOT NULL, -- singleton key, e.g. 'default'
    title TEXT,                   -- NULL = use frontend fallback/i18n
    created_at TEXT NOT NULL,      -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL       -- UTC timestamp in RFC 3339 format
);

-- Ensure default singleton row exists
INSERT INTO shutdown_ritual_settings (id, title, created_at, updated_at)
SELECT
    'default',
    NULL,
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE NOT EXISTS (SELECT 1 FROM shutdown_ritual_settings WHERE id = 'default');


