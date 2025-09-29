use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::{OrderingRepository, Transaction};
use crate::common::error::DbError;
use crate::common::utils::sort_order_utils::{get_mid_lexo_rank, get_rank_after};
use crate::core::models::{ContextType, Ordering};

/// OrderingRepository的SQLx实现
pub struct SqlxOrderingRepository {
    pool: SqlitePool,
}

impl SqlxOrderingRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为Ordering对象
    fn row_to_ordering(row: &sqlx::sqlite::SqliteRow) -> Result<Ordering, sqlx::Error> {
        let context_type_str: String = row.try_get("context_type")?;
        let context_type = match context_type_str.as_str() {
            "DAILY_KANBAN" => ContextType::DailyKanban,
            "PROJECT_LIST" => ContextType::ProjectList,
            "AREA_FILTER" => ContextType::AreaFilter,
            "MISC" => ContextType::Misc,
            _ => {
                return Err(sqlx::Error::ColumnDecode {
                    index: "context_type".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid context type",
                    )),
                })
            }
        };

        let context_id: String = row.try_get("context_id")?;
        let task_id = Uuid::parse_str(&row.try_get::<String, _>("task_id")?).map_err(|_| {
            sqlx::Error::ColumnDecode {
                index: "task_id".to_string(),
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid UUID",
                )),
            }
        })?;

        // 验证context_id格式
        if let Err(e) = Ordering::validate_context_id(&context_type, &context_id) {
            return Err(sqlx::Error::ColumnDecode {
                index: "context_id".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
            });
        }

        Ok(Ordering {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|_| {
                sqlx::Error::ColumnDecode {
                    index: "id".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid UUID",
                    )),
                }
            })?,
            context_type,
            context_id,
            task_id,
            sort_order: row.try_get("sort_order")?,
            updated_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "updated_at".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&Utc),
        })
    }

    /// 将Ordering对象转换为数据库参数
    fn ordering_to_params(ordering: &Ordering) -> (String, String, String, String, String, String) {
        let context_type_str = match ordering.context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        (
            ordering.id.to_string(),
            context_type_str.to_string(),
            ordering.context_id.clone(),
            ordering.task_id.to_string(),
            ordering.sort_order.clone(),
            ordering.updated_at.to_rfc3339(),
        )
    }
}

#[async_trait]
impl OrderingRepository for SqlxOrderingRepository {
    async fn upsert(
        &self,
        tx: &mut Transaction<'_>,
        ordering: &Ordering,
    ) -> Result<Ordering, DbError> {
        let params = Self::ordering_to_params(ordering);

        // 先尝试更新
        let update_result = sqlx::query(
            r#"
            UPDATE ordering 
            SET sort_order = ?, updated_at = ?
            WHERE context_type = ? AND context_id = ? AND task_id = ?
            "#,
        )
        .bind(&params.4)
        .bind(&params.5)
        .bind(&params.1)
        .bind(&params.2)
        .bind(&params.3)
        .execute(&mut **tx)
        .await;

        match update_result {
            Ok(query_result) => {
                if query_result.rows_affected() > 0 {
                    // 更新成功
                    Ok(ordering.clone())
                } else {
                    // 没有更新任何行，执行插入
                    let insert_result = sqlx::query(
                        r#"
                        INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at)
                        VALUES (?, ?, ?, ?, ?, ?)
                        "#
                    )
                    .bind(&params.0).bind(&params.1).bind(&params.2)
                    .bind(&params.3).bind(&params.4).bind(&params.5)
                    .execute(&mut **tx)
                    .await;

                    match insert_result {
                        Ok(_) => Ok(ordering.clone()),
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn delete_for_task_in_context(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<(), DbError> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        let result = sqlx::query(
            "DELETE FROM ordering WHERE task_id = ? AND context_type = ? AND context_id = ?",
        )
        .bind(task_id.to_string())
        .bind(context_type_str)
        .bind(context_id)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_for_context(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<Vec<Ordering>, DbError> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        let result = sqlx::query(
            "SELECT * FROM ordering WHERE context_type = ? AND context_id = ? ORDER BY sort_order ASC"
        )
        .bind(context_type_str)
        .bind(context_id)
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let orderings: Result<Vec<Ordering>, _> =
                    rows.iter().map(|row| Self::row_to_ordering(row)).collect();
                orderings.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn batch_upsert(
        &self,
        tx: &mut Transaction<'_>,
        orderings: &[Ordering],
    ) -> Result<Vec<Ordering>, DbError> {
        let mut results = Vec::new();

        for ordering in orderings {
            let result = self.upsert(tx, ordering).await?;
            results.push(result);
        }

        Ok(results)
    }

    async fn delete_all_for_task(
        &self,
        tx: &mut Transaction<'_>,
        task_id: Uuid,
    ) -> Result<(), DbError> {
        let result = sqlx::query("DELETE FROM ordering WHERE task_id = ?")
            .bind(task_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn clear_context(
        &self,
        tx: &mut Transaction<'_>,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<(), DbError> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        let result = sqlx::query("DELETE FROM ordering WHERE context_type = ? AND context_id = ?")
            .bind(context_type_str)
            .bind(context_id)
            .execute(&mut **tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_for_task(&self, task_id: Uuid) -> Result<Vec<Ordering>, DbError> {
        let result =
            sqlx::query("SELECT * FROM ordering WHERE task_id = ? ORDER BY updated_at DESC")
                .bind(task_id.to_string())
                .fetch_all(&self.pool)
                .await;

        match result {
            Ok(rows) => {
                let orderings: Result<Vec<Ordering>, _> =
                    rows.iter().map(|row| Self::row_to_ordering(row)).collect();
                orderings.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn get_next_sort_order(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<String, DbError> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        // 获取当前上下文中最大的sort_order
        let result = sqlx::query(
            "SELECT sort_order FROM ordering WHERE context_type = ? AND context_id = ? ORDER BY sort_order DESC LIMIT 1"
        )
        .bind(context_type_str)
        .bind(context_id)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let last_sort_order: String = row
                    .try_get("sort_order")
                    .map_err(DbError::ConnectionError)?;
                Ok(get_rank_after(&last_sort_order))
            }
            Ok(None) => {
                // 没有现有排序，返回初始排序
                Ok(crate::common::utils::sort_order_utils::generate_initial_sort_order())
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn get_sort_order_between(
        &self,
        context_type: &ContextType,
        context_id: &str,
        prev_sort_order: Option<&str>,
        next_sort_order: Option<&str>,
    ) -> Result<String, DbError> {
        match (prev_sort_order, next_sort_order) {
            (Some(prev), Some(next)) => Ok(get_mid_lexo_rank(prev, next)),
            (Some(prev), None) => Ok(get_rank_after(prev)),
            (None, Some(next)) => Ok(crate::common::utils::sort_order_utils::get_rank_before(
                next,
            )),
            (None, None) => {
                // 没有参考点，获取下一个排序位置
                self.get_next_sort_order(context_type, context_id).await
            }
        }
    }

    async fn count_tasks_in_context(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> Result<i64, DbError> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        let result = sqlx::query(
            "SELECT COUNT(*) as count FROM ordering WHERE context_type = ? AND context_id = ?",
        )
        .bind(context_type_str)
        .bind(context_id)
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => Ok(row.try_get("count").map_err(DbError::ConnectionError)?),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }
}
