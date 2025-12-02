/// TimeBlockRecurrence endpoints
///
/// 时间块循环规则相关的HTTP端点
pub use create_time_block_recurrence::handle as create_time_block_recurrence;
pub use delete_time_block_recurrence::handle as delete_time_block_recurrence;
pub use edit_time_block_recurrence::handle as edit_time_block_recurrence;
pub use get_time_block_recurrence::handle as get_time_block_recurrence;
pub use list_time_block_recurrences::handle as list_time_block_recurrences;
pub use resume_time_block_recurrence::handle as resume_time_block_recurrence;
pub use stop_time_block_recurrence::handle as stop_time_block_recurrence;
pub use update_time_block_recurrence::handle as update_time_block_recurrence;

mod create_time_block_recurrence;
mod delete_time_block_recurrence;
mod edit_time_block_recurrence;
mod get_time_block_recurrence;
mod list_time_block_recurrences;
mod resume_time_block_recurrence;
mod stop_time_block_recurrence;
mod update_time_block_recurrence;
