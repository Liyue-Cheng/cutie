/// 请求ID中间件
/// 
/// 为每个HTTP请求生成或提取唯一的请求ID，用于日志追踪和调试

use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// 请求ID头部名称
pub const REQUEST_ID_HEADER: &str = "X-Request-ID";

/// 请求ID中间件
/// 
/// **预期行为简介:** 为每个请求确保有唯一的请求ID，用于日志追踪
/// 
/// ## 行为逻辑
/// 1. 检查请求头中是否已有X-Request-ID
/// 2. 如果有，验证其为有效UUID格式，无效则生成新的
/// 3. 如果没有，生成新的UUID作为请求ID
/// 4. 将请求ID添加到响应头中
/// 5. 在日志中包含请求ID用于追踪
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // 提取或生成请求ID
    let request_id = extract_or_generate_request_id(request.headers());
    
    // 将请求ID添加到请求头中，供后续处理器使用
    request.headers_mut().insert(
        REQUEST_ID_HEADER,
        request_id.parse().unwrap_or_else(|_| {
            // 如果解析失败，使用默认值
            "invalid-request-id".parse().unwrap()
        })
    );
    
    // 记录请求开始
    log::debug!("Request started [{}] {} {}", 
        request_id,
        request.method(),
        request.uri()
    );
    
    // 调用下一个中间件或处理器
    let mut response = next.run(request).await;
    
    // 将请求ID添加到响应头中
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        request_id.parse().unwrap_or_else(|_| {
            "invalid-request-id".parse().unwrap()
        })
    );
    
    // 记录请求完成
    log::debug!("Request completed [{}] Status: {}", 
        request_id,
        response.status()
    );
    
    response
}

/// 提取或生成请求ID
fn extract_or_generate_request_id(headers: &HeaderMap) -> String {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| {
            // 验证是否为有效的UUID
            if Uuid::parse_str(s).is_ok() {
                Some(s.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            // 生成新的请求ID
            Uuid::new_v4().to_string()
        })
}

/// 从请求头中获取请求ID
pub fn get_request_id_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_valid_request_id() {
        let mut headers = HeaderMap::new();
        let test_uuid = Uuid::new_v4().to_string();
        headers.insert(REQUEST_ID_HEADER, HeaderValue::from_str(&test_uuid).unwrap());
        
        let request_id = extract_or_generate_request_id(&headers);
        assert_eq!(request_id, test_uuid);
    }

    #[test]
    fn test_extract_invalid_request_id() {
        let mut headers = HeaderMap::new();
        headers.insert(REQUEST_ID_HEADER, HeaderValue::from_static("invalid-uuid"));
        
        let request_id = extract_or_generate_request_id(&headers);
        assert_ne!(request_id, "invalid-uuid");
        assert!(Uuid::parse_str(&request_id).is_ok());
    }

    #[test]
    fn test_generate_request_id() {
        let headers = HeaderMap::new();
        
        let request_id = extract_or_generate_request_id(&headers);
        assert!(Uuid::parse_str(&request_id).is_ok());
    }

    #[test]
    fn test_get_request_id_from_headers() {
        let mut headers = HeaderMap::new();
        let test_uuid = Uuid::new_v4().to_string();
        headers.insert(REQUEST_ID_HEADER, HeaderValue::from_str(&test_uuid).unwrap());
        
        let request_id = get_request_id_from_headers(&headers);
        assert_eq!(request_id, Some(test_uuid));
    }

    #[test]
    fn test_get_request_id_from_empty_headers() {
        let headers = HeaderMap::new();
        
        let request_id = get_request_id_from_headers(&headers);
        assert_eq!(request_id, None);
    }
}
