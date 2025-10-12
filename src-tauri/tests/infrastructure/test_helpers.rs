/// 测试辅助函数
///
/// 提供创建测试环境的便捷函数
use explore_lib::{
    config::AppConfig,
    infra::{
        events::sse::SseState,
        ports::{SystemClock, UuidV4Generator},
    },
    startup::AppState,
};
use sqlx::SqlitePool;
use std::sync::Arc;

/// 创建测试用的 AppState
///
/// 使用默认配置和测试适配器
pub fn create_test_app_state(db_pool: SqlitePool) -> AppState {
    let config = AppConfig::default();
    AppState::new(
        config,
        db_pool,
        Arc::new(SystemClock::new()),
        Arc::new(UuidV4Generator::new()),
        Arc::new(SseState::new()),
    )
}
