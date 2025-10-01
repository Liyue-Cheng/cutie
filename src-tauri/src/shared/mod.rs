/// Shared模块 - 提供跨功能模块的通用组件
///
/// 包含：
/// - core: 核心领域模型、错误类型、工具函数
/// - database: 数据库连接和通用仓库trait
/// - http: HTTP中间件、错误处理、通用响应
/// - ports: 外部依赖抽象层（时钟、ID生成器等）
/// - events: 领域事件与 SSE 基础设施
pub mod core;
pub mod database;
pub mod events;
pub mod http;
pub mod ports;

// 显式导出最常用的核心类型，避免 ambiguous glob re-exports 警告

// Core - 错误类型和结果类型（最常用）
pub use core::{AppError, AppResult, DbError, DbResult, SortOrderError, SortResult};

// Core - 构建信息
pub use core::BuildInfo;

// HTTP - 响应构建函数（最常用）
pub use http::{created_response, success_response};

// HTTP - 响应结构
pub use http::{ApiResponse, EmptyResponse, ErrorResponse, HealthCheckResponse, MessageResponse};

// Ports - 依赖抽象接口（最常用）
pub use ports::{Clock, IdGenerator, SystemClock, UuidV4Generator};

// Database - 连接配置
pub use database::{DatabaseConfig, SynchronousMode};

// 注意：
// - PaginationQuery 和 SortOrder 在 http::extractors 和 database::pagination 中都有定义
// - 为避免歧义，不在顶层导出，使用时需要指定完整路径：
//   - use crate::shared::http::extractors::{PaginationQuery, SortOrder};
//   - use crate::shared::database::pagination::{PaginationQuery, SortOrder};
