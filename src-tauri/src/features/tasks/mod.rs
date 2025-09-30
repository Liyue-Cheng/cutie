/// 任务功能模块 - 单文件组件架构
///
/// 架构原则：
/// - endpoints/ 目录存放纯粹的 SFC 文件（无需 mod.rs）
/// - 每个 SFC 文件导出 `pub async fn handle(...)`
/// - 本文件直接声明 endpoints 子模块并在路由中使用
/// - DTOs 在 entities 模块中定义
use axum::{routing::post, Router};

use crate::startup::AppState;

// 共享模块（装配器等工具）
pub mod shared;

// 直接声明 endpoints 子模块（无需 pub，只内部使用）
mod endpoints {
    pub mod legacy; // complete_task 的示例实现
                    // 待实现的其他端点：
                    // pub mod create_task;
                    // pub mod get_task;
                    // pub mod update_task;
                    // pub mod delete_task;
                    // pub mod reopen_task;
}

/// 创建任务功能模块的路由
///
/// 直接使用 endpoints::文件名::handle 的方式引用处理器
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // 基本 CRUD 操作（待实现）
        // .route("/", post(endpoints::create_task::handle))
        // .route("/:id",
        //     get(endpoints::get_task::handle)
        //         .patch(endpoints::update_task::handle)
        //         .delete(endpoints::delete_task::handle)
        // )
        // 任务状态操作
        .route(
            "/:id/completion",
            post(endpoints::legacy::handle), // complete_task
                                             // .delete(endpoints::reopen_task::handle)  // 待实现
        )

    // 查询操作（待实现）
    // .route("/search", get(endpoints::search_tasks::handle))
    // .route("/stats", get(endpoints::get_stats::handle))
}
