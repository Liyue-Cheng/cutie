# Cutie 数据库字段修改指南

## 概述

本文档详细说明了在Cutie后端系统中修改数据库字段的完整流程，包括所有需要同步修改的代码位置和具体的修改步骤。严格遵循本指南可以确保数据库变更的安全性和系统的一致性。

## 修改原则

### 1. 安全性原则

- **数据迁移**: 所有字段变更必须提供数据迁移路径
- **向后兼容**: 新字段必须有合理的默认值
- **事务安全**: 所有变更在事务中执行

### 2. 一致性原则

- **类型一致**: 数据库类型与Rust类型保持一致
- **约束一致**: 数据库约束与业务验证保持一致
- **命名一致**: 字段命名遵循统一规范

### 3. 性能原则

- **索引优化**: 新字段需要考虑索引需求
- **查询优化**: 修改后的查询性能不能退化
- **存储优化**: 考虑存储空间和I/O影响

## 数据库字段修改流程

### Step 1: 创建数据库迁移

#### 1.1 创建迁移文件

```bash
# 生成迁移文件名
MIGRATION_NAME=$(date +%Y%m%d%H%M%S)_modify_table_fields.sql
touch src-tauri/migrations/${MIGRATION_NAME}
```

#### 1.2 编写迁移SQL

```sql
-- migrations/YYYYMMDDHHMMSS_modify_table_fields.sql

-- 场景1: 添加新字段
ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 1;
ALTER TABLE tasks ADD COLUMN tags TEXT; -- JSON array
ALTER TABLE tasks ADD COLUMN external_url TEXT;

-- 场景2: 修改字段类型
-- SQLite不支持直接修改字段类型，需要重建表
BEGIN TRANSACTION;

-- 创建新表结构
CREATE TABLE tasks_new (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    -- 修改字段类型
    estimated_duration REAL, -- 从INTEGER改为REAL
    -- 其他现有字段...
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE
);

-- 迁移数据
INSERT INTO tasks_new (
    id, title, estimated_duration, -- 其他字段...
)
SELECT
    id, title,
    CAST(estimated_duration AS REAL), -- 类型转换
    -- 其他字段...
FROM tasks;

-- 删除旧表并重命名
DROP TABLE tasks;
ALTER TABLE tasks_new RENAME TO tasks;

-- 重建索引
CREATE INDEX idx_tasks_updated_at ON tasks(updated_at);
CREATE INDEX idx_tasks_is_deleted ON tasks(is_deleted);
-- 其他索引...

COMMIT;

-- 场景3: 删除字段（谨慎操作）
-- 同样需要重建表的方式

-- 场景4: 添加约束
ALTER TABLE tasks ADD CONSTRAINT chk_priority
    CHECK (priority >= 1 AND priority <= 5);

-- 场景5: 添加索引
CREATE INDEX idx_tasks_priority ON tasks(priority);
CREATE INDEX idx_tasks_external_url ON tasks(external_url);
```

### Step 2: 更新数据模型

#### 2.1 修改核心模型结构

```rust
// src/core/models/task.rs

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,

    // 修改字段类型
    pub estimated_duration: Option<f64>, // 从i32改为f64

    // 新增字段
    pub priority: i32, // 新增优先级字段
    pub tags: Option<Vec<String>>, // 新增标签字段
    pub external_url: Option<String>, // 新增外部链接字段

    pub subtasks: Option<Vec<Subtask>>,
    pub project_id: Option<Uuid>,
    pub area_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub due_date_type: Option<DueDateType>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,

    // 现有的其他字段...
}

impl Task {
    pub fn new(
        id: Uuid,
        title: String,
        glance_note: Option<String>,
        detail_note: Option<String>,
        estimated_duration: Option<f64>, // 更新参数类型
        priority: Option<i32>, // 新增参数
        tags: Option<Vec<String>>, // 新增参数
        external_url: Option<String>, // 新增参数
        subtasks: Option<Vec<Subtask>>,
        project_id: Option<Uuid>,
        area_id: Option<Uuid>,
        due_date: Option<DateTime<Utc>>,
        due_date_type: Option<DueDateType>,
        now: DateTime<Utc>,
    ) -> Result<Self, String> {
        // 验证标题
        if title.trim().is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if title.len() > 255 {
            return Err("Title too long".to_string());
        }

        // 验证优先级
        let priority = priority.unwrap_or(1);
        if priority < 1 || priority > 5 {
            return Err("Priority must be between 1 and 5".to_string());
        }

        // 验证预估时长
        if let Some(duration) = estimated_duration {
            if duration < 0.0 {
                return Err("Estimated duration cannot be negative".to_string());
            }
        }

        // 验证外部URL
        if let Some(ref url) = external_url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("External URL must be a valid HTTP(S) URL".to_string());
            }
        }

        Ok(Self {
            id,
            title,
            glance_note,
            detail_note,
            estimated_duration,
            priority,
            tags,
            external_url,
            subtasks,
            project_id,
            area_id,
            due_date,
            due_date_type,
            completed_at: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
            // 其他字段的默认值...
        })
    }

    // 新增字段的更新方法
    pub fn update_priority(&mut self, new_priority: i32, now: DateTime<Utc>) -> Result<(), String> {
        if new_priority < 1 || new_priority > 5 {
            return Err("Priority must be between 1 and 5".to_string());
        }

        self.priority = new_priority;
        self.updated_at = now;
        Ok(())
    }

    pub fn update_tags(&mut self, new_tags: Option<Vec<String>>, now: DateTime<Utc>) {
        self.tags = new_tags;
        self.updated_at = now;
    }

    pub fn update_external_url(&mut self, new_url: Option<String>, now: DateTime<Utc>) -> Result<(), String> {
        if let Some(ref url) = new_url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                return Err("External URL must be a valid HTTP(S) URL".to_string());
            }
        }

        self.external_url = new_url;
        self.updated_at = now;
        Ok(())
    }
}
```

### Step 3: 更新仓库层

#### 3.1 更新仓库接口

```rust
// src/repositories/task_repository.rs

#[async_trait]
pub trait TaskRepository: Send + Sync {
    // 现有方法...

    /// 根据优先级查找任务
    async fn find_by_priority(&self, priority: i32) -> Result<Vec<Task>, DbError>;

    /// 根据标签查找任务
    async fn find_by_tags(&self, tags: &[String]) -> Result<Vec<Task>, DbError>;

    /// 根据外部URL查找任务
    async fn find_by_external_url(&self, url: &str) -> Result<Option<Task>, DbError>;

    /// 统计各优先级的任务数量
    async fn count_by_priority(&self) -> Result<Vec<PriorityCount>, DbError>;
}

/// 优先级统计结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct PriorityCount {
    pub priority: i32,
    pub count: i64,
}
```

#### 3.2 更新SQLx实现

```rust
// src/repositories/sqlx_task_repository.rs

impl TaskRepository for SqlxTaskRepository {
    async fn create(&self, tx: &mut Transaction<'_>, task: &Task) -> Result<Task, DbError> {
        let query = r#"
            INSERT INTO tasks (
                id, title, glance_note, detail_note, estimated_duration,
                priority, tags, external_url, -- 新增字段
                subtasks, project_id, area_id, due_date, due_date_type,
                completed_at, created_at, updated_at, is_deleted,
                source_info, external_source_id, external_source_provider,
                external_source_metadata, recurrence_rule, recurrence_parent_id,
                recurrence_original_date, recurrence_exclusions
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25
            )
            RETURNING *
        "#;

        let row = sqlx::query(query)
            .bind(task.id.to_string())
            .bind(&task.title)
            .bind(&task.glance_note)
            .bind(&task.detail_note)
            .bind(task.estimated_duration) // 类型已更新
            .bind(task.priority) // 新字段
            .bind(task.tags.as_ref().map(|tags| serde_json::to_string(tags).unwrap())) // 新字段
            .bind(&task.external_url) // 新字段
            .bind(task.subtasks.as_ref().map(|st| serde_json::to_string(st).unwrap()))
            .bind(task.project_id.map(|id| id.to_string()))
            .bind(task.area_id.map(|id| id.to_string()))
            .bind(task.due_date.map(|dt| dt.to_rfc3339()))
            .bind(task.due_date_type.as_ref().map(|dt| serde_json::to_string(dt).unwrap()))
            .bind(task.completed_at.map(|dt| dt.to_rfc3339()))
            .bind(task.created_at.to_rfc3339())
            .bind(task.updated_at.to_rfc3339())
            .bind(task.is_deleted)
            .bind(task.source_info.as_ref().map(|si| serde_json::to_string(si).unwrap()))
            .bind(&task.external_source_id)
            .bind(&task.external_source_provider)
            .bind(task.external_source_metadata.as_ref().map(|esm| serde_json::to_string(esm).unwrap()))
            .bind(&task.recurrence_rule)
            .bind(task.recurrence_parent_id.map(|id| id.to_string()))
            .bind(task.recurrence_original_date.map(|dt| dt.to_rfc3339()))
            .bind(task.recurrence_exclusions.as_ref().map(|re| serde_json::to_string(re).unwrap()))
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::SqlxError)?;

        Ok(row_to_task(row)?)
    }

    // 新增的查询方法
    async fn find_by_priority(&self, priority: i32) -> Result<Vec<Task>, DbError> {
        let query = r#"
            SELECT * FROM tasks
            WHERE priority = ?1 AND is_deleted = FALSE
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(query)
            .bind(priority)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(DbError::SqlxError)?;

        let tasks = rows
            .into_iter()
            .map(row_to_task)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    async fn find_by_tags(&self, tags: &[String]) -> Result<Vec<Task>, DbError> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        // 构建JSON查询条件
        let tag_conditions = tags
            .iter()
            .map(|tag| format!("JSON_EXTRACT(tags, '$') LIKE '%{}%'", tag))
            .collect::<Vec<_>>()
            .join(" OR ");

        let query = format!(
            r#"
            SELECT * FROM tasks
            WHERE ({}) AND is_deleted = FALSE
            ORDER BY created_at DESC
            "#,
            tag_conditions
        );

        let rows = sqlx::query(&query)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(DbError::SqlxError)?;

        let tasks = rows
            .into_iter()
            .map(row_to_task)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    async fn count_by_priority(&self) -> Result<Vec<PriorityCount>, DbError> {
        let query = r#"
            SELECT priority, COUNT(*) as count
            FROM tasks
            WHERE is_deleted = FALSE
            GROUP BY priority
            ORDER BY priority
        "#;

        let rows = sqlx::query(query)
            .fetch_all(self.pool.as_ref())
            .await
            .map_err(DbError::SqlxError)?;

        let counts = rows
            .into_iter()
            .map(|row| {
                Ok(PriorityCount {
                    priority: row.try_get::<i32, _>("priority")?,
                    count: row.try_get::<i64, _>("count")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()
            .map_err(DbError::SqlxError)?;

        Ok(counts)
    }
}

// 更新行转换函数
fn row_to_task(row: sqlx::sqlite::SqliteRow) -> Result<Task, DbError> {
    use sqlx::Row;

    let id_str: String = row.try_get("id")?;
    let id = Uuid::parse_str(&id_str).map_err(|e| DbError::DataConversion(e.to_string()))?;

    // 解析新字段
    let priority: i32 = row.try_get("priority").unwrap_or(1);

    let tags: Option<Vec<String>> = row
        .try_get::<Option<String>, _>("tags")?
        .map(|json_str| serde_json::from_str(&json_str))
        .transpose()
        .map_err(|e| DbError::DataConversion(e.to_string()))?;

    let external_url: Option<String> = row.try_get("external_url")?;

    // 解析修改类型的字段
    let estimated_duration: Option<f64> = row.try_get("estimated_duration")?;

    Ok(Task {
        id,
        title: row.try_get("title")?,
        glance_note: row.try_get("glance_note")?,
        detail_note: row.try_get("detail_note")?,
        estimated_duration,
        priority,
        tags,
        external_url,
        // 其他现有字段...
        subtasks: row
            .try_get::<Option<String>, _>("subtasks")?
            .map(|json_str| serde_json::from_str(&json_str))
            .transpose()
            .map_err(|e| DbError::DataConversion(e.to_string()))?,
        // 继续其他字段...
        created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
            .map_err(|e| DbError::DataConversion(e.to_string()))?
            .with_timezone(&Utc),
        updated_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
            .map_err(|e| DbError::DataConversion(e.to_string()))?
            .with_timezone(&Utc),
        is_deleted: row.try_get("is_deleted")?,
    })
}
```

#### 3.3 更新内存仓库实现

```rust
// src/repositories/memory_repositories.rs

impl TaskRepository for MemoryRepositories {
    async fn find_by_priority(&self, priority: i32) -> Result<Vec<Task>, DbError> {
        let tasks = self.tasks.lock().await;
        let filtered_tasks = tasks
            .values()
            .filter(|task| !task.is_deleted && task.priority == priority)
            .cloned()
            .collect();
        Ok(filtered_tasks)
    }

    async fn find_by_tags(&self, tags: &[String]) -> Result<Vec<Task>, DbError> {
        let tasks = self.tasks.lock().await;
        let filtered_tasks = tasks
            .values()
            .filter(|task| {
                !task.is_deleted &&
                task.tags.as_ref().map_or(false, |task_tags| {
                    tags.iter().any(|tag| task_tags.contains(tag))
                })
            })
            .cloned()
            .collect();
        Ok(filtered_tasks)
    }

    async fn count_by_priority(&self) -> Result<Vec<PriorityCount>, DbError> {
        let tasks = self.tasks.lock().await;
        let mut priority_counts = std::collections::HashMap::new();

        for task in tasks.values() {
            if !task.is_deleted {
                *priority_counts.entry(task.priority).or_insert(0) += 1;
            }
        }

        let mut counts: Vec<PriorityCount> = priority_counts
            .into_iter()
            .map(|(priority, count)| PriorityCount { priority, count })
            .collect();

        counts.sort_by_key(|pc| pc.priority);
        Ok(counts)
    }
}
```

### Step 4: 更新服务层

#### 4.1 更新DTO结构

```rust
// src/services/dtos.rs

#[derive(Debug, Clone)]
pub struct CreateTaskData {
    pub title: String,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,
    pub estimated_duration: Option<f64>, // 类型更新
    pub priority: Option<i32>, // 新增字段
    pub tags: Option<Vec<String>>, // 新增字段
    pub external_url: Option<String>, // 新增字段
    pub subtasks: Option<Vec<Subtask>>,
    pub area_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub due_date_type: Option<DueDateType>,
}

impl CreateTaskData {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 现有验证...

        // 新增字段验证
        if let Some(priority) = self.priority {
            if priority < 1 || priority > 5 {
                errors.push(ValidationError::new(
                    "priority",
                    "Priority must be between 1 and 5",
                    "INVALID_PRIORITY"
                ));
            }
        }

        if let Some(ref url) = self.external_url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                errors.push(ValidationError::new(
                    "external_url",
                    "External URL must be a valid HTTP(S) URL",
                    "INVALID_URL"
                ));
            }
        }

        if let Some(ref tags) = self.tags {
            if tags.len() > 20 {
                errors.push(ValidationError::new(
                    "tags",
                    "Too many tags (maximum 20)",
                    "TOO_MANY_TAGS"
                ));
            }

            for tag in tags {
                if tag.len() > 50 {
                    errors.push(ValidationError::new(
                        "tags",
                        "Tag name too long (maximum 50 characters)",
                        "TAG_TOO_LONG"
                    ));
                    break;
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateTaskData {
    pub title: Option<String>,
    pub glance_note: Option<Option<String>>,
    pub detail_note: Option<Option<String>>,
    pub estimated_duration: Option<Option<f64>>, // 类型更新
    pub priority: Option<i32>, // 新增字段
    pub tags: Option<Option<Vec<String>>>, // 新增字段
    pub external_url: Option<Option<String>>, // 新增字段
    pub subtasks: Option<Option<Vec<Subtask>>>,
    pub project_id: Option<Option<Uuid>>,
    pub area_id: Option<Option<Uuid>>,
    pub due_date: Option<Option<DateTime<Utc>>>,
    pub due_date_type: Option<Option<DueDateType>>,
}
```

#### 4.2 更新服务实现

```rust
// src/services/task_service.rs

impl TaskService {
    pub async fn create_in_context(&self, data: CreateTaskData, context: &CreationContext) -> AppResult<Task> {
        // 1. 验证输入
        data.validate().map_err(AppError::ValidationFailed)?;

        // 2. 生成核心属性
        let new_task_id = self.id_generator.new_uuid();
        let now = self.clock.now_utc();

        // 3. 构建Task对象（包含新字段）
        let new_task = Task::new(
            new_task_id,
            data.title,
            data.glance_note,
            data.detail_note,
            data.estimated_duration,
            data.priority, // 新字段
            data.tags, // 新字段
            data.external_url, // 新字段
            data.subtasks,
            None, // project_id 根据context设置
            data.area_id,
            data.due_date,
            data.due_date_type,
            now,
        ).map_err(AppError::StringError)?;

        // 4. 启动事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 5. 处理上下文（现有逻辑）
        // ...

        // 6. 持久化Task
        let created_task = self.task_repository.create(&mut tx, &new_task).await?;

        // 7. 处理后续安排（现有逻辑）
        // ...

        // 8. 提交事务
        tx.commit().await?;

        Ok(created_task)
    }

    // 新增的服务方法
    /// 根据优先级获取任务
    pub async fn get_tasks_by_priority(&self, priority: i32) -> AppResult<Vec<Task>> {
        if priority < 1 || priority > 5 {
            return Err(AppError::validation_error("priority", "Invalid priority", "INVALID_PRIORITY"));
        }

        let tasks = self.task_repository.find_by_priority(priority).await?;
        Ok(tasks)
    }

    /// 根据标签搜索任务
    pub async fn search_tasks_by_tags(&self, tags: Vec<String>) -> AppResult<Vec<Task>> {
        if tags.is_empty() {
            return Ok(Vec::new());
        }

        let tasks = self.task_repository.find_by_tags(&tags).await?;
        Ok(tasks)
    }

    /// 更新任务优先级
    pub async fn update_task_priority(&self, task_id: Uuid, new_priority: i32) -> AppResult<Task> {
        if new_priority < 1 || new_priority > 5 {
            return Err(AppError::validation_error("priority", "Invalid priority", "INVALID_PRIORITY"));
        }

        let mut tx = self.task_repository.begin_transaction().await?;

        let mut task = self.task_repository
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let now = self.clock.now_utc();
        task.update_priority(new_priority, now).map_err(AppError::StringError)?;

        let updated_task = self.task_repository.update(&mut tx, &task).await?;

        tx.commit().await?;

        Ok(updated_task)
    }
}
```

### Step 5: 更新网络层

#### 5.1 更新HTTP载荷

```rust
// src/handlers/payloads.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskPayload {
    pub title: String,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,
    pub estimated_duration: Option<f64>, // 类型更新
    pub priority: Option<i32>, // 新增字段
    pub tags: Option<Vec<String>>, // 新增字段
    pub external_url: Option<String>, // 新增字段
    pub subtasks: Option<Vec<Subtask>>,
    pub area_id: Option<Uuid>,
    pub due_date: Option<DateTime<Utc>>,
    pub due_date_type: Option<DueDateType>,
    pub context: CreationContextPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskPayload {
    pub title: Option<String>,
    pub glance_note: Option<Option<String>>,
    pub detail_note: Option<Option<String>>,
    pub estimated_duration: Option<Option<f64>>, // 类型更新
    pub priority: Option<i32>, // 新增字段
    pub tags: Option<Option<Vec<String>>>, // 新增字段
    pub external_url: Option<Option<String>>, // 新增字段
    pub subtasks: Option<Option<Vec<Subtask>>>,
    pub project_id: Option<Option<Uuid>>,
    pub area_id: Option<Option<Uuid>>,
    pub due_date: Option<Option<DateTime<Utc>>>,
    pub due_date_type: Option<Option<DueDateType>>,
}

// 新增的查询载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskPriorityPayload {
    pub priority: i32,
}

#[derive(Debug, Deserialize)]
pub struct SearchTasksByTagsQuery {
    pub tags: String, // 逗号分隔的标签列表
}

#[derive(Debug, Deserialize)]
pub struct GetTasksByPriorityQuery {
    pub priority: i32,
}

// 更新From实现
impl From<CreateTaskPayload> for crate::services::CreateTaskData {
    fn from(payload: CreateTaskPayload) -> Self {
        Self {
            title: payload.title,
            glance_note: payload.glance_note,
            detail_note: payload.detail_note,
            estimated_duration: payload.estimated_duration,
            priority: payload.priority,
            tags: payload.tags,
            external_url: payload.external_url,
            subtasks: payload.subtasks,
            area_id: payload.area_id,
            due_date: payload.due_date,
            due_date_type: payload.due_date_type,
        }
    }
}
```

#### 5.2 更新HTTP处理器

```rust
// src/handlers/task_handlers.rs

// 新增的处理器
/// 根据优先级获取任务处理器
///
/// **端点**: `GET /tasks/by-priority`
pub async fn get_tasks_by_priority_handler(
    State(app_state): State<AppState>,
    Query(query): Query<GetTasksByPriorityQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting tasks by priority: {}", query.priority);

    let tasks = app_state.task_service.get_tasks_by_priority(query.priority).await?;

    Ok(success_response(tasks))
}

/// 根据标签搜索任务处理器
///
/// **端点**: `GET /tasks/by-tags`
pub async fn search_tasks_by_tags_handler(
    State(app_state): State<AppState>,
    Query(query): Query<SearchTasksByTagsQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Searching tasks by tags: {}", query.tags);

    let tags: Vec<String> = query.tags
        .split(',')
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect();

    let tasks = app_state.task_service.search_tasks_by_tags(tags).await?;

    Ok(success_response(tasks))
}

/// 更新任务优先级处理器
///
/// **端点**: `PUT /tasks/{id}/priority`
pub async fn update_task_priority_handler(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(payload): Json<UpdateTaskPriorityPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Updating task {} priority to {}", task_id, payload.priority);

    let updated_task = app_state.task_service
        .update_task_priority(task_id, payload.priority)
        .await?;

    log::info!("Task priority updated successfully: {}", task_id);

    Ok(success_response(updated_task))
}
```

#### 5.3 更新路由配置

```rust
// src/routes/task_routes.rs

pub fn create_task_routes() -> Router<AppState> {
    Router::new()
        // 现有路由...
        .route("/tasks", post(create_task_handler))
        .route("/tasks/search", get(search_tasks_handler))
        .route("/tasks/unscheduled", get(get_unscheduled_tasks_handler))
        .route("/tasks/stats", get(get_task_stats_handler))

        // 新增路由
        .route("/tasks/by-priority", get(get_tasks_by_priority_handler))
        .route("/tasks/by-tags", get(search_tasks_by_tags_handler))
        .route("/tasks/:id/priority", put(update_task_priority_handler))

        // 其他现有路由...
}
```

### Step 6: 更新测试

#### 6.1 单元测试更新

```rust
// src/core/models/task.rs 测试部分

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation_with_new_fields() {
        let now = Utc::now();
        let task = Task::new(
            Uuid::new_v4(),
            "Test Task".to_string(),
            None,
            None,
            Some(60.5), // 测试浮点数时长
            Some(3), // 测试优先级
            Some(vec!["work".to_string(), "urgent".to_string()]), // 测试标签
            Some("https://example.com".to_string()), // 测试外部URL
            None,
            None,
            None,
            None,
            None,
            now,
        ).unwrap();

        assert_eq!(task.estimated_duration, Some(60.5));
        assert_eq!(task.priority, 3);
        assert_eq!(task.tags, Some(vec!["work".to_string(), "urgent".to_string()]));
        assert_eq!(task.external_url, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_priority_validation() {
        let now = Utc::now();

        // 测试无效优先级
        let result = Task::new(
            Uuid::new_v4(),
            "Test".to_string(),
            None, None, None,
            Some(6), // 无效优先级
            None, None, None, None, None, None, None, now,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Priority must be between 1 and 5"));
    }

    #[test]
    fn test_url_validation() {
        let now = Utc::now();

        // 测试无效URL
        let result = Task::new(
            Uuid::new_v4(),
            "Test".to_string(),
            None, None, None, None, None,
            Some("invalid-url".to_string()), // 无效URL
            None, None, None, None, None, now,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be a valid HTTP(S) URL"));
    }
}
```

#### 6.2 服务层测试更新

```rust
// src/services/task_service.rs 测试部分

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_task_with_priority() {
        let service = create_test_task_service().await;

        let data = CreateTaskData {
            title: "High Priority Task".to_string(),
            priority: Some(5),
            tags: Some(vec!["urgent".to_string()]),
            external_url: Some("https://example.com".to_string()),
            // 其他字段...
        };

        let context = CreationContext {
            context_type: ContextType::Misc,
            context_id: "floating".to_string(),
        };

        let result = service.create_in_context(data, &context).await;
        assert!(result.is_ok());

        let task = result.unwrap();
        assert_eq!(task.priority, 5);
        assert_eq!(task.tags, Some(vec!["urgent".to_string()]));
    }

    #[tokio::test]
    async fn test_get_tasks_by_priority() {
        let service = create_test_task_service().await;

        // 创建不同优先级的任务
        // ...

        let high_priority_tasks = service.get_tasks_by_priority(5).await.unwrap();
        assert!(high_priority_tasks.iter().all(|t| t.priority == 5));
    }

    #[tokio::test]
    async fn test_search_tasks_by_tags() {
        let service = create_test_task_service().await;

        // 创建带标签的任务
        // ...

        let tagged_tasks = service.search_tasks_by_tags(vec!["work".to_string()]).await.unwrap();
        assert!(tagged_tasks.iter().all(|t| {
            t.tags.as_ref().map_or(false, |tags| tags.contains(&"work".to_string()))
        }));
    }
}
```

#### 6.3 HTTP处理器测试更新

```rust
// src/handlers/task_handlers.rs 测试部分

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_payload_with_new_fields() {
        let payload = CreateTaskPayload {
            title: "Test Task".to_string(),
            estimated_duration: Some(90.5),
            priority: Some(4),
            tags: Some(vec!["test".to_string(), "demo".to_string()]),
            external_url: Some("https://test.example.com".to_string()),
            // 其他字段...
            context: CreationContextPayload {
                context_type: ContextType::Misc,
                context_id: "floating".to_string(),
            },
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("90.5"));
        assert!(json.contains("\"priority\":4"));
        assert!(json.contains("test"));
        assert!(json.contains("https://test.example.com"));

        let deserialized: CreateTaskPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.estimated_duration, Some(90.5));
        assert_eq!(deserialized.priority, Some(4));
    }

    #[test]
    fn test_search_by_tags_query_parsing() {
        let query_str = "tags=work,urgent,important";
        let query: SearchTasksByTagsQuery = serde_urlencoded::from_str(query_str).unwrap();

        assert_eq!(query.tags, "work,urgent,important");
    }
}
```

## 常见字段修改场景

### 场景1: 添加枚举字段

#### 数据库迁移

```sql
-- 添加状态字段
ALTER TABLE tasks ADD COLUMN status TEXT DEFAULT 'ACTIVE'
    CHECK (status IN ('ACTIVE', 'PAUSED', 'CANCELLED'));
CREATE INDEX idx_tasks_status ON tasks(status);
```

#### 数据模型更新

```rust
// src/core/models/enums.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Active,
    Paused,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Active => "ACTIVE",
            TaskStatus::Paused => "PAUSED",
            TaskStatus::Cancelled => "CANCELLED",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "ACTIVE" => Ok(TaskStatus::Active),
            "PAUSED" => Ok(TaskStatus::Paused),
            "CANCELLED" => Ok(TaskStatus::Cancelled),
            _ => Err(format!("Invalid task status: {}", s)),
        }
    }
}

// src/core/models/task.rs
pub struct Task {
    // 现有字段...
    pub status: TaskStatus, // 新增状态字段
}
```

### 场景2: 添加关联字段

#### 数据库迁移

```sql
-- 添加关联字段
ALTER TABLE tasks ADD COLUMN assigned_user_id TEXT;
ALTER TABLE tasks ADD COLUMN team_id TEXT;

-- 添加外键约束（如果相关表存在）
-- ALTER TABLE tasks ADD CONSTRAINT fk_tasks_assigned_user
--     FOREIGN KEY (assigned_user_id) REFERENCES users(id);

-- 添加索引
CREATE INDEX idx_tasks_assigned_user_id ON tasks(assigned_user_id);
CREATE INDEX idx_tasks_team_id ON tasks(team_id);
```

#### 数据模型更新

```rust
// src/core/models/task.rs
pub struct Task {
    // 现有字段...
    pub assigned_user_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
}

impl Task {
    pub fn assign_to_user(&mut self, user_id: Uuid, now: DateTime<Utc>) {
        self.assigned_user_id = Some(user_id);
        self.updated_at = now;
    }

    pub fn unassign(&mut self, now: DateTime<Utc>) {
        self.assigned_user_id = None;
        self.updated_at = now;
    }
}
```

### 场景3: 修改字段约束

#### 数据库迁移

```sql
-- 修改字段约束需要重建表
BEGIN TRANSACTION;

-- 创建新表结构
CREATE TABLE tasks_new (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    -- 修改约束
    estimated_duration REAL CHECK (estimated_duration > 0), -- 添加正数约束
    priority INTEGER DEFAULT 1 CHECK (priority BETWEEN 1 AND 10), -- 扩展优先级范围
    -- 其他字段...
);

-- 迁移数据（可能需要数据清理）
INSERT INTO tasks_new SELECT
    id, title,
    CASE
        WHEN estimated_duration <= 0 THEN 1.0
        ELSE estimated_duration
    END as estimated_duration,
    CASE
        WHEN priority > 5 THEN 5
        WHEN priority < 1 THEN 1
        ELSE priority
    END as priority,
    -- 其他字段...
FROM tasks;

-- 替换表
DROP TABLE tasks;
ALTER TABLE tasks_new RENAME TO tasks;

-- 重建索引
CREATE INDEX idx_tasks_updated_at ON tasks(updated_at);
-- 其他索引...

COMMIT;
```

## 数据迁移策略

### 简单数据迁移

```sql
-- 添加字段并设置默认值
ALTER TABLE tasks ADD COLUMN priority INTEGER DEFAULT 1;

-- 基于现有数据计算新字段值
UPDATE tasks SET priority =
    CASE
        WHEN due_date IS NOT NULL AND due_date < datetime('now', '+1 day') THEN 5
        WHEN due_date IS NOT NULL AND due_date < datetime('now', '+3 days') THEN 4
        WHEN due_date IS NOT NULL AND due_date < datetime('now', '+1 week') THEN 3
        WHEN due_date IS NOT NULL THEN 2
        ELSE 1
    END;
```

### 复杂数据迁移

```sql
-- 数据清理和转换
UPDATE tasks SET tags = '[]' WHERE tags IS NULL;

UPDATE tasks SET estimated_duration =
    CASE
        WHEN estimated_duration < 0 THEN 0
        WHEN estimated_duration > 480 THEN 480 -- 最大8小时
        ELSE estimated_duration
    END;

-- 数据验证
SELECT COUNT(*) FROM tasks WHERE priority NOT BETWEEN 1 AND 5;
-- 应该返回0
```

### 数据备份策略

```sql
-- 在重大变更前创建备份表
CREATE TABLE tasks_backup_YYYYMMDD AS SELECT * FROM tasks;

-- 验证备份
SELECT COUNT(*) FROM tasks;
SELECT COUNT(*) FROM tasks_backup_YYYYMMDD;
-- 两个数字应该相等
```

## 回滚策略

### 迁移回滚

```sql
-- 回滚迁移文件模板
-- migrations/YYYYMMDDHHMMSS_rollback_modify_table_fields.sql

-- 回滚场景1: 删除新增字段
ALTER TABLE tasks DROP COLUMN priority;
ALTER TABLE tasks DROP COLUMN tags;
ALTER TABLE tasks DROP COLUMN external_url;

-- 回滚场景2: 恢复字段类型
-- 需要重建表的逆向操作

-- 回滚场景3: 从备份恢复
DROP TABLE tasks;
ALTER TABLE tasks_backup_YYYYMMDD RENAME TO tasks;
```

### 代码回滚

```bash
# Git回滚到迁移前的状态
git log --oneline | grep "before migration"
git reset --hard <commit_hash>

# 或者使用分支策略
git checkout main
git branch -D feature/add-new-fields
```

## 性能影响评估

### 查询性能测试

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[tokio::test]
    async fn test_query_performance_with_new_fields() {
        let repository = create_test_repository().await;

        // 插入测试数据
        for i in 0..1000 {
            let task = create_test_task_with_priority(i % 5 + 1);
            repository.create(&mut tx, &task).await.unwrap();
        }

        // 测试查询性能
        let start = Instant::now();
        let results = repository.find_by_priority(5).await.unwrap();
        let duration = start.elapsed();

        assert!(duration < std::time::Duration::from_millis(100));
        assert!(!results.is_empty());
    }
}
```

### 索引效果验证

```sql
-- 查询执行计划
EXPLAIN QUERY PLAN
SELECT * FROM tasks
WHERE priority = 5 AND is_deleted = FALSE;

-- 应该显示使用了索引
-- SCAN TABLE tasks USING INDEX idx_tasks_priority
```

## 监控和告警

### 迁移监控

```rust
// 添加迁移监控
impl DatabaseManager {
    pub async fn run_migration_with_monitoring(&self, migration: &str) -> Result<(), DbError> {
        let start_time = Instant::now();

        log::info!("Starting migration: {}", migration);

        let result = self.run_migration(migration).await;

        let duration = start_time.elapsed();
        log::info!("Migration completed in {:?}: {}", duration, migration);

        if duration > Duration::from_secs(30) {
            log::warn!("Slow migration detected: {} took {:?}", migration, duration);
        }

        result
    }
}
```

### 字段使用监控

```rust
// 监控新字段的使用情况
impl TaskService {
    pub async fn get_field_usage_stats(&self) -> AppResult<FieldUsageStats> {
        let stats = self.task_repository.get_field_usage_stats().await?;

        log::info!("Field usage stats: {:?}", stats);

        Ok(stats)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct FieldUsageStats {
    pub total_tasks: i64,
    pub tasks_with_priority: i64,
    pub tasks_with_tags: i64,
    pub tasks_with_external_url: i64,
    pub priority_distribution: Vec<PriorityCount>,
    pub most_common_tags: Vec<TagUsage>,
}
```

## 最佳实践总结

### 1. 字段添加最佳实践

- **使用Optional类型**: 新字段应该是可选的
- **提供默认值**: 数据库层面提供合理默认值
- **渐进式部署**: 先添加字段，再添加业务逻辑

### 2. 字段修改最佳实践

- **保持兼容性**: 尽量保持向后兼容
- **数据验证**: 修改前验证数据完整性
- **分步执行**: 复杂修改分多个步骤执行

### 3. 字段删除最佳实践

- **废弃标记**: 先标记字段为废弃
- **观察期**: 观察一段时间确保无影响
- **最终删除**: 确认安全后再物理删除

### 4. 测试最佳实践

- **全面测试**: 测试所有受影响的功能
- **性能测试**: 验证性能不会退化
- **边界测试**: 测试所有边界情况

通过遵循本指南，可以安全、高效地修改Cutie后端的数据库字段，同时保持系统的稳定性和一致性。
