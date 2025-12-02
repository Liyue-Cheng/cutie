/// 继续时间块循环 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `resume_time_block_recurrence`

## 1. 端点签名
POST /api/time-block-recurrences/:id/resume

## 2. 预期行为简介
继续已停止的时间块循环规则，清除 end_date，从原 end_date 开始继续生成新实例

## 3. 输入输出规范

### 3.1 请求 (Request)
无请求体（或空对象）

### 3.2 响应 (Responses)
**200 OK:** TimeBlockRecurrenceDto

## 4. 业务逻辑详解
1. 验证规则存在
2. 验证规则当前有 end_date（即已停止状态）
3. 清除 end_date（设置为 null）
4. 返回更新后的循环规则
   （新实例将在下次查询时通过懒加载机制自动生成）

## 5. 预期副作用
- UPDATE: time_block_recurrences 表 (清除 end_date)
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::TimeBlockRecurrenceDto,
    features::shared::{TimeBlockRecurrenceRepository, TransactionHelper},
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>, Path(id): Path<Uuid>) -> Response {
    match logic::execute(&app_state, id).await {
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
    ) -> AppResult<TimeBlockRecurrenceDto> {
        // 1. 验证循环规则是否存在
        let existing =
            TimeBlockRecurrenceRepository::find_by_id(app_state.db_pool(), recurrence_id).await?;
        let recurrence = existing.ok_or_else(|| AppError::NotFound {
            entity_type: "TimeBlockRecurrence".to_string(),
            entity_id: recurrence_id.to_string(),
        })?;

        // 2. 验证规则当前有 end_date
        if recurrence.end_date.is_none() {
            return Err(AppError::validation_error(
                "end_date",
                "此循环规则尚未停止，无需继续",
                "RECURRENCE_NOT_STOPPED",
            ));
        }

        // 3. 获取依赖
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可
        let _permit = app_state.acquire_write_permit().await;

        // 4. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        tracing::info!(
            "▶️ [RESUME_TB_RECURRENCE] Resuming recurrence {}, clearing end_date from {:?}",
            recurrence_id,
            recurrence.end_date
        );

        // 5. 更新循环规则，清除 end_date
        // 使用 Some(None) 表示显式设置 end_date 为 NULL
        let update_request = crate::entities::UpdateTimeBlockRecurrenceRequest {
            template_id: None,
            rule: None,
            time_type: None,
            start_date: None,
            end_date: Some(None), // 显式设置为 NULL
            timezone: None,
            skip_conflicts: None,
            is_active: Some(true), // 同时确保激活状态
        };
        let updated = TimeBlockRecurrenceRepository::update_in_tx(
            &mut tx,
            recurrence_id,
            &update_request,
            now,
        )
        .await?;

        // 6. 提交事务
        TransactionHelper::commit(tx).await?;

        tracing::info!(
            "▶️ [RESUME_TB_RECURRENCE] Successfully resumed recurrence {}",
            recurrence_id
        );

        // 7. 组装 DTO
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
