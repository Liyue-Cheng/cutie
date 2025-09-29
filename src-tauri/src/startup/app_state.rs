/// 应用状态模块 - 为sidecar架构设计
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::repositories::{
    AreaRepository, OrderingRepository, SqliteAreaRepository, SqliteOrderingRepository,
    SqliteTaskRepository, SqliteTaskScheduleRepository, SqliteTemplateRepository,
    SqliteTimeBlockRepository, TaskRepository, TaskScheduleRepository, TemplateRepository,
    TimeBlockRepository,
};
use crate::shared::core::AppError;

/// 应用状态容器
///
/// 专为sidecar架构设计的轻量级状态容器，包含所有repository实例
#[derive(Clone)]
pub struct AppState {
    /// 应用配置
    config: Arc<AppConfig>,

    /// 数据库连接池
    db_pool: Arc<SqlitePool>,

    /// Repository实例 - 作为依赖注入容器
    task_repository: Arc<SqliteTaskRepository>,
    ordering_repository: Arc<SqliteOrderingRepository>,
    area_repository: Arc<SqliteAreaRepository>,
    template_repository: Arc<SqliteTemplateRepository>,
    time_block_repository: Arc<SqliteTimeBlockRepository>,
    task_schedule_repository: Arc<SqliteTaskScheduleRepository>,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(config: AppConfig, db_pool: SqlitePool) -> Self {
        let db_pool_arc = Arc::new(db_pool);

        // 创建所有repository实例
        let task_repository = Arc::new(SqliteTaskRepository::new((*db_pool_arc).clone()));
        let ordering_repository = Arc::new(SqliteOrderingRepository::new((*db_pool_arc).clone()));
        let area_repository = Arc::new(SqliteAreaRepository::new((*db_pool_arc).clone()));
        let template_repository = Arc::new(SqliteTemplateRepository::new((*db_pool_arc).clone()));
        let time_block_repository =
            Arc::new(SqliteTimeBlockRepository::new((*db_pool_arc).clone()));
        let task_schedule_repository =
            Arc::new(SqliteTaskScheduleRepository::new((*db_pool_arc).clone()));

        Self {
            config: Arc::new(config),
            db_pool: db_pool_arc,
            task_repository,
            ordering_repository,
            area_repository,
            template_repository,
            time_block_repository,
            task_schedule_repository,
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

    /// 获取TaskRepository实例
    pub fn task_repository(&self) -> &dyn TaskRepository {
        self.task_repository.as_ref()
    }

    /// 获取OrderingRepository实例
    pub fn ordering_repository(&self) -> &dyn OrderingRepository {
        self.ordering_repository.as_ref()
    }

    /// 获取AreaRepository实例
    pub fn area_repository(&self) -> &dyn AreaRepository {
        self.area_repository.as_ref()
    }

    /// 获取TemplateRepository实例
    pub fn template_repository(&self) -> &dyn TemplateRepository {
        self.template_repository.as_ref()
    }

    /// 获取TimeBlockRepository实例
    pub fn time_block_repository(&self) -> &dyn TimeBlockRepository {
        self.time_block_repository.as_ref()
    }

    /// 获取TaskScheduleRepository实例
    pub fn task_schedule_repository(&self) -> &dyn TaskScheduleRepository {
        self.task_schedule_repository.as_ref()
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
