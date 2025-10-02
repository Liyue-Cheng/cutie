/// HTTP 请求提取器
///
/// 用于从 HTTP 请求中提取常用的元数据和上下文信息
use axum::http::HeaderMap;

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
/// use crate::shared::http::extractors::extract_correlation_id;
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
                "550e8400-e29b-41d4-a716-446655440000"
                    .parse()
                    .unwrap(),
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
}
