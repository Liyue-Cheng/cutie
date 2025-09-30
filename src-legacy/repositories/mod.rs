pub mod implementations;
/// 数据访问/仓库层 (Data Access / Repository Layer)
///
/// 本层封装了所有与数据库的直接交互。它提供了一系列面向领域对象的、强类型的接口（Trait），
/// 将SQL查询、数据库事务管理等底层细节完全隐藏。本层是数据库Schema和服务层业务逻辑之间的唯一桥梁。
pub mod traits;

// 重新导出所有trait
pub use traits::*;

// 重新导出所有实现
pub use implementations::*;
