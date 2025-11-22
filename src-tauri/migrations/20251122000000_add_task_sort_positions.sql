-- Migration: 添加 tasks.sort_positions 支持 LexoRank 排序

PRAGMA foreign_keys = ON;

ALTER TABLE tasks
    ADD COLUMN sort_positions TEXT NOT NULL DEFAULT '{}';

CREATE INDEX idx_tasks_sort_positions
    ON tasks(sort_positions);

CREATE INDEX idx_tasks_sort_staging
    ON tasks(json_extract(sort_positions, '$.misc::staging'))
    WHERE json_extract(sort_positions, '$.misc::staging') IS NOT NULL;

