pub mod error;
/// 核心领域模块
///
/// 包含：
/// - models: 核心领域实体
/// - error: 错误类型定义
/// - utils: 通用工具函数
pub mod models;
pub mod utils;

// 重新导出常用类型
pub use error::*;
pub use models::*;
pub use utils::*;

