/// 应用状态模块 - 为sidecar架构设计
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::shared::core::AppError;

/// 应用状态容器
///
/// 专为sidecar架构设计的轻量级状态容器
#[derive(Clone)]
pub struct AppState {
    /// 应用配置
    config: Arc<AppConfig>,

    /// 数据库连接池
    db_pool: Arc<SqlitePool>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(config: AppConfig, db_pool: SqlitePool) -> Self {
        Self {
            config: Arc::new(config),
            db_pool: Arc::new(db_pool),
        }
    }

    /// 创建生产环境的应用状态（别名，保持兼容性）
    pub fn new_production(config: AppConfig, db_pool: SqlitePool) -> Self {
        Self::new(config, db_pool)
    }

    /// 获取配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取数据库连接池
    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus, AppError> {
        // 简单的数据库连接检查
        let result = sqlx::query("SELECT 1").fetch_one(self.db_pool()).await;

        match result {
            Ok(_) => Ok(HealthStatus::Healthy),
            Err(e) => {
                tracing::warn!("Database health check failed: {}", e);
                Ok(HealthStatus::Unhealthy)
            }
        }
    }
}

/// 健康状态枚举
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_creation() {
        // 基本的结构测试
        assert_eq!(
            std::mem::size_of::<AppState>(),
            std::mem::size_of::<(Arc<AppConfig>, Arc<SqlitePool>)>()
        );
    }

    #[test]
    fn test_health_status() {
        let status = HealthStatus::Healthy;
        matches!(status, HealthStatus::Healthy);
    }
}
