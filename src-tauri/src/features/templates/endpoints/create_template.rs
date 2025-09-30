/// 创建模板 API - 单文件组件
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::{task::Subtask, Template},
    shared::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `create_template`

## API端点
POST /api/templates

## 预期行为简介
创建一个新的任务模板。

## 输入输出规范
- **前置条件**: 请求体必须包含有效的 `name` 和 `title_template`。
- **后置条件**: 返回 `201 Created` 和新创建的 `Template` 对象。
- **不变量**: 无。

## 边界情况
- name 为空: 返回 `422 Unprocessable Entity`。
- title_template 为空: 返回 `422 Unprocessable Entity`。
- area_id 不存在: 返回 `404 Not Found`。

## 预期副作用
- 在 `templates` 表插入1条记录。
- 所有数据库写入在单个事务中。

## 请求体
```json
{
  "name": "每日站会模板",
  "title_template": "{{date}} 站会",
  "glance_note_template": "讨论昨天、今天和阻碍",
  "detail_note_template": "详细记录...",
  "estimated_duration_template": 15,
  "subtasks_template": [{"title": "讨论昨天", "completed": false}],
  "area_id": "uuid-string"
}
```
*/

#[derive(Deserialize)]
pub struct CreateTemplateRequest {
    name: String,
    title_template: String,
    glance_note_template: Option<String>,
    detail_note_template: Option<String>,
    estimated_duration_template: Option<i32>,
    subtasks_template: Option<Vec<Subtask>>,
    area_id: Option<String>,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTemplateRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(template) => created_response(template).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub struct ValidatedTemplateData {
        pub name: String,
        pub title_template: String,
        pub glance_note_template: Option<String>,
        pub detail_note_template: Option<String>,
        pub estimated_duration_template: Option<i32>,
        pub subtasks_template: Option<Vec<Subtask>>,
        pub area_id: Option<Uuid>,
    }

    pub fn validate_request(
        request: &CreateTemplateRequest,
    ) -> Result<ValidatedTemplateData, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 1. 验证 name
        if request.name.trim().is_empty() {
            errors.push(ValidationError::new(
                "name",
                "模板名称不能为空",
                "NAME_REQUIRED",
            ));
        }

        // 2. 验证 title_template
        if request.title_template.trim().is_empty() {
            errors.push(ValidationError::new(
                "title_template",
                "标题模板不能为空",
                "TITLE_TEMPLATE_REQUIRED",
            ));
        }

        // 3. 验证 area_id
        let area_id = if let Some(ref area_id_str) = request.area_id {
            match Uuid::parse_str(area_id_str) {
                Ok(id) => Some(id),
                Err(_) => {
                    errors.push(ValidationError::new(
                        "area_id",
                        "Area ID 格式无效",
                        "INVALID_AREA_ID",
                    ));
                    None
                }
            }
        } else {
            None
        };

        // 4. 验证 estimated_duration_template
        if let Some(duration) = request.estimated_duration_template {
            if duration < 0 {
                errors.push(ValidationError::new(
                    "estimated_duration_template",
                    "预估时长不能为负数",
                    "INVALID_DURATION",
                ));
            }
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(ValidatedTemplateData {
            name: request.name.trim().to_string(),
            title_template: request.title_template.clone(),
            glance_note_template: request.glance_note_template.clone(),
            detail_note_template: request.detail_note_template.clone(),
            estimated_duration_template: request.estimated_duration_template,
            subtasks_template: request.subtasks_template.clone(),
            area_id,
        })
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateTemplateRequest,
    ) -> AppResult<Template> {
        // 1. 验证请求
        let validated =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 2. 验证 area_id 是否存在（如果提供）
        if let Some(area_id) = validated.area_id {
            let area_exists = database::area_exists_in_tx(&mut tx, area_id).await?;
            if !area_exists {
                return Err(AppError::not_found("Area", area_id.to_string()));
            }
        }

        // 3. 生成 ID 和时间戳
        let new_template_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 4. 核心操作：创建模板
        let mut new_template = Template::new(
            new_template_id,
            validated.name,
            validated.title_template,
            now,
        );

        new_template.glance_note_template = validated.glance_note_template;
        new_template.detail_note_template = validated.detail_note_template;
        new_template.estimated_duration_template = validated.estimated_duration_template;
        new_template.subtasks_template = validated.subtasks_template;
        new_template.area_id = validated.area_id;

        let created_template = database::create_template_in_tx(&mut tx, &new_template).await?;

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(created_template)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::template::TemplateRow;

    pub async fn area_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM areas WHERE id = ? AND is_deleted = false")
                .bind(area_id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
                })?;

        Ok(count > 0)
    }

    pub async fn create_template_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template: &Template,
    ) -> AppResult<Template> {
        let subtasks_json = template
            .subtasks_template
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        let row = sqlx::query_as::<_, TemplateRow>(
            r#"
            INSERT INTO templates (
                id, name, title_template, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id,
                created_at, updated_at, is_deleted
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id, name, title_template, glance_note_template, detail_note_template,
                      estimated_duration_template, subtasks_template, area_id,
                      created_at, updated_at, is_deleted
            "#,
        )
        .bind(template.id.to_string())
        .bind(&template.name)
        .bind(&template.title_template)
        .bind(&template.glance_note_template)
        .bind(&template.detail_note_template)
        .bind(template.estimated_duration_template)
        .bind(subtasks_json)
        .bind(template.area_id.map(|id| id.to_string()))
        .bind(template.created_at.to_rfc3339())
        .bind(template.updated_at.to_rfc3339())
        .bind(template.is_deleted)
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Template::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}
