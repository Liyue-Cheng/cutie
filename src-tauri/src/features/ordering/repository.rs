/// 排序数据访问层
///
/// 实现排序的数据库操作
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::shared::{
    core::{AppResult, ContextType, DbError, Ordering},
    database::{OrderingRepository, Repository},
};

/// 排序仓库的SQLx实现
#[derive(Clone)]
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
            _ => ContextType::Misc, // 默认值
        };

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
            context_id: row.try_get("context_id")?,
            task_id: Uuid::parse_str(&row.try_get::<String, _>("task_id")?).map_err(|_| {
                sqlx::Error::ColumnDecode {
                    index: "task_id".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid UUID",
                    )),
                }
            })?,
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
impl Repository<Ordering> for SqlxOrderingRepository {
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Ordering>> {
        let row = sqlx::query("SELECT * FROM ordering WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        match row {
            Some(row) => {
                let ordering = Self::row_to_ordering(&row).map_err(DbError::ConnectionError)?;
                Ok(Some(ordering))
            }
            None => Ok(None),
        }
    }

    async fn create(&self, ordering: &Ordering) -> AppResult<Ordering> {
        let params = Self::ordering_to_params(ordering);

        sqlx::query(
            r#"
            INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.0) // id
        .bind(&params.1) // context_type
        .bind(&params.2) // context_id
        .bind(&params.3) // task_id
        .bind(&params.4) // sort_order
        .bind(&params.5) // updated_at
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(ordering.clone())
    }

    async fn update(&self, ordering: &Ordering) -> AppResult<Ordering> {
        let params = Self::ordering_to_params(ordering);

        let result = sqlx::query(
            r#"
            UPDATE orderings SET
                context_type = ?, context_id = ?, task_id = ?, sort_order = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&params.1) // context_type
        .bind(&params.2) // context_id
        .bind(&params.3) // task_id
        .bind(&params.4) // sort_order
        .bind(&params.5) // updated_at
        .bind(&params.0) // id
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "Ordering",
                ordering.id.to_string(),
            ));
        }

        Ok(ordering.clone())
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM orderings WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "Ordering",
                id.to_string(),
            ));
        }

        Ok(())
    }

    async fn find_all(&self) -> AppResult<Vec<Ordering>> {
        let rows = sqlx::query("SELECT * FROM orderings ORDER BY sort_order ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        let orderings = rows
            .iter()
            .map(Self::row_to_ordering)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(orderings)
    }
}

#[async_trait]
impl OrderingRepository for SqlxOrderingRepository {
    async fn find_by_context(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> AppResult<Vec<Ordering>> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        let rows = sqlx::query(
            r#"
            SELECT * FROM orderings 
            WHERE context_type = ? AND context_id = ?
            ORDER BY sort_order ASC
            "#,
        )
        .bind(context_type_str)
        .bind(context_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let orderings = rows
            .iter()
            .map(Self::row_to_ordering)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(orderings)
    }

    async fn find_by_task_id(&self, task_id: Uuid) -> AppResult<Vec<Ordering>> {
        let rows =
            sqlx::query("SELECT * FROM orderings WHERE task_id = ? ORDER BY updated_at DESC")
                .bind(task_id.to_string())
                .fetch_all(&self.pool)
                .await
                .map_err(DbError::ConnectionError)?;

        let orderings = rows
            .iter()
            .map(Self::row_to_ordering)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(orderings)
    }

    async fn update_sort_order(
        &self,
        context_type: &ContextType,
        context_id: &str,
        task_id: Uuid,
        new_sort_order: &str,
    ) -> AppResult<()> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        let result = sqlx::query(
            r#"
            UPDATE orderings SET sort_order = ?, updated_at = ?
            WHERE context_type = ? AND context_id = ? AND task_id = ?
            "#,
        )
        .bind(new_sort_order)
        .bind(Utc::now().to_rfc3339())
        .bind(context_type_str)
        .bind(context_id)
        .bind(task_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            // 如果记录不存在，创建新的排序记录
            let ordering = Ordering::new(
                Uuid::new_v4(),
                context_type.clone(),
                context_id.to_string(),
                task_id,
                new_sort_order.to_string(),
                Utc::now(),
            )?;

            self.create(&ordering).await?;
        }

        Ok(())
    }

    async fn batch_update(&self, orderings: &[Ordering]) -> AppResult<()> {
        // 使用事务进行批量更新
        let mut tx = self.pool.begin().await.map_err(DbError::ConnectionError)?;

        for ordering in orderings {
            let params = Self::ordering_to_params(ordering);

            sqlx::query(
                r#"
                INSERT OR REPLACE INTO orderings (id, context_type, context_id, task_id, sort_order, updated_at)
                VALUES (?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&params.0) // id
            .bind(&params.1) // context_type
            .bind(&params.2) // context_id
            .bind(&params.3) // task_id
            .bind(&params.4) // sort_order
            .bind(&params.5) // updated_at
            .execute(&mut *tx)
            .await
            .map_err(DbError::ConnectionError)?;
        }

        tx.commit().await.map_err(DbError::ConnectionError)?;
        Ok(())
    }

    async fn clear_context(&self, context_type: &ContextType, context_id: &str) -> AppResult<()> {
        let context_type_str = match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        };

        sqlx::query("DELETE FROM orderings WHERE context_type = ? AND context_id = ?")
            .bind(context_type_str)
            .bind(context_id)
            .execute(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        Ok(())
    }

    async fn delete_by_task_id(&self, task_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM orderings WHERE task_id = ?")
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::database::connection::create_test_database;

    #[tokio::test]
    async fn test_ordering_crud_operations() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxOrderingRepository::new(pool);

        // 创建测试排序记录
        let ordering = Ordering::new(
            Uuid::new_v4(),
            ContextType::Misc,
            "floating".to_string(),
            Uuid::new_v4(),
            "n".to_string(),
            Utc::now(),
        )
        .unwrap();

        // 测试创建
        let created_ordering = repo.create(&ordering).await.unwrap();
        assert_eq!(created_ordering.task_id, ordering.task_id);

        // 测试查找
        let found_ordering = repo.find_by_id(ordering.id).await.unwrap().unwrap();
        assert_eq!(found_ordering.id, ordering.id);

        // 测试更新
        let mut updated_ordering = found_ordering.clone();
        updated_ordering.update_sort_order("o".to_string(), Utc::now());

        let updated = repo.update(&updated_ordering).await.unwrap();
        assert_eq!(updated.sort_order, "o");

        // 测试删除
        repo.delete(ordering.id).await.unwrap();
        let deleted_ordering = repo.find_by_id(ordering.id).await.unwrap();
        assert!(deleted_ordering.is_none());
    }

    #[tokio::test]
    async fn test_find_by_context() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxOrderingRepository::new(pool);

        let context_type = ContextType::Misc;
        let context_id = "floating";

        // 创建多个排序记录
        for i in 0..3 {
            let ordering = Ordering::new(
                Uuid::new_v4(),
                context_type.clone(),
                context_id.to_string(),
                Uuid::new_v4(),
                format!("order_{}", i),
                Utc::now(),
            )
            .unwrap();
            repo.create(&ordering).await.unwrap();
        }

        // 测试按上下文查找
        let orderings = repo
            .find_by_context(&context_type, context_id)
            .await
            .unwrap();
        assert_eq!(orderings.len(), 3);
        assert!(orderings.iter().all(|o| o.context_type == context_type));
        assert!(orderings.iter().all(|o| o.context_id == context_id));
    }

    #[tokio::test]
    async fn test_batch_update() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxOrderingRepository::new(pool);

        // 创建多个排序记录
        let mut orderings = Vec::new();
        for i in 0..3 {
            let ordering = Ordering::new(
                Uuid::new_v4(),
                ContextType::Misc,
                "floating".to_string(),
                Uuid::new_v4(),
                format!("order_{}", i),
                Utc::now(),
            )
            .unwrap();
            orderings.push(ordering);
        }

        // 批量更新
        repo.batch_update(&orderings).await.unwrap();

        // 验证记录已创建
        for ordering in &orderings {
            let found = repo.find_by_id(ordering.id).await.unwrap();
            assert!(found.is_some());
        }
    }

    #[tokio::test]
    async fn test_clear_context() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxOrderingRepository::new(pool);

        let context_type = ContextType::Misc;
        let context_id = "test_context";

        // 创建排序记录
        let ordering = Ordering::new(
            Uuid::new_v4(),
            context_type.clone(),
            context_id.to_string(),
            Uuid::new_v4(),
            "n".to_string(),
            Utc::now(),
        )
        .unwrap();
        repo.create(&ordering).await.unwrap();

        // 清理上下文
        repo.clear_context(&context_type, context_id).await.unwrap();

        // 验证记录已删除
        let orderings = repo
            .find_by_context(&context_type, context_id)
            .await
            .unwrap();
        assert!(orderings.is_empty());
    }
}
