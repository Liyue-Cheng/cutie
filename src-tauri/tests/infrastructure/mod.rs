/// 测试基础设施模块
///
/// 提供测试中使用的所有工具类和辅助函数
pub mod database;
pub mod fixtures;
pub mod http_client;
pub mod test_helpers;

// 重新导出常用工具
pub use database::{create_test_db, TestDb};
pub use fixtures::{AreaFixture, TaskFixture};
pub use http_client::TestClient;
pub use test_helpers::create_test_app_state;
