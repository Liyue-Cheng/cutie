/// 统一日志系统模块
///
/// 提供分层标签化的日志机制，支持：
/// - 多层级标签（LAYER:FEATURE:COMPONENT）
/// - 文件落盘与轮转（按天，保留14天）
/// - Panic 捕获与独立日志文件
/// - 请求链路追踪（req_id）
/// - 结构化日志字段
pub mod config;
pub mod init;
pub mod macros;
pub mod middleware;
pub mod panic_handler;
pub mod tags;

// 重新导出核心功能
pub use config::LogConfig;
pub use init::{init_logging, init_logging_with_config};
pub use middleware::{request_tracing_middleware, slow_request_middleware, REQUEST_ID_HEADER};
pub use panic_handler::setup_panic_handler;
pub use tags::*;
