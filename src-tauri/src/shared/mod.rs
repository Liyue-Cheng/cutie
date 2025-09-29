/// Shared模块 - 提供跨功能模块的通用组件
///
/// 包含：
/// - core: 核心领域模型、错误类型、工具函数
/// - database: 数据库连接和通用仓库trait
/// - http: HTTP中间件、错误处理、通用响应
pub mod core;
pub mod database;
pub mod http;

// 重新导出常用类型
pub use core::*;
pub use database::*;
pub use http::*;
