/// 测试工具模块
///
/// 提供测试所需的基础设施：
/// - 测试数据库创建与清理
/// - 测试数据构造器（fixtures）
/// - HTTP 客户端辅助函数
pub mod database;
pub mod fixtures;
pub mod http_client;

// 重新导出常用测试工具
pub use database::{create_test_db, TestDb};
pub use fixtures::TaskFixture;
