/// 更新时间块循环规则 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `update_time_block_recurrence`

## 1. 端点签名
PATCH /api/time-block-recurrences/:id

## 2. 预期行为简介
更新指定的时间块循环规则

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "rule": "string (optional)",
  "start_date": "YYYY-MM-DD | null (optional)",
  "end_date": "YYYY-MM-DD | null (optional)",
  "skip_conflicts": "boolean (optional)",
  "is_active": "boolean (optional)"
}

### 3.2 响应 (Responses)
**200 OK:** TimeBlockRecurrenceDto

## 4. 预期副作用
- UPDATE: 1条记录到 time_block_recurrences 表
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    entities::{TimeBlockRecurrenceDto, UpdateTimeBlockRecurrenceRequest},
    features::shared::{TimeBlockRecurrenceRepository, TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTimeBlockRecurrenceRequest>,
) -> Response {
    match logic::execute(&app_state, id, request).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        recurrence_id: Uuid,
        request: UpdateTimeBlockRecurrenceRequest,
    ) -> AppResult<TimeBlockRecurrenceDto> {
        // 1. 验证循环规则是否存在
        let existing =
            TimeBlockRecurrenceRepository::find_by_id(app_state.db_pool(), recurrence_id).await?;
        if existing.is_none() {
            return Err(AppError::NotFound {
                entity_type: "TimeBlockRecurrence".to_string(),
                entity_id: recurrence_id.to_string(),
            });
        }

        // 2. 获取依赖
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        // 3. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 4. 更新循环规则
        let updated =
            TimeBlockRecurrenceRepository::update_in_tx(&mut tx, recurrence_id, &request, now)
                .await?;

        // 5. 提交事务
        TransactionHelper::commit(tx).await?;

        // 6. 组装 DTO
        let dto = TimeBlockRecurrenceDto {
            id: updated.id,
            template_id: updated.template_id,
            rule: updated.rule,
            time_type: updated.time_type,
            start_date: updated.start_date,
            end_date: updated.end_date,
            timezone: updated.timezone,
            skip_conflicts: updated.skip_conflicts,
            is_active: updated.is_active,
            created_at: updated.created_at,
            updated_at: updated.updated_at,
        };

        Ok(dto)
    }
}
