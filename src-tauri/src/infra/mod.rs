/// 基础设施模块（Infrastructure Layer）
///
/// 提供与业务无关的技术性基础组件
///
/// # 架构定位
///
/// 本模块位于分层架构的最底层，为上层业务逻辑提供技术支持。
/// 不包含任何业务规则，只负责技术实现细节。
///
/// # 模块组成
///
/// - `core`: 核心基础设施（错误类型、工具函数、构建信息）
/// - `database`: 数据库连接和事务管理
/// - `http`: HTTP 基础设施（中间件、响应构建、错误处理）
/// - `ports`: 外部依赖抽象层（时钟、ID生成器等）
/// - `events`: 事件系统基础设施（SSE、事件分发、Outbox）
/// - `logging`: 统一日志系统（分层标签、文件轮转、panic捕获）
///
/// # 与业务层的区别
///
/// - `infra/`: 技术关注点（如何实现）- 数据库、HTTP、日志等
/// - `features/shared/`: 业务关注点（做什么）- Repositories、Services、Validators
pub mod core;
pub mod database;
pub mod events;
pub mod http;
pub mod logging;
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
//   - use crate::infra::http::extractors::{PaginationQuery, SortOrder};
//   - use crate::infra::database::pagination::{PaginationQuery, SortOrder};
