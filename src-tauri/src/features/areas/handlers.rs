/// 领域HTTP处理器
///
/// 处理领域相关的HTTP请求

use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use uuid::Uuid;

use crate::shared::{
    database::AreaRepository,
    http::{
        error_handler::{created_response, no_content_response, success_response},
        extractors::{SearchQuery, ValidatedJson},
    },
};

use super::{
    payloads::{
        AreaQuery, BulkAreaOperation, BulkAreaOperationType, CreateAreaPayload, MoveAreaPayload,
        UpdateAreaPayload,
    },
    service::AreaService,
};

/// 领域处理器
pub struct AreaHandlers<R: AreaRepository> {
    service: AreaService<R>,
}

impl<R: AreaRepository> AreaHandlers<R> {
    pub fn new(service: AreaService<R>) -> Self {
        Self { service }
    }

    /// 创建领域
    pub async fn create_area(
        &self,
        ValidatedJson(payload): ValidatedJson<CreateAreaPayload>,
    ) -> Response {
        match self.service.create_area(payload).await {
            Ok(area) => created_response(area).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取领域详情
    pub async fn get_area(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.get_area(id).await {
            Ok(area) => success_response(area).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 更新领域
    pub async fn update_area(
        &self,
        Path(id): Path<Uuid>,
        ValidatedJson(payload): ValidatedJson<UpdateAreaPayload>,
    ) -> Response {
        match self.service.update_area(id, payload).await {
            Ok(area) => success_response(area).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 删除领域
    pub async fn delete_area(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.delete_area(id).await {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取领域列表
    pub async fn get_areas(&self, Query(query): Query<AreaQuery>) -> Response {
        match self
            .service
            .get_areas(
                query.parent_id,
                query.roots_only,
                query.include_descendants,
                query.q,
            )
            .await
        {
            Ok(areas) => success_response(areas).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取领域路径
    pub async fn get_area_path(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.get_area_path(id).await {
            Ok(path_response) => success_response(path_response).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 移动领域
    pub async fn move_area(
        &self,
        Path(id): Path<Uuid>,
        ValidatedJson(payload): ValidatedJson<MoveAreaPayload>,
    ) -> Response {
        match self.service.move_area(id, payload).await {
            Ok(area) => success_response(area).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 检查领域是否可删除
    pub async fn check_area_can_delete(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.check_area_can_delete(id).await {
            Ok(response) => success_response(response).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取领域统计
    pub async fn get_area_stats(&self) -> Response {
        match self.service.get_area_stats().await {
            Ok(stats) => success_response(stats).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 批量操作领域
    pub async fn bulk_operation(
        &self,
        ValidatedJson(payload): ValidatedJson<BulkAreaOperation>,
    ) -> Response {
        let result = match payload.operation {
            BulkAreaOperationType::Delete => {
                self.service.bulk_delete_areas(payload.area_ids).await
            }
            BulkAreaOperationType::MoveToParent => {
                // 从参数中提取新父领域ID
                if let Some(params) = &payload.parameters {
                    if let Some(new_parent_value) = params.get("new_parent_id") {
                        let new_parent_id = if new_parent_value.is_null() {
                            None
                        } else {
                            serde_json::from_value::<Uuid>(new_parent_value.clone()).ok()
                        };

                        self.service
                            .bulk_move_to_parent(payload.area_ids, new_parent_id)
                            .await
                    } else {
                        Err(crate::shared::core::AppError::validation_error(
                            "new_parent_id",
                            "移动操作必须提供新的父领域ID",
                            "NEW_PARENT_ID_REQUIRED",
                        ))
                    }
                } else {
                    Err(crate::shared::core::AppError::validation_error(
                        "parameters",
                        "移动操作必须提供参数",
                        "PARAMETERS_REQUIRED",
                    ))
                }
            }
            BulkAreaOperationType::UpdateColor => {
                // 从参数中提取颜色
                if let Some(params) = &payload.parameters {
                    if let Some(color_value) = params.get("color") {
                        if let Ok(color) = serde_json::from_value::<String>(color_value.clone()) {
                            self.service.bulk_update_color(payload.area_ids, color).await
                        } else {
                            Err(crate::shared::core::AppError::validation_error(
                                "color",
                                "颜色必须是字符串",
                                "COLOR_NOT_STRING",
                            ))
                        }
                    } else {
                        Err(crate::shared::core::AppError::validation_error(
                            "color",
                            "更新颜色操作必须提供颜色值",
                            "COLOR_REQUIRED",
                        ))
                    }
                } else {
                    Err(crate::shared::core::AppError::validation_error(
                        "parameters",
                        "更新颜色操作必须提供参数",
                        "PARAMETERS_REQUIRED",
                    ))
                }
            }
        };

        match result {
            Ok(affected_count) => success_response(serde_json::json!({
                "affected_count": affected_count,
                "operation": payload.operation
            }))
            .into_response(),
            Err(err) => err.into_response(),
        }
    }
}

/// 创建领域路由
pub fn create_area_routes<R, S>(
    service: AreaService<R>,
) -> Router<S>
where
    R: AreaRepository + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    let handlers = AreaHandlers::new(service);

    // 使用Arc包装handlers以便在多个路由中共享
    use std::sync::Arc;
    let handlers = Arc::new(handlers);

    Router::new()
        .route(
            "/",
            post({
                let handlers = handlers.clone();
                move |payload| async move { handlers.create_area(payload).await }
            }),
        )
        .route(
            "/",
            get({
                let handlers = handlers.clone();
                move |query| async move { handlers.get_areas(query).await }
            }),
        )
        .route(
            "/stats",
            get({
                let handlers = handlers.clone();
                move || async move { handlers.get_area_stats().await }
            }),
        )
        .route(
            "/bulk",
            post({
                let handlers = handlers.clone();
                move |payload| async move { handlers.bulk_operation(payload).await }
            }),
        )
        .route(
            "/:id",
            get({
                let handlers = handlers.clone();
                move |path| async move { handlers.get_area(path).await }
            }),
        )
        .route(
            "/:id",
            put({
                let handlers = handlers.clone();
                move |path, payload| async move { handlers.update_area(path, payload).await }
            }),
        )
        .route(
            "/:id",
            delete({
                let handlers = handlers.clone();
                move |path| async move { handlers.delete_area(path).await }
            }),
        )
        .route(
            "/:id/path",
            get({
                let handlers = handlers.clone();
                move |path| async move { handlers.get_area_path(path).await }
            }),
        )
        .route(
            "/:id/move",
            post({
                let handlers = handlers.clone();
                move |path, payload| async move { handlers.move_area(path, payload).await }
            }),
        )
        .route(
            "/:id/can-delete",
            get({
                let handlers = handlers.clone();
                move |path| async move { handlers.check_area_can_delete(path).await }
            }),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    use crate::{
        features::areas::repository::SqlxAreaRepository,
        shared::database::connection::create_test_database,
    };

    async fn create_test_service() -> AreaService<SqlxAreaRepository> {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxAreaRepository::new(pool);
        AreaService::new(repository)
    }

    #[tokio::test]
    async fn test_create_area_endpoint() {
        let service = create_test_service().await;
        let app = create_area_routes(service);

        let payload = CreateAreaPayload {
            name: "Test Area".to_string(),
            color: "#FF0000".to_string(),
            parent_area_id: None,
        };

        let request = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_get_areas_endpoint() {
        let service = create_test_service().await;
        let app = create_area_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_area_stats_endpoint() {
        let service = create_test_service().await;
        let app = create_area_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/stats")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
