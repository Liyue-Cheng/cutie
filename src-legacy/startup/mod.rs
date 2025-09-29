/// 应用启动模块
/// 
/// 负责应用的初始化、依赖注入容器的构建、数据库连接池的创建等启动相关工作

pub mod app_state;
pub mod database;
pub mod sidecar;

pub use app_state::*;
pub use database::*;
pub use sidecar::*;
