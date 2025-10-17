/// 获取所有模板列表 - 单文件组件
// ==================== CABC 文档 ====================
/*
CABC for `list_templates`

## 1. 端点签名
GET /api/templates

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要查看所有可用的模板,以便选择合适的模板创建任务

### 2.2 核心业务逻辑
查询数据库中所有未删除的模板,返回模板列表

## 3. 输入输出规范

### 3.1 请求 (Request)
无请求体

### 3.2 响应 (Responses)
**200 OK:**
[
  {
    "id": "uuid",
    "title": "string",
    ...
  }
]

## 4. 验证规则
无

## 5. 业务逻辑详解
1. 查询所有未删除的模板
2. 转换为 DTO 列表
3. 返回结果

## 6. 边界情况
- 无模板: 返回空数组

## 7. 预期副作用
### 数据库操作:
- SELECT: 查询 templates 表

### SSE 事件:
无

## 8. 契约
### 前置条件:
无

### 后置条件:
- 返回所有未删除的模板列表
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::State,
    response::{IntoResponse, Response},
};

use crate::{
    entities::template::{TemplateDto, TemplateRow},
    infra::core::{AppError, AppResult},
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(State(app_state): State<AppState>) -> Response {
    match logic::execute(&app_state).await {
        Ok(dtos) => {
            use crate::infra::http::error_handler::success_response;
            success_response(dtos).into_response()
        }
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState) -> AppResult<Vec<TemplateDto>> {
        // 1. 查询所有模板
        let templates = database::list_all(app_state.db_pool()).await?;

        // 2. 转换为 DTO
        let dtos = templates
            .into_iter()
            .map(|t| TemplateDto {
                id: t.id,
                title: t.title,
                glance_note_template: t.glance_note_template,
                detail_note_template: t.detail_note_template,
                estimated_duration_template: t.estimated_duration_template,
                subtasks_template: t.subtasks_template,
                area_id: t.area_id,
                category: t.category,
                created_at: t.created_at,
                updated_at: t.updated_at,
            })
            .collect();

        Ok(dtos)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use crate::entities::template::Template;
    use sqlx::SqlitePool;

    pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<Template>> {
        let query = r#"
            SELECT
                id, title, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id, category,
                created_at, updated_at, is_deleted
            FROM templates
            WHERE is_deleted = FALSE
            ORDER BY created_at DESC
        "#;

        let rows: Vec<TemplateRow> = sqlx::query_as(query)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        let templates = rows
            .into_iter()
            .map(|row| row.try_into().expect("Failed to convert TemplateRow"))
            .collect();

        Ok(templates)
    }
}
