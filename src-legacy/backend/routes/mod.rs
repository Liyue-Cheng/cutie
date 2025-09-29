pub mod api_router;
pub mod area_routes;
pub mod ordering_routes;
pub mod schedule_routes;
/// 路由模块
///
/// 组织和配置所有API路由
pub mod task_routes;
pub mod template_routes;
pub mod time_block_routes;

pub use api_router::*;
pub use area_routes::*;
pub use ordering_routes::*;
pub use schedule_routes::*;
pub use task_routes::*;
pub use template_routes::*;
pub use time_block_routes::*;
