/// View Preference endpoints
/// 
/// 视图偏好设置相关的HTTP端点

pub use get_view_preference::handle as get_view_preference;
pub use save_view_preference::handle as save_view_preference;

mod get_view_preference;
mod save_view_preference;
