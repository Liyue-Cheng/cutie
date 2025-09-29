/// 新的功能切片API路由器
///
/// 使用新的功能模块架构组织API路由
use axum::Router;

use super::create_feature_routes;
use crate::startup::AppState;

/// 创建新架构的API路由器
///
/// 这个函数替换了旧的分层架构路由，使用新的功能切片模块
pub fn create_new_api_router() -> Router<AppState> {
    Router::new().merge(create_feature_routes())
}
