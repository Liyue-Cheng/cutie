/// 获取单个时间块循环规则详情 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `get_time_block_recurrence`

## 1. 端点签名
GET /api/time-block-recurrences/:id

## 2. 预期行为简介
获取指定时间块循环规则的详情信息（包含模板摘要）

## 3. 输入输出规范

### 3.1 请求 (Request)
路径参数 `id`：循环规则 ID (UUID)

### 3.2 响应 (Responses)
**200 OK:** TimeBlockRecurrenceDetailDto

## 4. 业务逻辑详解
1. 查询循环规则
2. 查询关联模板
3. 组装详情 DTO

## 5. 预期副作用
- 无写操作
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::{TimeBlockRecurrenceDetailDto, TimeBlockTemplateInfo},
    features::shared::{TimeBlockRecurrenceRepository, TimeBlockTemplateRepository},
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
    ) -> AppResult<TimeBlockRecurrenceDetailDto> {
        // 1. 查询循环规则
        let recurrence = TimeBlockRecurrenceRepository::find_by_id(app_state.db_pool(), recurrence_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "TimeBlockRecurrence".to_string(),
                entity_id: recurrence_id.to_string(),
            })?;

        // 2. 查询关联模板
        let template_info = TimeBlockTemplateRepository::find_by_id(app_state.db_pool(), recurrence.template_id)
            .await?
            .map(|template| TimeBlockTemplateInfo {
                id: template.id,
                title: template.title,
                glance_note_template: template.glance_note_template,
                detail_note_template: template.detail_note_template,
                duration_minutes: template.duration_minutes,
                start_time_local: template.start_time_local,
                is_all_day: template.is_all_day,
                area_id: template.area_id,
            });

        // 3. 组装 DTO
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
            template: template_info,
        };

        Ok(dto)
    }
}
