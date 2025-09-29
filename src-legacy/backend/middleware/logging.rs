/// 请求日志中间件
///
/// 记录HTTP请求和响应的详细信息
use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

use super::request_id::{get_request_id_from_headers, REQUEST_ID_HEADER};

/// 请求日志中间件
///
/// **预期行为简介:** 记录HTTP请求的详细信息，包括请求方法、路径、耗时、状态码等
///
/// ## 日志内容
/// - 请求开始：方法、路径、请求ID
/// - 请求完成：状态码、耗时、响应大小
/// - 错误情况：详细的错误信息
pub async fn request_logging_middleware(request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id =
        get_request_id_from_headers(request.headers()).unwrap_or_else(|| "unknown".to_string());

    // 记录请求开始
    log::info!("Request started [{}] {} {}", request_id, method, uri);

    // 记录请求体大小（如果有）
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<u64>() {
                log::debug!("Request body size [{}]: {} bytes", request_id, length);
            }
        }
    }

    // 调用下一个处理器
    let response = next.run(request).await;

    // 计算耗时
    let duration = start_time.elapsed();
    let status = response.status();

    // 记录响应信息
    if status.is_success() {
        log::info!(
            "Request completed [{}] {} {} - {} in {:.2}ms",
            request_id,
            method,
            uri,
            status,
            duration.as_millis()
        );
    } else if status.is_client_error() {
        log::warn!(
            "Request failed [{}] {} {} - {} in {:.2}ms",
            request_id,
            method,
            uri,
            status,
            duration.as_millis()
        );
    } else if status.is_server_error() {
        log::error!(
            "Request error [{}] {} {} - {} in {:.2}ms",
            request_id,
            method,
            uri,
            status,
            duration.as_millis()
        );
    } else {
        log::debug!(
            "Request completed [{}] {} {} - {} in {:.2}ms",
            request_id,
            method,
            uri,
            status,
            duration.as_millis()
        );
    }

    // 记录慢请求
    if duration.as_millis() > 1000 {
        log::warn!(
            "Slow request detected [{}] {} {} - {:.2}ms",
            request_id,
            method,
            uri,
            duration.as_millis()
        );
    }

    response
}

/// 性能监控中间件
///
/// **预期行为简介:** 收集请求的性能指标
pub async fn performance_monitoring_middleware(request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let request_id =
        get_request_id_from_headers(request.headers()).unwrap_or_else(|| "unknown".to_string());

    // 调用下一个处理器
    let response = next.run(request).await;

    // 收集性能指标
    let duration = start_time.elapsed();
    let status = response.status();

    // 记录性能指标（在实际实现中，这些数据可以发送到监控系统）
    log::debug!(
        "Performance [{}] {} {} - Status: {}, Duration: {:.2}ms",
        request_id,
        method,
        path,
        status,
        duration.as_millis()
    );

    // 在这里可以添加指标收集逻辑：
    // - 响应时间分布
    // - 错误率统计
    // - 吞吐量统计
    // - 热点API识别

    response
}

/// 请求大小限制中间件
///
/// **预期行为简介:** 检查请求体大小是否超过限制
pub async fn request_size_middleware(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // 检查Content-Length头部
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<u64>() {
                // 默认限制10MB（可以从配置中读取）
                const MAX_REQUEST_SIZE: u64 = 10 * 1024 * 1024;

                if length > MAX_REQUEST_SIZE {
                    log::warn!(
                        "Request body too large: {} bytes (limit: {} bytes)",
                        length,
                        MAX_REQUEST_SIZE
                    );
                    return Err(axum::http::StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_request_id_extraction() {
        let mut headers = HeaderMap::new();
        let test_uuid = uuid::Uuid::new_v4().to_string();
        headers.insert(
            REQUEST_ID_HEADER,
            HeaderValue::from_str(&test_uuid).unwrap(),
        );

        let request_id = get_request_id_from_headers(&headers);
        assert_eq!(request_id, Some(test_uuid));
    }

    #[test]
    fn test_request_id_missing() {
        let headers = HeaderMap::new();

        let request_id = get_request_id_from_headers(&headers);
        assert_eq!(request_id, None);
    }
}
