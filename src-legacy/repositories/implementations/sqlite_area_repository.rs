/// AreaRepository的SQLite实现
///
/// 提供Area实体的具体数据库操作实现

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::shared::core::{AppResult, DbError};
use crate::entities::Area;
use crate::repositories::traits::AreaRepository;

/// 区域仓库的SQLite实现
#[derive(Clone)]
pub struct SqliteAreaRepository {
    pool: SqlitePool,
}

impl SqliteAreaRepository {
    /// 创建新的AreaRepository实例
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
}

#[async_trait]
impl AreaRepository for SqliteAreaRepository {
    // --- 写操作 ---
    async fn create(&self, tx: &mut Transaction<'_, Sqlite>, area: &Area) -> AppResult<Area> {
        sqlx::query(
            r#"
            INSERT INTO areas (
                id, name, color, parent_area_id, created_at, updated_at, is_deleted
            ) VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(area.id.to_string())
        .bind(&area.name)
        .bind(&area.color)
        .bind(area.parent_area_id.map(|id| id.to_string()))
        .bind(area.created_at.to_rfc3339())
        .bind(area.updated_at.to_rfc3339())
        .bind(area.is_deleted)
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(area.clone())
    }

    async fn update(&self, tx: &mut Transaction<'_, Sqlite>, area: &Area) -> AppResult<Area> {
        let result = sqlx::query(
            r#"
            UPDATE areas SET 
                name = ?, color = ?, parent_area_id = ?, updated_at = ?
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(&area.name)
        .bind(&area.color)
        .bind(area.parent_area_id.map(|id| id.to_string()))
        .bind(area.updated_at.to_rfc3339())
        .bind(area.id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "Area",
                area.id.to_string(),
            ));
        }

        Ok(area.clone())
    }

    async fn delete(&self, tx: &mut Transaction<'_, Sqlite>, id: Uuid) -> AppResult<()> {
        let now = Utc::now();
        let result = sqlx::query(
            r#"
            UPDATE areas SET 
                is_deleted = TRUE, 
                updated_at = ? 
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(now.to_rfc3339())
        .bind(id.to_string())
        .execute(&mut **tx)
        .await
        .map_err(DbError::ConnectionError)?;

        if result.rows_affected() == 0 {
            return Err(crate::shared::core::AppError::not_found(
                "Area",
                id.to_string(),
            ));
        }

        Ok(())
    }

    // --- 读操作 ---
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Area>> {
        let row = sqlx::query("SELECT * FROM areas WHERE id = ? AND is_deleted = FALSE")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        match row {
            Some(row) => {
                let area = Self::row_to_area(&row).map_err(DbError::ConnectionError)?;
                Ok(Some(area))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> AppResult<Vec<Area>> {
        let rows = sqlx::query("SELECT * FROM areas WHERE is_deleted = FALSE ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        let mut areas = Vec::new();
        for row in rows {
            let area = Self::row_to_area(&row).map_err(DbError::ConnectionError)?;
            areas.push(area);
        }

        Ok(areas)
    }
}
