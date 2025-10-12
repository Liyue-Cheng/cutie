/// 关联任务摘要装配器
use uuid::Uuid;

use crate::{
    entities::LinkedTaskSummary,
    infra::core::{AppError, AppResult, DbError},
};

pub struct LinkedTaskAssembler;

impl LinkedTaskAssembler {
    /// 批量获取任务摘要
    pub async fn get_summaries_batch(
        executor: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
        task_ids: &[Uuid],
    ) -> AppResult<Vec<LinkedTaskSummary>> {
        if task_ids.is_empty() {
            return Ok(Vec::new());
        }

        // 构建 IN 查询
        let placeholders = task_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            r#"
            SELECT id, title, completed_at
            FROM tasks
            WHERE id IN ({}) AND deleted_at IS NULL
            "#,
            placeholders
        );

        let mut query_builder = sqlx::query_as::<_, (String, String, Option<String>)>(&query);
        for task_id in task_ids {
            query_builder = query_builder.bind(task_id.to_string());
        }

        let rows = query_builder
            .fetch_all(executor)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let summaries = rows
            .into_iter()
            .map(|(id, title, completed_at)| {
                let parsed_id =
                    Uuid::parse_str(&id).map_err(|e| AppError::StringError(e.to_string()))?;
                Ok(LinkedTaskSummary {
                    id: parsed_id,
                    title,
                    is_completed: completed_at.is_some(),
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(summaries)
    }

    /// 获取时间块关联的任务摘要
    pub async fn get_for_time_block(
        executor: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
        block_id: Uuid,
    ) -> AppResult<Vec<LinkedTaskSummary>> {
        let query = r#"
            SELECT t.id, t.title, t.completed_at
            FROM tasks t
            INNER JOIN task_time_block_links ttbl ON t.id = ttbl.task_id
            WHERE ttbl.time_block_id = ? AND t.deleted_at IS NULL
            ORDER BY t.created_at ASC
        "#;

        let rows = sqlx::query_as::<_, (String, String, Option<String>)>(query)
            .bind(block_id.to_string())
            .fetch_all(executor)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let summaries = rows
            .into_iter()
            .map(|(id, title, completed_at)| {
                let parsed_id =
                    Uuid::parse_str(&id).map_err(|e| AppError::StringError(e.to_string()))?;
                Ok(LinkedTaskSummary {
                    id: parsed_id,
                    title,
                    is_completed: completed_at.is_some(),
                })
            })
            .collect::<AppResult<Vec<_>>>()?;

        Ok(summaries)
    }
}
