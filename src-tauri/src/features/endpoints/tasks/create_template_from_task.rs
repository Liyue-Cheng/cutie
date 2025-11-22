/// 从任务创建模板 - 单文件组件
///
/// POST /api/tasks/:id/to-template
// ==================== CABC 文档 ====================
/*
CABC for `create_template_from_task`

## 1. 端点签名
POST /api/tasks/:id/to-template

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要从现有任务创建模板,以便将任务内容保存为可重复使用的模板

### 2.2 核心业务逻辑
1. 查询任务
2. 提取任务内容创建模板
3. 任务字段映射到模板字段
4. 返回新创建的模板

## 3. 输入输出规范

### 3.1 请求 (Request)
**URL Parameters:**
- `id` (UUID, required): 任务ID

**请求体 (Request Body):** `application/json`
{
  "title": "string (optional, 默认使用任务标题)",
  "category": "GENERAL | RECURRENCE (optional, default: GENERAL)"
}

### 3.2 响应 (Responses)
**201 Created:**
返回完整的 Template DTO

**404 Not Found:**
任务不存在

## 4. 验证规则
- 任务必须存在且未删除

## 5. 业务逻辑详解
1. 查询任务
2. 提取任务字段:
   - title → title (或使用请求中的自定义标题)
   - glance_note → glance_note_template
   - detail_note → detail_note_template
   - estimated_duration → estimated_duration_template
   - subtasks → subtasks_template
   - area_id → area_id
3. 创建模板记录
4. 返回模板DTO

## 6. 边界情况
- 任务不存在: 返回 404
- 任务已删除: 返回 404

## 7. 预期副作用
### 数据库操作:
- SELECT: 查询任务
- INSERT: 创建模板
- 事务边界: begin() → commit()

### SSE 事件:
- template.created

## 8. 契约
### 前置条件:
- 任务存在且未删除

### 后置条件:
- 新模板已创建
- 返回完整的模板DTO
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
    entities::template::{Template, TemplateCategory, TemplateDto},
    features::shared::{
        repositories::{TaskRepository, TemplateSortRepository},
        TransactionHelper,
    },
    infra::{
        core::{AppError, AppResult, DbError},
        http::error_handler::created_response,
        LexoRankService,
    },
    startup::AppState,
};

// ==================== 请求结构体 ====================
#[derive(Debug, Deserialize)]
pub struct CreateTemplateFromTaskRequest {
    /// 自定义模板标题（可选，默认使用任务标题）
    pub title: Option<String>,
    /// 模板类别（可选，默认为 GENERAL）
    pub category: Option<String>,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<CreateTemplateFromTaskRequest>,
) -> Response {
    match logic::execute(&app_state, task_id, request).await {
        Ok(dto) => created_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        task_id: Uuid,
        request: CreateTemplateFromTaskRequest,
    ) -> AppResult<TemplateDto> {
        // 1. 查询任务（使用TaskRepository）
        let task = TaskRepository::find_by_id(app_state.db_pool(), task_id)
            .await?
            .ok_or_else(|| AppError::NotFound {
                entity_type: "Task".to_string(),
                entity_id: task_id.to_string(),
            })?;

        // 2. 解析模板类别
        let category = match request.category.as_deref() {
            Some("RECURRENCE") => TemplateCategory::Recurrence,
            _ => TemplateCategory::General,
        };

        // 3. 获取依赖
        let template_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 4. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 5. 创建模板
        let mut template = Template {
            id: template_id,
            title: request.title.unwrap_or(task.title.clone()),
            glance_note_template: task.glance_note.clone(),
            detail_note_template: task.detail_note.clone(),
            estimated_duration_template: task.estimated_duration,
            subtasks_template: task.subtasks.clone(),
            area_id: task.area_id,
            category,
            sort_rank: None,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        };

        let last_rank = TemplateSortRepository::get_highest_sort_rank_in_tx(&mut tx).await?;
        let new_rank = match last_rank {
            Some(rank) => LexoRankService::generate_between(Some(rank.as_str()), None)?,
            None => LexoRankService::initial_rank(),
        };
        template.sort_rank = Some(new_rank.clone());

        // 6. 插入数据库
        database::insert_in_tx(&mut tx, &template).await?;

        // 7. 写入事件到 outbox（在事务内）
        use crate::infra::events::{
            models::DomainEvent,
            outbox::{EventOutboxRepository, SqlxEventOutboxRepository},
        };

        let outbox_repo = SqlxEventOutboxRepository::new(app_state.db_pool().clone());

        let payload = serde_json::json!({
            "id": template.id,
            "title": template.title,
            "glance_note_template": template.glance_note_template,
            "detail_note_template": template.detail_note_template,
            "estimated_duration_template": template.estimated_duration_template,
            "subtasks_template": template.subtasks_template,
            "area_id": template.area_id,
            "category": template.category.as_str(),
            "sort_rank": template.sort_rank,
            "created_at": template.created_at,
            "updated_at": template.updated_at,
        });

        let event = DomainEvent::new(
            "template.created",
            "Template",
            template.id.to_string(),
            payload,
        );

        outbox_repo.append_in_tx(&mut tx, &event).await?;

        // 8. 提交事务
        TransactionHelper::commit(tx).await?;

        // 9. 组装 DTO
        let dto = TemplateDto {
            id: template.id,
            title: template.title,
            glance_note_template: template.glance_note_template,
            detail_note_template: template.detail_note_template,
            estimated_duration_template: template.estimated_duration_template,
            subtasks_template: template.subtasks_template,
            area_id: template.area_id,
            category: template.category,
            created_at: template.created_at,
            updated_at: template.updated_at,
            sort_rank: template.sort_rank,
        };

        Ok(dto)
    }
}

// ==================== 数据库层 ====================
mod database {
    use super::*;
    use sqlx::Sqlite;
    use sqlx::Transaction;

    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template: &Template,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO templates (
                id, title, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id,
                category, sort_rank, created_at, updated_at, is_deleted
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(template.id.to_string())
        .bind(&template.title)
        .bind(&template.glance_note_template)
        .bind(&template.detail_note_template)
        .bind(template.estimated_duration_template)
        .bind(
            template
                .subtasks_template
                .as_ref()
                .map(|s| serde_json::to_string(s).unwrap()),
        )
        .bind(template.area_id.map(|id| id.to_string()))
        .bind(template.category.as_str())
        .bind(&template.sort_rank)
        .bind(&template.created_at)
        .bind(&template.updated_at)
        .bind(template.is_deleted)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(())
    }
}
