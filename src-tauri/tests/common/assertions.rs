/// 自定义测试断言
///
/// 提供更语义化的断言方法
use reqwest::{Response, StatusCode};

/// HTTP 响应断言扩展
pub trait ResponseAssertions {
    fn assert_success(&self) -> &Self;
    fn assert_created(&self) -> &Self;
    fn assert_not_found(&self) -> &Self;
    fn assert_conflict(&self) -> &Self;
    fn assert_unprocessable(&self) -> &Self;
    fn assert_status(&self, expected: StatusCode) -> &Self;
}

impl ResponseAssertions for Response {
    fn assert_success(&self) -> &Self {
        assert!(
            self.status().is_success(),
            "Expected success status (2xx), got {}",
            self.status()
        );
        self
    }

    fn assert_created(&self) -> &Self {
        assert_eq!(
            self.status(),
            StatusCode::CREATED,
            "Expected 201 Created, got {}",
            self.status()
        );
        self
    }

    fn assert_not_found(&self) -> &Self {
        assert_eq!(
            self.status(),
            StatusCode::NOT_FOUND,
            "Expected 404 Not Found, got {}",
            self.status()
        );
        self
    }

    fn assert_conflict(&self) -> &Self {
        assert_eq!(
            self.status(),
            StatusCode::CONFLICT,
            "Expected 409 Conflict, got {}",
            self.status()
        );
        self
    }

    fn assert_unprocessable(&self) -> &Self {
        assert_eq!(
            self.status(),
            StatusCode::UNPROCESSABLE_ENTITY,
            "Expected 422 Unprocessable Entity, got {}",
            self.status()
        );
        self
    }

    fn assert_status(&self, expected: StatusCode) -> &Self {
        assert_eq!(
            self.status(),
            expected,
            "Expected {}, got {}",
            expected,
            self.status()
        );
        self
    }
}

/// JSON 比较断言
#[allow(dead_code)]
pub fn assert_json_eq(actual: &serde_json::Value, expected: &serde_json::Value, path: &str) {
    if actual != expected {
        panic!(
            "JSON mismatch at {}\nActual:\n{}\nExpected:\n{}",
            path,
            serde_json::to_string_pretty(actual).unwrap(),
            serde_json::to_string_pretty(expected).unwrap()
        );
    }
}

/// 字符串包含断言
#[allow(dead_code)]
pub fn assert_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "Expected '{}' to contain '{}'",
        haystack,
        needle
    );
}

