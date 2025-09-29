/// 日程功能模块
///
/// 包含完整的日程管理功能，按照单文件组件的方式组织
///
/// 包含：
/// - repository: 日程数据访问层
/// - service: 日程业务逻辑层
/// - handlers: 日程HTTP处理器
/// - payloads: 日程请求/响应载荷
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

/// 创建日程功能模块的路由
pub fn create_routes<S>(pool: SqlitePool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let repository = SqlxTaskScheduleRepository::new(pool);
    let service = ScheduleService::new(repository);

    // 直接返回日程路由
    handlers::create_schedule_routes(service)
}
