/// 数据库共享模块
/// 
/// 包含：
/// - connection: 数据库连接管理
/// - traits: 通用仓库trait定义
/// - pagination: 分页支持

pub mod connection;
pub mod traits;
pub mod pagination;

// 重新导出常用类型
pub use connection::*;
pub use traits::*;
pub use pagination::*;

