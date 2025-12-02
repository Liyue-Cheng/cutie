-- ============================================================
-- 时间块循环功能迁移
-- ============================================================

-- ============================================================
-- 时间块循环模板表 (Time Block Templates)
-- ============================================================
-- 存储循环时间块的模板信息
-- 与任务模板分开，因为时间块有不同的字段（时间段、时长等）
CREATE TABLE time_block_templates (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT,                           -- 标题模板
    glance_note_template TEXT,            -- 快览笔记模板
    detail_note_template TEXT,            -- 详细笔记模板
    duration_minutes INTEGER NOT NULL,    -- 时长（分钟）
    start_time_local TEXT NOT NULL,       -- 每天开始时间 (HH:MM:SS，如 "08:00:00")
    time_type TEXT NOT NULL DEFAULT 'FLOATING' CHECK (time_type IN ('FLOATING', 'FIXED')),
    is_all_day BOOLEAN NOT NULL DEFAULT FALSE,
    area_id TEXT,
    created_at TEXT NOT NULL,             -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL,             -- UTC timestamp in RFC 3339 format
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    
    FOREIGN KEY (area_id) REFERENCES areas(id)
);

-- 为 time_block_templates 表创建索引
CREATE INDEX idx_time_block_templates_updated_at ON time_block_templates(updated_at);
CREATE INDEX idx_time_block_templates_is_deleted ON time_block_templates(is_deleted);
CREATE INDEX idx_time_block_templates_area_id ON time_block_templates(area_id);

-- ============================================================
-- 时间块循环规则表 (Time Block Recurrences)
-- ============================================================
-- 存储时间块的循环规则
CREATE TABLE time_block_recurrences (
    id TEXT PRIMARY KEY NOT NULL,
    template_id TEXT NOT NULL,
    rule TEXT NOT NULL,                   -- RRULE 标准字符串 (如 "FREQ=DAILY", "FREQ=WEEKLY;BYDAY=MO,WE,FR")
    time_type TEXT NOT NULL DEFAULT 'FLOATING' CHECK (time_type IN ('FLOATING', 'FIXED')),
    start_date TEXT,                      -- 生效起始日期 (YYYY-MM-DD)
    end_date TEXT,                        -- 生效结束日期 (YYYY-MM-DD)
    timezone TEXT,                        -- 时区 (如 "Asia/Shanghai")
    skip_conflicts BOOLEAN NOT NULL DEFAULT TRUE, -- 遇到冲突时是否跳过（而不是报错）
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TEXT NOT NULL,             -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL,             -- UTC timestamp in RFC 3339 format

    FOREIGN KEY (template_id) REFERENCES time_block_templates(id) ON DELETE CASCADE
);

-- 为 time_block_recurrences 表创建索引
CREATE INDEX idx_time_block_recurrences_template_id ON time_block_recurrences(template_id);
CREATE INDEX idx_time_block_recurrences_is_active ON time_block_recurrences(is_active);
CREATE INDEX idx_time_block_recurrences_start_date ON time_block_recurrences(start_date);
CREATE INDEX idx_time_block_recurrences_end_date ON time_block_recurrences(end_date);

-- ============================================================
-- 时间块循环实例链接表 (Time Block Recurrence Links)
-- ============================================================
-- 记录循环规则在特定日期生成的时间块实例
-- 语义：
--   - (recurrence_id, instance_date) 联合唯一，保证同一规则同一天只有一个时间块
--   - time_block_id 唯一，防止同一时间块被多条规则/多天重复链接
CREATE TABLE time_block_recurrence_links (
    recurrence_id TEXT NOT NULL,
    instance_date TEXT NOT NULL,          -- YYYY-MM-DD (日历日期字符串)
    time_block_id TEXT NOT NULL,
    created_at TEXT NOT NULL,             -- UTC timestamp in RFC 3339 format
    
    PRIMARY KEY (recurrence_id, instance_date),
    FOREIGN KEY (recurrence_id) REFERENCES time_block_recurrences(id) ON DELETE CASCADE,
    FOREIGN KEY (time_block_id) REFERENCES time_blocks(id) ON DELETE CASCADE
);

-- 为 time_block_recurrence_links 表创建索引
CREATE UNIQUE INDEX idx_time_block_recurrence_links_time_block_id ON time_block_recurrence_links(time_block_id);
CREATE INDEX idx_time_block_recurrence_links_date ON time_block_recurrence_links(instance_date);
CREATE INDEX idx_time_block_recurrence_links_recurrence ON time_block_recurrence_links(recurrence_id);

-- 为现有 time_blocks 表添加索引（如果不存在）
CREATE INDEX IF NOT EXISTS idx_time_blocks_recurrence_parent_id ON time_blocks(recurrence_parent_id);
CREATE INDEX IF NOT EXISTS idx_time_blocks_recurrence_original_date ON time_blocks(recurrence_original_date);
