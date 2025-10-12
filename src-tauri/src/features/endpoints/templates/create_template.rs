/// 创建模板 - 单文件组件
///
/// ⚠️ 开发前必读:
/// 1. 查看 Schema: migrations/20241001000000_initial_schema.sql
/// 2. 查看共享资源清单: COMPLETE_FEATURE_DEVELOPMENT_GUIDE.md
/// 3. 使用已有的 Repository/Assembler,禁止重复实现
// ==================== CABC 文档 ====================
/*
CABC for `create_template`

## 1. 端点签名
POST /api/templates

## 2. 预期行为简介

### 2.1 用户故事
> 作为用户,我想要创建一个新的任务模板,以便快速创建具有相同结构的任务

### 2.2 核心业务逻辑
创建一个新的模板记录,包含模板标题、各项可选字段和类别

## 3. 输入输出规范

### 3.1 请求 (Request)
{
  "title": "string (required)",
  "glance_note_template": "string (optional)",
  "detail_note_template": "string (optional)",
  "estimated_duration_template": "integer (optional)",
  "subtasks_template": "array (optional)",
  "area_id": "uuid (optional)",
  "category": "GENERAL | RECURRENCE (optional, default: GENERAL)"
}

### 3.2 响应 (Responses)
**201 Created:**
{
  "id": "uuid",
  "title": "string",
  "glance_note_template": "string | null",
  "detail_note_template": "string | null",
  "estimated_duration_template": "integer | null",
  "subtasks_template": "array | null",
  "area_id": "uuid | null",
  "category": "GENERAL | RECURRENCE",
  "created_at": "timestamp",
  "updated_at": "timestamp"
}

## 4. 验证规则
- title: 必须,非空,长度 <= 255
- estimated_duration_template: 如果提供,必须 > 0

## 5. 业务逻辑详解
1. 验证输入
2. 开启事务
3. 生成 UUID 和时间戳
4. 创建 Template 实体
5. 插入数据库
6. 提交事务
7. 组装 DTO
8. 返回结果

## 6. 边界情况
- title 为空: 返回 422
- estimated_duration_template <= 0: 返回 422

## 7. 预期副作用
### 数据库操作:
- INSERT: 1条记录到 templates 表
- 事务边界: begin() → commit()

### SSE 事件:
- template.created

## 8. 契约
### 前置条件:
- request.title 不为空

### 后置条件:
- 数据库中存在新记录
- 返回完整的 TemplateDto

### 不变量:
- id 和 created_at 一旦创建永不改变
*/
// ==================== 依赖引入 ====================
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use crate::{
    entities::template::{CreateTemplateRequest, Template, TemplateCategory, TemplateDto},
    features::shared::TransactionHelper,
    infra::{
        core::{AppError, AppResult, ValidationError},
        http::error_handler::created_response,
    },
    startup::AppState,
};

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTemplateRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(dto) => created_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate_request(request: &CreateTemplateRequest) -> AppResult<()> {
        let mut errors = Vec::new();

        // 验证 title
        if request.title.trim().is_empty() {
            errors.push(ValidationError {
                field: "title".to_string(),
                code: "TITLE_EMPTY".to_string(),
                message: "title cannot be empty".to_string(),
            });
        }

        if request.title.len() > 255 {
            errors.push(ValidationError {
                field: "title".to_string(),
                code: "TITLE_TOO_LONG".to_string(),
                message: "title too long (max 255 characters)".to_string(),
            });
        }

        // 验证 estimated_duration_template
        if let Some(duration) = request.estimated_duration_template {
            if duration <= 0 {
                errors.push(ValidationError {
                    field: "estimated_duration_template".to_string(),
                    code: "DURATION_INVALID".to_string(),
                    message: "estimated_duration_template must be positive".to_string(),
                });
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
        request: CreateTemplateRequest,
    ) -> AppResult<TemplateDto> {
        // 1. 验证
        validation::validate_request(&request)?;

        // 2. 获取依赖
        let id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        // 3. 开启事务
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 4. 创建实体
        let template = Template {
            id,
            title: request.title,
            glance_note_template: request.glance_note_template,
            detail_note_template: request.detail_note_template,
            estimated_duration_template: request.estimated_duration_template,
            subtasks_template: request.subtasks_template,
            area_id: request.area_id,
            category: request.category.unwrap_or(TemplateCategory::General),
            created_at: now,
            updated_at: now,
            is_deleted: false,
        };

        // 5. 插入数据库
        database::insert_in_tx(&mut tx, &template).await?;

        // 6. 写入事件到 outbox（在事务内）
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

        // 7. 提交事务
        TransactionHelper::commit(tx).await?;

        // 8. 组装 DTO
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
        };

        Ok(dto)
    }
}

// ==================== 数据访问层 ====================
mod database {
    use super::*;
    use sqlx::{Sqlite, Transaction};

    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        template: &Template,
    ) -> AppResult<()> {
        let subtasks_json = template
            .subtasks_template
            .as_ref()
            .map(|s| serde_json::to_string(s).unwrap());

        let query = r#"
            INSERT INTO templates (
                id, title, glance_note_template, detail_note_template,
                estimated_duration_template, subtasks_template, area_id, category,
                created_at, updated_at, is_deleted
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(template.id.to_string())
            .bind(&template.title)
            .bind(&template.glance_note_template)
            .bind(&template.detail_note_template)
            .bind(template.estimated_duration_template)
            .bind(subtasks_json)
            .bind(template.area_id.map(|id| id.to_string()))
            .bind(template.category.as_str())
            .bind(template.created_at)
            .bind(template.updated_at)
            .bind(template.is_deleted)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
