/// 更新模板 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `update_template`

## 1. 端点签名
PATCH /api/templates/:id

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要更新现有模板的信息,以便调整模板内容

### 2.2 核心业务逻辑
根据模板ID更新模板的各项字段

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "title": "string (optional)",
  "glance_note_template": "string (optional)",
  "detail_note_template": "string (optional)",
  "estimated_duration_template": "integer (optional)",
  "subtasks_template": "array (optional)",
  "area_id": "uuid (optional)",
  "category": "GENERAL | RECURRENCE (optional)"
}

### 3.2 响应 (Responses)
**200 OK:**
{
  "id": "uuid",
  "title": "string",
  ...
}

**404 Not Found:**
模板不存在

## 4. 验证规则
- title: 如果提供,不能为空
- estimated_duration_template: 如果提供,必须 > 0

## 5. 业务逻辑详解
1. 验证输入
2. 开启事务
3. 查询模板是否存在
4. 更新模板字段
5. 提交事务
6. 组装 DTO
7. 返回结果

## 6. 边界情况
- 模板不存在: 返回 404
- title 为空: 返回 422

## 7. 预期副作用
### 数据库操作:
- SELECT: 查询 templates 表
- UPDATE: 更新 templates 表
- 事务边界: begin() → commit()

### SSE 事件:
- template.updated

## 8. 契约
### 前置条件:
- 模板存在且未删除

### 后置条件:
- 模板字段已更新
- 返回完整的 TemplateDto

### 不变量:
- id 和 created_at 永不改变
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    entities::template::{Template, TemplateDto, TemplateRow, UpdateTemplateRequest},
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTemplateRequest>,
) -> Response {
    match logic::execute(&app_state, id, request).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &UpdateTemplateRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        // 验证 title
        if let Some(title) = &request.title {
            if title.trim().is_empty() {
                errors.push(ValidationError {
                    field: "title".to_string(),
                    code: "TITLE_EMPTY".to_string(),
                    message: "title cannot be empty".to_string(),
                });
            }
            if title.len() > 255 {
                errors.push(ValidationError {
                    field: "title".to_string(),
                    code: "TITLE_TOO_LONG".to_string(),
                    message: "title too long (max 255 characters)".to_string(),
                });
            }
        }

        // 验证 estimated_duration_template
        if let Some(Some(duration)) = request.estimated_duration_template {
            if duration <= 0 {
                errors.push(ValidationError {
                    field: "estimated_duration_template".to_string(),
                    code: "DURATION_INVALID".to_string(),
                    message: "estimated_duration_template must be positive".to_string(),
                });
            }
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(errors));
        }

        Ok(())
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        id: Uuid,
        request: UpdateTemplateRequest,
    ) -> AppResult<TemplateDto> {
        // 1. 验证
        validation::validate_request(&request)?;

        // 2. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 查询模板
        let template = database::find_by_id_in_tx(&mut tx, id).await?;

        // 4. 获取更新时间
        let now = app_state.clock().now_utc();

        // 5. 更新模板
        database::update_in_tx(&mut tx, id, &request, now).await?;

        // 6. 提交事务
        TransactionHelper::commit(tx).await?;

        // 7. 查询更新后的模板
        let updated = database::find_by_id(app_state.db_pool(), id).await?;

        // 8. 组装 DTO
        let dto = TemplateDto {
            id: updated.id,
            title: updated.title,
            glance_note_template: updated.glance_note_template,
            detail_note_template: updated.detail_note_template,
            estimated_duration_template: updated.estimated_duration_template,
            subtasks_template: updated.subtasks_template,
            area_id: updated.area_id,
            category: updated.category,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
        };

        Ok(dto)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use chrono::{DateTime, Utc};
    use sqlx::{Sqlite, SqlitePool, Transaction};

    pub async fn find_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        id: Uuid,
    ) -> AppResult<Template> {
        let query = r#"
            SELECT
                id, title, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id, category,
                created_at, updated_at, is_deleted
            FROM templates
            WHERE id = ? AND is_deleted = FALSE
        "#;

        let row: TemplateRow = sqlx::query_as(query)
            .bind(id.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "Template".to_string(),
                entity_id: id.to_string(),
            })?;

        Ok(row.try_into().expect("Failed to convert TemplateRow"))
    }

    pub async fn find_by_id(pool: &SqlitePool, id: Uuid) -> AppResult<Template> {
        let query = r#"
            SELECT
                id, title, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id, category,
                created_at, updated_at, is_deleted
            FROM templates
            WHERE id = ? AND is_deleted = FALSE
        "#;

        let row: TemplateRow = sqlx::query_as(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "Template".to_string(),
                entity_id: id.to_string(),
            })?;

        Ok(row.try_into().expect("Failed to convert TemplateRow"))
    }

    pub async fn update_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        id: Uuid,
        request: &UpdateTemplateRequest,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let mut updates = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, Sqlite> + Send>> = Vec::new();

        if request.title.is_some() {
            updates.push("title = ?");
        }
        if request.glance_note_template.is_some() {
            updates.push("glance_note_template = ?");
        }
        if request.detail_note_template.is_some() {
            updates.push("detail_note_template = ?");
        }
        if request.estimated_duration_template.is_some() {
            updates.push("estimated_duration_template = ?");
        }
        if request.subtasks_template.is_some() {
            updates.push("subtasks_template = ?");
        }
        if request.area_id.is_some() {
            updates.push("area_id = ?");
        }
        if request.category.is_some() {
            updates.push("category = ?");
        }

        if updates.is_empty() {
            return Ok(());
        }

        updates.push("updated_at = ?");

        let query = format!("UPDATE templates SET {} WHERE id = ?", updates.join(", "));

        let mut q = sqlx::query(&query);

        if let Some(ref title) = request.title {
            q = q.bind(title);
        }
        if let Some(ref glance_note_opt) = request.glance_note_template {
            q = q.bind(glance_note_opt.as_ref());
        }
        if let Some(ref detail_note_opt) = request.detail_note_template {
            q = q.bind(detail_note_opt.as_ref());
        }
        if let Some(duration_opt) = request.estimated_duration_template {
            q = q.bind(duration_opt);
        }
        if let Some(ref subtasks_opt) = request.subtasks_template {
            let subtasks_json = subtasks_opt
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap());
            q = q.bind(subtasks_json);
        }
        if let Some(area_id_opt) = request.area_id {
            q = q.bind(area_id_opt.map(|id| id.to_string()));
        }
        if let Some(ref category) = request.category {
            q = q.bind(category.as_str());
        }

        q = q.bind(updated_at);
        q = q.bind(id.to_string());

        q.execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
