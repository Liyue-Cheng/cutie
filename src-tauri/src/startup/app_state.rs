use sqlx::SqlitePool;
use std::sync::Arc;

use crate::common::error::AppError;
use crate::config::AppConfig;
use crate::ports::{Clock, IdGenerator, SettingRepository, SystemClock, UuidV4Generator};
use crate::repositories::*;
use crate::services::*;

/// 应用状态容器 - 依赖注入的核心
///
/// **预期行为简介:** 作为整个应用的依赖注入容器，持有所有服务、仓库和外部依赖的实例
///
/// ## 设计原则
/// - 所有依赖都通过Arc包装，支持多线程共享
/// - 使用trait object实现依赖注入的灵活性
/// - 在应用启动时一次性构建，运行时不可变
#[derive(Clone)]
pub struct AppState {
    /// 应用配置
    pub config: Arc<AppConfig>,

    /// 数据库连接池
    pub db_pool: Arc<SqlitePool>,

    /// 时钟服务
    pub clock: Arc<dyn Clock>,

    /// ID生成器
    pub id_generator: Arc<dyn IdGenerator>,

    /// 设置仓库
    pub setting_repository: Arc<dyn SettingRepository>,

    /// 任务仓库
    pub task_repository: Arc<dyn TaskRepository>,

    /// 任务日程仓库
    pub task_schedule_repository: Arc<dyn TaskScheduleRepository>,

    /// 排序仓库
    pub ordering_repository: Arc<dyn OrderingRepository>,

    /// 领域仓库
    pub area_repository: Arc<dyn AreaRepository>,

    /// 模板仓库
    pub template_repository: Arc<dyn TemplateRepository>,

    /// 时间块仓库
    pub time_block_repository: Arc<dyn TimeBlockRepository>,

    // === 服务层 ===
    /// 任务服务
    pub task_service: Arc<TaskService>,

    /// 日程服务
    pub schedule_service: Arc<ScheduleService>,

    /// 排序服务
    pub ordering_service: Arc<OrderingService>,

    /// 时间块服务
    pub time_block_service: Arc<TimeBlockService>,

    /// 模板服务
    pub template_service: Arc<TemplateService>,

    /// 领域服务
    pub area_service: Arc<AreaService>,
}

impl AppState {
    /// 创建生产环境的AppState
    ///
    /// **预期行为简介:** 构建一个完整的、用于生产环境的依赖注入容器
    /// **输入输出规范:**
    /// - **前置条件:** config必须已经验证过，db_pool必须是已连接的有效连接池
    /// - **后置条件:** 返回一个完全初始化的AppState实例，所有依赖都已正确注入
    /// **预期副作用:** 无直接副作用，但创建的实例将在整个应用生命周期中被使用
    pub fn new_production(config: AppConfig, db_pool: SqlitePool) -> Self {
        let config = Arc::new(config);
        let db_pool = Arc::new(db_pool);

        // 创建外部依赖的生产适配器
        let clock: Arc<dyn Clock> = Arc::new(SystemClock::new());
        let id_generator: Arc<dyn IdGenerator> = Arc::new(UuidV4Generator::new());

        // 创建设置仓库（TOML适配器）
        let setting_repository: Arc<dyn SettingRepository> = Arc::new(
            crate::adapters::TomlSettingRepository::new(config.settings_path()),
        );

        // 创建所有数据仓库的SQLx适配器
        let task_repository: Arc<dyn TaskRepository> =
            Arc::new(SqlxTaskRepository::new((*db_pool).clone()));

        let task_schedule_repository: Arc<dyn TaskScheduleRepository> =
            Arc::new(SqlxTaskScheduleRepository::new((*db_pool).clone()));

        let ordering_repository: Arc<dyn OrderingRepository> =
            Arc::new(SqlxOrderingRepository::new((*db_pool).clone()));

        let area_repository: Arc<dyn AreaRepository> =
            Arc::new(SqlxAreaRepository::new((*db_pool).clone()));

        let template_repository: Arc<dyn TemplateRepository> =
            Arc::new(SqlxTemplateRepository::new((*db_pool).clone()));

        let time_block_repository: Arc<dyn TimeBlockRepository> =
            Arc::new(SqlxTimeBlockRepository::new((*db_pool).clone()));

        // 创建服务层
        let task_service = Arc::new(TaskService::new(
            clock.clone(),
            id_generator.clone(),
            task_repository.clone(),
            task_schedule_repository.clone(),
            ordering_repository.clone(),
        ));

        let schedule_service = Arc::new(ScheduleService::new(
            clock.clone(),
            id_generator.clone(),
            task_repository.clone(),
            task_schedule_repository.clone(),
            ordering_repository.clone(),
        ));

        let ordering_service = Arc::new(OrderingService::new(
            clock.clone(),
            id_generator.clone(),
            task_repository.clone(),
            ordering_repository.clone(),
        ));

        let time_block_service = Arc::new(TimeBlockService::new(
            clock.clone(),
            id_generator.clone(),
            task_repository.clone(),
            time_block_repository.clone(),
        ));

        let template_service = Arc::new(TemplateService::new(
            clock.clone(),
            id_generator.clone(),
            template_repository.clone(),
            task_repository.clone(),
        ));

        let area_service = Arc::new(AreaService::new(
            clock.clone(),
            id_generator.clone(),
            area_repository.clone(),
            task_repository.clone(),
        ));

        Self {
            config,
            db_pool,
            clock,
            id_generator,
            setting_repository,
            task_repository,
            task_schedule_repository,
            ordering_repository,
            area_repository,
            template_repository,
            time_block_repository,
            task_service,
            schedule_service,
            ordering_service,
            time_block_service,
            template_service,
            area_service,
        }
    }

    /// 创建测试环境的AppState
    ///
    /// **预期行为简介:** 构建一个用于测试的依赖注入容器，使用内存适配器
    /// **输入输出规范:**
    /// - **前置条件:** 无
    /// - **后置条件:** 返回一个使用内存适配器的AppState实例，适合单元测试
    /// **预期副作用:** 无
    pub async fn new_test() -> Result<Self, AppError> {
        let config = Arc::new(AppConfig::default());

        // 创建内存数据库连接池
        let db_pool = Arc::new(crate::startup::database::create_test_database().await?);

        // 创建测试适配器
        let clock: Arc<dyn Clock> = Arc::new(crate::ports::FixedClock::new(chrono::Utc::now()));

        let id_generator: Arc<dyn IdGenerator> =
            Arc::new(crate::ports::SequentialIdGenerator::default_test_sequence());

        let setting_repository: Arc<dyn SettingRepository> =
            Arc::new(crate::ports::InMemorySettingRepository::new());

        // 创建内存仓库适配器
        let task_repository: Arc<dyn TaskRepository> = Arc::new(MemoryTaskRepository::new());

        let task_schedule_repository: Arc<dyn TaskScheduleRepository> =
            Arc::new(MemoryTaskScheduleRepository::new());

        // 创建SQLx仓库（使用内存数据库）
        let ordering_repository: Arc<dyn OrderingRepository> =
            Arc::new(SqlxOrderingRepository::new((*db_pool).clone()));

        let area_repository: Arc<dyn AreaRepository> =
            Arc::new(SqlxAreaRepository::new((*db_pool).clone()));

        let template_repository: Arc<dyn TemplateRepository> =
            Arc::new(SqlxTemplateRepository::new((*db_pool).clone()));

        let time_block_repository: Arc<dyn TimeBlockRepository> =
            Arc::new(SqlxTimeBlockRepository::new((*db_pool).clone()));

        // 创建服务层
        let task_service = Arc::new(TaskService::new(
            clock.clone(),
            id_generator.clone(),
            task_repository.clone(),
            task_schedule_repository.clone(),
            ordering_repository.clone(),
        ));

        let schedule_service = Arc::new(ScheduleService::new(
            clock.clone(),
            id_generator.clone(),
            task_repository.clone(),
            task_schedule_repository.clone(),
            ordering_repository.clone(),
        ));

        let ordering_service = Arc::new(OrderingService::new(
            clock.clone(),
            id_generator.clone(),
            task_repository.clone(),
            ordering_repository.clone(),
        ));

        let time_block_service = Arc::new(TimeBlockService::new(
            clock.clone(),
            id_generator.clone(),
            task_repository.clone(),
            time_block_repository.clone(),
        ));

        let template_service = Arc::new(TemplateService::new(
            clock.clone(),
            id_generator.clone(),
            template_repository.clone(),
            task_repository.clone(),
        ));

        let area_service = Arc::new(AreaService::new(
            clock.clone(),
            id_generator.clone(),
            area_repository.clone(),
            task_repository.clone(),
        ));

        Ok(Self {
            config,
            db_pool,
            clock,
            id_generator,
            setting_repository,
            task_repository,
            task_schedule_repository,
            ordering_repository,
            area_repository,
            template_repository,
            time_block_repository,
            task_service,
            schedule_service,
            ordering_service,
            time_block_service,
            template_service,
            area_service,
        })
    }

    /// 获取应用配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取数据库连接池
    pub fn db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }

    /// 检查应用状态的健康性
    pub async fn health_check(&self) -> Result<AppHealthStatus, crate::common::error::AppError> {
        let mut status = AppHealthStatus {
            overall: HealthStatus::Healthy,
            database: HealthStatus::Healthy,
            repositories: HealthStatus::Healthy,
            external_dependencies: HealthStatus::Healthy,
            details: Vec::new(),
        };

        // 检查数据库连接
        match sqlx::query("SELECT 1")
            .fetch_one(self.db_pool.as_ref())
            .await
        {
            Ok(_) => {
                status.details.push("Database connection: OK".to_string());
            }
            Err(e) => {
                status.database = HealthStatus::Unhealthy;
                status.overall = HealthStatus::Unhealthy;
                status
                    .details
                    .push(format!("Database connection: ERROR - {}", e));
            }
        }

        // 检查设置仓库
        match self.setting_repository.count().await {
            Ok(count) => {
                status
                    .details
                    .push(format!("Settings repository: OK ({} settings)", count));
            }
            Err(e) => {
                status.repositories = HealthStatus::Degraded;
                if status.overall == HealthStatus::Healthy {
                    status.overall = HealthStatus::Degraded;
                }
                status
                    .details
                    .push(format!("Settings repository: WARNING - {}", e));
            }
        }

        // 检查任务仓库
        match self.task_repository.count_by_status().await {
            Ok(stats) => {
                status
                    .details
                    .push(format!("Task repository: OK ({} tasks)", stats.total));
            }
            Err(e) => {
                status.repositories = HealthStatus::Unhealthy;
                status.overall = HealthStatus::Unhealthy;
                status
                    .details
                    .push(format!("Task repository: ERROR - {}", e));
            }
        }

        Ok(status)
    }

    /// 优雅关闭应用状态
    pub async fn shutdown(&self) -> Result<(), crate::common::error::AppError> {
        log::info!("Shutting down application state...");

        // 关闭数据库连接池
        self.db_pool.close().await;
        log::info!("Database connection pool closed");

        log::info!("Application state shutdown complete");
        Ok(())
    }
}

/// 应用健康状态
#[derive(Debug, Clone)]
pub struct AppHealthStatus {
    /// 整体状态
    pub overall: HealthStatus,

    /// 数据库状态
    pub database: HealthStatus,

    /// 仓库状态
    pub repositories: HealthStatus,

    /// 外部依赖状态
    pub external_dependencies: HealthStatus,

    /// 详细信息
    pub details: Vec<String>,
}

/// 健康状态枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 降级（部分功能不可用）
    Degraded,
    /// 不健康
    Unhealthy,
}

impl HealthStatus {
    /// 转换为HTTP状态码
    pub fn to_http_status(&self) -> u16 {
        match self {
            HealthStatus::Healthy => 200,
            HealthStatus::Degraded => 206,  // Partial Content
            HealthStatus::Unhealthy => 503, // Service Unavailable
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            HealthStatus::Healthy => "healthy",
            HealthStatus::Degraded => "degraded",
            HealthStatus::Unhealthy => "unhealthy",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_http_mapping() {
        assert_eq!(HealthStatus::Healthy.to_http_status(), 200);
        assert_eq!(HealthStatus::Degraded.to_http_status(), 206);
        assert_eq!(HealthStatus::Unhealthy.to_http_status(), 503);
    }

    #[test]
    fn test_health_status_string() {
        assert_eq!(HealthStatus::Healthy.as_str(), "healthy");
        assert_eq!(HealthStatus::Degraded.as_str(), "degraded");
        assert_eq!(HealthStatus::Unhealthy.as_str(), "unhealthy");
    }

    #[test]
    fn test_app_health_status_creation() {
        let status = AppHealthStatus {
            overall: HealthStatus::Healthy,
            database: HealthStatus::Healthy,
            repositories: HealthStatus::Healthy,
            external_dependencies: HealthStatus::Healthy,
            details: vec!["All systems operational".to_string()],
        };

        assert_eq!(status.overall, HealthStatus::Healthy);
        assert_eq!(status.details.len(), 1);
    }
}
