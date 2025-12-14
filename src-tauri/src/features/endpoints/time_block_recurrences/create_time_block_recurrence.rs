/// 创建时间块循环规则 - 单文件组件
///
/// ⚠️ 开发前必读:
/// 1. 查看 Schema: migrations/20251128000000_add_time_block_recurrences.sql
/// 2. 使用已有的 Repository,禁止重复实现
// ==================== CABC 文档 ====================
/*
CABC for `create_time_block_recurrence`

## 1. 端点签名
POST /api/time-block-recurrences

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要创建一个时间块循环规则,以便系统自动生成每天/每周的重复时间块

### 2.2 核心业务逻辑
1. 先创建一个时间块模板
2. 再创建循环规则关联到该模板
3. 可选地将源时间块作为第一个实例

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  // 模板信息
  "title": "string (optional)",
  "glance_note_template": "string (optional)",
  "detail_note_template": "string (optional)",
  "duration_minutes": "integer (required)",
  "start_time_local": "HH:MM:SS (required)",
  "time_type": "FLOATING | FIXED (optional, default: FLOATING)",
  "is_all_day": "boolean (optional, default: false)",
  "area_id": "uuid (optional)",

  // 循环规则信息
  "rule": "string (required, RRULE格式)",
  "start_date": "YYYY-MM-DD (optional)",
  "end_date": "YYYY-MM-DD (optional)",

  // 源时间块（可选）
  "source_time_block_id": "uuid (optional)"
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
  "is_active": boolean,
  "created_at": "ISO8601",
  "updated_at": "ISO8601",
  "template": {
    "id": "uuid",
    "title": "string | null",
    "duration_minutes": integer,
    "start_time_local": "HH:MM:SS",
    "is_all_day": boolean,
    "area_id": "uuid | null"
  }
}

## 4. 验证规则
- duration_minutes: 必须 > 0
- start_time_local: 必须是有效的 HH:MM:SS 格式
- rule: 必须非空

## 5. 业务逻辑详解
1. 验证输入
2. 开启事务
3. 创建时间块模板
4. 创建循环规则
5. 如果有source_time_block_id，创建链接
6. 提交事务
7. 返回结果

## 6. 预期副作用
### 数据库操作:
- INSERT: 1条记录到 time_block_templates 表
- INSERT: 1条记录到 time_block_recurrences 表
- INSERT: (可选) 1条记录到 time_block_recurrence_links 表
- 事务边界: begin() → commit()
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::{
        time_block::TimeType, TimeBlockRecurrence, TimeBlockRecurrenceDetailDto,
        TimeBlockRecurrenceLink, TimeBlockTemplate, TimeBlockTemplateInfo,
    },
    features::shared::{
        TimeBlockRecurrenceLinkRepository, TimeBlockRecurrenceRepository,
        TimeBlockTemplateRepository, TransactionHelper,
    },
    infra::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== 请求结构 ====================
#[derive(Debug, Deserialize)]
pub struct CreateTimeBlockRecurrenceFullRequest {
    // 模板信息
    pub title: Option<String>,
    pub glance_note_template: Option<String>,
    pub detail_note_template: Option<String>,
    pub duration_minutes: i32,
    pub start_time_local: String,
    pub time_type: Option<TimeType>,
    pub is_all_day: Option<bool>,
    pub area_id: Option<Uuid>,

    // 循环规则信息
    pub rule: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub timezone: Option<String>,

    // 源时间块（可选）
    pub source_time_block_id: Option<Uuid>,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTimeBlockRecurrenceFullRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(dto) => created_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &CreateTimeBlockRecurrenceFullRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        // 验证 duration_minutes
        if request.duration_minutes <= 0 {
            errors.push(ValidationError {
                field: "duration_minutes".to_string(),
                code: "DURATION_INVALID".to_string(),
                message: "duration_minutes must be positive".to_string(),
            });
        }

        // 验证 start_time_local 格式 (HH:MM:SS)
        if chrono::NaiveTime::parse_from_str(&request.start_time_local, "%H:%M:%S").is_err() {
            errors.push(ValidationError {
                field: "start_time_local".to_string(),
                code: "TIME_FORMAT_INVALID".to_string(),
                message: "start_time_local must be in HH:MM:SS format".to_string(),
            });
        }

        // 验证 rule
        if request.rule.trim().is_empty() {
            errors.push(ValidationError {
                field: "rule".to_string(),
                code: "RULE_EMPTY".to_string(),
                message: "rule cannot be empty".to_string(),
            });
        }

        // 验证日期格式（如果提供）
        if let Some(ref start_date) = request.start_date {
            if chrono::NaiveDate::parse_from_str(start_date, "%Y-%m-%d").is_err() {
                errors.push(ValidationError {
                    field: "start_date".to_string(),
                    code: "DATE_FORMAT_INVALID".to_string(),
                    message: "start_date must be in YYYY-MM-DD format".to_string(),
                });
            }
        }

        if let Some(ref end_date) = request.end_date {
            if chrono::NaiveDate::parse_from_str(end_date, "%Y-%m-%d").is_err() {
                errors.push(ValidationError {
                    field: "end_date".to_string(),
                    code: "DATE_FORMAT_INVALID".to_string(),
                    message: "end_date must be in YYYY-MM-DD format".to_string(),
                });
            }
        }

        // 验证 start_date <= end_date
        if let (Some(ref start), Some(ref end)) = (&request.start_date, &request.end_date) {
            if let (Ok(start_d), Ok(end_d)) = (
                chrono::NaiveDate::parse_from_str(start, "%Y-%m-%d"),
                chrono::NaiveDate::parse_from_str(end, "%Y-%m-%d"),
            ) {
                if start_d > end_d {
                    errors.push(ValidationError {
                        field: "end_date".to_string(),
                        code: "DATE_RANGE_INVALID".to_string(),
                        message: "end_date must be after or equal to start_date".to_string(),
                    });
                }
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
        request: CreateTimeBlockRecurrenceFullRequest,
    ) -> AppResult<TimeBlockRecurrenceDetailDto> {
        // 1. 验证
        validation::validate_request(&request)?;

        // 2. 获取依赖
        let template_id = app_state.id_generator().new_uuid();
        let recurrence_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 3. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 4. 创建时间块模板
        let time_type = request.time_type.unwrap_or(TimeType::Floating);
        let template = TimeBlockTemplate {
            id: template_id,
            title: request.title.clone(),
            glance_note_template: request.glance_note_template.clone(),
            detail_note_template: request.detail_note_template.clone(),
            duration_minutes: request.duration_minutes,
            start_time_local: request.start_time_local.clone(),
            time_type,
            is_all_day: request.is_all_day.unwrap_or(false),
            area_id: request.area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        };

        TimeBlockTemplateRepository::insert_in_tx(&mut tx, &template).await?;

        // 5. 创建循环规则
        let recurrence = TimeBlockRecurrence {
            id: recurrence_id,
            template_id,
            rule: request.rule.clone(),
            time_type,
            start_date: request.start_date.clone(),
            end_date: request.end_date.clone(),
            timezone: request.timezone.clone(),
            is_active: true,
            created_at: now,
            updated_at: now,
        };

        TimeBlockRecurrenceRepository::insert_in_tx(&mut tx, &recurrence).await?;

        // 6. 提交事务
        TransactionHelper::commit(tx).await?;

        // 7. 如果提供了 source_time_block_id，将其作为第一个循环实例
        if let Some(source_time_block_id) = request.source_time_block_id {
            if let Some(ref start_date) = recurrence.start_date {
                tracing::info!(
                    "🔄 [CREATE_TB_RECURRENCE] Linking source time block {} as first instance on {}",
                    source_time_block_id,
                    start_date
                );

                // 创建链接（在新事务中）
                let mut link_tx = TransactionHelper::begin(app_state.db_pool()).await?;

                let link = TimeBlockRecurrenceLink::new(
                    recurrence.id,
                    start_date.clone(),
                    source_time_block_id,
                    now,
                );

                TimeBlockRecurrenceLinkRepository::insert_in_tx(&mut link_tx, &link).await?;

                // 更新源时间块的循环字段
                // 注意：recurrence_parent_id 应指向父时间块（外键约束），而不是循环规则
                // 作为第一个实例，它没有父时间块，所以设为 NULL
                database::update_time_block_recurrence_fields(
                    &mut link_tx,
                    source_time_block_id,
                    &recurrence.rule,
                    start_date,
                    now,
                )
                .await?;

                TransactionHelper::commit(link_tx).await?;

                tracing::info!(
                    "🔄 [CREATE_TB_RECURRENCE] ✅ Linked source time block {} as first instance",
                    source_time_block_id
                );
            }
        }

        // 8. 组装 DTO
        let dto = TimeBlockRecurrenceDetailDto {
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
            template: Some(TimeBlockTemplateInfo {
                id: template.id,
                title: template.title,
                glance_note_template: template.glance_note_template,
                detail_note_template: template.detail_note_template,
                duration_minutes: template.duration_minutes,
                start_time_local: template.start_time_local,
                is_all_day: template.is_all_day,
                area_id: template.area_id,
            }),
        };

        Ok(dto)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use chrono::{DateTime, Utc};
    use sqlx::{Sqlite, Transaction};

    /// 更新时间块的循环相关字段
    /// 注意：recurrence_parent_id 有外键约束指向 time_blocks(id)，不能设置为循环规则ID
    /// 对于源时间块（第一个实例），recurrence_parent_id 保持 NULL
    pub async fn update_time_block_recurrence_fields(
        tx: &mut Transaction<'_, Sqlite>,
        time_block_id: Uuid,
        rule: &str,
        original_date: &str,
        updated_at: DateTime<Utc>,
    ) -> AppResult<()> {
        let query = r#"
            UPDATE time_blocks
            SET recurrence_rule = ?,
                recurrence_original_date = ?,
                updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(rule)
            .bind(original_date)
            .bind(updated_at)
            .bind(time_block_id.to_string())
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(crate::infra::core::DbError::ConnectionError(e))
            })?;

        Ok(())
    }
}
