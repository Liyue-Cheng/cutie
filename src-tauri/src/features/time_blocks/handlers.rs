/// 时间块HTTP处理器
use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use uuid::Uuid;

use crate::shared::{
    database::{TimeBlockRepository, TimeBlockTaskRepository},
    http::{
        error_handler::{created_response, no_content_response, success_response},
        extractors::ValidatedJson,
    },
};

use super::{
    payloads::{
        CreateTimeBlockPayload, FreeSlotsQuery, LinkTaskPayload, TimeBlockQuery, TimeConflictQuery,
        UpdateTimeBlockPayload,
    },
    service::TimeBlockService,
};

/// 时间块处理器
pub struct TimeBlockHandlers<R: TimeBlockRepository, T: TimeBlockTaskRepository> {
    service: TimeBlockService<R, T>,
}

impl<R: TimeBlockRepository, T: TimeBlockTaskRepository> TimeBlockHandlers<R, T> {
    pub fn new(service: TimeBlockService<R, T>) -> Self {
        Self { service }
    }

    /// 创建时间块
    pub async fn create_time_block(
        &self,
        ValidatedJson(payload): ValidatedJson<CreateTimeBlockPayload>,
    ) -> Response {
        match self.service.create_time_block(payload).await {
            Ok(time_block) => created_response(time_block).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取时间块详情
    pub async fn get_time_block(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.get_time_block(id).await {
            Ok(time_block) => success_response(time_block).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 更新时间块
    pub async fn update_time_block(
        &self,
        Path(id): Path<Uuid>,
        ValidatedJson(payload): ValidatedJson<UpdateTimeBlockPayload>,
    ) -> Response {
        match self.service.update_time_block(id, payload).await {
            Ok(time_block) => success_response(time_block).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 删除时间块
    pub async fn delete_time_block(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.delete_time_block(id).await {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取时间块列表
    pub async fn get_time_blocks(&self, Query(query): Query<TimeBlockQuery>) -> Response {
        match self
            .service
            .get_time_blocks(
                query.date,
                query.start_date,
                query.end_date,
                query.task_id,
                query.area_id,
            )
            .await
        {
            Ok(time_blocks) => success_response(time_blocks).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 检查时间冲突
    pub async fn check_time_conflict(&self, Query(query): Query<TimeConflictQuery>) -> Response {
        match self
            .service
            .check_time_conflict(query.start_time, query.end_time, query.exclude_id)
            .await
        {
            Ok(response) => success_response(response).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 查找空闲时间段
    pub async fn find_free_slots(&self, Query(query): Query<FreeSlotsQuery>) -> Response {
        match self
            .service
            .find_free_slots(query.start_time, query.end_time, query.min_duration_minutes)
            .await
        {
            Ok(free_slots) => success_response(free_slots).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 链接任务到时间块
    pub async fn link_task_to_block(
        &self,
        Path(id): Path<Uuid>,
        ValidatedJson(payload): ValidatedJson<LinkTaskPayload>,
    ) -> Response {
        match self.service.link_task_to_block(id, payload.task_id).await {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 取消任务关联
    pub async fn unlink_task_from_block(
        &self,
        Path((time_block_id, task_id)): Path<(Uuid, Uuid)>,
    ) -> Response {
        match self
            .service
            .unlink_task_from_block(time_block_id, task_id)
            .await
        {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取时间块统计
    pub async fn get_time_block_stats(&self) -> Response {
        match self.service.get_time_block_stats().await {
            Ok(stats) => success_response(stats).into_response(),
            Err(err) => err.into_response(),
        }
    }
}

/// 创建时间块路由
pub fn create_time_block_routes<
    R: TimeBlockRepository + Clone + Send + Sync + 'static,
    T: TimeBlockTaskRepository + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
>(
    service: TimeBlockService<R, T>,
) -> Router<S> {
    let handlers = TimeBlockHandlers::new(service);

    use std::sync::Arc;
    let handlers = Arc::new(handlers);

    Router::new()
        .route(
            "/",
            post({
                let handlers = handlers.clone();
                move |payload| async move { handlers.create_time_block(payload).await }
            }),
        )
        .route(
            "/",
            get({
                let handlers = handlers.clone();
                move |query| async move { handlers.get_time_blocks(query).await }
            }),
        )
        .route(
            "/conflicts",
            get({
                let handlers = handlers.clone();
                move |query| async move { handlers.check_time_conflict(query).await }
            }),
        )
        .route(
            "/free-slots",
            get({
                let handlers = handlers.clone();
                move |query| async move { handlers.find_free_slots(query).await }
            }),
        )
        .route(
            "/stats",
            get({
                let handlers = handlers.clone();
                move || async move { handlers.get_time_block_stats().await }
            }),
        )
        .route(
            "/:id",
            get({
                let handlers = handlers.clone();
                move |path| async move { handlers.get_time_block(path).await }
            }),
        )
        .route(
            "/:id",
            put({
                let handlers = handlers.clone();
                move |path, payload| async move { handlers.update_time_block(path, payload).await }
            }),
        )
        .route(
            "/:id",
            delete({
                let handlers = handlers.clone();
                move |path| async move { handlers.delete_time_block(path).await }
            }),
        )
        .route(
            "/:id/tasks",
            post({
                let handlers = handlers.clone();
                move |path, payload| async move { handlers.link_task_to_block(path, payload).await }
            }),
        )
        .route(
            "/:id/tasks/:task_id",
            delete({
                let handlers = handlers.clone();
                move |path| async move { handlers.unlink_task_from_block(path).await }
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
        features::time_blocks::repository::{SqlxTimeBlockRepository, SqlxTimeBlockTaskRepository},
        shared::database::connection::create_test_database,
    };

    async fn create_test_service(
    ) -> TimeBlockService<SqlxTimeBlockRepository, SqlxTimeBlockTaskRepository> {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTimeBlockRepository::new(pool.clone());
        let task_repository = SqlxTimeBlockTaskRepository::new(pool);
        TimeBlockService::new(repository, task_repository)
    }

    #[tokio::test]
    async fn test_create_time_block_endpoint() {
        let service = create_test_service().await;
        let app = create_time_block_routes(service);

        let now = chrono::Utc::now();
        let payload = CreateTimeBlockPayload {
            title: Some("Test Block".to_string()),
            glance_note: None,
            detail_note: None,
            start_time: now + chrono::Duration::hours(1),
            end_time: now + chrono::Duration::hours(2),
            area_id: None,
            task_ids: vec![],
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
    async fn test_get_time_blocks_endpoint() {
        let service = create_test_service().await;
        let app = create_time_block_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
