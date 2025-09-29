/// HTTP请求提取器
///
/// 定义各种自定义的请求提取器，用于从HTTP请求中提取数据
use axum::{
    async_trait,
    extract::{FromRequest, Query, Request},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::shared::{
    core::{AppError, ValidationError},
    database::PaginationParams,
    http::{middleware::RequestId, responses::ErrorResponse},
};

/// 验证的JSON提取器
///
/// 自动验证JSON请求体的结构和内容
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: for<'de> Deserialize<'de> + Validate,
    S: Send + Sync,
{
    type Rejection = ValidationRejection;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        // 首先提取JSON
        let Json(payload): Json<T> = Json::from_request(req, state)
            .await
            .map_err(|err| ValidationRejection::JsonError(err.to_string()))?;

        // 验证数据
        payload
            .validate()
            .map_err(ValidationRejection::ValidationError)?;

        Ok(ValidatedJson(payload))
    }
}

/// 验证trait
pub trait Validate {
    fn validate(&self) -> Result<(), Vec<ValidationError>>;
}

/// 验证拒绝错误
pub enum ValidationRejection {
    JsonError(String),
    ValidationError(Vec<ValidationError>),
}

impl IntoResponse for ValidationRejection {
    fn into_response(self) -> Response {
        match self {
            ValidationRejection::JsonError(err) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "JsonParseError".to_string(),
                    format!("JSON解析失败: {}", err),
                )),
            )
                .into_response(),
            ValidationRejection::ValidationError(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(ErrorResponse::validation_error(errors)),
            )
                .into_response(),
        }
    }
}

/// 分页查询提取器
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

impl PaginationQuery {
    pub fn to_params(&self) -> PaginationParams {
        PaginationParams::new(self.page, self.page_size)
    }
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

/// 排序查询提取器
#[derive(Debug, Clone, Deserialize)]
pub struct SortQuery {
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default = "default_sort_order")]
    pub sort_order: SortOrder,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

fn default_sort_order() -> SortOrder {
    SortOrder::Asc
}

impl Default for SortQuery {
    fn default() -> Self {
        Self {
            sort_by: None,
            sort_order: SortOrder::Asc,
        }
    }
}

/// 搜索查询提取器
#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            q: None,
            limit: None,
        }
    }
}

/// 日期范围查询提取器
#[derive(Debug, Clone, Deserialize)]
pub struct DateRangeQuery {
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    pub date: Option<chrono::DateTime<chrono::Utc>>,
}

impl DateRangeQuery {
    /// 验证日期范围的有效性
    pub fn validate(&self) -> Result<(), ValidationError> {
        if let (Some(start), Some(end)) = (&self.start_date, &self.end_date) {
            if start >= end {
                return Err(ValidationError::new(
                    "date_range",
                    "开始日期必须早于结束日期",
                    "INVALID_DATE_RANGE",
                ));
            }
        }
        Ok(())
    }

    /// 检查是否为单日查询
    pub fn is_single_date(&self) -> bool {
        self.date.is_some()
    }

    /// 检查是否为日期范围查询
    pub fn is_date_range(&self) -> bool {
        self.start_date.is_some() && self.end_date.is_some()
    }
}

/// 过滤查询提取器
#[derive(Debug, Clone, Deserialize)]
pub struct FilterQuery {
    #[serde(flatten)]
    pub filters: HashMap<String, String>,
}

impl FilterQuery {
    /// 获取特定的过滤值
    pub fn get_filter(&self, key: &str) -> Option<&String> {
        self.filters.get(key)
    }

    /// 检查是否有过滤条件
    pub fn has_filters(&self) -> bool {
        !self.filters.is_empty()
    }

    /// 获取所有过滤条件
    pub fn get_all_filters(&self) -> &HashMap<String, String> {
        &self.filters
    }
}

/// 组合查询提取器
///
/// 将分页、排序、搜索和过滤查询组合在一起
#[derive(Debug, Clone)]
pub struct CombinedQuery {
    pub pagination: PaginationQuery,
    pub sort: SortQuery,
    pub search: SearchQuery,
    pub date_range: DateRangeQuery,
    pub filters: FilterQuery,
}

#[async_trait]
impl<S> FromRequest<S> for CombinedQuery
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Query(mut query_params): Query<HashMap<String, String>> =
            Query::from_request(req, state).await.map_err(|_| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse::new(
                        "QueryParseError".to_string(),
                        "查询参数解析失败".to_string(),
                    )),
                )
            })?;

        // 提取分页参数
        let page = query_params
            .remove("page")
            .and_then(|p| p.parse().ok())
            .unwrap_or(1);
        let page_size = query_params
            .remove("page_size")
            .and_then(|p| p.parse().ok())
            .unwrap_or(20);

        // 提取排序参数
        let sort_by = query_params.remove("sort_by");
        let sort_order = query_params
            .remove("sort_order")
            .and_then(|s| serde_json::from_str(&format!("\"{}\"", s)).ok())
            .unwrap_or(SortOrder::Asc);

        // 提取搜索参数
        let q = query_params.remove("q");
        let limit = query_params.remove("limit").and_then(|l| l.parse().ok());

        // 提取日期参数
        let start_date = query_params
            .remove("start_date")
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
            .map(|d| d.with_timezone(&chrono::Utc));
        let end_date = query_params
            .remove("end_date")
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
            .map(|d| d.with_timezone(&chrono::Utc));
        let date = query_params
            .remove("date")
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
            .map(|d| d.with_timezone(&chrono::Utc));

        let date_range = DateRangeQuery {
            start_date,
            end_date,
            date,
        };

        // 验证日期范围
        if let Err(validation_error) = date_range.validate() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::validation_error(vec![validation_error])),
            ));
        }

        // 剩余的参数作为过滤条件
        let filters = FilterQuery {
            filters: query_params,
        };

        Ok(CombinedQuery {
            pagination: PaginationQuery { page, page_size },
            sort: SortQuery {
                sort_by,
                sort_order,
            },
            search: SearchQuery { q, limit },
            date_range,
            filters,
        })
    }
}

/// 请求ID提取器
pub struct ExtractedRequestId(pub String);

#[async_trait]
impl<S> FromRequest<S> for ExtractedRequestId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        let request_id = req
            .extensions()
            .get::<RequestId>()
            .map(|id| id.0.clone())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        Ok(ExtractedRequestId(request_id))
    }
}

/// UUID路径参数提取器
///
/// 自动验证路径中的UUID参数
pub struct ValidatedUuid(pub uuid::Uuid);

#[async_trait]
impl<S> FromRequest<S> for ValidatedUuid
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(id_str): axum::extract::Path<String> =
            axum::extract::Path::from_request(req, state)
                .await
                .map_err(|_| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse::new(
                            "PathParseError".to_string(),
                            "路径参数解析失败".to_string(),
                        )),
                    )
                })?;

        let uuid = uuid::Uuid::parse_str(&id_str).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "InvalidUuid".to_string(),
                    "无效的UUID格式".to_string(),
                )),
            )
        })?;

        Ok(ValidatedUuid(uuid))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_query_defaults() {
        let query = PaginationQuery::default();
        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 20);
    }

    #[test]
    fn test_sort_query_defaults() {
        let query = SortQuery::default();
        assert!(query.sort_by.is_none());
        matches!(query.sort_order, SortOrder::Asc);
    }

    #[test]
    fn test_date_range_validation() {
        let valid_range = DateRangeQuery {
            start_date: Some(chrono::Utc::now()),
            end_date: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            date: None,
        };
        assert!(valid_range.validate().is_ok());

        let invalid_range = DateRangeQuery {
            start_date: Some(chrono::Utc::now()),
            end_date: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            date: None,
        };
        assert!(invalid_range.validate().is_err());
    }

    #[test]
    fn test_filter_query() {
        let mut filters = HashMap::new();
        filters.insert("status".to_string(), "active".to_string());
        filters.insert("category".to_string(), "work".to_string());

        let filter_query = FilterQuery { filters };

        assert!(filter_query.has_filters());
        assert_eq!(
            filter_query.get_filter("status"),
            Some(&"active".to_string())
        );
        assert_eq!(filter_query.get_filter("nonexistent"), None);
    }

    #[test]
    fn test_uuid_validation() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(uuid::Uuid::parse_str(valid_uuid).is_ok());

        let invalid_uuid = "invalid-uuid";
        assert!(uuid::Uuid::parse_str(invalid_uuid).is_err());
    }
}

