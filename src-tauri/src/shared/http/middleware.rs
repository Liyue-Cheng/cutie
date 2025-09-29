/// HTTP中间件模块
///
/// 包含各种HTTP中间件实现
use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use uuid::Uuid;

/// 请求ID中间件
///
/// 为每个请求生成或提取唯一的请求ID，用于日志追踪
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // 尝试从请求头中获取请求ID
    let request_id = request
        .headers()
        .get("X-Request-ID")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // 将请求ID添加到请求扩展中，以便后续处理器使用
    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    // 调用下一个中间件或处理器
    let mut response = next.run(request).await;

    // 在响应头中添加请求ID
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("X-Request-ID", header_value);
    }

    response
}

/// 请求ID包装器
#[derive(Debug, Clone)]
pub struct RequestId(pub String);

/// 日志中间件
///
/// 记录请求和响应的基本信息
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let start_time = std::time::Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let version = request.version();

    // 获取请求ID（如果存在）
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map(|id| id.0.clone())
        .unwrap_or_else(|| "unknown".to_string());

    log::info!(
        "Started {} {} HTTP/{:?} [{}]",
        method,
        uri,
        version,
        request_id
    );

    // 调用下一个中间件或处理器
    let response = next.run(request).await;

    let duration = start_time.elapsed();
    let status = response.status();

    log::info!(
        "Completed {} {} {} in {:.2}ms [{}]",
        method,
        uri,
        status,
        duration.as_millis(),
        request_id
    );

    response
}

/// CORS中间件
///
/// 处理跨域请求
pub async fn cors_middleware(request: Request, next: Next) -> Response {
    let origin = request
        .headers()
        .get("Origin")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("*")
        .to_string();

    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // 设置CORS头
    if let Ok(origin_value) = HeaderValue::from_str(&origin) {
        headers.insert("Access-Control-Allow-Origin", origin_value);
    }

    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );

    headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("Content-Type, Authorization, X-Request-ID"),
    );

    headers.insert(
        "Access-Control-Expose-Headers",
        HeaderValue::from_static("X-Request-ID"),
    );

    headers.insert("Access-Control-Max-Age", HeaderValue::from_static("3600"));

    response
}

/// 安全头中间件
///
/// 添加安全相关的HTTP头
pub async fn security_headers_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // 安全头
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );

    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));

    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // CSP头（根据需要调整）
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"),
    );

    response
}

/// 速率限制中间件（简单实现）
///
/// 基于IP地址的简单速率限制
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration,
        }
    }

    pub fn is_allowed(&self, client_id: &str) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();

        // 获取或创建客户端的请求记录
        let client_requests = requests
            .entry(client_id.to_string())
            .or_insert_with(Vec::new);

        // 清理过期的请求记录
        client_requests.retain(|&timestamp| now.duration_since(timestamp) < self.window_duration);

        // 检查是否超过限制
        if client_requests.len() >= self.max_requests {
            false
        } else {
            client_requests.push(now);
            true
        }
    }
}

/// 速率限制中间件函数
pub fn rate_limit_middleware(
    rate_limiter: RateLimiter,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone {
    move |request: Request, next: Next| {
        let rate_limiter = rate_limiter.clone();
        Box::pin(async move {
            // 获取客户端IP地址
            let client_ip = request
                .headers()
                .get("X-Forwarded-For")
                .and_then(|value| value.to_str().ok())
                .or_else(|| {
                    request
                        .headers()
                        .get("X-Real-IP")
                        .and_then(|value| value.to_str().ok())
                })
                .unwrap_or("unknown")
                .to_string();

            // 检查速率限制
            if !rate_limiter.is_allowed(&client_ip) {
                // 返回429 Too Many Requests
                return Response::builder()
                    .status(429)
                    .header("Content-Type", "application/json")
                    .body(
                        serde_json::json!({
                            "error_type": "RateLimitExceeded",
                            "message": "请求过于频繁，请稍后再试",
                            "code": "RATE_LIMIT_EXCEEDED",
                            "timestamp": Utc::now().to_rfc3339()
                        })
                        .to_string()
                        .into(),
                    )
                    .unwrap();
            }

            next.run(request).await
        })
    }
}

/// 超时中间件
///
/// 为请求设置超时时间
pub fn timeout_middleware(
    timeout: Duration,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            match tokio::time::timeout(timeout, next.run(request)).await {
                Ok(response) => response,
                Err(_) => {
                    // 请求超时
                    Response::builder()
                        .status(408)
                        .header("Content-Type", "application/json")
                        .body(
                            serde_json::json!({
                                "error_type": "RequestTimeout",
                                "message": "请求超时",
                                "code": "REQUEST_TIMEOUT",
                                "timestamp": Utc::now().to_rfc3339()
                            })
                            .to_string()
                            .into(),
                        )
                        .unwrap()
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request},
        response::Response,
    };
    use std::time::Duration;

    #[test]
    fn test_rate_limiter() {
        let rate_limiter = RateLimiter::new(3, Duration::from_secs(60));

        // 前3个请求应该被允许
        assert!(rate_limiter.is_allowed("client1"));
        assert!(rate_limiter.is_allowed("client1"));
        assert!(rate_limiter.is_allowed("client1"));

        // 第4个请求应该被拒绝
        assert!(!rate_limiter.is_allowed("client1"));

        // 不同客户端应该有独立的限制
        assert!(rate_limiter.is_allowed("client2"));
    }

    #[test]
    fn test_request_id_extraction() {
        // 这里可以添加更多的单元测试
        let request_id = RequestId("test-id".to_string());
        assert_eq!(request_id.0, "test-id");
    }
}
