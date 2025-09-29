/// 功能模块
///
/// 按功能切片组织的业务逻辑模块
/// 每个功能模块都是独立的，包含完整的数据访问、业务逻辑和HTTP处理
use axum::Router;

use crate::startup::AppState;

pub mod api_router;
pub mod tasks;

/// 创建所有功能模块的路由
pub fn create_feature_routes() -> Router<AppState> {
    Router::new().nest("/tasks", tasks::create_routes())
}

// 重新导出主要功能
pub use tasks::*;
