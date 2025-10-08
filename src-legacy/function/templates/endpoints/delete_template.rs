/// 删除模板 API - 单文件组件
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    entities::Template,
    shared::core::{AppError, AppResult},
    startup::AppState,
};

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `delete_template`

## API端点
DELETE /api/templates/{id}

## 预期行为简介
删除一个模板。幂等操作。

## 输入输出规范
- **前置条件**: `id` 必须是有效的模板ID。
- **后置条件**: 返回 `204 No Content`。
- **不变量**: 无。

## 边界情况
- 模板不存在: 幂等地返回 `204`。

## 预期副作用
- 软删除 `templates` 表中的1条记录（设置 is_deleted = true）。
- 所有数据库写入在单个事务中。
*/

// ==================== 路由层 (Router Layer) ====================
pub async fn handle(State(app_state): State<AppState>, Path(template_id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, template_id).await {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, template_id: Uuid) -> AppResult<()> {
        let mut tx = app_state.db_pool().begin().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        // 1. 验证模板存在（幂等）
        let template = match database::find_template_by_id_in_tx(&mut tx, template_id).await? {
            Some(t) => t,
            None => {
                // 幂等：模板不存在也返回成功
                return Ok(());
            }
        };

        // 2. 核心操作：软删除模板
        let now = app_state.clock().now_utc();
        database::delete_template_in_tx(&mut tx, template.id, now).await?;

        // 3. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(())
    }
}

// ==================== 数据访问层 (Data Access Layer) ====================
mod database {
    use super::*;
    use crate::entities::template::TemplateRow;
    use chrono::DateTime;

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

    pub async fn delete_template_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template_id: Uuid,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query("UPDATE templates SET is_deleted = true, updated_at = ? WHERE id = ?")
            .bind(updated_at.to_rfc3339())
            .bind(template_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }
}
