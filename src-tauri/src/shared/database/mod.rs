/// 数据库共享模块
///
/// 包含：
/// - connection: 数据库连接管理
/// - pagination: 分页支持
pub mod connection;
pub mod pagination;

// 重新导出常用类型
pub use connection::*;
pub use pagination::*;
