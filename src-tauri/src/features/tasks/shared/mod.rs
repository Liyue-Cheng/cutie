pub mod dtos;
/// Tasks功能模块的共享基础设施
///
/// 包含所有API端点共享的组件：
/// - dtos: 数据传输对象
/// - validation: 验证逻辑
///
/// 注意：repository层已移动到根目录的repositories模块
pub mod validation;

pub use dtos::*;
pub use validation::*;
