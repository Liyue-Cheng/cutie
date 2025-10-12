/// Area 通用查询仓库
/// 使用场景：所有需要查询 Area 信息的地方
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    infra::core::{AppError, AppResult, DbError},
    entities::task::response_dtos::AreaSummary,
};

pub struct AreaRepository;

impl AreaRepository {
    /// 获取 Area 摘要（支持任何 Executor，包括事务和连接池）
    ///
    /// # 使用示例
    /// ```rust
    /// // 在事务中使用
    /// let area = AreaRepository::get_summary(&mut *tx, area_id).await?;
    ///
    /// // 在连接池中使用
    /// let area = AreaRepository::get_summary(pool, area_id).await?;
    /// ```
    pub async fn get_summary(
        executor: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
        area_id: Uuid,
    ) -> AppResult<Option<AreaSummary>> {
        let query = r#"
            SELECT id, name, color
            FROM areas
            WHERE id = ? AND is_deleted = false
        "#;

        let result = sqlx::query_as::<_, (String, String, String)>(query)
            .bind(area_id.to_string())
            .fetch_optional(executor)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(result.map(|(id, name, color)| AreaSummary {
            id: Uuid::parse_str(&id).unwrap(),
            name,
            color,
        }))
    }

    /// 批量获取 Area 摘要
    ///
    /// 返回 HashMap<area_id, AreaSummary>，便于快速查找
    pub async fn get_summaries_batch(
        executor: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
        area_ids: &[Uuid],
    ) -> AppResult<HashMap<Uuid, AreaSummary>> {
        if area_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let placeholders = area_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            r#"
            SELECT id, name, color
            FROM areas
            WHERE id IN ({}) AND is_deleted = false
            "#,
            placeholders
        );

        let mut query_builder = sqlx::query_as::<_, (String, String, String)>(&query);
        for area_id in area_ids {
            query_builder = query_builder.bind(area_id.to_string());
        }

        let rows = query_builder
            .fetch_all(executor)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        let map = rows
            .into_iter()
            .map(|(id, name, color)| {
                (
                    Uuid::parse_str(&id).unwrap(),
                    AreaSummary {
                        id: Uuid::parse_str(&id).unwrap(),
                        name,
                        color,
                    },
                )
            })
            .collect();

        Ok(map)
    }
}
