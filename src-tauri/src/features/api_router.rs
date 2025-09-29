/// 新的功能切片API路由器
///
/// 使用新的功能模块架构组织API路由
use axum::Router;
use sqlx::SqlitePool;

use super::create_feature_routes;

/// 创建新架构的API路由器
///
/// 这个函数替换了旧的分层架构路由，使用新的功能切片模块
pub fn create_new_api_router<S>(pool: SqlitePool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .merge(create_feature_routes(pool))
        // 添加系统级端点
        .route("/ping", axum::routing::get(ping_handler))
        .route("/health", axum::routing::get(health_handler))
        .route("/info", axum::routing::get(info_handler))
}

/// Ping处理器
async fn ping_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "message": "pong",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// 健康检查处理器
async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "1.0.0"
    }))
}

/// 服务器信息处理器
async fn info_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": "Cutie API",
        "version": "1.0.0",
        "build_time": "2024-09-29T00:00:00Z", // 简化版本，避免环境变量依赖
        "rust_version": "1.70+", // 简化版本
        "features": [
            "task_management",
            "feature_slicing",
            "async_processing"
        ]
    }))
}
