/// 更新模板 API - 单文件组件
use axum::{
    extract::{Path, State},
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
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `update_template`

## API端点
PATCH /api/templates/{id}

## 预期行为简介
更新一个现有模板的属性。

## 输入输出规范
- **前置条件**: `id` 必须是有效的模板ID。请求体中所有非 `None` 的字段都必须通过验证。
- **后置条件**: 返回 `200 OK` 和更新后的 `Template` 对象。
- **不变量**: 无。

## 边界情况
- 模板不存在: 返回 `404 Not Found`。
- 输入数据验证失败: 返回 `422 Unprocessable Entity`。

## 预期副作用
- 更新 `templates` 表中的1条记录。
- 所有数据库写入在单个事务中。
*/

#[derive(Deserialize)]
pub struct UpdateTemplateRequest {
    name: Option<String>,
    title_template: Option<String>,
    glance_note_template: Option<Option<String>>,
    detail_note_template: Option<Option<String>>,
    estimated_duration_template: Option<Option<i32>>,
    subtasks_template: Option<Option<Vec<Subtask>>>,
    area_id: Option<Option<String>>,
}

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(request): Json<UpdateTemplateRequest>,
) -> Response {
    match logic::execute(&app_state, template_id, request).await {
        Ok(template) => success_response(template).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 (Validation Layer) ====================
mod validation {
    use super::*;

    pub struct ValidatedUpdates {
        pub name: Option<String>,
        pub title_template: Option<String>,
        pub glance_note_template: Option<Option<String>>,
        pub detail_note_template: Option<Option<String>>,
        pub estimated_duration_template: Option<Option<i32>>,
        pub subtasks_template: Option<Option<Vec<Subtask>>>,
        pub area_id: Option<Option<Uuid>>,
    }

    pub fn validate_request(
        request: &UpdateTemplateRequest,
    ) -> Result<ValidatedUpdates, Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 1. 验证 name
        let name = if let Some(ref n) = request.name {
            if n.trim().is_empty() {
                errors.push(ValidationError::new(
                    "name",
                    "模板名称不能为空",
                    "NAME_REQUIRED",
                ));
                None
            } else {
                Some(n.trim().to_string())
            }
        } else {
            None
        };

        // 2. 验证 title_template
        let title_template = if let Some(ref tt) = request.title_template {
            if tt.trim().is_empty() {
                errors.push(ValidationError::new(
                    "title_template",
                    "标题模板不能为空",
                    "TITLE_TEMPLATE_REQUIRED",
                ));
                None
            } else {
                Some(tt.clone())
            }
        } else {
            None
        };

        // 3. 验证 area_id
        let area_id = if let Some(ref maybe_area_id) = request.area_id {
            Some(if let Some(ref area_id_str) = maybe_area_id {
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
            })
        } else {
            None
        };

        // 4. 验证 estimated_duration_template
        if let Some(Some(duration)) = request.estimated_duration_template {
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

        Ok(ValidatedUpdates {
            name,
            title_template,
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
        template_id: Uuid,
        request: UpdateTemplateRequest,
    ) -> AppResult<Template> {
        // 1. 验证请求
        let validated =
            validation::validate_request(&request).map_err(AppError::ValidationFailed)?;

        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 2. 获取现有模板
        let mut template = database::find_template_by_id_in_tx(&mut tx, template_id)
            .await?
            .ok_or_else(|| AppError::not_found("Template", template_id.to_string()))?;

        // 3. 验证 area_id 是否存在（如果要更新）
        if let Some(Some(area_id)) = validated.area_id {
            let area_exists = database::area_exists_in_tx(&mut tx, area_id).await?;
            if !area_exists {
                return Err(AppError::not_found("Area", area_id.to_string()));
            }
        }

        // 4. 应用更新
        let now = app_state.clock().now_utc();

        if let Some(name) = validated.name {
            template.name = name;
        }
        if let Some(title_template) = validated.title_template {
            template.title_template = title_template;
        }
        if let Some(glance_note_template) = validated.glance_note_template {
            template.glance_note_template = glance_note_template;
        }
        if let Some(detail_note_template) = validated.detail_note_template {
            template.detail_note_template = detail_note_template;
        }
        if let Some(estimated_duration_template) = validated.estimated_duration_template {
            template.estimated_duration_template = estimated_duration_template;
        }
        if let Some(subtasks_template) = validated.subtasks_template {
            template.subtasks_template = subtasks_template;
        }
        if let Some(area_id) = validated.area_id {
            template.area_id = area_id;
        }

        template.updated_at = now;

        // 5. 核心操作：持久化更新
        let updated_template = database::update_template_in_tx(&mut tx, &template).await?;

        // 6. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(updated_template)
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::template::TemplateRow;

    pub async fn find_template_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template_id: Uuid,
    ) -> AppResult<Option<Template>> {
        let row = sqlx::query_as::<_, TemplateRow>(
            r#"
            SELECT id, name, title_template, glance_note_template, detail_note_template,
                   estimated_duration_template, subtasks_template, area_id,
                   created_at, updated_at, is_deleted
            FROM templates WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(template_id.to_string())
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        row.map(|r| Template::try_from(r))
            .transpose()
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }

    pub async fn area_exists_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        area_id: Uuid,
    ) -> AppResult<bool> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM areas WHERE id = ? AND deleted_at IS NULL")
                .bind(area_id.to_string())
                .fetch_one(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
                })?;

        Ok(count > 0)
    }

    pub async fn update_template_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template: &Template,
    ) -> AppResult<Template> {
        let subtasks_json = template
            .subtasks_template
            .as_ref()
            .and_then(|s| serde_json::to_string(s).ok());

        let row = sqlx::query_as::<_, TemplateRow>(
            r#"
            UPDATE templates SET
                name = ?, title_template = ?, glance_note_template = ?,
                detail_note_template = ?, estimated_duration_template = ?,
                subtasks_template = ?, area_id = ?, updated_at = ?
            WHERE id = ? AND deleted_at IS NULL
            RETURNING id, name, title_template, glance_note_template, detail_note_template,
                      estimated_duration_template, subtasks_template, area_id,
                      created_at, updated_at, is_deleted
            "#,
        )
        .bind(&template.name)
        .bind(&template.title_template)
        .bind(&template.glance_note_template)
        .bind(&template.detail_note_template)
        .bind(template.estimated_duration_template)
        .bind(subtasks_json)
        .bind(template.area_id.map(|id| id.to_string()))
        .bind(template.updated_at.to_rfc3339())
        .bind(template.id.to_string())
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        Template::try_from(row)
            .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::QueryError(e)))
    }
}
