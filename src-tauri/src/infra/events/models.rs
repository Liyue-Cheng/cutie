/// 事件模型定义
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 领域事件信封
///
/// 采用标准事件溯源信封模式，包含元数据与载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEvent {
    /// 事件唯一ID（UUID）
    pub event_id: Uuid,

    /// 事件类型（如 task.completed, time_blocks.truncated）
    pub event_type: String,

    /// 事件契约版本
    pub version: i32,

    /// 聚合类型（如 task, time_block）
    pub aggregate_type: String,

    /// 聚合根ID
    pub aggregate_id: String,

    /// 聚合版本（用于幂等，可为空）
    pub aggregate_version: Option<i64>,

    /// 关联的命令ID（HTTP 请求 correlation_id）
    pub correlation_id: Option<String>,

    /// 事件发生时间（UTC）
    pub occurred_at: DateTime<Utc>,

    /// 事件载荷（JSON）
    pub payload: serde_json::Value,
}

impl DomainEvent {
    /// 创建新事件
    pub fn new(
        event_type: impl Into<String>,
        aggregate_type: impl Into<String>,
        aggregate_id: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type: event_type.into(),
            version: 1,
            aggregate_type: aggregate_type.into(),
            aggregate_id: aggregate_id.into(),
            aggregate_version: None,
            correlation_id: None,
            occurred_at: Utc::now(),
            payload,
        }
    }

    /// 设置聚合版本（用于幂等）
    pub fn with_aggregate_version(mut self, version: i64) -> Self {
        self.aggregate_version = Some(version);
        self
    }

    /// 设置关联ID（用于去重）
    pub fn with_correlation_id(mut self, correlation_id: impl Into<String>) -> Self {
        self.correlation_id = Some(correlation_id.into());
        self
    }

    /// 转换为 SSE 格式字符串
    pub fn to_sse_data(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Outbox 表行结构
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EventOutboxRow {
    pub id: i64,
    pub event_id: String,
    pub event_type: String,
    pub version: i32,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub aggregate_version: Option<i64>,
    pub correlation_id: Option<String>,
    pub occurred_at: String,
    pub payload: String,
    pub dispatched_at: Option<String>,
    pub created_at: String,
}

impl EventOutboxRow {
    /// 转换为 DomainEvent
    pub fn to_domain_event(&self) -> Result<DomainEvent, serde_json::Error> {
        Ok(DomainEvent {
            event_id: Uuid::parse_str(&self.event_id).unwrap_or_else(|_| Uuid::new_v4()),
            event_type: self.event_type.clone(),
            version: self.version,
            aggregate_type: self.aggregate_type.clone(),
            aggregate_id: self.aggregate_id.clone(),
            aggregate_version: self.aggregate_version,
            correlation_id: self.correlation_id.clone(),
            occurred_at: DateTime::parse_from_rfc3339(&self.occurred_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            payload: serde_json::from_str(&self.payload)?,
        })
    }
}
