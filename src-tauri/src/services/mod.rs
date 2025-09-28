pub mod area_service;
/// 业务/服务层
///
/// 本层包含系统的核心业务逻辑。每个服务封装了一组相关的业务操作，
/// 并负责事务管理、业务规则验证、以及跨仓库的复杂操作协调。
pub mod dtos;
pub mod ordering_service;
pub mod schedule_service;
pub mod task_service;
pub mod template_service;
pub mod time_block_service;

pub use area_service::*;
pub use dtos::*;
pub use ordering_service::*;
pub use schedule_service::*;
pub use task_service::*;
pub use template_service::*;
pub use time_block_service::*;
