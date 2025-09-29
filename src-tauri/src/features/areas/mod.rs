/// 领域功能模块
///
/// 包含完整的领域管理功能，按照单文件组件的方式组织
///
/// 包含：
/// - repository: 领域数据访问层
/// - service: 领域业务逻辑层
/// - handlers: 领域HTTP处理器
/// - payloads: 领域请求/响应载荷

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

/// 创建领域功能模块的路由
pub fn create_routes<S>(pool: SqlitePool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let repository = SqlxAreaRepository::new(pool);
    let service = AreaService::new(repository);

    // 直接返回领域路由
    handlers::create_area_routes(service)
}
