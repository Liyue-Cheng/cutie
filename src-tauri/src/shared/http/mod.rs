pub mod error_handler;
pub mod extractors;
/// HTTP共享模块
///
/// 包含：
/// - middleware: HTTP中间件
/// - error_handler: 错误处理
/// - responses: 通用响应结构
/// - extractors: 请求提取器
pub mod middleware;
pub mod responses;

// 重新导出常用类型
pub use error_handler::*;
pub use extractors::*;
pub use middleware::*;
pub use responses::*;
