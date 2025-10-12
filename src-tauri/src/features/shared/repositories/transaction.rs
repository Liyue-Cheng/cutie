/// 事务助手 - 减少样板代码
use sqlx::{Sqlite, SqlitePool, Transaction};

use crate::infra::core::{AppError, AppResult, DbError};

pub struct TransactionHelper;

impl TransactionHelper {
    /// 开始事务（统一错误处理）
    pub async fn begin(pool: &SqlitePool) -> AppResult<Transaction<'_, Sqlite>> {
        pool.begin()
            .await
            .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))
    }

    /// 提交事务（统一错误处理）
    pub async fn commit(tx: Transaction<'_, Sqlite>) -> AppResult<()> {
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(DbError::TransactionFailed {
                message: e.to_string(),
            })
        })
    }
}

