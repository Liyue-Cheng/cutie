/// 数据库初始化模块 - 基于新架构重写
use sqlx::SqlitePool;

use crate::config::AppConfig;
use crate::shared::{
    core::AppError,
    database::{initialize_database as shared_initialize_database, DatabaseConfig},
};

/// 初始化数据库连接池
///
/// 使用新的shared模块中的数据库初始化功能
pub async fn initialize_database(config: &AppConfig) -> Result<SqlitePool, AppError> {
    tracing::info!("Initializing database with new architecture...");

    let db_path = config.database_path();

    // 使用From trait进行配置转换，简洁且不易出错
    let db_config: DatabaseConfig = (&config.database).into();

    // 使用shared模块的数据库初始化功能
    let pool = shared_initialize_database(&db_path, &db_config).await?;

    tracing::info!("Database initialized successfully with new architecture");
    Ok(pool)
}

/// 运行数据库迁移
pub async fn run_migrations(pool: &SqlitePool) -> Result<(), AppError> {
    crate::shared::database::run_migrations(pool).await
}

/// 创建测试数据库
pub async fn create_test_database() -> Result<SqlitePool, AppError> {
    crate::shared::database::create_test_database().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_database() {
        let pool = create_test_database().await.unwrap();

        // 测试数据库连接
        let result: (i32,) = sqlx::query_as("SELECT 1").fetch_one(&pool).await.unwrap();

        assert_eq!(result.0, 1);
        pool.close().await;
    }
}
