pub mod build_info;
pub mod error;
/// 核心领域模块
///
/// 包含：
/// - build_info: 构建信息
/// - error: 错误类型定义
/// - utils: 通用工具函数
pub mod utils;

// 重新导出常用类型
pub use build_info::*;
pub use error::*;
pub use utils::*;
