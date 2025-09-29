/// 模板功能模块
///
/// 包含完整的模板系统功能，按照单文件组件的方式组织

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

/// 创建模板功能模块的路由
pub fn create_routes<S>(pool: SqlitePool) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    let repository = SqlxTemplateRepository::new(pool);
    let service = TemplateService::new(repository);

    handlers::create_template_routes(service)
}
