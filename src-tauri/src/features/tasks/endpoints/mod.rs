/// 任务API端点 - 单文件组件集合
///
/// 每个文件包含一个完整的API实现，包括路由、业务逻辑和数据访问
pub mod complete_task;
pub mod create_task;
pub mod get_task;
// TODO: 添加其他API端点
// pub mod update_task;
// pub mod delete_task;
// pub mod search_tasks;
// pub mod get_unscheduled_tasks;
// pub mod get_task_stats;
// pub mod reopen_task;

// 重新导出处理器函数
pub use complete_task::handle as complete_task_handler;
pub use create_task::handle as create_task_handler;
pub use get_task::handle as get_task_handler;
