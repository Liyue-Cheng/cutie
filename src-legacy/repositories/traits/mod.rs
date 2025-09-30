pub mod area_repository;
pub mod ordering_repository;
/// Repository trait定义
///
/// 定义了所有数据访问层的接口规范
pub mod task_repository;
pub mod task_schedule_repository;
pub mod template_repository;
pub mod time_block_repository;

// 重新导出所有trait
pub use area_repository::*;
pub use ordering_repository::*;
pub use task_repository::*;
pub use task_schedule_repository::*;
pub use template_repository::*;
pub use time_block_repository::*;
