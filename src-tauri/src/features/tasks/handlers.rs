/// 任务HTTP处理器
///
/// 处理任务相关的HTTP请求
use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use uuid::Uuid;

use crate::shared::{
    database::TaskRepository,
    http::{
        error_handler::{created_response, no_content_response, success_response},
        extractors::{SearchQuery, ValidatedJson},
    },
};

use super::{
    payloads::{BulkOperationType, BulkTaskOperation, CreateTaskPayload, UpdateTaskPayload},
    service::TaskService,
};

/// 任务处理器
pub struct TaskHandlers<R: TaskRepository> {
    service: TaskService<R>,
}

impl<R: TaskRepository> TaskHandlers<R> {
    pub fn new(service: TaskService<R>) -> Self {
        Self { service }
    }

    /// 创建任务
    pub async fn create_task(
        &self,
        ValidatedJson(payload): ValidatedJson<CreateTaskPayload>,
    ) -> Response {
        match self.service.create_task(payload).await {
            Ok(task) => created_response(task).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取任务详情
    pub async fn get_task(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.get_task(id).await {
            Ok(task) => success_response(task).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 更新任务
    pub async fn update_task(
        &self,
        Path(id): Path<Uuid>,
        ValidatedJson(payload): ValidatedJson<UpdateTaskPayload>,
    ) -> Response {
        match self.service.update_task(id, payload).await {
            Ok(task) => success_response(task).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 删除任务
    pub async fn delete_task(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.delete_task(id).await {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 完成任务
    pub async fn complete_task(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.complete_task(id).await {
            Ok(task) => success_response(task).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 重新打开任务
    pub async fn reopen_task(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.reopen_task(id).await {
            Ok(task) => success_response(task).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 搜索任务
    pub async fn search_tasks(&self, Query(query): Query<SearchQuery>) -> Response {
        match self.service.search_tasks(query.q, query.limit).await {
            Ok(tasks) => success_response(tasks).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取未安排任务
    pub async fn get_unscheduled_tasks(&self) -> Response {
        match self.service.get_unscheduled_tasks().await {
            Ok(tasks) => success_response(tasks).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取任务统计
    pub async fn get_task_stats(&self) -> Response {
        match self.service.get_task_stats().await {
            Ok(stats) => success_response(stats).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 批量操作任务
    pub async fn bulk_operation(
        &self,
        ValidatedJson(payload): ValidatedJson<BulkTaskOperation>,
    ) -> Response {
        let result = match payload.operation {
            BulkOperationType::Delete => self.service.bulk_delete_tasks(payload.task_ids).await,
            BulkOperationType::Complete => self.service.bulk_complete_tasks(payload.task_ids).await,
            BulkOperationType::Reopen => self.service.bulk_reopen_tasks(payload.task_ids).await,
            BulkOperationType::UpdateArea => {
                // TODO: 实现批量更新领域
                Err(crate::shared::core::AppError::StringError(
                    "Not implemented".to_string(),
                ))
            }
            BulkOperationType::UpdateProject => {
                // TODO: 实现批量更新项目
                Err(crate::shared::core::AppError::StringError(
                    "Not implemented".to_string(),
                ))
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

/// 创建任务路由
pub fn create_task_routes<R, S>(service: TaskService<R>) -> Router<S>
where
    R: TaskRepository + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    let handlers = TaskHandlers::new(service);

    // 由于Rust的所有权系统，我们需要将handlers包装在Arc中以便在多个路由中共享
    use std::sync::Arc;
    let handlers = Arc::new(handlers);

    Router::new()
        .route(
            "/",
            post({
                let handlers = handlers.clone();
                move |payload| async move { handlers.create_task(payload).await }
            }),
        )
        .route(
            "/",
            get({
                let handlers = handlers.clone();
                move |query| async move { handlers.search_tasks(query).await }
            }),
        )
        .route(
            "/unscheduled",
            get({
                let handlers = handlers.clone();
                move || async move { handlers.get_unscheduled_tasks().await }
            }),
        )
        .route(
            "/stats",
            get({
                let handlers = handlers.clone();
                move || async move { handlers.get_task_stats().await }
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
                move |path| async move { handlers.get_task(path).await }
            }),
        )
        .route(
            "/:id",
            put({
                let handlers = handlers.clone();
                move |path, payload| async move { handlers.update_task(path, payload).await }
            }),
        )
        .route(
            "/:id",
            delete({
                let handlers = handlers.clone();
                move |path| async move { handlers.delete_task(path).await }
            }),
        )
        .route(
            "/:id/completion",
            post({
                let handlers = handlers.clone();
                move |path| async move { handlers.complete_task(path).await }
            }),
        )
        .route(
            "/:id/reopen",
            post({
                let handlers = handlers.clone();
                move |path| async move { handlers.reopen_task(path).await }
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
        features::tasks::{payloads::CreationContextPayload, repository::SqlxTaskRepository},
        shared::{core::ContextType, database::connection::create_test_database},
    };

    async fn create_test_service() -> TaskService<SqlxTaskRepository> {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTaskRepository::new(pool);
        TaskService::new(repository)
    }

    #[tokio::test]
    async fn test_create_task_endpoint() {
        let service = create_test_service().await;
        let app = create_task_routes(service);

        let payload = CreateTaskPayload {
            title: "Test Task".to_string(),
            glance_note: None,
            detail_note: None,
            estimated_duration: Some(60),
            subtasks: None,
            area_id: None,
            due_date: None,
            due_date_type: None,
            context: CreationContextPayload {
                context_type: ContextType::Misc,
                context_id: "floating".to_string(),
            },
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
    async fn test_get_unscheduled_tasks_endpoint() {
        let service = create_test_service().await;
        let app = create_task_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/unscheduled")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_task_stats_endpoint() {
        let service = create_test_service().await;
        let app = create_task_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/stats")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
