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

-- 创建 projects 表 (项目表)
CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    
    -- 状态管理（仅 ACTIVE 和 COMPLETED）
    status TEXT NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'COMPLETED')),
    
    -- 时间信息
    due_date TEXT, -- 截止日期 (YYYY-MM-DD)
    completed_at TEXT, -- 完成时间 (UTC RFC 3339)
    
    -- 关联（颜色从 area 继承）
    area_id TEXT,
    
    -- 元数据
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    
    FOREIGN KEY (area_id) REFERENCES areas(id)
);

-- 为 projects 表创建索引
CREATE INDEX idx_projects_updated_at ON projects(updated_at);
CREATE INDEX idx_projects_is_deleted ON projects(is_deleted);
CREATE INDEX idx_projects_area_id ON projects(area_id);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_due_date ON projects(due_date);

-- 创建 project_sections 表 (项目章节表)
CREATE TABLE project_sections (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    sort_order TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 为 project_sections 表创建索引
CREATE INDEX idx_project_sections_project_id ON project_sections(project_id);
CREATE INDEX idx_project_sections_is_deleted ON project_sections(is_deleted);
CREATE INDEX idx_project_sections_updated_at ON project_sections(updated_at);

-- 创建 tasks 表 (任务表)
CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    glance_note TEXT,
    detail_note TEXT,
    estimated_duration INTEGER,
    subtasks TEXT, -- JSON: [{"id": UUID, "title": String, "is_completed": Boolean, "sort_order": String}]
    project_id TEXT,
    section_id TEXT, -- 项目章节ID
    area_id TEXT,
    due_date TEXT, -- YYYY-MM-DD format (NaiveDate, 符合用户意图时间模型)
    due_date_type TEXT CHECK (due_date_type IN ('SOFT', 'HARD')),
    completed_at TEXT, -- UTC timestamp in RFC 3339 format
    archived_at TEXT, -- UTC timestamp in RFC 3339 format
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    deleted_at TEXT, -- UTC timestamp in RFC 3339 format (NULL = not deleted)
    source_info TEXT, -- JSON
    external_source_id TEXT,
    external_source_provider TEXT,
    external_source_metadata TEXT, -- JSON
    recurrence_original_date TEXT, -- YYYY-MM-DD (日历日期字符串)
    recurrence_id TEXT, -- 关联循环规则表
    
    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (section_id) REFERENCES project_sections(id),
    FOREIGN KEY (area_id) REFERENCES areas(id),
    
    -- 确保due_date和due_date_type的一致性
    CHECK (
        (due_date IS NULL AND due_date_type IS NULL) OR 
        (due_date IS NOT NULL AND due_date_type IS NOT NULL)
    ),
    -- 业务约束：如果有 section_id，必须有 project_id
    CHECK (section_id IS NULL OR project_id IS NOT NULL)
);

-- 为 tasks 表创建索引
CREATE INDEX idx_tasks_updated_at ON tasks(updated_at);
CREATE INDEX idx_tasks_deleted_at ON tasks(deleted_at);
CREATE INDEX idx_tasks_project_id ON tasks(project_id);
CREATE INDEX idx_tasks_section_id ON tasks(section_id);
CREATE INDEX idx_tasks_area_id ON tasks(area_id);
CREATE INDEX idx_tasks_external_source_id ON tasks(external_source_id);
CREATE INDEX idx_tasks_completed_at ON tasks(completed_at);
CREATE INDEX idx_tasks_archived_at ON tasks(archived_at);
CREATE INDEX idx_tasks_due_date ON tasks(due_date);
CREATE INDEX idx_tasks_recurrence_id ON tasks(recurrence_id);
CREATE INDEX idx_tasks_recurrence_original_date ON tasks(recurrence_original_date);

-- 创建 time_blocks 表 (时间块表)
CREATE TABLE time_blocks (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT,
    glance_note TEXT,
    detail_note TEXT,
    start_time TEXT NOT NULL, -- UTC timestamp in RFC 3339 format (解释方式取决于time_type)
    end_time TEXT NOT NULL, -- UTC timestamp in RFC 3339 format (解释方式取决于time_type)
    start_time_local TEXT, -- HH:MM:SS 本地时间 (仅FLOATING类型使用)
    end_time_local TEXT, -- HH:MM:SS 本地时间 (仅FLOATING类型使用)
    time_type TEXT NOT NULL DEFAULT 'FLOATING' CHECK (time_type IN ('FLOATING', 'FIXED')), -- 时间类型
    creation_timezone TEXT, -- 创建时的时区 (占位字段)
    is_all_day BOOLEAN NOT NULL DEFAULT FALSE, -- 是否为全天事件
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
    recurrence_original_date TEXT, -- YYYY-MM-DD (日历日期字符串)
    
    FOREIGN KEY (area_id) REFERENCES areas(id),
    FOREIGN KEY (recurrence_parent_id) REFERENCES time_blocks(id),
    
    -- 确保时间范围有效
    CHECK (start_time <= end_time),
    -- 当time_type='FLOATING'时，必须有local时间
    CHECK (time_type != 'FLOATING' OR (start_time_local IS NOT NULL AND end_time_local IS NOT NULL))
);

-- 为 time_blocks 表创建索引
CREATE INDEX idx_time_blocks_updated_at ON time_blocks(updated_at);
CREATE INDEX idx_time_blocks_is_deleted ON time_blocks(is_deleted);
CREATE INDEX idx_time_blocks_area_id ON time_blocks(area_id);
CREATE INDEX idx_time_blocks_start_time ON time_blocks(start_time);
CREATE INDEX idx_time_blocks_end_time ON time_blocks(end_time);
CREATE INDEX idx_time_blocks_time_type ON time_blocks(time_type);

-- 创建 templates 表 (模板表)
CREATE TABLE templates (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    glance_note_template TEXT,
    detail_note_template TEXT,
    estimated_duration_template INTEGER,
    subtasks_template TEXT, -- JSON
    area_id TEXT,
    category TEXT NOT NULL DEFAULT 'GENERAL' CHECK (category IN ('GENERAL', 'RECURRENCE')),
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    
    FOREIGN KEY (area_id) REFERENCES areas(id)
);

-- 为 templates 表创建索引
CREATE INDEX idx_templates_updated_at ON templates(updated_at);
CREATE INDEX idx_templates_is_deleted ON templates(is_deleted);
CREATE INDEX idx_templates_area_id ON templates(area_id);
CREATE INDEX idx_templates_category ON templates(category);

-- 创建 task_schedules 表 (任务日程表)
CREATE TABLE task_schedules (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    -- 📅 scheduled_date: 日历日期（YYYY-MM-DD 纯字符串，无时区）
    -- 语义：表示"用户本地时区的某一天"
    -- 存储格式：YYYY-MM-DD（如 "2025-10-08"）
    -- 前后端传输：统一使用此格式，不做时区转换
    scheduled_date TEXT NOT NULL,
    outcome TEXT NOT NULL DEFAULT 'PLANNED' CHECK (outcome IN ('PLANNED', 'PRESENCE_LOGGED', 'COMPLETED_ON_DAY', 'CARRIED_OVER')),
    created_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL, -- UTC timestamp in RFC 3339 format
    
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- 为 task_schedules 表创建索引
CREATE INDEX idx_task_schedules_task_id ON task_schedules(task_id);
CREATE INDEX idx_task_schedules_scheduled_date ON task_schedules(scheduled_date);
CREATE INDEX idx_task_schedules_outcome ON task_schedules(outcome);

-- ============================================================
-- 循环任务规则表 (Task Recurrences)
-- ============================================================
-- 存储生效的循环规则
-- 说明：
-- - rule: 循环字符串（如 RRULE:FREQ=DAILY 或自定义简化串）
-- - time_type: 'FLOATING' | 'FIXED'，表示是浮动时间还是绝对时间
-- - start_date/end_date: 可选生效边界（YYYY-MM-DD）
-- - timezone: 仅当 FIXED 时需要明确解释为某时区本地日

CREATE TABLE task_recurrences (
    id TEXT PRIMARY KEY NOT NULL,
    template_id TEXT NOT NULL,
    rule TEXT NOT NULL,
    time_type TEXT NOT NULL DEFAULT 'FLOATING' CHECK (time_type IN ('FLOATING', 'FIXED')),
    start_date TEXT,                         -- YYYY-MM-DD (日历日期字符串)
    end_date TEXT,                           -- YYYY-MM-DD (日历日期字符串)
    timezone TEXT,                           -- e.g. "Asia/Shanghai"
    expiry_behavior TEXT NOT NULL DEFAULT 'CARRYOVER_TO_STAGING' CHECK (expiry_behavior IN ('CARRYOVER_TO_STAGING', 'EXPIRE')), -- 过期处理策略
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TEXT NOT NULL,                -- UTC timestamp in RFC 3339 format
    updated_at TEXT NOT NULL,                -- UTC timestamp in RFC 3339 format

    FOREIGN KEY (template_id) REFERENCES templates(id) ON DELETE CASCADE
);

-- 为 task_recurrences 表创建索引
CREATE INDEX idx_task_recurrences_template_id ON task_recurrences(template_id);
CREATE INDEX idx_task_recurrences_is_active ON task_recurrences(is_active);
CREATE INDEX idx_task_recurrences_time_type ON task_recurrences(time_type);

-- ============================================================
-- 循环任务实例链接表 (Task Recurrence Links)
-- ============================================================
-- 为每条循环规则在"某一天"的实例与任务建立一条链接
-- 语义：
--   - (recurrence_id, instance_date) 联合唯一，保证同一规则同一天只有一个任务
--   - task_id 唯一，防止同一任务被多条规则/多天重复链接
-- 用法：
--   - 查今天是否已有实例：SELECT * FROM task_recurrence_links WHERE recurrence_id=? AND instance_date=?
--   - 无则创建任务 + 插入链接；有则取出 task_id 并校验任务是否仍属于今天

CREATE TABLE task_recurrence_links (
    recurrence_id TEXT NOT NULL,
    instance_date TEXT NOT NULL,             -- YYYY-MM-DD (日历日期字符串)
    task_id TEXT NOT NULL,
    created_at TEXT NOT NULL,                -- UTC timestamp in RFC 3339 format
    
    PRIMARY KEY (recurrence_id, instance_date),
    FOREIGN KEY (recurrence_id) REFERENCES task_recurrences(id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- 为 task_recurrence_links 表创建索引
CREATE UNIQUE INDEX idx_recurrence_links_task_id ON task_recurrence_links(task_id);
CREATE INDEX idx_recurrence_links_date ON task_recurrence_links(instance_date);
CREATE INDEX idx_recurrence_links_recurrence ON task_recurrence_links(recurrence_id);
-- 注意：(recurrence_id, instance_date) 的唯一性由 PRIMARY KEY 约束自动保证

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

-- ============================================================
-- 用户设置表 (User Settings) - V1.0
-- ============================================================
-- 存储用户的个性化配置
-- 采用 Key-Value 结构，每个设置项为一行记录

CREATE TABLE user_settings (
    -- 设置项的唯一标识符 (主键)
    setting_key TEXT PRIMARY KEY NOT NULL,
    
    -- 设置值 (JSON 格式存储，支持复杂数据类型)
    -- 示例:
    -- - 字符串: '"zh-CN"'
    -- - 数字: '100'
    -- - 布尔: 'true'
    -- - 对象: '{"format": "24h", "showSeconds": true}'
    setting_value TEXT NOT NULL,
    
    -- 设置项的数据类型 (用于前端反序列化)
    value_type TEXT NOT NULL CHECK (value_type IN ('string', 'number', 'boolean', 'object', 'array')),
    
    -- 设置项的分类 (用于UI分组显示)
    -- 'appearance' - 外观设置
    -- 'behavior' - 行为设置
    -- 'data' - 数据设置
    -- 'account' - 账号设置
    -- 'debug' - 调试设置
    -- 'system' - 系统设置
    -- 'ai' - AI设置
    category TEXT NOT NULL CHECK (category IN ('appearance', 'behavior', 'data', 'account', 'debug', 'system', 'ai')),
    
    -- 最后更新时间 (UTC timestamp in RFC 3339 format)
    updated_at TEXT NOT NULL,
    
    -- 创建时间 (UTC timestamp in RFC 3339 format)
    created_at TEXT NOT NULL
);

-- 为常用查询创建索引
CREATE INDEX idx_user_settings_category ON user_settings(category);
CREATE INDEX idx_user_settings_updated_at ON user_settings(updated_at);