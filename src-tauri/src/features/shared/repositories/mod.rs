/// 共享数据仓库层
///
/// 包含所有功能模块共享的数据访问逻辑
pub mod area_repository;
pub mod task_recurrence_link_repository;
pub mod task_recurrence_repository;
pub mod task_repository;
pub mod task_schedule_repository;
pub mod task_time_block_link_repository;
pub mod time_block_repository;
pub mod transaction;

// 重新导出常用类型
pub use area_repository::AreaRepository;
pub use task_recurrence_link_repository::TaskRecurrenceLinkRepository;
pub use task_recurrence_repository::TaskRecurrenceRepository;
pub use task_repository::TaskRepository;
pub use task_schedule_repository::TaskScheduleRepository;
pub use task_time_block_link_repository::TaskTimeBlockLinkRepository;
pub use time_block_repository::TimeBlockRepository;
pub use transaction::*;
