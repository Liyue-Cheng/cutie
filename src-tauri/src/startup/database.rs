/// 数据库初始化模块 - 基于新架构重写

use sqlx::SqlitePool;

use crate::shared::{
    core::AppError,
    database::{initialize_database as shared_initialize_database, DatabaseConfig, SynchronousMode},
};
use crate::config::AppConfig;

/// 初始化数据库连接池
/// 
/// 使用新的shared模块中的数据库初始化功能
pub async fn initialize_database(config: &AppConfig) -> Result<SqlitePool, AppError> {
    log::info!("Initializing database with new architecture...");

    let db_path = config.database_path();
    
    // 转换配置格式
    let db_config = DatabaseConfig {
        max_connections: config.database.max_connections,
        min_connections: config.database.min_connections,
        connect_timeout_seconds: config.database.connect_timeout_seconds,
        idle_timeout_seconds: config.database.idle_timeout_seconds,
        auto_migrate: config.database.auto_migrate,
        synchronous: match config.database.synchronous {
            crate::config::SynchronousMode::Off => SynchronousMode::Off,
            crate::config::SynchronousMode::Normal => SynchronousMode::Normal,
            crate::config::SynchronousMode::Full => SynchronousMode::Full,
        },
        cache_size_kb: config.database.cache_size_kb,
        foreign_keys: config.database.foreign_keys,
        wal_mode: config.database.wal_mode,
    };

    // 使用shared模块的数据库初始化功能
    let pool = shared_initialize_database(&db_path, &db_config).await?;

    log::info!("Database initialized successfully with new architecture");
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
        let result: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.0, 1);
        pool.close().await;
    }
}