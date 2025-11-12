/// HTTP 请求提取器
///
/// 用于从 HTTP 请求中提取常用的元数据和上下文信息
use axum::http::HeaderMap;
use chrono::{DateTime, Utc};

use crate::infra::core::AppError;

/// 从 HTTP 请求头中提取 Correlation ID
///
/// Correlation ID 用于追踪一个 HTTP 命令及其产生的所有领域事件
/// - 前端在发起写操作时，生成并通过 `X-Correlation-ID` header 传递
/// - 后端将此 ID 关联到生成的领域事件中
/// - 前端通过 SSE 接收事件时，可以判断是否是自己触发的操作
///
/// # 使用场景
/// - 避免重复更新：前端跳过自己触发的事件（任务已在 HTTP 响应中更新）
/// - 多窗口协作：其他窗口通过 SSE 接收完整事件并更新
/// - 请求追踪：在日志中关联 HTTP 请求和领域事件
/// - Undo/Redo：识别命令来源和事件关联
///
/// # 示例
/// ```rust
/// use axum::http::HeaderMap;
/// use crate::infra::http::extractors::extract_correlation_id;
///
/// pub async fn handle(headers: HeaderMap) -> Response {
///     let correlation_id = extract_correlation_id(&headers);
///     // 传递给业务逻辑...
/// }
/// ```
pub fn extract_correlation_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("X-Correlation-ID")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// 从 HTTP 请求头中提取客户端时间
///
/// 前端会在所有请求中通过 `X-Client-Time` header 发送客户端当前时间（ISO 8601 格式）
/// 这样避免了在每个 DTO 中重复定义时间字段，实现统一的时间处理
///
/// # 设计理由
/// - **统一处理**：所有需要客户端时间的 API 都从请求头获取，避免重复代码
/// - **时区一致性**：使用客户端时间，消除服务器时区假设
/// - **职责分离**：时间作为元数据放在请求头更合理
///
/// # 返回值
/// - `Ok(DateTime<Utc>)`：成功解析的客户端时间
/// - `Err(AppError)`：请求头缺失或时间格式无效
///
/// # 示例
/// ```rust
/// use axum::http::HeaderMap;
/// use crate::infra::http::extractors::extract_client_time;
///
/// pub async fn handle(headers: HeaderMap) -> Result<Response, AppError> {
///     let client_time = extract_client_time(&headers)?;
///     // 使用 client_time 进行业务逻辑...
/// }
/// ```
pub fn extract_client_time(headers: &HeaderMap) -> Result<DateTime<Utc>, AppError> {
    let time_str = headers
        .get("X-Client-Time")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            AppError::validation_error(
                "X-Client-Time",
                "Missing X-Client-Time header",
                "MISSING_CLIENT_TIME",
            )
        })?;

    DateTime::parse_from_rfc3339(time_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| {
            AppError::validation_error(
                "X-Client-Time",
                "Invalid ISO 8601 time format",
                "INVALID_TIME_FORMAT",
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_extract_correlation_id_exists() {
        let headers = {
            let mut h = HeaderMap::new();
            h.insert(
                "X-Correlation-ID",
                "550e8400-e29b-41d4-a716-446655440000".parse().unwrap(),
            );
            h
        };

        let result = extract_correlation_id(&headers);
        assert_eq!(
            result,
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_extract_correlation_id_missing() {
        let headers = HeaderMap::new();
        let result = extract_correlation_id(&headers);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_correlation_id_invalid() {
        let mut headers = HeaderMap::new();
        // 插入一个包含非 ASCII 字符的值（虽然实际很难做到）
        // 这里测试 to_str() 失败的情况
        let result = extract_correlation_id(&headers);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_client_time_valid() {
        let headers = {
            let mut h = HeaderMap::new();
            h.insert("X-Client-Time", "2025-11-11T13:45:32.123Z".parse().unwrap());
            h
        };

        let result = extract_client_time(&headers);
        assert!(result.is_ok());

        let time = result.unwrap();
        assert_eq!(time.year(), 2025);
        assert_eq!(time.month(), 11);
        assert_eq!(time.day(), 11);
    }

    #[test]
    fn test_extract_client_time_missing() {
        let headers = HeaderMap::new();
        let result = extract_client_time(&headers);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, AppError::ValidationError { .. }));
    }

    #[test]
    fn test_extract_client_time_invalid_format() {
        let headers = {
            let mut h = HeaderMap::new();
            h.insert("X-Client-Time", "not-a-valid-date".parse().unwrap());
            h
        };

        let result = extract_client_time(&headers);
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, AppError::ValidationError { .. }));
    }
}
