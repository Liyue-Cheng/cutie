/// 新的启动模块
///
/// 基于功能切片架构的全新启动逻辑，专为sidecar架构设计
pub mod app_state;
pub mod database;
pub mod sidecar;

pub use app_state::*;
pub use database::*;
pub use sidecar::*;
