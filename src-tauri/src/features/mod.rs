/// 功能模块 - 单文件组件架构
///
/// 按功能切片组织的业务逻辑模块
/// 每个功能模块都是独立的，包含完整的数据访问、业务逻辑和HTTP处理
///
/// 架构原则：
/// - 每个功能目录下有 mod.rs 和 endpoints/ 目录
/// - endpoints/ 目录存放纯粹的 SFC 文件（无需 mod.rs）
/// - 功能的 mod.rs 直接声明和使用 endpoints 子模块
use axum::Router;

use crate::startup::AppState;

pub mod areas;
pub mod tasks;
pub mod time_blocks;
pub mod view_preferences;
pub mod views;
// 其他功能模块（待迁移）
// pub mod schedules;
// pub mod templates;

/// 创建所有功能模块的API路由器
///
/// 这是应用的主路由入口，聚合所有功能模块的路由
pub fn create_api_router() -> Router<AppState> {
    use axum::routing::get;
    use crate::shared::events::sse;
    
    Router::new()
        .nest("/areas", areas::create_routes())
        .nest("/tasks", tasks::create_routes())
        .nest("/time-blocks", time_blocks::create_routes())
        .nest("/view-preferences", view_preferences::create_routes())
        .nest("/views", views::create_routes())
        .route("/events/stream", get(sse::handle))
    // 其他路由（待迁移）
    // .nest("/schedules", schedules::create_routes())
    // .nest("/templates", templates::create_routes())
}
