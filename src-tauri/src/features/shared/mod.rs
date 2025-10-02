/// 跨功能模块共享基础设施
///
/// 包含所有功能模块都可能用到的通用工具和仓库
pub mod repositories;
pub mod transaction;

// 重新导出常用类型
pub use transaction::TransactionHelper;
