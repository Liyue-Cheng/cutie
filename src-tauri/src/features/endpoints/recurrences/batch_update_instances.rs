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
  "subtasks": "array | null (optional)", // 子任务列表
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
    features::{shared::TaskRecurrenceRepository, shared::TransactionHelper},
    infra::{
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
    #[serde(default, deserialize_with = "deserialize_nullable_field")]
    pub subtasks: Option<Option<Vec<crate::entities::Subtask>>>,
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
pub mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        recurrence_id: Uuid,
        request: BatchUpdateInstancesRequest,
    ) -> AppResult<BatchUpdateInstancesResponse> {
        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

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
        .map_err(|e| AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e)))?;

        // 解析 UUID
        task_id_strs
            .into_iter()
            .map(|s| {
                Uuid::parse_str(&s).map_err(|e| {
                    AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                        "task_id".to_string(),
                        format!("Invalid UUID: {}", e),
                        "INVALID_UUID".to_string(),
                    )])
                })
            })
            .collect()
    }

    /// 批量更新任务
    pub async fn batch_update_tasks(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        task_ids: &[Uuid],
        request: &BatchUpdateInstancesRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<usize> {
        let mut updated_count = 0;

        // 🔥 对于 subtasks，需要逐个任务处理（智能合并完成状态）
        if request.subtasks.is_some() {
            updated_count += batch_update_subtasks_preserving_completion(
                tx,
                task_ids,
                request.subtasks.as_ref().unwrap(),
            )
            .await?;
        }

        // 🔥 对于其他字段，使用批量 UPDATE
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

        if set_clauses.len() > 1 {
            // 有字段需要更新（除了 updated_at）
            let set_clause = set_clauses.join(", ");
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
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

            updated_count = result.rows_affected() as usize;
        }

        Ok(updated_count)
    }

    /// 🔥 批量更新子任务，但保留每个任务实例的已有完成状态
    ///
    /// 策略：
    /// 1. 根据 title 或 id 匹配新旧子任务
    /// 2. 如果匹配成功，保留旧的 is_completed 状态
    /// 3. 如果是新增子任务，使用模板的 is_completed (通常为 false)
    /// 4. 更新 title、sort_order 等结构字段
    async fn batch_update_subtasks_preserving_completion(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        task_ids: &[Uuid],
        new_subtasks_opt: &Option<Vec<crate::entities::Subtask>>,
    ) -> AppResult<usize> {
        use crate::entities::Subtask;

        let mut updated_count = 0;

        for task_id in task_ids {
            // 1. 查询当前任务的 subtasks
            let current_task_query = "SELECT subtasks FROM tasks WHERE id = ?";
            let current_subtasks_json: Option<String> = sqlx::query_scalar(current_task_query)
                .bind(task_id.to_string())
                .fetch_optional(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
                })?;

            let current_subtasks: Vec<Subtask> = if let Some(json) = current_subtasks_json {
                serde_json::from_str(&json).unwrap_or_default()
            } else {
                vec![]
            };

            // 2. 合并新旧子任务（保留完成状态）
            let merged_subtasks = match new_subtasks_opt {
                Some(new_subtasks) => {
                    merge_subtasks_preserving_completion(&current_subtasks, new_subtasks)
                }
                None => vec![], // 清空子任务
            };

            // 3. 更新任务的 subtasks 字段
            let merged_json = if merged_subtasks.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&merged_subtasks).map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::QueryError(e.to_string()))
                })?)
            };

            let update_query = "UPDATE tasks SET subtasks = ?, updated_at = ? WHERE id = ?";
            let result = sqlx::query(update_query)
                .bind(merged_json)
                .bind(chrono::Utc::now().to_rfc3339())
                .bind(task_id.to_string())
                .execute(&mut **tx)
                .await
                .map_err(|e| {
                    AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
                })?;

            updated_count += result.rows_affected() as usize;
        }

        Ok(updated_count)
    }

    /// 合并子任务列表，保留完成状态
    ///
    /// 策略：
    /// - 按 id 匹配：如果新旧子任务 id 相同，保留旧的 is_completed
    /// - 按 title 匹配：如果 id 不同但 title 相同，也保留旧的 is_completed
    /// - 新增子任务：使用模板的 is_completed
    fn merge_subtasks_preserving_completion(
        current: &[crate::entities::Subtask],
        new: &[crate::entities::Subtask],
    ) -> Vec<crate::entities::Subtask> {
        use std::collections::HashMap;

        // 建立旧子任务的索引（id -> is_completed, title -> is_completed）
        let mut completion_by_id: HashMap<uuid::Uuid, bool> = HashMap::new();
        let mut completion_by_title: HashMap<String, bool> = HashMap::new();

        for subtask in current {
            completion_by_id.insert(subtask.id, subtask.is_completed);
            completion_by_title.insert(subtask.title.clone(), subtask.is_completed);
        }

        // 遍历新子任务，恢复完成状态
        new.iter()
            .map(|subtask| {
                let is_completed = completion_by_id
                    .get(&subtask.id)
                    .or_else(|| completion_by_title.get(&subtask.title))
                    .copied()
                    .unwrap_or(subtask.is_completed); // 如果匹配不到，使用模板的状态

                crate::entities::Subtask {
                    id: subtask.id,
                    title: subtask.title.clone(),
                    is_completed, // 🔥 保留或恢复完成状态
                    sort_order: subtask.sort_order.clone(),
                }
            })
            .collect()
    }
}
