pub use batch_init_ranks::handle as batch_init_ranks;
/// Template endpoints
///
/// 模板相关的HTTP端点
pub use create_task_from_template::handle as create_task_from_template;
pub use create_template::handle as create_template;
pub use delete_template::handle as delete_template;
pub use list_templates::handle as list_templates;
pub use update_sort_rank::handle as update_sort_rank;
pub use update_template::handle as update_template;

mod batch_init_ranks;
mod create_task_from_template;
mod create_template;
mod delete_template;
mod list_templates;
mod update_sort_rank;
mod update_template;
