/// 请求追踪中间件
///
/// 为每个 HTTP 请求生成唯一的 request_id，并在日志中统一输出标准字段
use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use std::time::Instant;
use uuid::Uuid;

/// 请求 ID 的 header 键
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// 请求追踪中间件
///
/// 功能：
/// 1. 为每个请求生成或透传 x-request-id
/// 2. 记录请求开始（method, path, remote_ip）
/// 3. 记录请求结束（status, latency_ms）
/// 4. 将 request_id 添加到响应头
pub async fn request_tracing_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();

    // 1. 获取或生成 request_id
    let request_id = headers
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 2. 提取请求信息
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();
    let query = uri.query().map(|q| q.to_string());

    // 3. 记录请求开始
    tracing::info!(
        target: "MIDDLEWARE:request_tracing",
        req_id = %request_id,
        method = %method,
        path = %path,
        query = ?query,
        "Request started"
    );

    // 4. 处理请求
    let response = next.run(request).await;

    // 5. 计算耗时
    let latency_ms = start.elapsed().as_millis();

    // 6. 提取响应状态
    let status = response.status();

    // 7. 记录请求结束
    if status.is_server_error() {
        tracing::error!(
            target: "MIDDLEWARE:request_tracing",
            req_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            latency_ms = %latency_ms,
            "Request completed with server error"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            target: "MIDDLEWARE:request_tracing",
            req_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            latency_ms = %latency_ms,
            "Request completed with client error"
        );
    } else {
        tracing::info!(
            target: "MIDDLEWARE:request_tracing",
            req_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            latency_ms = %latency_ms,
            "Request completed"
        );
    }

    // 8. 在响应头中添加 request_id
    let mut response = response;
    response
        .headers_mut()
        .insert(REQUEST_ID_HEADER, request_id.parse().unwrap());

    response
}

/// 慢请求检测中间件
///
/// 记录超过阈值的慢请求
pub async fn slow_request_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
    threshold_ms: u64,
) -> Response {
    let start = Instant::now();

    let request_id = headers
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let method = request.method().clone();
    let path = request.uri().path().to_string();

    let response = next.run(request).await;
    let latency_ms = start.elapsed().as_millis() as u64;

    if latency_ms > threshold_ms {
        tracing::warn!(
            target: "MIDDLEWARE:slow_request",
            req_id = %request_id,
            method = %method,
            path = %path,
            latency_ms = %latency_ms,
            threshold_ms = %threshold_ms,
            "Slow request detected"
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, middleware, routing::get, Router};
    use tower::ServiceExt;

    async fn dummy_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_request_tracing_middleware() {
        let app = Router::new()
            .route("/test", get(dummy_handler))
            .layer(middleware::from_fn(request_tracing_middleware));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();

        // 验证响应头包含 request_id
        assert!(response.headers().contains_key(REQUEST_ID_HEADER));
    }

    #[tokio::test]
    async fn test_request_id_passthrough() {
        let app = Router::new()
            .route("/test", get(dummy_handler))
            .layer(middleware::from_fn(request_tracing_middleware));

        let custom_id = "test-request-id-123";
        let request = Request::builder()
            .uri("/test")
            .header(REQUEST_ID_HEADER, custom_id)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        // 验证透传了自定义 request_id
        let response_id = response
            .headers()
            .get(REQUEST_ID_HEADER)
            .unwrap()
            .to_str()
            .unwrap();
        assert_eq!(response_id, custom_id);
    }
}
