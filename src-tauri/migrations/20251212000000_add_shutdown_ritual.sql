-- Migration: add daily shutdown ritual (steps + daily progress)

PRAGMA foreign_keys = ON;

-- =========================
-- shutdown_ritual_steps
-- =========================
CREATE TABLE IF NOT EXISTS shutdown_ritual_steps (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    order_rank TEXT NOT NULL,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL  -- UTC timestamp in RFC 3339 format
);

CREATE INDEX IF NOT EXISTS idx_shutdown_ritual_steps_order_rank
    ON shutdown_ritual_steps(order_rank);

-- =========================
-- shutdown_ritual_progress
-- =========================
CREATE TABLE IF NOT EXISTS shutdown_ritual_progress (
    id TEXT PRIMARY KEY NOT NULL,
    step_id TEXT NOT NULL,
    date TEXT NOT NULL,          -- YYYY-MM-DD (local day string)
    completed_at TEXT,           -- UTC timestamp in RFC 3339 format, NULL = incomplete
    created_at TEXT NOT NULL,    -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL,    -- UTC timestamp in RFC 3339 format

    FOREIGN KEY (step_id) REFERENCES shutdown_ritual_steps(id) ON DELETE CASCADE,
    UNIQUE (step_id, date)
);

CREATE INDEX IF NOT EXISTS idx_shutdown_ritual_progress_date
    ON shutdown_ritual_progress(date);

CREATE INDEX IF NOT EXISTS idx_shutdown_ritual_progress_step_id
    ON shutdown_ritual_progress(step_id);


