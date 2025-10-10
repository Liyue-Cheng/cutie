/// 应用状态模块 - 为sidecar架构设计
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::{Semaphore, OwnedSemaphorePermit};

use crate::config::AppConfig;
use crate::shared::core::AppError;
use crate::shared::events::SseState;

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

    /// SSE 状态（事件广播）
    sse_state: Arc<SseState>,

    /// 写入串行化信号量（SQLite 写锁优化）
    /// 确保所有写操作在应用层串行执行，避免数据库锁冲突
    write_semaphore: Arc<Semaphore>,
}

impl AppState {
    /// 创建新的应用状态（完整构造）
    pub fn new(
        config: AppConfig,
        db_pool: SqlitePool,
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        sse_state: Arc<SseState>,
    ) -> Self {
        Self {
            config: Arc::new(config),
            db_pool: Arc::new(db_pool),
            clock,
            id_generator,
            sse_state,
            // ✅ 写入串行化：permits=1，确保同一时刻只有一个写事务
            write_semaphore: Arc::new(Semaphore::new(1)),
        }
    }

    /// 创建生产环境的应用状态（使用默认的生产适配器）
    pub fn new_production(config: AppConfig, db_pool: SqlitePool) -> Self {
        Self::new(
            config,
            db_pool,
            Arc::new(SystemClock::new()),
            Arc::new(UuidV4Generator::new()),
            Arc::new(SseState::new()),
        )
    }

    /// 创建测试环境的应用状态（使用默认配置和测试适配器）
    #[cfg(test)]
    pub fn new_test(db_pool: SqlitePool) -> Self {
        let config = AppConfig::default();
        Self::new(
            config,
            db_pool,
            Arc::new(SystemClock::new()),
            Arc::new(UuidV4Generator::new()),
            Arc::new(SseState::new()),
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

    /// 获取 SSE 状态
    pub fn sse_state(&self) -> &Arc<SseState> {
        &self.sse_state
    }

    /// 获取写入信号量（用于外部注入，如 EventDispatcher）
    pub fn write_semaphore(&self) -> Arc<Semaphore> {
        self.write_semaphore.clone()
    }

    /// 获取写入许可（串行化写操作）
    ///
    /// 在所有写事务开始前调用此方法，确保应用层写操作串行执行。
    /// 许可在返回的 OwnedSemaphorePermit 被 drop 时自动释放。
    ///
    /// # Example
    /// ```rust
    /// let _permit = app_state.acquire_write_permit().await;
    /// let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;
    /// // ... 写操作 ...
    /// TransactionHelper::commit(tx).await?;
    /// // _permit 自动 drop，释放许可
    /// ```
    pub async fn acquire_write_permit(&self) -> OwnedSemaphorePermit {
        self.write_semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("Write semaphore should never be closed")
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
