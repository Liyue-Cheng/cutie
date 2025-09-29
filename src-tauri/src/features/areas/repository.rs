/// 领域数据访问层
///
/// 实现领域的数据库操作

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::shared::{
    core::{AppResult, Area, DbError},
    database::{AreaRepository, Repository},
};

/// 领域仓库的SQLx实现
#[derive(Clone)]
pub struct SqlxAreaRepository {
    pub pool: SqlitePool,
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
    fn area_to_params(area: &Area) -> (String, String, String, Option<String>, String, String, bool) {
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
impl Repository<Area> for SqlxAreaRepository {
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

    async fn create(&self, area: &Area) -> AppResult<Area> {
        let params = Self::area_to_params(area);

        sqlx::query(
            r#"
            INSERT INTO areas (id, name, color, parent_area_id, created_at, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.0) // id
        .bind(&params.1) // name
        .bind(&params.2) // color
        .bind(&params.3) // parent_area_id
        .bind(&params.4) // created_at
        .bind(&params.5) // updated_at
        .bind(&params.6) // is_deleted
        .execute(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        Ok(area.clone())
    }

    async fn update(&self, area: &Area) -> AppResult<Area> {
        let params = Self::area_to_params(area);

        let result = sqlx::query(
            r#"
            UPDATE areas SET
                name = ?, color = ?, parent_area_id = ?, updated_at = ?
            WHERE id = ? AND is_deleted = FALSE
            "#,
        )
        .bind(&params.1) // name
        .bind(&params.2) // color
        .bind(&params.3) // parent_area_id
        .bind(&params.5) // updated_at
        .bind(&params.0) // id
        .execute(&self.pool)
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

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            "UPDATE areas SET is_deleted = TRUE, updated_at = ? WHERE id = ? AND is_deleted = FALSE",
        )
        .bind(Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(&self.pool)
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

    async fn find_all(&self) -> AppResult<Vec<Area>> {
        let rows = sqlx::query("SELECT * FROM areas WHERE is_deleted = FALSE ORDER BY name ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::ConnectionError)?;

        let areas = rows
            .iter()
            .map(Self::row_to_area)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(areas)
    }
}

#[async_trait]
impl AreaRepository for SqlxAreaRepository {
    async fn find_roots(&self) -> AppResult<Vec<Area>> {
        let rows = sqlx::query(
            "SELECT * FROM areas WHERE parent_area_id IS NULL AND is_deleted = FALSE ORDER BY name ASC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let areas = rows
            .iter()
            .map(Self::row_to_area)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(areas)
    }

    async fn find_by_parent_id(&self, parent_id: Option<Uuid>) -> AppResult<Vec<Area>> {
        let rows = match parent_id {
            Some(parent_id) => {
                sqlx::query(
                    "SELECT * FROM areas WHERE parent_area_id = ? AND is_deleted = FALSE ORDER BY name ASC",
                )
                .bind(parent_id.to_string())
                .fetch_all(&self.pool)
                .await
                .map_err(DbError::ConnectionError)?
            }
            None => {
                sqlx::query(
                    "SELECT * FROM areas WHERE parent_area_id IS NULL AND is_deleted = FALSE ORDER BY name ASC",
                )
                .fetch_all(&self.pool)
                .await
                .map_err(DbError::ConnectionError)?
            }
        };

        let areas = rows
            .iter()
            .map(Self::row_to_area)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(areas)
    }

    async fn get_path(&self, area_id: Uuid) -> AppResult<Vec<Area>> {
        let mut path = Vec::new();
        let mut current_id = Some(area_id);

        while let Some(id) = current_id {
            if let Some(area) = self.find_by_id(id).await? {
                current_id = area.parent_area_id;
                path.insert(0, area); // 插入到开头，构建从根到目标的路径
            } else {
                break;
            }
        }

        Ok(path)
    }

    async fn get_descendants(&self, area_id: Uuid) -> AppResult<Vec<Area>> {
        // 使用递归CTE查询所有后代
        let rows = sqlx::query(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT * FROM areas WHERE parent_area_id = ? AND is_deleted = FALSE
                UNION ALL
                SELECT a.* FROM areas a
                INNER JOIN descendants d ON a.parent_area_id = d.id
                WHERE a.is_deleted = FALSE
            )
            SELECT * FROM descendants ORDER BY name ASC
            "#,
        )
        .bind(area_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        let areas = rows
            .iter()
            .map(Self::row_to_area)
            .collect::<Result<Vec<_>, _>>()
            .map_err(DbError::ConnectionError)?;

        Ok(areas)
    }

    async fn can_delete(&self, area_id: Uuid) -> AppResult<bool> {
        // 检查是否有子领域
        let children = self.find_by_parent_id(Some(area_id)).await?;
        if !children.is_empty() {
            return Ok(false);
        }

        // 检查是否有任务使用此领域
        let task_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM tasks WHERE area_id = ? AND is_deleted = FALSE",
        )
        .bind(area_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::ConnectionError)?;

        if task_count.0 > 0 {
            return Ok(false);
        }

        // TODO: 检查是否有项目使用此领域
        // let project_count: (i64,) = sqlx::query_as(
        //     "SELECT COUNT(*) FROM projects WHERE area_id = ? AND is_deleted = FALSE",
        // )
        // .bind(area_id.to_string())
        // .fetch_one(&self.pool)
        // .await
        // .map_err(DbError::ConnectionError)?;

        // if project_count.0 > 0 {
        //     return Ok(false);
        // }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::database::connection::create_test_database;

    #[tokio::test]
    async fn test_area_crud_operations() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxAreaRepository::new(pool);

        // 创建测试领域
        let area = Area::new(
            Uuid::new_v4(),
            "Test Area".to_string(),
            "#FF0000".to_string(),
            Utc::now(),
        );

        // 测试创建
        let created_area = repo.create(&area).await.unwrap();
        assert_eq!(created_area.name, area.name);

        // 测试查找
        let found_area = repo.find_by_id(area.id).await.unwrap().unwrap();
        assert_eq!(found_area.id, area.id);

        // 测试更新
        let mut updated_area = found_area.clone();
        updated_area.name = "Updated Area".to_string();
        updated_area.updated_at = Utc::now();

        let updated = repo.update(&updated_area).await.unwrap();
        assert_eq!(updated.name, "Updated Area");

        // 测试删除
        repo.delete(area.id).await.unwrap();
        let deleted_area = repo.find_by_id(area.id).await.unwrap();
        assert!(deleted_area.is_none());
    }

    #[tokio::test]
    async fn test_area_hierarchy() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxAreaRepository::new(pool);

        // 创建根领域
        let root_area = Area::new(
            Uuid::new_v4(),
            "Root Area".to_string(),
            "#FF0000".to_string(),
            Utc::now(),
        );
        repo.create(&root_area).await.unwrap();

        // 创建子领域
        let mut child_area = Area::new(
            Uuid::new_v4(),
            "Child Area".to_string(),
            "#00FF00".to_string(),
            Utc::now(),
        );
        child_area.parent_area_id = Some(root_area.id);
        repo.create(&child_area).await.unwrap();

        // 测试查找根领域
        let roots = repo.find_roots().await.unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, root_area.id);

        // 测试按父ID查找
        let children = repo.find_by_parent_id(Some(root_area.id)).await.unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child_area.id);

        // 测试获取路径
        let path = repo.get_path(child_area.id).await.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0].id, root_area.id);
        assert_eq!(path[1].id, child_area.id);

        // 测试获取后代
        let descendants = repo.get_descendants(root_area.id).await.unwrap();
        assert_eq!(descendants.len(), 1);
        assert_eq!(descendants[0].id, child_area.id);
    }

    #[tokio::test]
    async fn test_can_delete() {
        let pool = create_test_database().await.unwrap();
        let repo = SqlxAreaRepository::new(pool);

        // 创建领域
        let area = Area::new(
            Uuid::new_v4(),
            "Test Area".to_string(),
            "#FF0000".to_string(),
            Utc::now(),
        );
        repo.create(&area).await.unwrap();

        // 新创建的领域应该可以删除
        let can_delete = repo.can_delete(area.id).await.unwrap();
        assert!(can_delete);

        // 创建子领域
        let mut child_area = Area::new(
            Uuid::new_v4(),
            "Child Area".to_string(),
            "#00FF00".to_string(),
            Utc::now(),
        );
        child_area.parent_area_id = Some(area.id);
        repo.create(&child_area).await.unwrap();

        // 有子领域的领域不能删除
        let can_delete_parent = repo.can_delete(area.id).await.unwrap();
        assert!(!can_delete_parent);
    }
}
