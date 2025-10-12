/// 批量更新循环模板和任务实例 - 单文件组件
///
/// 在同一事务中更新模板和所有未完成实例，避免数据库锁冲突
// ==================== CABC 文档 ====================
/*
CABC for `batch_update_template_and_instances`

## 1. 端点签名
PATCH /api/recurrences/:id/template-and-instances

## 2. 预期行为简介
在同一事务中批量更新循环规则的模板和所有未完成任务实例

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "title": "string (optional)",
  "glance_note": "string | null (optional)",
  "detail_note": "string | null (optional)",
  "estimated_duration": "number | null (optional)",
  "area_id": "uuid | null (optional)",
  "subtasks": "array | null (optional)",
  "update_from_date": "YYYY-MM-DD (optional)" // 只更新该日期之后的实例
}

### 3.2 响应 (Responses)
**200 OK:**
{
  "template_updated": true,
  "instances_updated_count": 10
}

**404 Not Found:**
循环规则不存在

## 4. 业务逻辑详解
1. 验证循环规则存在
2. 开启事务
3. 更新模板（如果字段提供）
4. 批量更新任务实例（复用现有逻辑）
5. 提交事务
6. 返回更新统计

## 5. 预期副作用
- UPDATE: templates 表
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
    infra::{
        core::{AppError, AppResult, DbError},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(recurrence_id): Path<Uuid>,
    Json(request): Json<BatchUpdateTemplateAndInstancesRequest>,
) -> Response {
    match logic::execute(&app_state, recurrence_id, request).await {
        Ok(result) => success_response(result).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== DTOs ====================
#[derive(Debug, Deserialize)]
pub struct BatchUpdateTemplateAndInstancesRequest {
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
pub struct BatchUpdateTemplateAndInstancesResponse {
    pub template_updated: bool,
    pub instances_updated_count: usize,
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
    use std::collections::HashMap;

    pub async fn execute(
        app_state: &AppState,
        recurrence_id: Uuid,
        request: BatchUpdateTemplateAndInstancesRequest,
    ) -> AppResult<BatchUpdateTemplateAndInstancesResponse> {
        // 1. 开启事务
        let mut tx = app_state
            .db_pool()
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        // 2. 验证循环规则存在并获取模板ID
        let template_id = find_template_id_by_recurrence(&mut tx, recurrence_id).await?;
        let now = app_state.clock().now_utc();

        tracing::info!(
            "🔄 [TEMPLATE_AND_INSTANCES] Updating template {} and instances for recurrence {}",
            template_id,
            recurrence_id
        );

        // 3. 更新模板
        let template_updated =
            update_template_if_needed(&mut tx, template_id, &request, now).await?;

        // 4. 批量更新任务实例
        let instances_updated_count =
            update_instances(&mut tx, recurrence_id, &request, now).await?;

        // 5. 提交事务
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        tracing::info!(
            "🔄 [TEMPLATE_AND_INSTANCES] ✅ Updated template: {}, instances: {}",
            template_updated,
            instances_updated_count
        );

        Ok(BatchUpdateTemplateAndInstancesResponse {
            template_updated,
            instances_updated_count,
        })
    }

    /// 根据循环规则ID查找模板ID
    async fn find_template_id_by_recurrence(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
    ) -> AppResult<Uuid> {
        let query = "SELECT template_id FROM task_recurrences WHERE id = ?";

        let template_id_str: String = sqlx::query_scalar(query)
            .bind(recurrence_id.to_string())
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "TaskRecurrence".to_string(),
                entity_id: recurrence_id.to_string(),
            })?;

        Uuid::parse_str(&template_id_str).map_err(|e| {
            AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                "template_id".to_string(),
                format!("Invalid UUID: {}", e),
                "INVALID_UUID".to_string(),
            )])
        })
    }

    /// 更新模板（如果有字段需要更新）
    async fn update_template_if_needed(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        template_id: Uuid,
        request: &BatchUpdateTemplateAndInstancesRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<bool> {
        // 检查是否有字段需要更新
        let has_updates = request.title.is_some()
            || request.glance_note.is_some()
            || request.detail_note.is_some()
            || request.estimated_duration.is_some()
            || request.area_id.is_some()
            || request.subtasks.is_some();

        if !has_updates {
            return Ok(false);
        }

        // 构建动态 SQL 更新语句
        let mut set_clauses = vec![];
        if request.title.is_some() {
            set_clauses.push("title = ?");
        }
        if request.glance_note.is_some() {
            set_clauses.push("glance_note_template = ?");
        }
        if request.detail_note.is_some() {
            set_clauses.push("detail_note_template = ?");
        }
        if request.estimated_duration.is_some() {
            set_clauses.push("estimated_duration_template = ?");
        }
        if request.area_id.is_some() {
            set_clauses.push("area_id = ?");
        }
        if request.subtasks.is_some() {
            set_clauses.push("subtasks_template = ?");
        }
        set_clauses.push("updated_at = ?");

        let set_clause = set_clauses.join(", ");
        let query = format!("UPDATE templates SET {} WHERE id = ?", set_clause);

        let mut q = sqlx::query(&query);

        // 按顺序绑定参数
        if let Some(ref title) = request.title {
            q = q.bind(title);
        }
        if let Some(ref glance_note) = request.glance_note {
            q = q.bind(glance_note.as_ref());
        }
        if let Some(ref detail_note) = request.detail_note {
            q = q.bind(detail_note.as_ref());
        }
        if let Some(ref estimated_duration) = request.estimated_duration {
            q = q.bind(estimated_duration.as_ref());
        }
        if let Some(ref area_id) = request.area_id {
            q = q.bind(area_id.as_ref().map(|id| id.to_string()));
        }
        if let Some(ref subtasks) = request.subtasks {
            let subtasks_json = match subtasks.as_ref() {
                Some(subtasks) => Some(serde_json::to_string(subtasks).map_err(|e| {
                    AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                        "subtasks".to_string(),
                        format!("Failed to serialize subtasks: {}", e),
                        "SERIALIZATION_ERROR".to_string(),
                    )])
                })?),
                None => None,
            };
            q = q.bind(subtasks_json);
        }
        q = q.bind(now.to_rfc3339());
        q = q.bind(template_id.to_string());

        let result = q
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        tracing::info!(
            "🔄 [TEMPLATE_UPDATE] ✅ Template {} updated successfully, rows affected: {}",
            template_id,
            result.rows_affected()
        );

        Ok(true)
    }

    /// 更新任务实例
    async fn update_instances(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
        request: &BatchUpdateTemplateAndInstancesRequest,
        now: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<usize> {
        // 1. 查询所有未完成的任务实例ID
        let task_ids =
            find_uncompleted_instance_ids(tx, recurrence_id, &request.update_from_date).await?;

        tracing::info!(
            "🔄 [INSTANCES_UPDATE] Found {} uncompleted instances for recurrence {}",
            task_ids.len(),
            recurrence_id
        );

        if task_ids.is_empty() {
            return Ok(0);
        }

        let mut updated_count = 0;

        // 2. 🔥 对于 subtasks，需要逐个任务处理（智能合并完成状态）
        if request.subtasks.is_some() {
            updated_count += batch_update_subtasks_preserving_completion(
                tx,
                &task_ids,
                request.subtasks.as_ref().unwrap(),
            )
            .await?;
        }

        // 3. 🔥 对于其他字段，使用批量 UPDATE
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

            // 绑定 SET 子句的参数
            if let Some(ref title) = request.title {
                q = q.bind(title);
            }
            if let Some(ref glance_note) = request.glance_note {
                q = q.bind(glance_note.as_ref());
            }
            if let Some(ref detail_note) = request.detail_note {
                q = q.bind(detail_note.as_ref());
            }
            if let Some(ref estimated_duration) = request.estimated_duration {
                q = q.bind(estimated_duration.as_ref());
            }
            if let Some(ref area_id) = request.area_id {
                q = q.bind(area_id.as_ref().map(|id| id.to_string()));
            }
            q = q.bind(now.to_rfc3339());

            // 绑定 WHERE 子句的参数
            for task_id in &task_ids {
                q = q.bind(task_id.to_string());
            }

            let result = q
                .execute(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

            updated_count += result.rows_affected() as usize;
        }

        tracing::info!(
            "🔄 [INSTANCES_UPDATE] ✅ Updated {} task instances",
            updated_count
        );

        Ok(updated_count)
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
        .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

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

    /// 🔥 批量更新子任务，但保留每个任务实例的已有完成状态
    async fn batch_update_subtasks_preserving_completion(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        task_ids: &[Uuid],
        new_subtasks_opt: &Option<Vec<crate::entities::Subtask>>,
    ) -> AppResult<usize> {
        use crate::entities::Subtask;
        let mut updated_count = 0;

        for task_id in task_ids {
            // 1. 查询当前任务的 subtasks
            let current_subtasks_json: Option<String> =
                sqlx::query_scalar("SELECT subtasks FROM tasks WHERE id = ?")
                    .bind(task_id.to_string())
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

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
                    AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                        "subtasks".to_string(),
                        format!("Failed to serialize subtasks: {}", e),
                        "SERIALIZATION_ERROR".to_string(),
                    )])
                })?)
            };

            let update_query = "UPDATE tasks SET subtasks = ?, updated_at = ? WHERE id = ?";
            let result = sqlx::query(update_query)
                .bind(merged_json)
                .bind(chrono::Utc::now().to_rfc3339())
                .bind(task_id.to_string())
                .execute(&mut **tx)
                .await
                .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

            updated_count += result.rows_affected() as usize;
        }

        Ok(updated_count)
    }

    /// 合并子任务列表，保留完成状态
    fn merge_subtasks_preserving_completion(
        current: &[crate::entities::Subtask],
        new: &[crate::entities::Subtask],
    ) -> Vec<crate::entities::Subtask> {
        let mut completion_by_id: HashMap<uuid::Uuid, bool> = HashMap::new();
        let mut completion_by_title: HashMap<String, bool> = HashMap::new();

        // 建立完成状态索引
        for subtask in current {
            completion_by_id.insert(subtask.id, subtask.is_completed);
            completion_by_title.insert(subtask.title.clone(), subtask.is_completed);
        }

        // 合并新子任务，保留完成状态
        new.iter()
            .map(|subtask| {
                let is_completed = completion_by_id
                    .get(&subtask.id)
                    .or_else(|| completion_by_title.get(&subtask.title))
                    .copied()
                    .unwrap_or(subtask.is_completed);

                crate::entities::Subtask {
                    id: subtask.id,
                    title: subtask.title.clone(),
                    is_completed,
                    sort_order: subtask.sort_order.clone(),
                }
            })
            .collect()
    }
}
