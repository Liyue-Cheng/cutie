/// 添加任务日程 API - 单文件组件
///
/// POST /api/tasks/:id/schedules
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    entities::TaskCardDto,
    features::tasks::shared::{
        repositories::{TaskRepository, TaskScheduleRepository},
        TaskAssembler,
    },
    shared::{
        core::{AppError, AppResult},
        http::{error_handler::created_response, extractors::extract_correlation_id},
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `add_schedule`

## API端点
POST /api/tasks/:id/schedules

## 预期行为简介
为任务添加日程安排，指定任务在某天需要完成。

## 业务逻辑
1. 验证任务存在且未删除
2. 验证该日期还没有日程记录
3. 创建 schedule 记录（outcome = 'PLANNED'）
4. 如果是任务的第一个日程，更新 schedule_status 为 'planned'
5. 通过 SSE 推送 task.scheduled 事件

## 输入输出规范
- **前置条件**:
  - 任务存在且未删除
  - scheduled_day 是有效日期
  - 该日期还没有 schedule 记录
- **后置条件**:
  - 插入 task_schedules 记录
  - 更新任务的 schedule_status（如果需要）

## 边界情况
- 任务不存在 → 404
- 该日期已有日程 → 409 Conflict
- 日期格式错误 → 400
*/

// ==================== 请求/响应结构体 ====================
#[derive(Debug, Deserialize)]
pub struct AddScheduleRequest {
    /// 安排日期（YYYY-MM-DD 格式）
    pub scheduled_day: String,
}

#[derive(Debug, Serialize)]
pub struct AddScheduleResponse {
    pub task_card: TaskCardDto,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<AddScheduleRequest>,
) -> Response {
    let correlation_id = extract_correlation_id(&headers);
    match logic::execute(&app_state, task_id, request, correlation_id).await {
        Ok(response) => created_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn parse_date(date_str: &str) -> AppResult<chrono::DateTime<Utc>> {
        let naive_date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
            AppError::validation_error(
                "scheduled_day",
                "日期格式错误，请使用 YYYY-MM-DD 格式",
                "INVALID_DATE_FORMAT",
            )
        })?;

        // 🔧 FIX: 直接使用 NaiveDate 转换为 UTC 零点
        // 因为前端传递的日期已经是"用户本地日期"，不需要再做时区转换
        use crate::shared::core::utils::time_utils::local_date_to_utc_midnight;
        Ok(local_date_to_utc_midnight(naive_date))
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;
    use crate::features::shared::TransactionHelper;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        request: AddScheduleRequest,
        correlation_id: Option<String>,
    ) -> AppResult<AddScheduleResponse> {
        let now = app_state.clock().now_utc();

        // 1. 解析日期
        let scheduled_day = validation::parse_date(&request.scheduled_day)?;

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 2. 开始事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. 检查任务是否存在
        let _task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 4. 检查该日期是否已有日程
        let has_schedule =
            TaskScheduleRepository::has_schedule_for_day_in_tx(&mut tx, task_id, scheduled_day)
                .await?;

        if has_schedule {
            return Err(AppError::conflict("该日期已有日程安排"));
        }

        // 5. 创建日程记录
        TaskScheduleRepository::create_in_tx(&mut tx, task_id, scheduled_day).await?;

        // 6. 重新查询任务并组装 TaskCard
        // 注意：schedule_status 是派生字段，由装配器根据 task_schedules 表计算
        let updated_task = TaskRepository::find_by_id_in_tx(&mut tx, task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        let mut task_card = TaskAssembler::task_to_card_basic(&updated_task);

        // 7. ✅ 在事务内填充 schedules 字段
        // ⚠️ 必须在写入 SSE 之前填充，确保 SSE 和 HTTP 返回的数据一致！
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 8. ✅ 根据 schedules 设置正确的 schedule_status
        // staging 定义：今天和未来没有排期的任务，过去的排期不影响
        use crate::entities::ScheduleStatus;
        use chrono::Utc;
        let today = Utc::now().date_naive();
        
        let has_future_schedule = task_card.schedules.as_ref().map(|schedules| {
            schedules.iter().any(|s| {
                if let Ok(schedule_date) = chrono::NaiveDate::parse_from_str(&s.scheduled_day, "%Y-%m-%d") {
                    schedule_date >= today
                } else {
                    false
                }
            })
        }).unwrap_or(false);
        
        task_card.schedule_status = if has_future_schedule {
            ScheduleStatus::Scheduled
        } else {
            ScheduleStatus::Staging
        };

        // 8. 写入领域事件到 outbox
        use crate::shared::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let payload = serde_json::json!({
            "task": task_card,
            "scheduled_day": request.scheduled_day,
        });

        let mut event = DomainEvent::new("task.scheduled", "task", task_id.to_string(), payload)
            .with_aggregate_version(now.timestamp_millis());

        if let Some(cid) = correlation_id {
            event = event.with_correlation_id(cid);
        }

        outbox_repo.append_in_tx(&mut tx, &event).await?;

        // 9. 提交事务
        TransactionHelper::commit(tx).await?;

        // 10. 返回结果
        Ok(AddScheduleResponse { task_card })
    }
}

// ==================== 数据访问层 ====================
// ✅ 所有数据库操作已迁移到共享 Repository
// schedule_status 是派生字段，不存储在数据库中，由装配器根据 task_schedules 表计算
