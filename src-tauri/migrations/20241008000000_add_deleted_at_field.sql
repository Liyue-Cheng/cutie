-- 为软删除功能添加 deleted_at 字段
-- 用于实现"最近删除"（回收站）功能

-- 为 tasks 表添加 deleted_at 字段
ALTER TABLE tasks ADD COLUMN deleted_at TEXT;

-- 为 areas 表添加 deleted_at 字段
ALTER TABLE areas ADD COLUMN deleted_at TEXT;

-- 为 time_blocks 表添加 deleted_at 字段
ALTER TABLE time_blocks ADD COLUMN deleted_at TEXT;

-- 为 deleted_at 创建索引（用于回收站查询和排序）
CREATE INDEX idx_tasks_deleted_at ON tasks(deleted_at);
CREATE INDEX idx_areas_deleted_at ON areas(deleted_at);
CREATE INDEX idx_time_blocks_deleted_at ON time_blocks(deleted_at);
