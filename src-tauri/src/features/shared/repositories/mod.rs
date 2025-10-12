/// 共享数据仓库层
///
/// 包含所有功能模块共享的数据访问逻辑
/// 
/// # 架构设计
/// 
/// - `traits.rs`: 定义 Repository 的通用接口
/// - `transaction.rs`: 事务辅助工具
/// - 各个具体的 Repository 实现

// Repository trait 定义
pub mod traits;

// 事务辅助工具
pub mod transaction;

// 具体的 Repository 实现
pub mod area_repository;
pub mod task_recurrence_link_repository;
pub mod task_recurrence_repository;
pub mod task_repository;
pub mod task_schedule_repository;
pub mod task_time_block_link_repository;
pub mod time_block_repository;

// 重新导出常用类型

// Repository traits
pub use traits::{Repository, QueryableRepository, BatchRepository};

// 事务辅助工具
pub use transaction::*;

// 具体的 Repository 实现
pub use area_repository::AreaRepository;
pub use task_recurrence_link_repository::TaskRecurrenceLinkRepository;
pub use task_recurrence_repository::TaskRecurrenceRepository;
pub use task_repository::TaskRepository;
pub use task_schedule_repository::TaskScheduleRepository;
pub use task_time_block_link_repository::TaskTimeBlockLinkRepository;
pub use time_block_repository::TimeBlockRepository;
