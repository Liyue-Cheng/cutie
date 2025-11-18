/// Projects 端点模块
pub mod list_projects;
pub mod create_project;
pub mod get_project;
pub mod update_project;
pub mod delete_project;

pub mod list_sections;
pub mod create_section;
pub mod update_section;
pub mod delete_section;

// 导出 handle 函数
pub use list_projects::handle as list_projects;
pub use create_project::handle as create_project;
pub use get_project::handle as get_project;
pub use update_project::handle as update_project;
pub use delete_project::handle as delete_project;

pub use list_sections::handle as list_sections;
pub use create_section::handle as create_section;
pub use update_section::handle as update_section;
pub use delete_section::handle as delete_section;

