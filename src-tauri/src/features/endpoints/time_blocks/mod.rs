/// Time Block endpoints
///
/// 时间块相关的HTTP端点
pub use create_from_task::handle as create_from_task;
pub use create_time_block::handle as create_time_block;
pub use delete_time_block::handle as delete_time_block;
pub use link_task::handle as link_task;
pub use list_time_blocks::handle as list_time_blocks;
pub use update_time_block::handle as update_time_block;

mod create_from_task;
mod create_time_block;
mod delete_time_block;
mod link_task;
mod list_time_blocks;
mod update_time_block;
