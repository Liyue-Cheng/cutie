/// 从模板创建任务 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `create_task_from_template`

## 1. 端点签名
POST /api/templates/:id/create-task

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要从模板快速创建任务,自动填充模板中的各项内容

### 2.2 核心业务逻辑
1. 查询模板
2. 根据模板内容创建新任务
3. 支持变量替换 (如 {{date}})
4. 返回新创建的任务

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "variables": {
    "date": "2025-10-09",
    "custom_var": "value"
  }
}

### 3.2 响应 (Responses)
**201 Created:**
返回完整的 TaskCardDto

**404 Not Found:**
模板不存在

## 4. 验证规则
- 模板必须存在且未删除

## 5. 业务逻辑详解
1. 查询模板
2. 替换模板变量
3. 创建任务
4. 返回任务

## 6. 边界情况
- 模板不存在: 返回 404
- 无变量: 直接使用模板内容

## 7. 预期副作用
### 数据库操作:
- SELECT: 查询模板
- INSERT: 创建任务
- 事务边界: begin() → commit()

### SSE 事件:
- task.created

## 8. 契约
### 前置条件:
- 模板存在且未删除

### 后置条件:
- 新任务已创建
- 返回完整的任务卡片DTO
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::{
    entities::{
        task::{Task, TaskCardDto},
        template::{Template, TemplateRow},
    },
    features::{
        shared::repositories::TaskRepository,
        shared::ViewTaskCardAssembler, shared::TransactionHelper,
    },
    infra::{
        core::{AppError, AppResult},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Path(template_id): Path<Uuid>,
    Json(variables): Json<std::collections::HashMap<String, String>>,
) -> Response {
    match logic::execute(&app_state, template_id, variables).await {
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
        variables: std::collections::HashMap<String, String>,
    ) -> AppResult<TaskCardDto> {
        // 1. 查询模板
        let template = database::find_template(app_state.db_pool(), template_id).await?;

        // 2. 替换变量
        let title = replace_variables(&template.title, &variables);
        let glance_note = template
            .glance_note_template
            .as_ref()
            .map(|s| replace_variables(s, &variables));
        let detail_note = template
            .detail_note_template
            .as_ref()
            .map(|s| replace_variables(s, &variables));

        // 3. 获取依赖
        let task_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // ✅ 获取写入许可，确保写操作串行执行
        let _permit = app_state.acquire_write_permit().await;

        // 4. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

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
            project_id: None,
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

        // 6. 提交事务
        TransactionHelper::commit(tx).await?;

        // 7. 组装完整的 TaskCardDto
        let task_card = ViewTaskCardAssembler::assemble_full(&task, app_state.db_pool()).await?;

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

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use sqlx::SqlitePool;

    pub async fn find_template(pool: &SqlitePool, id: Uuid) -> AppResult<Template> {
        let query = r#"
            SELECT
                id, title, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id, category,
                created_at, updated_at, is_deleted
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
