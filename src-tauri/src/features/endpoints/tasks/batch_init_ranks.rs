/// 批量初始化任务 LexoRank 排序 - 单文件组件
///
/// POST /api/tasks/batch-init-ranks
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    features::shared::{repositories::TaskSortRepository, TransactionHelper},
    infra::LexoRankService,
    infra::{
        core::{AppError, AppResult},
        http::error_handler::success_response,
    },
    startup::AppState,
};

// ==================== 文档层 ====================
/*
CABC for `batch_init_ranks`

## 1. 端点签名

POST /api/tasks/batch-init-ranks

## 2. 预期行为简介

在指定视图中为一批缺少 rank 的任务批量生成 LexoRank 排序值，按照请求中任务 ID 的顺序
写入 `tasks.sort_positions`，仅影响提供的任务。

## 3. 输入输出规范

### 3.1 请求

```json
{
  "view_context": "daily::2025-10-01",
  "task_ids": ["uuid-1", "uuid-2", "uuid-3"]
}
```

任务 ID 列表的顺序代表期望的显示顺序（前 -> 后）。

### 3.2 响应

```json
{
  "view_context": "daily::2025-10-01",
  "assigned": [
    {"task_id": "uuid-1", "new_rank": "0|hzzzzz:"},
    {"task_id": "uuid-2", "new_rank": "0|i00000:"}
  ],
  "updated_at": "2025-11-22T12:34:56Z"
}
```

## 4. 业务流程

1. 校验请求参数
2. 查询视图中现有任务的首个 rank 作为边界
3. 使用 `LexoRankService` 生成一组新的 rank
4. 在事务中依次更新每个任务的 `sort_positions`
5. 返回生成结果
*/

// ==================== 请求 / 响应定义 ====================
#[derive(Debug, Deserialize)]
pub struct BatchInitRanksRequest {
    pub view_context: String,
    pub task_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct TaskRankAssignment {
    pub task_id: Uuid,
    pub new_rank: String,
}

#[derive(Debug, Serialize)]
pub struct BatchInitRanksResponse {
    pub view_context: String,
    pub assigned: Vec<TaskRankAssignment>,
    pub updated_at: String,
}

// ==================== HTTP 处理器 ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<BatchInitRanksRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(response) => success_response(response).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    use super::*;

    pub async fn execute(
        app_state: &AppState,
        request: BatchInitRanksRequest,
    ) -> AppResult<BatchInitRanksResponse> {
        validation::validate(&request)?;

        if request.task_ids.is_empty() {
            return Ok(BatchInitRanksResponse {
                view_context: request.view_context,
                assigned: vec![],
                updated_at: Utc::now().to_rfc3339(),
            });
        }

        let pool = app_state.db_pool();
        let view_context = request.view_context.trim().to_string();

        // 获取视图中首个 rank，作为新 rank 的边界
        let existing_first =
            TaskSortRepository::get_first_rank_in_view(pool, &view_context).await?;

        let assignments = generate_ranks(existing_first, &request.task_ids)?;
        let now = app_state.clock().now_utc();

        let mut tx = TransactionHelper::begin(pool).await?;
        for assignment in &assignments {
            TaskSortRepository::update_task_rank_in_tx(
                &mut tx,
                assignment.task_id,
                &view_context,
                &assignment.new_rank,
                now,
            )
            .await?;
        }
        TransactionHelper::commit(tx).await?;

        Ok(BatchInitRanksResponse {
            view_context,
            assigned: assignments,
            updated_at: now.to_rfc3339(),
        })
    }

    fn generate_ranks(
        existing_first: Option<String>,
        task_ids: &[Uuid],
    ) -> AppResult<Vec<TaskRankAssignment>> {
        if let Some(first_rank) = existing_first {
            // 在现有最前面的任务之前插入，因此需要逆序生成
            let mut generated: Vec<TaskRankAssignment> = Vec::with_capacity(task_ids.len());
            let mut next_anchor = first_rank;
            let mut temp: Vec<TaskRankAssignment> = Vec::with_capacity(task_ids.len());

            for task_id in task_ids.iter().rev() {
                let new_rank = LexoRankService::generate_between(None, Some(&next_anchor))?;
                next_anchor = new_rank.clone();
                temp.push(TaskRankAssignment {
                    task_id: *task_id,
                    new_rank,
                });
            }

            temp.reverse();
            generated.extend(temp);
            Ok(generated)
        } else {
            // 视图为空，从中间开始依次向后生成
            let mut generated: Vec<TaskRankAssignment> = Vec::with_capacity(task_ids.len());
            let mut prev: Option<String> = None;

            for task_id in task_ids {
                let new_rank = LexoRankService::generate_between(prev.as_deref(), None)?;
                prev = Some(new_rank.clone());
                generated.push(TaskRankAssignment {
                    task_id: *task_id,
                    new_rank,
                });
            }

            Ok(generated)
        }
    }
}

// ==================== 验证层 ====================
mod validation {
    use super::*;

    pub fn validate(request: &BatchInitRanksRequest) -> AppResult<()> {
        if request.view_context.trim().is_empty() {
            return Err(AppError::validation_error(
                "view_context",
                "view_context cannot be empty",
                "VIEW_CONTEXT_REQUIRED",
            ));
        }

        if request.task_ids.is_empty() {
            return Ok(());
        }

        Ok(())
    }
}
