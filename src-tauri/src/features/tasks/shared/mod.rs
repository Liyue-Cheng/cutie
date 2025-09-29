pub mod dtos;
/// Tasks功能模块的共享基础设施
///
/// 包含所有API端点共享的组件：
/// - repository: 数据访问层
/// - dtos: 数据传输对象
/// - validation: 验证逻辑
pub mod repository;
pub mod validation;

pub use dtos::*;
pub use repository::*;
pub use validation::*;
