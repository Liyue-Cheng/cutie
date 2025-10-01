-- Cutie V1.0 数据库初始化迁移脚本
-- 基于架构纲领 V1.8 "定稿版" 数据库Schema
--
-- 时间存储约定:
-- 所有 *_at, *_time, *_date 列都存储 UTC 时间，使用 RFC 3339 格式 (例如: "2024-01-15T08:30:00Z")
-- SQLx 会自动在 DateTime<Utc> 和 TEXT 之间进行转换

-- 启用外键约束
PRAGMA foreign_keys = ON;

-- 创建 areas 表 (领域表)
CREATE TABLE areas (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    parent_area_id TEXT,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    
    FOREIGN KEY (parent_area_id) REFERENCES areas(id)
);

-- 为 areas 表创建索引
CREATE INDEX idx_areas_updated_at ON areas(updated_at);
CREATE INDEX idx_areas_is_deleted ON areas(is_deleted);
CREATE INDEX idx_areas_parent_area_id ON areas(parent_area_id);

-- 创建 projects 表 (项目表) - V1.0仅建表，不提供API
CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'PAUSED', 'COMPLETED', 'ARCHIVED')),
    type TEXT NOT NULL DEFAULT 'PROJECT' CHECK (type IN ('PROJECT', 'EXPERIENCE')),
    resources TEXT, -- JSON
    area_id TEXT,
    completed_at TEXT, -- UTC timestamp in RFC 3339 format
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    external_source_id TEXT,
    external_source_provider TEXT,
    external_source_metadata TEXT, -- JSON
    
    FOREIGN KEY (area_id) REFERENCES areas(id)
);

-- 为 projects 表创建索引
CREATE INDEX idx_projects_updated_at ON projects(updated_at);
CREATE INDEX idx_projects_is_deleted ON projects(is_deleted);
CREATE INDEX idx_projects_area_id ON projects(area_id);
CREATE INDEX idx_projects_external_source_id ON projects(external_source_id);

-- 创建 tasks 表 (任务表)
CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    glance_note TEXT,
    detail_note TEXT,
    estimated_duration INTEGER,
    subtasks TEXT, -- JSON: [{"id": UUID, "title": String, "is_completed": Boolean, "sort_order": String}]
    project_id TEXT,
    area_id TEXT,
    due_date TEXT, -- UTC timestamp in RFC 3339 format
    due_date_type TEXT CHECK (due_date_type IN ('SOFT', 'HARD')),
    completed_at TEXT, -- UTC timestamp in RFC 3339 format
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    source_info TEXT, -- JSON
    external_source_id TEXT,
    external_source_provider TEXT,
    external_source_metadata TEXT, -- JSON
    recurrence_rule TEXT,
    recurrence_parent_id TEXT,
    recurrence_original_date TEXT, -- UTC timestamp in RFC 3339 format
    recurrence_exclusions TEXT, -- JSON: Array of UTC timestamps in RFC 3339 format
    
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (area_id) REFERENCES areas(id),
    FOREIGN KEY (recurrence_parent_id) REFERENCES tasks(id),
    
    -- 确保due_date和due_date_type的一致性
    CHECK (
        (due_date IS NULL AND due_date_type IS NULL) OR 
        (due_date IS NOT NULL AND due_date_type IS NOT NULL)
    )
);

-- 为 tasks 表创建索引
CREATE INDEX idx_tasks_updated_at ON tasks(updated_at);
CREATE INDEX idx_tasks_is_deleted ON tasks(is_deleted);
CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_area_id ON tasks(area_id);
CREATE INDEX idx_tasks_external_source_id ON tasks(external_source_id);
CREATE INDEX idx_tasks_completed_at ON tasks(completed_at);
CREATE INDEX idx_tasks_due_date ON tasks(due_date);

-- 创建 time_blocks 表 (时间块表)
CREATE TABLE time_blocks (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT,
    glance_note TEXT,
    detail_note TEXT,
    start_time TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    end_time TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    area_id TEXT,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    source_info TEXT, -- JSON
    external_source_id TEXT,
    external_source_provider TEXT,
    external_source_metadata TEXT, -- JSON
    recurrence_rule TEXT,
    recurrence_parent_id TEXT,
    recurrence_original_date TEXT, -- UTC timestamp in RFC 3339 format
    recurrence_exclusions TEXT, -- JSON: Array of UTC timestamps
    
    FOREIGN KEY (area_id) REFERENCES areas(id),
    FOREIGN KEY (recurrence_parent_id) REFERENCES time_blocks(id),
    
    -- 确保时间范围有效
    CHECK (start_time <= end_time)
);

-- 为 time_blocks 表创建索引
CREATE INDEX idx_time_blocks_updated_at ON time_blocks(updated_at);
CREATE INDEX idx_time_blocks_is_deleted ON time_blocks(is_deleted);
CREATE INDEX idx_time_blocks_area_id ON time_blocks(area_id);
CREATE INDEX idx_time_blocks_start_time ON time_blocks(start_time);
CREATE INDEX idx_time_blocks_end_time ON time_blocks(end_time);

-- 创建 templates 表 (模板表)
CREATE TABLE templates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    title_template TEXT NOT NULL,
    glance_note_template TEXT,
    detail_note_template TEXT,
    estimated_duration_template INTEGER,
    subtasks_template TEXT, -- JSON
    area_id TEXT,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    
    FOREIGN KEY (area_id) REFERENCES areas(id)
);

-- 为 templates 表创建索引
CREATE INDEX idx_templates_updated_at ON templates(updated_at);
CREATE INDEX idx_templates_is_deleted ON templates(is_deleted);
CREATE INDEX idx_templates_area_id ON templates(area_id);

-- 创建 task_schedules 表 (任务日程表)
CREATE TABLE task_schedules (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    scheduled_day TEXT NOT NULL, -- UTC timestamp in RFC 3339 format (represents the start of day in UTC)
    outcome TEXT NOT NULL DEFAULT 'PLANNED' CHECK (outcome IN ('PLANNED', 'PRESENCE_LOGGED', 'COMPLETED_ON_DAY', 'CARRIED_OVER')),
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- 为 task_schedules 表创建索引
CREATE INDEX idx_task_schedules_task_id ON task_schedules(task_id);
CREATE INDEX idx_task_schedules_scheduled_day ON task_schedules(scheduled_day);
CREATE INDEX idx_task_schedules_outcome ON task_schedules(outcome);

-- 创建 orderings 表 (统一排序表)
CREATE TABLE orderings (
    id TEXT PRIMARY KEY NOT NULL,
    context_type TEXT NOT NULL CHECK (context_type IN ('DAILY_KANBAN', 'PROJECT_LIST', 'AREA_FILTER', 'MISC')),
    context_id TEXT NOT NULL,
    task_id TEXT NOT NULL,
    sort_order TEXT NOT NULL,
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    
    -- 复合唯一约束
    UNIQUE(context_type, context_id, task_id)
);

-- 为 orderings 表创建索引
CREATE INDEX idx_orderings_context ON orderings(context_type, context_id);
CREATE INDEX idx_orderings_task_id ON orderings(task_id);
CREATE INDEX idx_orderings_sort_order ON orderings(sort_order);
CREATE INDEX idx_orderings_updated_at ON orderings(updated_at);

-- 创建 task_time_block_links 表 (任务-时间块链接表)
CREATE TABLE task_time_block_links (
    task_id TEXT NOT NULL,
    time_block_id TEXT NOT NULL,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    
    PRIMARY KEY (task_id, time_block_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (time_block_id) REFERENCES time_blocks(id) ON DELETE CASCADE
);

-- 为链接表创建索引
CREATE INDEX idx_task_time_block_links_task_id ON task_time_block_links(task_id);
CREATE INDEX idx_task_time_block_links_time_block_id ON task_time_block_links(time_block_id);

-- 创建延迟实现的表 (V1.0仅建表，不提供API)

-- 创建 time_points 表 (时间点表)
CREATE TABLE time_points (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    point_time TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    area_id TEXT,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    
    FOREIGN KEY (area_id) REFERENCES areas(id)
);

CREATE INDEX idx_time_points_point_time ON time_points(point_time);
CREATE INDEX idx_time_points_area_id ON time_points(area_id);

-- 创建 tags 表 (标签表)
CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    color TEXT NOT NULL,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX idx_tags_name ON tags(name);

-- 创建 task_tag_links 表 (任务-标签链接表)
CREATE TABLE task_tag_links (
    task_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- 创建 time_block_tag_links 表 (时间块-标签链接表)
CREATE TABLE time_block_tag_links (
    time_block_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    
    PRIMARY KEY (time_block_id, tag_id),
    FOREIGN KEY (time_block_id) REFERENCES time_blocks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- 创建 reminders 表 (提醒表)
CREATE TABLE reminders (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT,
    time_block_id TEXT,
    reminder_time TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    message TEXT,
    is_sent BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (time_block_id) REFERENCES time_blocks(id) ON DELETE CASCADE,
    
    -- 确保关联到任务或时间块之一
    CHECK (
        (task_id IS NOT NULL AND time_block_id IS NULL) OR 
        (task_id IS NULL AND time_block_id IS NOT NULL)
    )
);

CREATE INDEX idx_reminders_reminder_time ON reminders(reminder_time);
CREATE INDEX idx_reminders_task_id ON reminders(task_id);
CREATE INDEX idx_reminders_time_block_id ON reminders(time_block_id);

-- ============================================================
-- 视图排序偏好表 (View Preferences)
-- ============================================================
-- 用于存储用户在各种视图中的任务排序配置
--
-- Context Key 格式规范：
-- - 杂项视图: misc::{id}           例如: misc::staging, misc::planned
-- - 日期看板: daily::{YYYY-MM-DD}   例如: daily::2025-10-01
-- - 区域看板: area::{area_uuid}     例如: area::a1b2c3d4-1234...
-- - 项目看板: project::{proj_uuid}  例如: project::proj-uuid-1234

CREATE TABLE view_preferences (
    -- 视图上下文唯一标识（复合主键）
    context_key TEXT PRIMARY KEY NOT NULL,
    
    -- 排序后的任务ID数组（JSON字符串格式）
    -- 示例: '["uuid-1", "uuid-2", "uuid-3"]'
    -- 数组顺序即为任务在该视图中的显示顺序
    sorted_task_ids TEXT NOT NULL,
    
    -- 最后更新时间（UTC timestamp in RFC 3339 format）
    updated_at TEXT NOT NULL
);

-- 为常用查询创建索引
CREATE INDEX idx_view_prefs_updated_at ON view_preferences(updated_at);

-- ============================================================
-- 事件发件箱表 (Event Outbox)
-- ============================================================
-- 用于实现可靠的事件投递（Transactional Outbox Pattern）
-- 业务事务内写入 event_outbox，提交后由后台分发器扫描并推送到 SSE 流
--
-- 事件信封规范：
-- - id: 全局唯一递增ID（用于 Last-Event-ID 续传）
-- - event_id: UUID，用于去重
-- - type: 事件类型（如 task.completed、time_blocks.truncated）
-- - version: 事件契约版本
-- - aggregate_type: 聚合类型（task、time_block 等）
-- - aggregate_id: 聚合根ID
-- - aggregate_version: 聚合版本或 updated_at 单调戳（用于幂等）
-- - correlation_id: 关联的命令ID（HTTP 请求）
-- - occurred_at: 事件发生时间（UTC RFC 3339）
-- - payload: 事件载荷（JSON）
-- - dispatched_at: 已分发时间（NULL 表示未分发）
--
CREATE TABLE event_outbox (
    -- 全局递增ID（主键，用于排序与续传）
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    
    -- 事件唯一标识（UUID）
    event_id TEXT NOT NULL UNIQUE,
    
    -- 事件类型（dot-separated，如 task.completed）
    event_type TEXT NOT NULL,
    
    -- 事件契约版本
    version INTEGER NOT NULL DEFAULT 1,
    
    -- 聚合类型与ID
    aggregate_type TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    
    -- 聚合版本（用于幂等，可为 NULL）
    aggregate_version INTEGER,
    
    -- 关联的命令ID（用于去重，可为 NULL）
    correlation_id TEXT,
    
    -- 事件发生时间（UTC timestamp in RFC 3339 format）
    occurred_at TEXT NOT NULL,
    
    -- 事件载荷（JSON）
    payload TEXT NOT NULL,
    
    -- 已分发时间（NULL 表示未分发，UTC timestamp in RFC 3339 format）
    dispatched_at TEXT,
    
    -- 创建时间（UTC timestamp in RFC 3339 format）
    created_at TEXT NOT NULL
);

-- 未分发事件索引（dispatcher 查询用）
CREATE INDEX idx_outbox_undispatched ON event_outbox(dispatched_at) WHERE dispatched_at IS NULL;

-- 事件ID索引（去重查询）
CREATE INDEX idx_outbox_event_id ON event_outbox(event_id);

-- 聚合索引（按聚合查询事件）
CREATE INDEX idx_outbox_aggregate ON event_outbox(aggregate_type, aggregate_id);

-- 时间索引（清理旧事件）
CREATE INDEX idx_outbox_created_at ON event_outbox(created_at);