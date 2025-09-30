/// 应用状态模块 - 为sidecar架构设计
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::shared::core::AppError;

// 导入抽象层接口
pub use crate::shared::ports::{Clock, IdGenerator, SystemClock, UuidV4Generator};

/// 应用状态容器
///
/// 专为sidecar架构设计的轻量级状态容器，包含所有外部依赖的抽象
#[derive(Clone)]
pub struct AppState {
    /// 应用配置
    config: Arc<AppConfig>,

    /// 数据库连接池
    db_pool: Arc<SqlitePool>,

    /// 时钟抽象（用于可测试的时间获取）
    clock: Arc<dyn Clock>,

    /// ID生成器抽象（用于可测试的UUID生成）
    id_generator: Arc<dyn IdGenerator>,
}

impl AppState {
    /// 创建新的应用状态（完整构造）
    pub fn new(
        config: AppConfig,
        db_pool: SqlitePool,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            db_pool: Arc::new(db_pool),
            clock,
            id_generator,
        }
    }

    /// 创建生产环境的应用状态（使用默认的生产适配器）
    pub fn new_production(config: AppConfig, db_pool: SqlitePool) -> Self {
        Self::new(
            config,
            db_pool,
            Arc::new(SystemClock::new()),
            Arc::new(UuidV4Generator::new()),
        )
    }

    /// 获取配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取数据库连接池
    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    /// 获取时钟抽象
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// 获取ID生成器抽象
    pub fn id_generator(&self) -> &Arc<dyn IdGenerator> {
        &self.id_generator
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
