pub mod area_repository;
pub mod ordering_repository;
/// 数据访问/仓库层
///
/// 本层封装了所有与数据库的直接交互。它提供了一系列面向领域对象的、强类型的接口（Trait），
/// 将SQL查询、数据库事务管理等底层细节完全隐藏。本层是数据库Schema和服务层业务逻辑之间的唯一桥梁。
pub mod task_repository;
pub mod task_schedule_repository;
pub mod template_repository;
pub mod time_block_repository;

// SQLx实现
pub mod sqlx_area_repository;
pub mod sqlx_ordering_repository;
pub mod sqlx_task_repository;
pub mod sqlx_task_schedule_repository;
pub mod sqlx_template_repository;
pub mod sqlx_time_block_repository;

// 内存测试实现
pub mod memory_repositories;

pub use area_repository::*;
pub use ordering_repository::*;
pub use task_repository::*;
pub use task_schedule_repository::*;
pub use template_repository::*;
pub use time_block_repository::*;

pub use sqlx_area_repository::*;
pub use sqlx_ordering_repository::*;
pub use sqlx_task_repository::*;
pub use sqlx_task_schedule_repository::*;
pub use sqlx_template_repository::*;
pub use sqlx_time_block_repository::*;

pub use memory_repositories::*;
