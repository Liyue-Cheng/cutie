/// 更新循环规则 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `update_recurrence`

## 1. 端点签名
PATCH /api/recurrences/:id

## 2. 预期行为简介
更新循环规则的属性

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "template_id": "uuid (optional)",
  "rule": "string (optional)",
  "time_type": "FLOATING | FIXED (optional)",
  "start_date": "YYYY-MM-DD | null (optional)",
  "end_date": "YYYY-MM-DD | null (optional)",
  "timezone": "string | null (optional)",
  "is_active": "boolean (optional)"
}

### 3.2 响应 (Responses)
**200 OK:**
返回更新后的 TaskRecurrenceDto

**404 Not Found:**
循环规则不存在

## 4. 业务逻辑详解
1. 验证输入
2. 开启事务
3. 🔥 如果设置了 end_date，清理该日期之后的未完成实例
   - 查询所有 recurrence_original_date > end_date 的未完成任务
   - 软删除这些任务
   - 删除对应的 task_recurrence_links 记录
4. 更新循环规则
5. 🔥 如果修改了 rule，清理不匹配新规则的未完成实例
   - 查询所有未完成的任务实例（通过 task_recurrence_links）
   - 对每个实例的日期，用新的 RRULE 规则判断是否应该在该日期生成
   - 如果不应该生成，软删除该任务实例并删除链接记录
6. 提交事务
7. 返回结果

## 5. 预期副作用
- UPDATE: task_recurrences 表
- SOFT_DELETE: tasks 表
  * 当设置 end_date 时：删除未来的未完成实例
  * 当修改 rule 时：删除不匹配新规则的未完成实例
- DELETE: task_recurrence_links 表
  * 当设置 end_date 时：删除未来的链接记录
  * 当修改 rule 时：删除不匹配的链接记录
- DELETE: task_schedules 表（通过任务的软删除级联清理）
- SSE 事件: recurrence.updated
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    entities::{TaskRecurrenceDto, UpdateTaskRecurrenceRequest},
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
    Json(request): Json<UpdateTaskRecurrenceRequest>,
) -> Response {
    match logic::execute(&app_state, recurrence_id, request).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::repositories::TaskRepository;
    use chrono::NaiveDate;

    pub async fn execute(
        app_state: &AppState,
        recurrence_id: Uuid,
        request: UpdateTaskRecurrenceRequest,
    ) -> AppResult<TaskRecurrenceDto> {
        // 1. 🔥 验证请求
        validate_update_request(&request)?;

        // 2. 获取时间
        let now = app_state.clock().now_utc();

        // 3. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 4. 查询现有的循环规则（用于验证 end_date）
        let existing_recurrence =
            TaskRecurrenceRepository::find_by_id_in_tx(&mut tx, recurrence_id)
                .await?
                .ok_or_else(|| AppError::NotFound {
                    entity_type: "TaskRecurrence".to_string(),
                    entity_id: recurrence_id.to_string(),
                })?;

        // 5. 🔥 验证 end_date（如果提供）
        if let Some(Some(ref end_date)) = request.end_date {
            // 验证 start_date <= end_date
            if let Some(ref start_date) = existing_recurrence.start_date {
                validate_end_date_after_start(start_date, end_date)?;
            }
        }

        // 6. 🔥 如果设置了 end_date，需要清理该日期之后的未完成实例
        if let Some(Some(ref end_date)) = request.end_date {
            tracing::info!(
                "🔄 [STOP_RECURRENCE] Stopping recurrence {} at date {}, cleaning up future instances...",
                recurrence_id,
                end_date
            );

            // 删除所有在 end_date 之后的未完成任务实例
            cleanup_future_instances(&mut tx, recurrence_id, end_date).await?;
        }

        // 7. 更新循环规则
        let recurrence =
            TaskRecurrenceRepository::update_in_tx(&mut tx, recurrence_id, &request, now).await?;

        // 8. 🔥 如果修改了循环规则，需要清理不匹配的实例
        if request.rule.is_some() {
            tracing::info!(
                "🔄 [RULE_CHANGE] Recurrence rule changed for {}, cleaning up mismatched instances...",
                recurrence_id
            );

            cleanup_mismatched_instances(&mut tx, &recurrence).await?;
        }

        // 9. 提交事务
        TransactionHelper::commit(tx).await?;

        // 10. 组装 DTO
        let dto = TaskRecurrenceDto {
            id: recurrence.id,
            template_id: recurrence.template_id,
            rule: recurrence.rule,
            time_type: recurrence.time_type,
            start_date: recurrence.start_date,
            end_date: recurrence.end_date,
            timezone: recurrence.timezone,
            is_active: recurrence.is_active,
            created_at: recurrence.created_at,
            updated_at: recurrence.updated_at,
        };

        // 11. (可选) 发送 SSE 事件
        // TODO: 实现 SSE 事件

        Ok(dto)
    }

    /// 验证更新请求
    fn validate_update_request(request: &UpdateTaskRecurrenceRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        // 🔥 禁止修改 start_date
        if request.start_date.is_some() {
            errors.push("Modifying start_date is not allowed");
        }

        // 验证 RRULE 格式（如果提供）
        if let Some(ref rule) = request.rule {
            if rule.trim().is_empty() {
                errors.push("rule cannot be empty");
            }

            // 🔥 验证 RRULE 中的 UNTIL 与 end_date 一致性
            if let Some(until_date) = extract_until_from_rrule(rule) {
                if let Some(Some(ref end_date)) = request.end_date {
                    if until_date != *end_date {
                        errors.push("RRULE UNTIL and end_date must be consistent (or omit UNTIL and use end_date only)");
                    }
                }
            }
        }

        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(
                errors
                    .into_iter()
                    .enumerate()
                    .map(|(i, msg)| {
                        crate::infra::core::ValidationError::new(
                            format!("field_{}", i),
                            msg.to_string(),
                            "VALIDATION_ERROR".to_string(),
                        )
                    })
                    .collect(),
            ));
        }

        Ok(())
    }

    /// 验证 end_date 必须在 start_date 之后
    fn validate_end_date_after_start(start_date: &str, end_date: &str) -> AppResult<()> {
        let start = chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d").map_err(|e| {
            AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                "start_date".to_string(),
                format!("Invalid start_date format: {}", e),
                "INVALID_DATE".to_string(),
            )])
        })?;

        let end = chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d").map_err(|e| {
            AppError::ValidationFailed(vec![crate::infra::core::ValidationError::new(
                "end_date".to_string(),
                format!("Invalid end_date format: {}", e),
                "INVALID_DATE".to_string(),
            )])
        })?;

        if start > end {
            return Err(AppError::ValidationFailed(vec![
                crate::infra::core::ValidationError::new(
                    "end_date".to_string(),
                    "end_date must be after or equal to start_date".to_string(),
                    "INVALID_DATE_RANGE".to_string(),
                ),
            ]));
        }

        Ok(())
    }

    /// 从 RRULE 中提取 UNTIL 参数（YYYY-MM-DD 格式）
    fn extract_until_from_rrule(rule: &str) -> Option<String> {
        // RRULE 格式示例：FREQ=DAILY;UNTIL=20251231
        for part in rule.split(';') {
            if let Some(until_value) = part.strip_prefix("UNTIL=") {
                // 将 YYYYMMDD 转换为 YYYY-MM-DD
                if until_value.len() == 8 {
                    return Some(format!(
                        "{}-{}-{}",
                        &until_value[0..4],
                        &until_value[4..6],
                        &until_value[6..8]
                    ));
                }
            }
        }
        None
    }

    /// 清理未来的未完成任务实例
    async fn cleanup_future_instances(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence_id: Uuid,
        end_date: &str,
    ) -> AppResult<()> {
        // 1. 查询所有在 end_date 之后的未完成任务
        let query = r#"
            SELECT id
            FROM tasks
            WHERE recurrence_id = ?
              AND recurrence_original_date > ?
              AND completed_at IS NULL
              AND deleted_at IS NULL
        "#;

        let task_ids: Vec<String> = sqlx::query_scalar(query)
            .bind(recurrence_id.to_string())
            .bind(end_date)
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| {
                crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                )
            })?;

        tracing::info!(
            "🔄 [CLEANUP] Found {} future uncompleted instances to delete",
            task_ids.len()
        );

        // 2. 软删除这些任务
        for task_id_str in task_ids {
            let task_id = Uuid::parse_str(&task_id_str).map_err(|e| {
                crate::infra::core::AppError::ValidationFailed(vec![
                    crate::infra::core::ValidationError::new(
                        "task_id".to_string(),
                        format!("Invalid UUID: {}", e),
                        "INVALID_UUID".to_string(),
                    ),
                ])
            })?;

            tracing::info!("🔄 [CLEANUP] Deleting task instance: {}", task_id);
            TaskRepository::soft_delete_in_tx(tx, task_id, chrono::Utc::now()).await?;
        }

        // 3. 删除对应的链接记录
        let delete_links_query = r#"
            DELETE FROM task_recurrence_links
            WHERE recurrence_id = ?
              AND instance_date > ?
        "#;

        let deleted_links = sqlx::query(delete_links_query)
            .bind(recurrence_id.to_string())
            .bind(end_date)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                )
            })?;

        tracing::info!(
            "🔄 [CLEANUP] Deleted {} recurrence links",
            deleted_links.rows_affected()
        );

        Ok(())
    }

    /// 清理不匹配新规则的任务实例
    ///
    /// 当循环规则被修改时（例如从每日改成每周），需要删除那些按照新规则不应该存在的未完成任务实例
    async fn cleanup_mismatched_instances(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        recurrence: &crate::entities::TaskRecurrence,
    ) -> AppResult<()> {
        use crate::infra::core::utils::time_utils;

        // 1. 查询所有未完成的任务实例及其日期
        let query = r#"
            SELECT trl.task_id, trl.instance_date, t.completed_at
            FROM task_recurrence_links trl
            JOIN tasks t ON t.id = trl.task_id
            WHERE trl.recurrence_id = ?
              AND t.completed_at IS NULL
              AND t.deleted_at IS NULL
        "#;

        #[derive(sqlx::FromRow)]
        struct TaskInstance {
            task_id: String,
            instance_date: String,
        }

        let instances: Vec<TaskInstance> = sqlx::query_as(query)
            .bind(recurrence.id.to_string())
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| {
                crate::infra::core::AppError::DatabaseError(
                    crate::infra::core::DbError::ConnectionError(e),
                )
            })?;

        tracing::info!(
            "🔄 [RULE_CHANGE] Found {} uncompleted instances to validate",
            instances.len()
        );

        // 2. 对每个实例，检查是否匹配新规则
        let mut to_delete = Vec::new();

        for instance in instances {
            // 解析日期
            let date = time_utils::parse_date_yyyy_mm_dd(&instance.instance_date).map_err(|e| {
                crate::infra::core::AppError::ValidationFailed(vec![
                    crate::infra::core::ValidationError::new(
                        "instance_date".to_string(),
                        format!("Invalid date format: {}", e),
                        "INVALID_DATE".to_string(),
                    ),
                ])
            })?;

            // 检查是否匹配新规则
            let matches = check_date_matches_rrule(&date, recurrence)?;

            if !matches {
                tracing::info!(
                    "🔄 [RULE_CHANGE] Instance {} on {} does not match new rule, marking for deletion",
                    instance.task_id,
                    instance.instance_date
                );
                to_delete.push((instance.task_id, instance.instance_date));
            } else {
                tracing::debug!(
                    "🔄 [RULE_CHANGE] Instance {} on {} still matches new rule",
                    instance.task_id,
                    instance.instance_date
                );
            }
        }

        // 3. 删除不匹配的任务实例
        tracing::info!(
            "🔄 [RULE_CHANGE] Deleting {} mismatched instances",
            to_delete.len()
        );

        for (task_id_str, instance_date) in to_delete {
            let task_id = Uuid::parse_str(&task_id_str).map_err(|e| {
                crate::infra::core::AppError::ValidationFailed(vec![
                    crate::infra::core::ValidationError::new(
                        "task_id".to_string(),
                        format!("Invalid UUID: {}", e),
                        "INVALID_UUID".to_string(),
                    ),
                ])
            })?;

            tracing::info!(
                "🔄 [RULE_CHANGE] Deleting mismatched task instance: {} on {}",
                task_id,
                instance_date
            );

            // 软删除任务
            TaskRepository::soft_delete_in_tx(tx, task_id, chrono::Utc::now()).await?;

            // 删除链接记录
            let delete_link_query = r#"
                DELETE FROM task_recurrence_links
                WHERE recurrence_id = ? AND instance_date = ?
            "#;

            sqlx::query(delete_link_query)
                .bind(recurrence.id.to_string())
                .bind(&instance_date)
                .execute(&mut **tx)
                .await
                .map_err(|e| {
                    crate::infra::core::AppError::DatabaseError(
                        crate::infra::core::DbError::ConnectionError(e),
                    )
                })?;
        }

        Ok(())
    }

    /// 检查日期是否匹配 RRULE（复用自 recurrence_instantiation_service）
    fn check_date_matches_rrule(
        date: &NaiveDate,
        recurrence: &crate::entities::TaskRecurrence,
    ) -> AppResult<bool> {
        use crate::infra::core::utils::time_utils;
        use rrule::RRuleSet;

        tracing::debug!(
            "🔄 [RRULE_CHECK] Checking date {} against rule: {}",
            date,
            recurrence.rule
        );

        // 确定 DTSTART：优先使用 start_date，否则使用 created_at 的日期部分
        let dtstart_date = if let Some(ref start_date) = recurrence.start_date {
            start_date.clone()
        } else {
            let created_date = recurrence.created_at.date_naive();
            time_utils::format_date_yyyy_mm_dd(&created_date)
        };

        // 构建完整的 RRULE 字符串（包含 DTSTART）
        let start_date_rrule = dtstart_date.replace("-", "");
        let full_rrule = format!("DTSTART:{}\nRRULE:{}", start_date_rrule, recurrence.rule);

        tracing::debug!("🔄 [RRULE_CHECK] Full RRULE:\n{}", full_rrule);

        let rrule_set: RRuleSet = full_rrule.parse().map_err(|e| {
            tracing::error!("🔄 [RRULE_CHECK] ❌ Failed to parse RRULE: {:?}", e);
            crate::infra::core::AppError::ValidationFailed(vec![
                crate::infra::core::ValidationError::new(
                    "rule".to_string(),
                    format!("Invalid RRULE: {:?}", e),
                    "INVALID_RRULE".to_string(),
                ),
            ])
        })?;

        // 检查该日期是否在 RRULE 生成的日期集合中
        let occurrences = rrule_set.into_iter();
        let mut count = 0;

        for occurrence in occurrences {
            count += 1;
            let occ_date = occurrence.date_naive();

            if occ_date == *date {
                tracing::debug!(
                    "🔄 [RRULE_CHECK] ✅ Found matching occurrence: {}",
                    occ_date
                );
                return Ok(true);
            }

            // 如果已经超过目标日期，停止检查
            if occ_date > *date {
                tracing::debug!(
                    "🔄 [RRULE_CHECK] Reached future date {}, stopping",
                    occ_date
                );
                break;
            }

            // 限制检查次数，防止无限循环
            if count > 1000 {
                tracing::warn!("🔄 [RRULE_CHECK] ⚠️ Checked 1000 occurrences, stopping");
                break;
            }
        }

        tracing::debug!("🔄 [RRULE_CHECK] ❌ No matching occurrence found");
        Ok(false)
    }
}
