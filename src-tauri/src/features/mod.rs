/// 功能模块
///
/// 按功能切片组织的业务逻辑模块
/// 每个功能模块都是独立的，包含完整的数据访问、业务逻辑和HTTP处理
use axum::Router;
use sqlx::SqlitePool;

pub mod api_router;
pub mod areas;
pub mod ordering;
pub mod schedules;
pub mod startup_new;
pub mod tasks;
pub mod templates;
pub mod time_blocks;

/// 创建所有功能模块的路由
pub fn create_feature_routes<S>(pool: SqlitePool) -> Router<S> 
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .nest("/tasks", tasks::create_routes(pool.clone()))
        .nest("/schedules", schedules::create_routes(pool.clone()))
        .nest("/time-blocks", time_blocks::create_routes(pool.clone()))
        .nest("/templates", templates::create_routes(pool.clone()))
        .nest("/areas", areas::create_routes(pool.clone()))
        .nest("/ordering", ordering::create_routes(pool.clone()))
}

// 重新导出主要功能
pub use tasks::*;
