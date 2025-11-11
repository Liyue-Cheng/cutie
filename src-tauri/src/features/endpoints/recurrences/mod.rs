/// Recurrence endpoints
///
/// 循环任务相关的HTTP端点
pub use batch_update_instances::handle as batch_update_instances;
pub use batch_update_template_and_instances::handle as batch_update_template_and_instances;
pub use create_recurrence::handle as create_recurrence;
pub use delete_recurrence::handle as delete_recurrence;
pub use list_recurrences::handle as list_recurrences;
pub use update_recurrence::handle as update_recurrence;

mod batch_update_instances;
mod batch_update_template_and_instances;
mod create_recurrence;
mod delete_recurrence;
mod list_recurrences;
mod update_recurrence;
