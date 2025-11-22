/// View endpoints
///
/// 视图相关的HTTP端点
pub use get_all_incomplete::handle as get_all_incomplete;
pub use get_daily_tasks::handle as get_daily_tasks;
pub use get_daily_tasks_batch::handle as get_daily_tasks_batch;
pub use get_planned::handle as get_planned;
pub use get_staging_view::handle as get_staging_view;

mod get_all_incomplete;
mod get_daily_tasks;
mod get_daily_tasks_batch;
mod get_planned;
mod get_staging_view;
