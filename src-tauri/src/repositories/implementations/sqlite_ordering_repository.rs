/// OrderingRepository的SQLite实现
///
/// 提供Ordering实体的具体数据库操作实现
use async_trait::async_trait;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::entities::{ContextType, Ordering};
use crate::repositories::traits::OrderingRepository;
use crate::shared::core::{AppResult, DbError};

/// 排序仓库的SQLite实现
#[derive(Clone)]
pub struct SqliteOrderingRepository {
    pool: SqlitePool,
}

impl SqliteOrderingRepository {
    /// 创建新的OrderingRepository实例
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
                        "Invalid context_type value",
                    )),
                })
            }
        };

        let updated_at =
            chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "updated_at".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&chrono::Utc);

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
            updated_at,
        })
    }

    /// 将ContextType转换为数据库字符串
    fn context_type_to_string(context_type: &ContextType) -> &'static str {
        match context_type {
            ContextType::DailyKanban => "DAILY_KANBAN",
            ContextType::ProjectList => "PROJECT_LIST",
            ContextType::AreaFilter => "AREA_FILTER",
            ContextType::Misc => "MISC",
        }
    }
}

#[async_trait]
impl OrderingRepository for SqliteOrderingRepository {
    // --- 写操作 ---
    async fn upsert(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        ordering: &Ordering,
    ) -> AppResult<Ordering> {
        // 先尝试更新
        let update_result = sqlx::query(
            r#"
            UPDATE ordering SET 
                sort_order = ?, 
                updated_at = ? 
            WHERE context_type = ? AND context_id = ? AND task_id = ?
            "#,
        )
        .bind(&ordering.sort_order)
        .bind(ordering.updated_at.to_rfc3339())
        .bind(Self::context_type_to_string(&ordering.context_type))
        .bind(&ordering.context_id)
        .bind(ordering.task_id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if update_result.rows_affected() > 0 {
            // 更新成功，返回更新后的对象
            return Ok(ordering.clone());
        }

        // 更新失败，执行插入
        sqlx::query(
            r#"
            INSERT INTO ordering (
                id, context_type, context_id, task_id, sort_order, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ordering.id.to_string())
        .bind(Self::context_type_to_string(&ordering.context_type))
        .bind(&ordering.context_id)
        .bind(ordering.task_id.to_string())
        .bind(&ordering.sort_order)
        .bind(ordering.updated_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(ordering.clone())
    }

    async fn delete_for_task_in_context(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        context_type: &ContextType,
        context_id: &str,
    ) -> AppResult<()> {
        sqlx::query(
            "DELETE FROM ordering WHERE task_id = ? AND context_type = ? AND context_id = ?",
        )
        .bind(task_id.to_string())
        .bind(Self::context_type_to_string(context_type))
        .bind(context_id)
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(())
    }

    // --- 读操作 ---
    async fn find_for_context(
        &self,
        context_type: &ContextType,
        context_id: &str,
    ) -> AppResult<Vec<Ordering>> {
        let rows = sqlx::query(
            "SELECT * FROM ordering WHERE context_type = ? AND context_id = ? ORDER BY sort_order ASC"
        )
        .bind(Self::context_type_to_string(context_type))
        .bind(context_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let mut orderings = Vec::new();
        for row in rows {
            let ordering = Self::row_to_ordering(&row).map_err(DbError::ConnectionError)?;
            orderings.push(ordering);
        }

        Ok(orderings)
    }

    async fn create_for_new_task(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        context_type: &ContextType,
        context_id: &str,
        task_id: Uuid,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> AppResult<Ordering> {
        // 验证上下文ID格式
        crate::entities::Ordering::validate_context_id(context_type, context_id)?;

        // 获取上下文中现有的排序记录数量
        let count_row = sqlx::query(
            "SELECT COUNT(*) as count FROM ordering WHERE context_type = ? AND context_id = ?",
        )
        .bind(Self::context_type_to_string(context_type))
        .bind(context_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        let count: i64 = count_row.try_get("count").unwrap_or(0);

        // 计算新的排序值（放在末尾）
        let sort_order = if count == 0 {
            crate::shared::core::generate_initial_sort_order()
        } else {
            // 获取最后一个排序值
            let last_row = sqlx::query(
                r#"
                SELECT sort_order FROM ordering 
                WHERE context_type = ? AND context_id = ?
                ORDER BY sort_order DESC LIMIT 1
                "#,
            )
            .bind(Self::context_type_to_string(context_type))
            .bind(context_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(DbError::ConnectionError)?;

            let last_sort_order: String = last_row
                .try_get("sort_order")
                .unwrap_or_else(|_| "n".to_string());
            crate::shared::core::get_rank_after(&last_sort_order)?
        };

        // 创建排序对象
        let ordering = crate::entities::Ordering {
            id: Uuid::new_v4(),
            context_type: context_type.clone(),
            context_id: context_id.to_string(),
            task_id,
            sort_order,
            updated_at: created_at,
        };

        // 插入到数据库
        sqlx::query(
            r#"
            INSERT INTO ordering (id, context_type, context_id, task_id, sort_order, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(ordering.id.to_string())
        .bind(Self::context_type_to_string(&ordering.context_type))
        .bind(&ordering.context_id)
        .bind(ordering.task_id.to_string())
        .bind(&ordering.sort_order)
        .bind(ordering.updated_at.to_rfc3339())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(ordering)
    }
}
