-- Migration: add sort_rank to templates for LexoRank ordering

ALTER TABLE templates ADD COLUMN sort_rank TEXT;

CREATE INDEX IF NOT EXISTS idx_templates_sort_rank ON templates(sort_rank);

