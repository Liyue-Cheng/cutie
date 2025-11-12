/// HTTP 测试客户端工具
///
/// 提供简化的 HTTP 请求辅助函数，用于端点和集成测试
use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde::{de::DeserializeOwned, Serialize};
use tower::ServiceExt; // for `oneshot`

/// 测试 HTTP 客户端
pub struct TestClient {
    router: Router,
}

impl TestClient {
    /// 创建新的测试客户端
    pub fn new(router: Router) -> Self {
        Self { router }
    }

    /// 发送 GET 请求
    pub async fn get(&self, uri: &str) -> TestResponse {
        let request = Request::builder()
            .uri(uri)
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = self.router.clone().oneshot(request).await.unwrap();

        TestResponse { response }
    }

    /// 发送 POST 请求（带 JSON body）
    pub async fn post<T: Serialize>(&self, uri: &str, body: &T) -> TestResponse {
        let json = serde_json::to_string(body).unwrap();

        let request = Request::builder()
            .uri(uri)
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap();

        let response = self.router.clone().oneshot(request).await.unwrap();

        TestResponse { response }
    }

    /// 发送 PATCH 请求（带 JSON body）
    pub async fn patch<T: Serialize>(&self, uri: &str, body: &T) -> TestResponse {
        let json = serde_json::to_string(body).unwrap();

        let request = Request::builder()
            .uri(uri)
            .method("PATCH")
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap();

        let response = self.router.clone().oneshot(request).await.unwrap();

        TestResponse { response }
    }

    /// 发送 DELETE 请求
    pub async fn delete(&self, uri: &str) -> TestResponse {
        let request = Request::builder()
            .uri(uri)
            .method("DELETE")
            .body(Body::empty())
            .unwrap();

        let response = self.router.clone().oneshot(request).await.unwrap();

        TestResponse { response }
    }
}

/// 测试响应包装器
pub struct TestResponse {
    response: axum::response::Response,
}

impl TestResponse {
    /// 获取响应状态码
    pub fn status(&self) -> StatusCode {
        self.response.status()
    }

    /// 解析 JSON 响应体
    pub async fn json<T: DeserializeOwned>(self) -> T {
        let body = axum::body::to_bytes(self.response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    /// 断言状态码
    pub fn assert_status(self, expected: StatusCode) -> Self {
        assert_eq!(
            self.status(),
            expected,
            "Expected status {}, got {}",
            expected,
            self.status()
        );
        self
    }

    /// 断言成功（2xx）
    pub fn assert_success(self) -> Self {
        assert!(
            self.status().is_success(),
            "Expected success status, got {}",
            self.status()
        );
        self
    }
}
