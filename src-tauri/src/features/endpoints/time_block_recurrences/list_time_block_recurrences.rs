/// 查询时间块循环规则列表 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `list_time_block_recurrences`

## 1. 端点签名
GET /api/time-block-recurrences
GET /api/time-block-recurrences?template_id=uuid

## 2. 预期行为简介
查询所有激活的时间块循环规则,或查询某个模板的所有循环规则

## 3. 输入输出规范

### 3.1 请求 (Request)
**Query Parameters:**
- template_id (optional): 过滤特定模板的循环规则

### 3.2 响应 (Responses)
**200 OK:**
[
  {
    "id": "uuid",
    "template_id": "uuid",
    "rule": "string",
    "time_type": "FLOATING | FIXED",
    "start_date": "YYYY-MM-DD | null",
    "end_date": "YYYY-MM-DD | null",
    "skip_conflicts": boolean,
    "is_active": boolean,
    "created_at": "ISO8601",
    "updated_at": "ISO8601",
    "template": { ... } | null
  }
]

## 4. 业务逻辑详解
1. 解析查询参数
2. 根据参数查询循环规则
3. 加载关联的模板信息
4. 返回结果

## 5. 预期副作用
- 无写操作,只读查询
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::{TimeBlockRecurrenceDetailDto, TimeBlockTemplateInfo},
    features::shared::{TimeBlockRecurrenceRepository, TimeBlockTemplateRepository},
    infra::{core::AppResult, http::error_handler::success_response},
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
#[derive(Debug, Deserialize)]
pub struct ListTimeBlockRecurrencesQuery {
    pub template_id: Option<Uuid>,
}

pub async fn handle(
    State(app_state): State<AppState>,
    Query(query): Query<ListTimeBlockRecurrencesQuery>,
) -> Response {
    match logic::execute(&app_state, query).await {
        Ok(dtos) => success_response(dtos).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        query: ListTimeBlockRecurrencesQuery,
    ) -> AppResult<Vec<TimeBlockRecurrenceDetailDto>> {
        // 根据参数查询循环规则
        let recurrences = if let Some(template_id) = query.template_id {
            TimeBlockRecurrenceRepository::find_by_template_id(app_state.db_pool(), template_id)
                .await?
        } else {
            TimeBlockRecurrenceRepository::find_all_active(app_state.db_pool()).await?
        };

        // 组装 DTOs，包含模板信息
        let mut dtos = Vec::with_capacity(recurrences.len());

        for r in recurrences {
            // 加载关联的模板
            let template_info =
                if let Some(template) =
                    TimeBlockTemplateRepository::find_by_id(app_state.db_pool(), r.template_id)
                        .await?
                {
                    Some(TimeBlockTemplateInfo {
                        id: template.id,
                        title: template.title,
                        glance_note_template: template.glance_note_template,
                        detail_note_template: template.detail_note_template,
                        duration_minutes: template.duration_minutes,
                        start_time_local: template.start_time_local,
                        is_all_day: template.is_all_day,
                        area_id: template.area_id,
                    })
                } else {
                    None
                };

            dtos.push(TimeBlockRecurrenceDetailDto {
                id: r.id,
                template_id: r.template_id,
                rule: r.rule,
                time_type: r.time_type,
                start_date: r.start_date,
                end_date: r.end_date,
                timezone: r.timezone,
                skip_conflicts: r.skip_conflicts,
                is_active: r.is_active,
                created_at: r.created_at,
                updated_at: r.updated_at,
                template: template_info,
            });
        }

        Ok(dtos)
    }
}
