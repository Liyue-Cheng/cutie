/// 排序HTTP处理器
///
/// 处理排序相关的HTTP请求
use axum::{
    extract::Query,
    response::{IntoResponse, Response},
    routing::{delete, get, put},
    Router,
};

use crate::shared::{
    database::OrderingRepository,
    http::{
        error_handler::{no_content_response, success_response},
        extractors::ValidatedJson,
    },
};

use super::{
    payloads::{
        BatchUpdateOrderingPayload, CalculateSortOrderQuery, ClearContextQuery,
        ContextOrderingQuery, UpdateOrderPayload,
    },
    service::OrderingService,
};

/// 排序处理器
pub struct OrderingHandlers<R: OrderingRepository> {
    service: OrderingService<R>,
}

impl<R: OrderingRepository> OrderingHandlers<R> {
    pub fn new(service: OrderingService<R>) -> Self {
        Self { service }
    }

    /// 更新排序
    pub async fn update_order(
        &self,
        ValidatedJson(payload): ValidatedJson<UpdateOrderPayload>,
    ) -> Response {
        match self.service.update_order(payload).await {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取上下文排序
    pub async fn get_context_ordering(
        &self,
        Query(query): Query<ContextOrderingQuery>,
    ) -> Response {
        match self
            .service
            .get_context_ordering(&query.context_type, &query.context_id)
            .await
        {
            Ok(orderings) => success_response(orderings).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 清理上下文排序
    pub async fn clear_context_ordering(&self, Query(query): Query<ClearContextQuery>) -> Response {
        match self
            .service
            .clear_context_ordering(&query.context_type, &query.context_id)
            .await
        {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 批量更新排序
    pub async fn batch_update_ordering(
        &self,
        ValidatedJson(payload): ValidatedJson<BatchUpdateOrderingPayload>,
    ) -> Response {
        match self.service.batch_update_ordering(payload.orderings).await {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 计算排序位置
    pub async fn calculate_sort_order(
        &self,
        Query(query): Query<CalculateSortOrderQuery>,
    ) -> Response {
        match self
            .service
            .calculate_sort_order(
                &query.context_type,
                &query.context_id,
                query.prev_sort_order.as_deref(),
                query.next_sort_order.as_deref(),
            )
            .await
        {
            Ok(response) => success_response(response).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取排序统计
    pub async fn get_ordering_stats(&self) -> Response {
        match self.service.get_ordering_stats().await {
            Ok(stats) => success_response(stats).into_response(),
            Err(err) => err.into_response(),
        }
    }
}

/// 创建排序路由
pub fn create_ordering_routes<R, S>(service: OrderingService<R>) -> Router<S>
where
    R: OrderingRepository + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    let handlers = OrderingHandlers::new(service);

    // 使用Arc包装handlers以便在多个路由中共享
    use std::sync::Arc;
    let handlers = Arc::new(handlers);

    Router::new()
        .route(
            "/",
            put({
                let handlers = handlers.clone();
                move |payload| async move { handlers.update_order(payload).await }
            }),
        )
        .route(
            "/",
            get({
                let handlers = handlers.clone();
                move |query| async move { handlers.get_context_ordering(query).await }
            }),
        )
        .route(
            "/",
            delete({
                let handlers = handlers.clone();
                move |query| async move { handlers.clear_context_ordering(query).await }
            }),
        )
        .route(
            "/batch",
            put({
                let handlers = handlers.clone();
                move |payload| async move { handlers.batch_update_ordering(payload).await }
            }),
        )
        .route(
            "/calculate",
            get({
                let handlers = handlers.clone();
                move |query| async move { handlers.calculate_sort_order(query).await }
            }),
        )
        .route(
            "/stats",
            get({
                let handlers = handlers.clone();
                move || async move { handlers.get_ordering_stats().await }
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
        features::ordering::repository::SqlxOrderingRepository,
        shared::{core::ContextType, database::connection::create_test_database},
    };

    async fn create_test_service() -> OrderingService<SqlxOrderingRepository> {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxOrderingRepository::new(pool);
        OrderingService::new(repository)
    }

    #[tokio::test]
    async fn test_update_order_endpoint() {
        let service = create_test_service().await;
        let app = create_ordering_routes(service);

        let payload = UpdateOrderPayload {
            context_type: ContextType::Misc,
            context_id: "floating".to_string(),
            task_id: uuid::Uuid::new_v4(),
            new_sort_order: "n".to_string(),
        };

        let request = Request::builder()
            .method("PUT")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&payload).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_calculate_sort_order_endpoint() {
        let service = create_test_service().await;
        let app = create_ordering_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/calculate?context_type=MISC&context_id=floating&prev_sort_order=a&next_sort_order=z")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_ordering_stats_endpoint() {
        let service = create_test_service().await;
        let app = create_ordering_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/stats")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
