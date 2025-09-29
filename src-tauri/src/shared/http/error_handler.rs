/// 统一错误处理模块
///
/// 实现AppError到HTTP响应的映射，确保错误处理的一致性
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use super::responses::ErrorResponse;
use crate::shared::core::AppError;

/// 实现AppError到HTTP响应的转换
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status_code, error_response) = match self {
            // 数据库错误 -> 500 Internal Server Error
            AppError::DatabaseError(db_error) => {
                tracing::error!("Database error: {}", db_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::internal_error(format!("数据库操作失败: {}", db_error)),
                )
            }

            // 未指定内部错误 -> 500 Internal Server Error
            AppError::UnspecifiedInternalError => {
                tracing::error!("Unspecified internal error occurred");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::internal_error("内部服务器错误".to_string()),
                )
            }

            // 实体未找到 -> 404 Not Found
            AppError::NotFound {
                entity_type,
                entity_id,
            } => (
                StatusCode::NOT_FOUND,
                ErrorResponse::not_found(entity_type, entity_id),
            ),

            // 验证失败 -> 422 Unprocessable Entity
            AppError::ValidationFailed(validation_errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorResponse::validation_error(validation_errors),
            ),

            // 权限被拒绝 -> 403 Forbidden
            AppError::PermissionDenied => (
                StatusCode::FORBIDDEN,
                ErrorResponse::new("PermissionDenied".to_string(), "权限不足".to_string()),
            ),

            // 冲突 -> 409 Conflict
            AppError::Conflict { message } => {
                (StatusCode::CONFLICT, ErrorResponse::conflict(message))
            }

            // 外部依赖失败 -> 503 Service Unavailable
            AppError::ExternalDependencyFailed {
                service_name,
                error_message,
            } => {
                tracing::error!(
                    "External service '{}' failed: {}",
                    service_name,
                    error_message
                );
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    ErrorResponse::new(
                        "ExternalDependencyFailed".to_string(),
                        format!("外部服务 '{}' 不可用: {}", service_name, error_message),
                    ),
                )
            }

            // 配置错误 -> 500 Internal Server Error
            AppError::ConfigurationError { message } => {
                tracing::error!("Configuration error: {}", message);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::internal_error(format!("配置错误: {}", message)),
                )
            }

            // 序列化错误 -> 400 Bad Request
            AppError::SerializationError(serde_error) => {
                tracing::error!("Serialization error: {}", serde_error);
                (
                    StatusCode::BAD_REQUEST,
                    ErrorResponse::new(
                        "SerializationError".to_string(),
                        format!("数据序列化错误: {}", serde_error),
                    ),
                )
            }

            // IO错误 -> 500 Internal Server Error
            AppError::IoError(io_error) => {
                tracing::error!("IO error: {}", io_error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse::internal_error(format!("IO错误: {}", io_error)),
                )
            }

            // 字符串错误 -> 400 Bad Request
            AppError::StringError(message) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse::new("StringError".to_string(), message),
            ),
        };

        (status_code, Json(error_response)).into_response()
    }
}

/// 创建标准的成功响应
pub fn success_response<T: serde::Serialize>(data: T) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(super::responses::ApiResponse::success(data)),
    )
}

/// 创建201 Created响应
pub fn created_response<T: serde::Serialize>(data: T) -> impl IntoResponse {
    (
        StatusCode::CREATED,
        Json(super::responses::ApiResponse::success(data)),
    )
}

/// 创建204 No Content响应
pub fn no_content_response() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

/// 创建202 Accepted响应（用于异步操作）
pub fn accepted_response<T: serde::Serialize>(data: T) -> impl IntoResponse {
    (
        StatusCode::ACCEPTED,
        Json(super::responses::ApiResponse::success(data)),
    )
}

/// 提取请求ID的中间件（可选）
pub async fn extract_request_id(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get("X-Request-ID")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
        .or_else(|| {
            // 如果没有提供请求ID，生成一个
            Some(uuid::Uuid::new_v4().to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::core::ValidationError;

    #[test]
    fn test_api_response_creation() {
        let data = "test_data";
        let response = super::responses::ApiResponse::success(data);

        assert_eq!(response.data, "test_data");
        assert!(response.request_id.is_none());
    }

    #[test]
    fn test_error_response_validation() {
        let errors = vec![
            ValidationError::new("title", "Title cannot be empty", "TITLE_EMPTY"),
            ValidationError::new("email", "Invalid email format", "EMAIL_INVALID"),
        ];

        let error_response = ErrorResponse::validation_error(errors);

        assert_eq!(error_response.error_type, "ValidationError");
        assert_eq!(error_response.message, "输入数据验证失败");
        assert!(error_response.details.is_some());
        assert_eq!(error_response.code, Some("VALIDATION_FAILED".to_string()));
    }

    #[test]
    fn test_error_response_not_found() {
        let error_response = ErrorResponse::not_found("Task".to_string(), "123".to_string());

        assert_eq!(error_response.error_type, "NotFound");
        assert!(error_response.message.contains("Task"));
        assert!(error_response.message.contains("123"));
        assert_eq!(error_response.code, Some("NOT_FOUND".to_string()));
    }

    #[test]
    fn test_error_response_conflict() {
        let error_response = ErrorResponse::conflict("Resource already exists".to_string());

        assert_eq!(error_response.error_type, "Conflict");
        assert_eq!(error_response.message, "Resource already exists");
        assert_eq!(error_response.code, Some("CONFLICT".to_string()));
    }

    #[test]
    fn test_request_id_extraction() {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("X-Request-ID", "test-request-123".parse().unwrap());

        let rt = tokio::runtime::Runtime::new().unwrap();
        let request_id = rt.block_on(extract_request_id(&headers));

        assert_eq!(request_id, Some("test-request-123".to_string()));
    }

    #[test]
    fn test_request_id_generation() {
        let headers = axum::http::HeaderMap::new();

        let rt = tokio::runtime::Runtime::new().unwrap();
        let request_id = rt.block_on(extract_request_id(&headers));

        assert!(request_id.is_some());
        assert!(uuid::Uuid::parse_str(&request_id.unwrap()).is_ok());
    }
}
