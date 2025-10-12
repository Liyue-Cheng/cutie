/// Trash endpoints
/// 
/// 回收站相关的HTTP端点

pub use empty_trash::handle as empty_trash;
pub use list_trash::handle as list_trash;

mod empty_trash;
mod list_trash;
