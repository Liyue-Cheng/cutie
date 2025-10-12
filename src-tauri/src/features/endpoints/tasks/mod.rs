/// Task endpoints
///
/// 任务相关的HTTP端点
pub use add_schedule::handle as add_schedule;
pub use archive_task::handle as archive_task;
pub use complete_task::handle as complete_task;
pub use create_task::handle as create_task;
pub use create_task_with_schedule::handle as create_task_with_schedule;
pub use delete_schedule::handle as delete_schedule;
pub use delete_task::handle as delete_task;
pub use get_task::handle as get_task;
pub use permanently_delete_task::handle as permanently_delete_task;
pub use reopen_task::handle as reopen_task;
pub use restore_task::handle as restore_task;
pub use return_to_staging::handle as return_to_staging;
pub use unarchive_task::handle as unarchive_task;
pub use update_schedule::handle as update_schedule;
pub use update_task::handle as update_task;

mod add_schedule;
mod archive_task;
mod complete_task;
mod create_task;
mod create_task_with_schedule;
mod delete_schedule;
mod delete_task;
mod get_task;
mod permanently_delete_task;
mod reopen_task;
mod restore_task;
mod return_to_staging;
mod unarchive_task;
mod update_schedule;
mod update_task;
