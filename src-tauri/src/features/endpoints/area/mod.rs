/// Area endpoints
///
/// 区域相关的HTTP端点
pub use create_area::handle as create_area;
pub use delete_area::handle as delete_area;
pub use get_area::handle as get_area;
pub use list_areas::handle as list_areas;
pub use suggest_color::handle as suggest_color;
pub use update_area::handle as update_area;

mod create_area;
mod delete_area;
mod get_area;
mod list_areas;
mod suggest_color;
mod update_area;
