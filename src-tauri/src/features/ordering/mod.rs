/// 排序功能模块
///
/// 包含完整的排序管理功能，按照单文件组件的方式组织
///
/// 包含：
/// - repository: 排序数据访问层
/// - service: 排序业务逻辑层
/// - handlers: 排序HTTP处理器
/// - payloads: 排序请求/响应载荷

use axum::Router;
use sqlx::SqlitePool;

pub mod handlers;
pub mod payloads;
pub mod repository;
pub mod service;

pub use handlers::*;
pub use payloads::*;
pub use repository::*;
pub use service::*;

/// 创建排序功能模块的路由
pub fn create_routes<S>(pool: SqlitePool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let repository = SqlxOrderingRepository::new(pool);
    let service = OrderingService::new(repository);

    // 直接返回排序路由
    handlers::create_ordering_routes(service)
}
