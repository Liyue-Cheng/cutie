/// HTTP响应结构定义
///
/// 定义所有API端点的响应体结构，确保响应格式的一致性
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::core::ValidationError;

/// 标准API响应包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// 响应数据
    pub data: T,

    /// 响应时间戳
    pub timestamp: DateTime<Utc>,

    /// 请求ID（用于追踪）
    pub request_id: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 创建成功响应
    pub fn success(data: T) -> Self {
        Self {
            data,
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    /// 创建带请求ID的成功响应
    pub fn success_with_id(data: T, request_id: String) -> Self {
        Self {
            data,
            timestamp: chrono::Utc::now(),
            request_id: Some(request_id),
        }
    }
}

/// 错误响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// 错误类型
    pub error_type: String,

    /// 错误消息
    pub message: String,

    /// 详细信息
    pub details: Option<serde_json::Value>,

    /// 错误代码
    pub code: Option<String>,

    /// 响应时间戳
    pub timestamp: DateTime<Utc>,

    /// 请求ID
    pub request_id: Option<String>,
}

impl ErrorResponse {
    /// 创建通用错误响应
    pub fn new(error_type: String, message: String) -> Self {
        Self {
            error_type,
            message,
            details: None,
            code: None,
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    /// 创建验证错误响应
    pub fn validation_error(errors: Vec<ValidationError>) -> Self {
        let details = serde_json::json!({
            "validation_errors": errors.iter().map(|e| {
                serde_json::json!({
                    "field": e.field,
                    "message": e.message,
                    "code": e.code
                })
            }).collect::<Vec<_>>()
        });

        Self {
            error_type: "ValidationError".to_string(),
            message: "输入数据验证失败".to_string(),
            details: Some(details),
            code: Some("VALIDATION_FAILED".to_string()),
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    /// 创建未找到错误响应
    pub fn not_found(entity_type: String, entity_id: String) -> Self {
        Self {
            error_type: "NotFound".to_string(),
            message: format!("{} with id {} not found", entity_type, entity_id),
            details: Some(serde_json::json!({
                "entity_type": entity_type,
                "entity_id": entity_id
            })),
            code: Some("NOT_FOUND".to_string()),
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    /// 创建冲突错误响应
    pub fn conflict(message: String) -> Self {
        Self {
            error_type: "Conflict".to_string(),
            message,
            details: None,
            code: Some("CONFLICT".to_string()),
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    /// 创建内部服务器错误响应
    pub fn internal_error(message: String) -> Self {
        Self {
            error_type: "InternalError".to_string(),
            message,
            details: None,
            code: Some("INTERNAL_ERROR".to_string()),
            timestamp: chrono::Utc::now(),
            request_id: None,
        }
    }

    /// 设置请求ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}

/// 分页响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// 数据列表
    pub items: Vec<T>,

    /// 总数量
    pub total: i64,

    /// 当前页
    pub page: i64,

    /// 每页大小
    pub page_size: i64,

    /// 是否有下一页
    pub has_next: bool,

    /// 是否有上一页
    pub has_prev: bool,
}

impl<T> PaginatedResponse<T> {
    /// 创建分页响应
    pub fn new(items: Vec<T>, total: i64, page: i64, page_size: i64) -> Self {
        let has_next = (page * page_size) < total;
        let has_prev = page > 1;

        Self {
            items,
            total,
            page,
            page_size,
            has_next,
            has_prev,
        }
    }
}

/// 统计响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    /// 统计数据
    pub stats: serde_json::Value,

    /// 生成时间
    pub generated_at: DateTime<Utc>,

    /// 统计范围（可选）
    pub range: Option<DateRange>,
}

/// 日期范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    /// 开始日期
    pub start: DateTime<Utc>,

    /// 结束日期
    pub end: DateTime<Utc>,
}

/// 批量操作响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperationResponse {
    /// 成功处理的数量
    pub success_count: i64,

    /// 失败的数量
    pub failed_count: i64,

    /// 失败的详细信息
    pub failures: Vec<BatchFailure>,
}

/// 批量操作失败项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFailure {
    /// 失败的项目ID
    pub item_id: String,

    /// 错误消息
    pub error: String,

    /// 错误代码
    pub code: Option<String>,
}

/// 健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// 状态
    pub status: String,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 版本信息
    pub version: String,

    /// 详细信息
    pub details: Option<serde_json::Value>,
}

/// Ping响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    /// 消息
    pub message: String,

    /// 时间戳
    pub timestamp: DateTime<Utc>,
}

/// 服务器信息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfoResponse {
    /// 名称
    pub name: String,

    /// 版本
    pub version: String,

    /// 构建时间
    pub build_time: String,

    /// Rust版本
    pub rust_version: String,

    /// 功能特性
    pub features: Vec<String>,
}

/// 空响应（用于204 No Content）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyResponse;

/// 成功消息响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    /// 消息内容
    pub message: String,

    /// 操作类型
    pub operation: String,

    /// 影响的实体数量
    pub affected_count: Option<i64>,
}

impl MessageResponse {
    /// 创建成功消息
    pub fn success(operation: String, message: String) -> Self {
        Self {
            message,
            operation,
            affected_count: None,
        }
    }

    /// 创建带影响数量的成功消息
    pub fn success_with_count(operation: String, message: String, count: i64) -> Self {
        Self {
            message,
            operation,
            affected_count: Some(count),
        }
    }
}

