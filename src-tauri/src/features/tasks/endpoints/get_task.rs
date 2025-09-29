/// 获取任务详情 API - 单文件组件
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use uuid::Uuid;

use crate::{
    entities::Task,
    shared::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

use super::super::shared::dtos::TaskResponse;
use crate::repositories::TaskRepository;

// ==================== 文档层 (Documentation Layer) ====================
/*
CABC for `get_task`

## API端点
GET /api/tasks/{id}

## 预期行为简介
根据任务ID获取单个任务的详细信息。

## 输入输出规范
- **前置条件**:
  - task_id必须是有效的UUID格式
- **后置条件**:
  - 如果任务存在且未删除，返回任务详情
  - 如果任务不存在，返回404错误

## 边界情况
- UUID格式无效：返回400 Bad Request
- 任务不存在：返回404 Not Found
- 任务已删除：返回404 Not Found
- 数据库错误：返回500 Internal Server Error

## 预期副作用
- 无副作用，只读操作

## 事务保证
- 单次查询，无需事务
*/

// ==================== 路由层 (Router Layer) ====================
/// 获取任务详情的HTTP处理器
pub async fn handle(State(app_state): State<AppState>, Path(task_id): Path<Uuid>) -> Response {
    match logic::execute(app_state.task_repository(), task_id).await {
        Ok(task) => success_response(TaskResponse::from(task)).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务层 (Service/Logic Layer) ====================
/// 获取任务的核心业务逻辑
pub mod logic {
    use super::*;

    /// 执行获取任务的业务逻辑
    pub async fn execute(task_repo: &dyn TaskRepository, task_id: Uuid) -> AppResult<Task> {
        // 1. 从数据库获取任务
        let task = task_repo
            .find_by_id(task_id)
            .await?
            .ok_or_else(|| AppError::not_found("Task", task_id.to_string()))?;

        // 2. 验证任务状态（可选的业务规则）
        if task.is_deleted {
            return Err(AppError::not_found("Task", task_id.to_string()));
        }

        Ok(task)
    }
}
