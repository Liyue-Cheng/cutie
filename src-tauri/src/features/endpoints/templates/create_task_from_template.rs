/// 从模板创建任务 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `create_task_from_template`

## 1. 端点签名
POST /api/templates/:id/create-task

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要从模板快速创建任务,自动填充模板中的各项内容
> 可选：同时安排到指定日期并设置排序位置（原子操作，避免竞争条件）

### 2.2 核心业务逻辑
1. 查询模板
2. 根据模板内容创建新任务
3. 支持变量替换 (如 {{date}})
4. 可选：创建日程记录
5. 可选：设置排序位置（LexoRank）
6. 返回新创建的任务

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "variables": {
    "date": "2025-10-09",
    "custom_var": "value"
  },
  "scheduled_day": "2025-10-09",           // 可选：安排日期
  "sort_position": {                        // 可选：排序位置
    "view_context": "daily::2025-10-09",
    "prev_task_id": "uuid | null",
    "next_task_id": "uuid | null"
  }
}

### 3.2 响应 (Responses)
**201 Created:**
返回完整的 TaskCardDto（包含 schedules 字段）

**404 Not Found:**
模板不存在

## 4. 验证规则
- 模板必须存在且未删除
- scheduled_day 如果提供，必须是有效的 YYYY-MM-DD 格式
- sort_position.view_context 如果提供，不能为空

## 5. 业务逻辑详解
1. 查询模板
2. 替换模板变量
3. 创建任务
4. 如果提供 scheduled_day，创建日程记录
5. 如果提供 sort_position，计算并设置 LexoRank
6. 返回任务

## 6. 边界情况
- 模板不存在: 返回 404
- 无变量: 直接使用模板内容
- 无 scheduled_day: 任务创建后无日程（进入 staging）
- 无 sort_position: 任务无排序位置

## 7. 预期副作用
### 数据库操作:
- SELECT: 查询模板
- INSERT: 创建任务
- INSERT: 创建日程（如果提供 scheduled_day）
- UPDATE: 更新排序位置（如果提供 sort_position）
- 事务边界: begin() → commit()

### SSE 事件:
- task.created（无日程时）
- task.created_with_schedule（有日程时）

## 8. 契约
### 前置条件:
- 模板存在且未删除

### 后置条件:
- 新任务已创建
- 如果提供 scheduled_day，日程已创建
- 如果提供 sort_position，排序位置已设置
- 返回完整的任务卡片DTO
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    entities::{
        task::{Task, TaskCardDto},
        template::{Template, TemplateRow},
    },
    features::shared::{
        repositories::{TaskRepository, TaskScheduleRepository, TaskSortRepository},
        TaskAssembler, TransactionHelper,
    },
    infra::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
        LexoRankService,
    },
    startup::AppState,
};

// ==================== 请求结构体 ====================
#[derive(Debug, Deserialize)]
pub struct CreateTaskFromTemplateRequest {
    /// 变量替换映射
    #[serde(default)]
    pub variables: std::collections::HashMap<String, String>,
    /// 可选：安排日期（YYYY-MM-DD 格式）
    pub scheduled_day: Option<String>,
    /// 可选：排序位置
    pub sort_position: Option<SortPositionInput>,
}

#[derive(Debug, Deserialize)]
pub struct SortPositionInput {
    /// 视图上下文（如 "daily::2025-10-09"）
    pub view_context: String,
    /// 前一个任务的 ID（null 表示移动到开头）
    pub prev_task_id: Option<Uuid>,
    /// 后一个任务的 ID（null 表示移动到末尾）
    pub next_task_id: Option<Uuid>,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(request): Json<CreateTaskFromTemplateRequest>,
) -> Response {
    match logic::execute(&app_state, template_id, request).await {
        Ok(dto) => created_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        template_id: Uuid,
        request: CreateTaskFromTemplateRequest,
    ) -> AppResult<TaskCardDto> {
        // 0. 验证请求
        validation::validate(&request)?;

        // 1. 查询模板
        let template = database::find_template(app_state.db_pool(), template_id).await?;

        // 2. 替换变量
        let title = replace_variables(&template.title, &request.variables);
        let glance_note = template
            .glance_note_template
            .as_ref()
            .map(|s| replace_variables(s, &request.variables));
        let detail_note = template
            .detail_note_template
            .as_ref()
            .map(|s| replace_variables(s, &request.variables));

        // 3. 获取依赖
        let task_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();
        let pool = app_state.db_pool();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 4. 开启事务
        let mut tx = TransactionHelper::begin(pool).await?;

        // 5. 创建任务
        let source_info_json = serde_json::json!({
            "source_type": "native::from_template",
            "template_id": template_id.to_string(),
            "template_title": template.title,
        });

        let task = Task {
            id: task_id,
            title,
            glance_note,
            detail_note,
            estimated_duration: template.estimated_duration_template,
            subtasks: template.subtasks_template.clone(),
            sort_positions: std::collections::HashMap::new(),
            project_id: None,
            section_id: None,
            area_id: template.area_id,
            due_date: None,
            due_date_type: None,
            completed_at: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
            deleted_at: None,
            source_info: Some(serde_json::from_value(source_info_json).unwrap()),
            external_source_id: None,
            external_source_provider: None,
            external_source_metadata: None,
            recurrence_id: None,
            recurrence_original_date: None,
        };

        TaskRepository::insert_in_tx(&mut tx, &task).await?;

        // 6. 可选：创建日程记录
        let scheduled_day = if let Some(ref day) = request.scheduled_day {
            let parsed = validation::parse_date(day)?;
            TaskScheduleRepository::create_in_tx(&mut tx, task_id, &parsed).await?;
            Some(parsed)
        } else {
            None
        };

        // 7. 可选：设置排序位置
        let sort_position_result = if let Some(ref sort_pos) = request.sort_position {
            let view_context = sort_pos.view_context.trim();

            // 获取相邻任务的 rank
            let prev_rank = if let Some(prev_id) = sort_pos.prev_task_id {
                TaskSortRepository::get_task_rank(pool, prev_id, view_context).await?
            } else {
                None
            };

            let next_rank = if let Some(next_id) = sort_pos.next_task_id {
                TaskSortRepository::get_task_rank(pool, next_id, view_context).await?
            } else {
                None
            };

            // 计算新的 rank
            let new_rank =
                LexoRankService::generate_between(prev_rank.as_deref(), next_rank.as_deref())?;

            // 更新任务排序位置
            TaskSortRepository::update_task_rank_in_tx(&mut tx, task_id, view_context, &new_rank, now)
                .await?;

            Some((view_context.to_string(), new_rank))
        } else {
            None
        };

        // 8. 组装 TaskCardDto
        let mut task_card = TaskAssembler::task_to_card_basic(&task);

        // 8.1 填充排序位置（如果有）
        if let Some((view_context, new_rank)) = sort_position_result {
            task_card.sort_positions.insert(view_context, new_rank);
        }

        // 9. 在事务内填充 schedules 字段
        task_card.schedules = TaskAssembler::assemble_schedules_in_tx(&mut tx, task_id).await?;

        // 9.1 填充 recurrence_expiry_behavior（使用 pool 查询，task_recurrences 表不在事务内修改）
        TaskAssembler::fill_recurrence_expiry_behavior(&mut task_card, pool).await?;

        // 10. 写入领域事件到 outbox
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };
        let outbox_repo = SqlxEventOutboxRepository::new(pool.clone());

        let event_type = if scheduled_day.is_some() {
            "task.created_with_schedule"
        } else {
            "task.created"
        };

        let payload = serde_json::json!({
            "task": task_card,
            "scheduled_day": scheduled_day,
        });

        let event = DomainEvent::new(event_type, "task", task_id.to_string(), payload)
            .with_aggregate_version(now.timestamp_millis());

        outbox_repo.append_in_tx(&mut tx, &event).await?;

        // 11. 提交事务
        TransactionHelper::commit(tx).await?;

        tracing::info!(
            "Task created from template: task_id={}, template_id={}, scheduled_day={:?}",
            task_id,
            template_id,
            scheduled_day
        );

        Ok(task_card)
    }

    /// 替换模板中的变量
    fn replace_variables(
        template: &str,
        variables: &std::collections::HashMap<String, String>,
    ) -> String {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate(request: &CreateTaskFromTemplateRequest) -> AppResult<()> {
        // 验证 scheduled_day 格式
        if let Some(ref day) = request.scheduled_day {
            parse_date(day)?;
        }

        // 验证 sort_position
        if let Some(ref sort_pos) = request.sort_position {
            if sort_pos.view_context.trim().is_empty() {
                return Err(AppError::validation_error(
                    "sort_position.view_context",
                    "view_context cannot be empty",
                    "VIEW_CONTEXT_REQUIRED",
                ));
            }
        }

        Ok(())
    }

    pub fn parse_date(date_str: &str) -> AppResult<String> {
        use crate::infra::core::utils::time_utils;
        time_utils::parse_date_yyyy_mm_dd(date_str)
            .map(|date| time_utils::format_date_yyyy_mm_dd(&date))
            .map_err(|_| {
                AppError::validation_error(
                    "scheduled_day",
                    "日期格式错误，请使用 YYYY-MM-DD 格式",
                    "INVALID_DATE_FORMAT",
                )
            })
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use sqlx::SqlitePool;

    pub async fn find_template(pool: &SqlitePool, id: Uuid) -> AppResult<Template> {
        let query = r#"
            SELECT
                id, title, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id, category,
                sort_rank, created_at, updated_at, is_deleted
            FROM templates
            WHERE id = ? AND is_deleted = FALSE
        "#;

        let row: TemplateRow = sqlx::query_as(query)
            .bind(id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "Template".to_string(),
                entity_id: id.to_string(),
            })?;

        Ok(row.try_into().expect("Failed to convert TemplateRow"))
    }
}
