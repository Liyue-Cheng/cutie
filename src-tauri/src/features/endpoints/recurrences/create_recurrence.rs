/// 创建循环规则 - 单文件组件
///
/// ⚠️ 开发前必读:
/// 1. 查看 Schema: migrations/20241001000000_initial_schema.sql
/// 2. 使用已有的 Repository,禁止重复实现
// ==================== CABC 文档 ====================
/*
CABC for `create_recurrence`

## 1. 端点签名
POST /api/recurrences

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要创建一个循环任务规则,以便系统自动生成每天/每周的重复任务

### 2.2 核心业务逻辑
创建一条新的循环规则记录,关联到指定的模板

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "template_id": "uuid (required)",
  "rule": "string (required, e.g. DAILY, WEEKLY:1,3,5)",
  "time_type": "FLOATING | FIXED (optional, default: FLOATING)",
  "start_date": "YYYY-MM-DD (optional)",
  "end_date": "YYYY-MM-DD (optional)",
  "timezone": "string (optional)",
  "is_active": "boolean (optional, default: true)"
}

### 3.2 响应 (Responses)
**201 Created:**
{
  "id": "uuid",
  "template_id": "uuid",
  "rule": "string",
  "time_type": "FLOATING | FIXED",
  "start_date": "YYYY-MM-DD | null",
  "end_date": "YYYY-MM-DD | null",
  "timezone": "string | null",
  "is_active": boolean,
  "created_at": "ISO8601",
  "updated_at": "ISO8601"
}

## 4. 验证规则
- template_id: 必须存在
- rule: 必须非空
- start_date/end_date: 必须符合 YYYY-MM-DD 格式（如果提供）

## 5. 业务逻辑详解
1. 验证输入
2. 验证模板是否存在
3. 开启事务
4. 创建循环规则
5. 提交事务
6. 返回结果

## 6. 边界情况
- 模板不存在: 返回 404
- 规则格式错误: 返回 422

## 7. 预期副作用
### 数据库操作:
- INSERT: 1条记录到 task_recurrences 表
- 事务边界: begin() → commit()

### SSE 事件:
- recurrence.created

## 8. 契约
### 前置条件:
- template_id 必须指向存在的模板
- rule 必须非空

### 后置条件:
- 数据库中存在新的循环规则
- 返回完整的 TaskRecurrenceDto
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::{
        CreateTaskRecurrenceRequest, ExpiryBehavior, TaskRecurrence, TaskRecurrenceDto, TimeType,
    },
    features::{shared::TaskRecurrenceRepository, shared::TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTaskRecurrenceRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(dto) => created_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &CreateTaskRecurrenceRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        // 验证 rule
        if request.rule.trim().is_empty() {
            errors.push("rule cannot be empty");
        }

        // 🔥 验证 start_date 必须存在（用于链接源任务）
        if request.source_task_id.is_some() && request.start_date.is_none() {
            errors.push("start_date is required when source_task_id is provided");
        }

        // 🔥 验证 RRULE 中的 UNTIL 与 end_date 一致性
        if let Some(until_date) = extract_until_from_rrule(&request.rule) {
            if let Some(ref end_date) = request.end_date {
                if until_date != *end_date {
                    errors.push("RRULE UNTIL and end_date must be consistent (or omit UNTIL and use end_date only)");
                }
            }
        }

        // 验证日期格式（如果提供）
        let start_date_parsed = if let Some(ref start_date) = request.start_date {
            match chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => {
                    errors.push("start_date must be in YYYY-MM-DD format");
                    None
                }
            }
        } else {
            None
        };

        let end_date_parsed = if let Some(ref end_date) = request.end_date {
            match chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d") {
                Ok(d) => Some(d),
                Err(_) => {
                    errors.push("end_date must be in YYYY-MM-DD format");
                    None
                }
            }
        } else {
            None
        };

        // 🔥 验证 start_date <= end_date
        if let (Some(start), Some(end)) = (start_date_parsed, end_date_parsed) {
            if start > end {
                errors.push("end_date must be after or equal to start_date");
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
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: CreateTaskRecurrenceRequest,
    ) -> AppResult<TaskRecurrenceDto> {
        // 1. 验证
        validation::validate_request(&request)?;

        // 2. 验证模板是否存在
        database::verify_template_exists(app_state.db_pool(), request.template_id).await?;

        // 3. 获取依赖
        let id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行（覆盖所有后续事务）
        let _permit = app_state.acquire_write_permit().await;

        // 4. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 5. 创建循环规则
        let time_type = request.time_type.unwrap_or(TimeType::Floating);
        let expiry_behavior = request.expiry_behavior.unwrap_or(ExpiryBehavior::CarryoverToStaging);
        let recurrence = TaskRecurrence {
            id,
            template_id: request.template_id,
            rule: request.rule,
            time_type,
            start_date: request.start_date,
            end_date: request.end_date,
            timezone: request.timezone,
            expiry_behavior,
            is_active: request.is_active.unwrap_or(true),
            created_at: now,
            updated_at: now,
        };

        TaskRecurrenceRepository::insert_in_tx(&mut tx, &recurrence).await?;

        // 6. 提交事务
        TransactionHelper::commit(tx).await?;

        // 7. 🔥 如果提供了source_task_id，将其作为第一个循环实例（避免重复创建）
        if let Some(source_task_id) = request.source_task_id {
            if let Some(ref start_date) = recurrence.start_date {
                // 🔥 验证 source_task_id 的日期与 start_date 匹配
                validate_source_task_date(app_state.db_pool(), source_task_id, start_date).await?;

                tracing::info!(
                    "🔄 [CREATE_RECURRENCE] Linking source task {} as first instance on {}",
                    source_task_id,
                    start_date
                );

                // 创建链接并更新源任务循环字段（在同一新事务中）
                let mut link_tx = TransactionHelper::begin(app_state.db_pool()).await?;

                use crate::entities::TaskRecurrenceLink;
                use crate::features::shared::TaskRecurrenceLinkRepository;

                let link =
                    TaskRecurrenceLink::new(recurrence.id, start_date.clone(), source_task_id, now);

                TaskRecurrenceLinkRepository::insert_in_tx(&mut link_tx, &link).await?;

                // 同步更新源任务的 recurrence 字段，确保前端识别为循环任务的首个实例
                use crate::features::shared::repositories::TaskRepository;
                TaskRepository::set_recurrence_fields_in_tx(
                    &mut link_tx,
                    source_task_id,
                    recurrence.id,
                    start_date,
                    now,
                )
                .await?;
                TransactionHelper::commit(link_tx).await?;

                tracing::info!(
                    "🔄 [CREATE_RECURRENCE] ✅ Linked source task {} as first instance",
                    source_task_id
                );
            }
        }

        // 8. 组装 DTO
        let dto = TaskRecurrenceDto {
            id: recurrence.id,
            template_id: recurrence.template_id,
            rule: recurrence.rule,
            time_type: recurrence.time_type,
            start_date: recurrence.start_date,
            end_date: recurrence.end_date,
            timezone: recurrence.timezone,
            expiry_behavior: recurrence.expiry_behavior,
            is_active: recurrence.is_active,
            created_at: recurrence.created_at,
            updated_at: recurrence.updated_at,
        };

        // 9. (可选) 发送 SSE 事件
        // TODO: 实现 SSE 事件

        Ok(dto)
    }

    /// 🔥 验证 source_task_id 的日期与 start_date 匹配
    async fn validate_source_task_date(
        pool: &sqlx::SqlitePool,
        source_task_id: uuid::Uuid,
        expected_start_date: &str,
    ) -> AppResult<()> {
        // 查询任务的第一个日程日期
        let query = r#"
            SELECT ts.scheduled_date
            FROM task_schedules ts
            WHERE ts.task_id = ?
            ORDER BY ts.scheduled_date ASC
            LIMIT 1
        "#;

        let actual_date: Option<String> = sqlx::query_scalar(query)
            .bind(source_task_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        match actual_date {
            Some(date) if date == expected_start_date => {
                tracing::info!(
                    "🔄 [VALIDATION] ✅ Source task {} date {} matches start_date",
                    source_task_id,
                    date
                );
                Ok(())
            }
            Some(date) => {
                tracing::error!(
                    "🔄 [VALIDATION] ❌ Source task {} date {} does not match start_date {}",
                    source_task_id,
                    date,
                    expected_start_date
                );
                Err(AppError::ValidationFailed(vec![
                    crate::infra::core::ValidationError::new(
                        "source_task_id".to_string(),
                        format!(
                            "Source task is scheduled on {}, but start_date is {}",
                            date, expected_start_date
                        ),
                        "DATE_MISMATCH".to_string(),
                    ),
                ]))
            }
            None => {
                tracing::error!(
                    "🔄 [VALIDATION] ❌ Source task {} has no schedule",
                    source_task_id
                );
                Err(AppError::ValidationFailed(vec![
                    crate::infra::core::ValidationError::new(
                        "source_task_id".to_string(),
                        "Source task has no schedule date".to_string(),
                        "NO_SCHEDULE".to_string(),
                    ),
                ]))
            }
        }
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use uuid::Uuid;

    pub async fn verify_template_exists(
        pool: &sqlx::SqlitePool,
        template_id: Uuid,
    ) -> AppResult<()> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM templates
            WHERE id = ? AND is_deleted = 0
        "#;

        let count: i64 = sqlx::query_scalar(query)
            .bind(template_id.to_string())
            .fetch_one(pool)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        if count == 0 {
            return Err(AppError::NotFound {
                entity_type: "Template".to_string(),
                entity_id: template_id.to_string(),
            });
        }

        Ok(())
    }
}
