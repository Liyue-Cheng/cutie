/// 批量更新循环任务实例 - 单文件组件
///
/// 一次性修改某个循环规则的所有未完成任务实例
// ==================== CABC 文档 ====================
/*
CABC for `batch_update_instances`

## 1. 端点签名
PATCH /api/recurrences/:id/instances/batch

## 2. 预期行为简介
批量更新某个循环规则的所有未完成任务实例的内容

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "title": "string (optional)",
  "glance_note": "string | null (optional)",
  "detail_note": "string | null (optional)",
  "estimated_duration": "number | null (optional)",
  "area_id": "uuid | null (optional)",
  "update_from_date": "YYYY-MM-DD (optional)" // 只更新该日期之后的实例
}

### 3.2 响应 (Responses)
**200 OK:**
{
  "updated_count": 10
}

**404 Not Found:**
循环规则不存在

## 4. 业务逻辑详解
1. 验证循环规则存在
2. 查询所有未完成的任务实例（可选：从指定日期开始）
3. 批量更新任务字段
4. 返回更新数量

## 5. 预期副作用
- UPDATE: tasks 表（批量更新）
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    features::{recurrences::shared::TaskRecurrenceRepository, shared::TransactionHelper},
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(recurrence_id): Path<Uuid>,
    Json(request): Json<BatchUpdateInstancesRequest>,
) -> Response {
    match logic::execute(&app_state, recurrence_id, request).await {
        Ok(result) => success_response(result).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== DTOs ====================
#[derive(Debug, Deserialize)]
pub struct BatchUpdateInstancesRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub glance_note: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub detail_note: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub estimated_duration: Option<Option<i32>>,
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub area_id: Option<Option<Uuid>>,
    /// 只更新该日期之后的实例（包括该日期），如果为 None 则更新所有未完成实例
    pub update_from_date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchUpdateInstancesResponse {
    pub updated_count: usize,
}

/// 自定义反序列化器，用于正确处理三态字段
fn deserialize_nullable_field<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    use serde::Deserialize;
    Ok(Some(Option::deserialize(deserializer)?))
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        recurrence_id: Uuid,
        request: BatchUpdateInstancesRequest,
    ) -> AppResult<BatchUpdateInstancesResponse> {
        // 1. 验证循环规则存在
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        let _recurrence = TaskRecurrenceRepository::find_by_id_in_tx(&mut tx, recurrence_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "TaskRecurrence".to_string(),
                entity_id: recurrence_id.to_string(),
            })?;

        // 2. 查询所有未完成的任务实例
        let task_ids =
            find_uncompleted_instance_ids(&mut tx, recurrence_id, &request.update_from_date)
                .await?;

        tracing::info!(
            "🔄 [BATCH_UPDATE] Found {} uncompleted instances for recurrence {}",
            task_ids.len(),
            recurrence_id
        );

        // 3. 批量更新任务
        let updated_count = if !task_ids.is_empty() {
            batch_update_tasks(&mut tx, &task_ids, &request, app_state.clock().now_utc()).await?
        } else {
            0
        };

        // 4. 提交事务
        TransactionHelper::commit(tx).await?;

        tracing::info!(
            "🔄 [BATCH_UPDATE] Successfully updated {} task instances",
            updated_count
        );

        Ok(BatchUpdateInstancesResponse { updated_count })
    }

    /// 查询所有未完成的任务实例ID
    async fn find_uncompleted_instance_ids(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
        from_date: &Option<String>,
    ) -> AppResult<Vec<Uuid>> {
        let query = if from_date.is_some() {
            r#"
                SELECT trl.task_id
                FROM task_recurrence_links trl
                JOIN tasks t ON t.id = trl.task_id
                WHERE trl.recurrence_id = ?
                  AND trl.instance_date >= ?
                  AND t.completed_at IS NULL
                  AND t.deleted_at IS NULL
            "#
        } else {
            r#"
                SELECT trl.task_id
                FROM task_recurrence_links trl
                JOIN tasks t ON t.id = trl.task_id
                WHERE trl.recurrence_id = ?
                  AND t.completed_at IS NULL
                  AND t.deleted_at IS NULL
            "#
        };

        let task_id_strs: Vec<String> = if let Some(ref date) = from_date {
            sqlx::query_scalar(query)
                .bind(recurrence_id.to_string())
                .bind(date)
                .fetch_all(&mut **tx)
                .await
        } else {
            sqlx::query_scalar(query)
                .bind(recurrence_id.to_string())
                .fetch_all(&mut **tx)
                .await
        }
        .map_err(|e| AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e)))?;

        // 解析 UUID
        task_id_strs
            .into_iter()
            .map(|s| {
                Uuid::parse_str(&s).map_err(|e| {
                    AppError::ValidationFailed(vec![crate::shared::core::ValidationError::new(
                        "task_id".to_string(),
                        format!("Invalid UUID: {}", e),
                        "INVALID_UUID".to_string(),
                    )])
                })
            })
            .collect()
    }

    /// 批量更新任务
    async fn batch_update_tasks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        task_ids: &[Uuid],
        request: &BatchUpdateInstancesRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<usize> {
        // 构建动态 SET 子句
        let mut set_clauses = vec![];
        if request.title.is_some() {
            set_clauses.push("title = ?");
        }
        if request.glance_note.is_some() {
            set_clauses.push("glance_note = ?");
        }
        if request.detail_note.is_some() {
            set_clauses.push("detail_note = ?");
        }
        if request.estimated_duration.is_some() {
            set_clauses.push("estimated_duration = ?");
        }
        if request.area_id.is_some() {
            set_clauses.push("area_id = ?");
        }
        set_clauses.push("updated_at = ?");

        if set_clauses.len() == 1 {
            // 只有 updated_at，无实际更新
            return Ok(0);
        }

        let set_clause = set_clauses.join(", ");

        // 构建 IN 子句
        let placeholders = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let query = format!(
            "UPDATE tasks SET {} WHERE id IN ({})",
            set_clause, placeholders
        );

        let mut q = sqlx::query(&query);

        // 绑定 SET 参数
        if let Some(ref title) = request.title {
            q = q.bind(title);
        }
        if let Some(ref glance_note_opt) = request.glance_note {
            q = q.bind(glance_note_opt.as_ref());
        }
        if let Some(ref detail_note_opt) = request.detail_note {
            q = q.bind(detail_note_opt.as_ref());
        }
        if let Some(ref duration_opt) = request.estimated_duration {
            q = q.bind(duration_opt);
        }
        if let Some(ref area_id_opt) = request.area_id {
            q = q.bind(area_id_opt.map(|id| id.to_string()));
        }
        q = q.bind(now);

        // 绑定 WHERE IN 参数
        for task_id in task_ids {
            q = q.bind(task_id.to_string());
        }

        let result = q.execute(&mut **tx).await.map_err(|e| {
            AppError::DatabaseError(crate::shared::core::DbError::ConnectionError(e))
        })?;

        Ok(result.rows_affected() as usize)
    }
}
