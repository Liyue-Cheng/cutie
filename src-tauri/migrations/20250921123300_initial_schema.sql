CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    color TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    metadata BLOB,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    remote_updated_at INTEGER
);

CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'todo',
    due_date INTEGER,
    completed_at INTEGER,
    sort_key TEXT NOT NULL,
    metadata BLOB,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    remote_updated_at INTEGER,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL 
);

CREATE TABLE checkpoints (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    title TEXT NOT NULL,
    is_completed BOOLEAN NOT NULL DEFAULT 0,
    sort_key TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    remote_updated_at INTEGER,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

CREATE TABLE activities (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    timezone TEXT,
    is_all_day BOOLEAN NOT NULL DEFAULT 0,
    color TEXT,
    metadata BLOB,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    remote_updated_at INTEGER,
    origin_id TEXT,
    connector_id TEXT
);

CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    color TEXT,
    sort_key TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    deleted_at INTEGER,
    remote_updated_at INTEGER
);

CREATE UNIQUE INDEX idx_tags_title ON tags(title);

CREATE TABLE task_activity_links (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    activity_id TEXT NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (activity_id) REFERENCES activities(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_task_activity_unique ON task_activity_links(task_id, activity_id);

CREATE TABLE project_tags (
    project_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (project_id, tag_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE task_tags (
    task_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY NOT NULL,
    value BLOB NOT NULL
);