/// 时间块冲突检查服务
use chrono::{DateTime, Utc};
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::infra::core::{AppError, AppResult, DbError};

pub struct TimeBlockConflictChecker;

impl TimeBlockConflictChecker {
    /// 检查时间冲突
    ///
    /// # 参数
    /// - `tx`: 事务引用
    /// - `start_time`: 开始时间
    /// - `end_time`: 结束时间
    /// - `is_all_day`: 是否为全天事件
    /// - `exclude_id`: 排除的时间块ID（用于更新时排除自身）
    ///
    /// # 冲突规则
    /// - 全天事件（is_all_day = true）：不与任何事件冲突
    /// - 分时事件（is_all_day = false）：只与其他分时事件检测冲突
    ///
    /// # 返回
    /// - `Ok(true)`: 有冲突
    /// - `Ok(false)`: 无冲突
    pub async fn check_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        start_time: &DateTime<Utc>,
        end_time: &DateTime<Utc>,
        is_all_day: bool,
        exclude_id: Option<Uuid>,
    ) -> AppResult<bool> {
        // 全天事件不与任何事件冲突
        if is_all_day {
            return Ok(false);
        }

        // 分时事件只与其他分时事件检测冲突
        let mut query = String::from(
            r#"
            SELECT COUNT(*) as count
            FROM time_blocks
            WHERE is_deleted = false
              AND is_all_day = false
              AND start_time < ?
              AND end_time > ?
        "#,
        );

        if let Some(id) = exclude_id {
            query.push_str(&format!(" AND id != '{}'", id));
        }

        let count: i64 = sqlx::query_scalar(&query)
            .bind(end_time.to_rfc3339())
            .bind(start_time.to_rfc3339())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

        Ok(count > 0)
    }
}
