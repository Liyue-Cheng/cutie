/// 时间块功能模块
///
/// 包含完整的时间块管理功能，按照单文件组件的方式组织

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

/// 创建时间块功能模块的路由
pub fn create_routes<S>(pool: SqlitePool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let repository = SqlxTimeBlockRepository::new(pool.clone());
    let task_repository = SqlxTimeBlockTaskRepository::new(pool);
    let service = TimeBlockService::new(repository, task_repository);

    handlers::create_time_block_routes(service)
}
