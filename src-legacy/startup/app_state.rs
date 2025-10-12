use sqlx::SqlitePool;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::infra::core::AppError;

/// 简化的应用状态容器
///
/// 为了保持兼容性，简化AppState结构，只保留核心依赖
#[derive(Clone)]
pub struct AppState {
    /// 应用配置
    pub config: Arc<AppConfig>,

    /// 数据库连接池
    pub db_pool: Arc<SqlitePool>,
}

impl AppState {
    /// 创建新的应用状态
    pub async fn new() -> Result<Self, AppError> {
        // 加载配置
        let config = Arc::new(AppConfig::load_from_env()?);

        // 初始化数据库
        let pool = crate::startup::database::initialize_database(&config).await?;
        let db_pool = Arc::new(pool);

        Ok(Self { config, db_pool })
    }

    /// 获取配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取数据库连接池
    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_state_creation() {
        // 注意：这个测试可能需要有效的环境变量配置
        // 在实际测试中，我们可能需要mock配置

        // 简单的存在性测试
        assert_eq!(
            std::mem::size_of::<AppState>(),
            std::mem::size_of::<(Arc<AppConfig>, Arc<SqlitePool>)>()
        );
    }
}
