/// Events模块 - 领域事件与 SSE 基础设施
///
/// 包含：
/// - models: 领域事件模型定义
/// - outbox: Event Outbox 仓库（Transactional Outbox Pattern）
/// - sse: SSE 端点与事件流
/// - dispatcher: 事件分发器（后台任务）
pub mod dispatcher;
pub mod models;
pub mod outbox;
pub mod sse;

// 导出核心类型
pub use dispatcher::EventDispatcher;
pub use models::{DomainEvent, EventOutboxRow};
pub use outbox::{EventOutboxRepository, SqlxEventOutboxRepository};
pub use sse::SseState;
