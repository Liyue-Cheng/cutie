use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use super::{AreaRepository, AreaUsageCount, Transaction};
use crate::common::error::DbError;
use crate::core::models::Area;

/// AreaRepository的SQLx实现
pub struct SqlxAreaRepository {
    pool: SqlitePool,
}

impl SqlxAreaRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// 将数据库行转换为Area对象
    fn row_to_area(row: &sqlx::sqlite::SqliteRow) -> Result<Area, sqlx::Error> {
        Ok(Area {
            id: Uuid::parse_str(&row.try_get::<String, _>("id")?).map_err(|_| {
                sqlx::Error::ColumnDecode {
                    index: "id".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid UUID",
                    )),
                }
            })?,
            name: row.try_get("name")?,
            color: row.try_get("color")?,
            parent_area_id: row
                .try_get::<Option<String>, _>("parent_area_id")?
                .and_then(|s| Uuid::parse_str(&s).ok()),
            created_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "created_at".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
                .map_err(|_| sqlx::Error::ColumnDecode {
                    index: "updated_at".to_string(),
                    source: Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid datetime",
                    )),
                })?
                .with_timezone(&Utc),
            is_deleted: row.try_get("is_deleted")?,
        })
    }

    /// 将Area对象转换为数据库参数
    fn area_to_params(
        area: &Area,
    ) -> (String, String, String, Option<String>, String, String, bool) {
        (
            area.id.to_string(),
            area.name.clone(),
            area.color.clone(),
            area.parent_area_id.map(|id| id.to_string()),
            area.created_at.to_rfc3339(),
            area.updated_at.to_rfc3339(),
            area.is_deleted,
        )
    }
}

#[async_trait]
impl AreaRepository for SqlxAreaRepository {
    async fn create(&self, tx: &mut Transaction<'_>, area: &Area) -> Result<Area, DbError> {
        let params = Self::area_to_params(area);

        let result = sqlx::query(
            r#"
            INSERT INTO areas (id, name, color, parent_area_id, created_at, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.0)
        .bind(&params.1)
        .bind(&params.2)
        .bind(&params.3)
        .bind(&params.4)
        .bind(&params.5)
        .bind(&params.6)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(_) => Ok(area.clone()),
            Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
                Err(DbError::ConstraintViolation {
                    message: format!("Area with id {} already exists", area.id),
                })
            }
            Err(sqlx::Error::Database(db_err)) if db_err.is_foreign_key_violation() => {
                Err(DbError::ConstraintViolation {
                    message: "Parent area does not exist".to_string(),
                })
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn update(&self, tx: &mut Transaction<'_>, area: &Area) -> Result<Area, DbError> {
        let params = Self::area_to_params(area);

        let result = sqlx::query(
            "UPDATE areas SET name = ?, color = ?, parent_area_id = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL"
        )
        .bind(&params.1).bind(&params.2).bind(&params.3).bind(&params.5)
        .bind(&params.0)
        .execute(&mut **tx)
        .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "Area".to_string(),
                        entity_id: area.id.to_string(),
                    })
                } else {
                    Ok(area.clone())
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_by_id(&self, area_id: Uuid) -> Result<Option<Area>, DbError> {
        let result = sqlx::query("SELECT * FROM areas WHERE id = ? AND deleted_at IS NULL")
            .bind(area_id.to_string())
            .fetch_optional(&self.pool)
            .await;

        match result {
            Ok(Some(row)) => Ok(Some(
                Self::row_to_area(&row).map_err(DbError::ConnectionError)?,
            )),
            Ok(None) => Ok(None),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_all(&self) -> Result<Vec<Area>, DbError> {
        let result = sqlx::query("SELECT * FROM areas WHERE deleted_at IS NULL ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => {
                let areas: Result<Vec<Area>, _> =
                    rows.iter().map(|row| Self::row_to_area(row)).collect();
                areas.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_root_areas(&self) -> Result<Vec<Area>, DbError> {
        let result = sqlx::query("SELECT * FROM areas WHERE parent_area_id IS NULL AND deleted_at IS NULL ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await;

        match result {
            Ok(rows) => {
                let areas: Result<Vec<Area>, _> =
                    rows.iter().map(|row| Self::row_to_area(row)).collect();
                areas.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_children(&self, parent_id: Uuid) -> Result<Vec<Area>, DbError> {
        let result = sqlx::query(
            "SELECT * FROM areas WHERE parent_area_id = ? AND deleted_at IS NULL ORDER BY name ASC",
        )
        .bind(parent_id.to_string())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let areas: Result<Vec<Area>, _> =
                    rows.iter().map(|row| Self::row_to_area(row)).collect();
                areas.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_descendants(&self, parent_id: Uuid) -> Result<Vec<Area>, DbError> {
        // 使用递归CTE查找所有后代
        let result = sqlx::query(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT * FROM areas WHERE parent_area_id = ? AND deleted_at IS NULL
                UNION ALL
                SELECT a.* FROM areas a
                INNER JOIN descendants d ON a.parent_area_id = d.id
                WHERE a.deleted_at IS NULL
            )
            SELECT * FROM descendants ORDER BY name ASC
            "#,
        )
        .bind(parent_id.to_string())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let areas: Result<Vec<Area>, _> =
                    rows.iter().map(|row| Self::row_to_area(row)).collect();
                areas.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn find_path_to_root(&self, area_id: Uuid) -> Result<Vec<Area>, DbError> {
        // 使用递归CTE查找到根节点的路径
        let result = sqlx::query(
            r#"
            WITH RECURSIVE path AS (
                SELECT * FROM areas WHERE id = ? AND deleted_at IS NULL
                UNION ALL
                SELECT a.* FROM areas a
                INNER JOIN path p ON a.id = p.parent_area_id
                WHERE a.deleted_at IS NULL
            )
            SELECT * FROM path
            "#,
        )
        .bind(area_id.to_string())
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let areas: Result<Vec<Area>, _> =
                    rows.iter().map(|row| Self::row_to_area(row)).collect();
                areas.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn soft_delete(&self, tx: &mut Transaction<'_>, area_id: Uuid) -> Result<(), DbError> {
        // 首先检查是否有子领域
        let has_children = self.has_children(area_id).await?;
        if has_children {
            return Err(DbError::ConstraintViolation {
                message: "Cannot delete area with children".to_string(),
            });
        }

        // 检查是否被使用
        let is_used = self.is_used(area_id).await?;
        if is_used {
            return Err(DbError::ConstraintViolation {
                message: "Cannot delete area that is in use".to_string(),
            });
        }

        let result = sqlx::query("UPDATE areas SET is_deleted = TRUE, updated_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(area_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn restore(&self, tx: &mut Transaction<'_>, area_id: Uuid) -> Result<Area, DbError> {
        let result =
            sqlx::query("UPDATE areas SET deleted_at IS NULL, updated_at = ? WHERE id = ?")
                .bind(Utc::now().to_rfc3339())
                .bind(area_id.to_string())
                .execute(&mut **tx)
                .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "Area".to_string(),
                        entity_id: area_id.to_string(),
                    })
                } else {
                    // 查询恢复后的领域
                    let area_result = sqlx::query("SELECT * FROM areas WHERE id = ?")
                        .bind(area_id.to_string())
                        .fetch_one(&mut **tx)
                        .await;

                    match area_result {
                        Ok(row) => Ok(Self::row_to_area(&row).map_err(DbError::ConnectionError)?),
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn has_children(&self, area_id: Uuid) -> Result<bool, DbError> {
        let result = sqlx::query(
            "SELECT COUNT(*) as count FROM areas WHERE parent_area_id = ? AND deleted_at IS NULL",
        )
        .bind(area_id.to_string())
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => {
                let count: i64 = row.try_get("count").map_err(DbError::ConnectionError)?;
                Ok(count > 0)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn is_used(&self, area_id: Uuid) -> Result<bool, DbError> {
        let result = sqlx::query(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM tasks WHERE area_id = ? AND deleted_at IS NULL) +
                (SELECT COUNT(*) FROM time_blocks WHERE area_id = ? AND deleted_at IS NULL) +
                (SELECT COUNT(*) FROM projects WHERE area_id = ? AND deleted_at IS NULL) as total_usage
            "#
        )
        .bind(area_id.to_string())
        .bind(area_id.to_string())
        .bind(area_id.to_string())
        .fetch_one(&self.pool)
        .await;

        match result {
            Ok(row) => {
                let total_usage: i64 = row
                    .try_get("total_usage")
                    .map_err(DbError::ConnectionError)?;
                Ok(total_usage > 0)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn move_to_parent(
        &self,
        tx: &mut Transaction<'_>,
        area_id: Uuid,
        new_parent_id: Option<Uuid>,
    ) -> Result<Area, DbError> {
        // 如果有新父级，检查是否会造成循环
        if let Some(parent_id) = new_parent_id {
            let would_cycle = self.would_create_cycle(area_id, parent_id).await?;
            if would_cycle {
                return Err(DbError::ConstraintViolation {
                    message: "Moving area would create a cycle".to_string(),
                });
            }
        }

        let result = sqlx::query("UPDATE areas SET parent_area_id = ?, updated_at = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(new_parent_id.map(|id| id.to_string()))
            .bind(Utc::now().to_rfc3339())
            .bind(area_id.to_string())
            .execute(&mut **tx)
            .await;

        match result {
            Ok(query_result) => {
                if query_result.rows_affected() == 0 {
                    Err(DbError::NotFound {
                        entity_type: "Area".to_string(),
                        entity_id: area_id.to_string(),
                    })
                } else {
                    // 查询更新后的领域
                    let area_result =
                        sqlx::query("SELECT * FROM areas WHERE id = ? AND deleted_at IS NULL")
                            .bind(area_id.to_string())
                            .fetch_one(&mut **tx)
                            .await;

                    match area_result {
                        Ok(row) => Ok(Self::row_to_area(&row).map_err(DbError::ConnectionError)?),
                        Err(e) => Err(DbError::ConnectionError(e)),
                    }
                }
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }

    async fn would_create_cycle(
        &self,
        area_id: Uuid,
        potential_parent_id: Uuid,
    ) -> Result<bool, DbError> {
        // 检查potential_parent_id是否是area_id的后代
        let descendants = self.find_descendants(area_id).await?;
        Ok(descendants
            .iter()
            .any(|area| area.id == potential_parent_id))
    }

    async fn count_usage(&self) -> Result<Vec<AreaUsageCount>, DbError> {
        let result = sqlx::query(
            r#"
            SELECT 
                a.id as area_id,
                a.name as area_name,
                COALESCE(t.task_count, 0) as task_count,
                COALESCE(tb.time_block_count, 0) as time_block_count,
                COALESCE(p.project_count, 0) as project_count,
                (COALESCE(t.task_count, 0) + COALESCE(tb.time_block_count, 0) + COALESCE(p.project_count, 0)) as total_usage
            FROM areas a
            LEFT JOIN (
                SELECT area_id, COUNT(*) as task_count 
                FROM tasks 
                WHERE deleted_at IS NULL 
                GROUP BY area_id
            ) t ON a.id = t.area_id
            LEFT JOIN (
                SELECT area_id, COUNT(*) as time_block_count 
                FROM time_blocks 
                WHERE deleted_at IS NULL 
                GROUP BY area_id
            ) tb ON a.id = tb.area_id
            LEFT JOIN (
                SELECT area_id, COUNT(*) as project_count 
                FROM projects 
                WHERE deleted_at IS NULL 
                GROUP BY area_id
            ) p ON a.id = p.area_id
            WHERE a.deleted_at IS NULL
            ORDER BY total_usage DESC, a.name ASC
            "#
        )
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                let usage_counts: Result<Vec<AreaUsageCount>, _> = rows
                    .iter()
                    .map(|row| {
                        let area_id_str: String = row.try_get("area_id")?;
                        Ok(AreaUsageCount {
                            area_id: Uuid::parse_str(&area_id_str).map_err(|_| {
                                sqlx::Error::ColumnDecode {
                                    index: "area_id".to_string(),
                                    source: Box::new(std::io::Error::new(
                                        std::io::ErrorKind::InvalidData,
                                        "Invalid UUID",
                                    )),
                                }
                            })?,
                            area_name: row.try_get("area_name")?,
                            task_count: row.try_get("task_count")?,
                            time_block_count: row.try_get("time_block_count")?,
                            project_count: row.try_get("project_count")?,
                            total_usage: row.try_get("total_usage")?,
                        })
                    })
                    .collect();
                usage_counts.map_err(DbError::ConnectionError)
            }
            Err(e) => Err(DbError::ConnectionError(e)),
        }
    }
}
