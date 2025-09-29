/// 模板HTTP处理器
use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Router,
};
use uuid::Uuid;

use crate::shared::{
    database::TemplateRepository,
    http::{
        error_handler::{created_response, no_content_response, success_response},
        extractors::ValidatedJson,
    },
};

use super::{
    payloads::{
        CloneTemplatePayload, CreateTaskFromTemplatePayload, CreateTemplatePayload, TemplateQuery,
        UpdateTemplatePayload,
    },
    service::TemplateService,
};

/// 模板处理器
pub struct TemplateHandlers<R: TemplateRepository> {
    service: TemplateService<R>,
}

impl<R: TemplateRepository> TemplateHandlers<R> {
    pub fn new(service: TemplateService<R>) -> Self {
        Self { service }
    }

    /// 创建模板
    pub async fn create_template(
        &self,
        ValidatedJson(payload): ValidatedJson<CreateTemplatePayload>,
    ) -> Response {
        match self.service.create_template(payload).await {
            Ok(template) => created_response(template).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取模板详情
    pub async fn get_template(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.get_template(id).await {
            Ok(template) => success_response(template).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 更新模板
    pub async fn update_template(
        &self,
        Path(id): Path<Uuid>,
        ValidatedJson(payload): ValidatedJson<UpdateTemplatePayload>,
    ) -> Response {
        match self.service.update_template(id, payload).await {
            Ok(template) => success_response(template).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 删除模板
    pub async fn delete_template(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.delete_template(id).await {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取模板列表
    pub async fn get_templates(&self, Query(query): Query<TemplateQuery>) -> Response {
        match self
            .service
            .search_templates(query.q, query.area_id, query.variable, query.limit)
            .await
        {
            Ok(templates) => success_response(templates).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 克隆模板
    pub async fn clone_template(
        &self,
        Path(id): Path<Uuid>,
        ValidatedJson(payload): ValidatedJson<CloneTemplatePayload>,
    ) -> Response {
        match self.service.clone_template(id, payload.new_name).await {
            Ok(template) => created_response(template).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 基于模板创建任务
    pub async fn create_task_from_template(
        &self,
        Path(id): Path<Uuid>,
        ValidatedJson(payload): ValidatedJson<CreateTaskFromTemplatePayload>,
    ) -> Response {
        match self.service.create_task_from_template(id, payload).await {
            Ok(task) => created_response(task).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取模板统计
    pub async fn get_template_stats(&self) -> Response {
        match self.service.get_template_stats().await {
            Ok(stats) => success_response(stats).into_response(),
            Err(err) => err.into_response(),
        }
    }
}

/// 创建模板路由
pub fn create_template_routes<R, S>(service: TemplateService<R>) -> Router<S>
where
    R: TemplateRepository + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    let handlers = TemplateHandlers::new(service);

    use std::sync::Arc;
    let handlers = Arc::new(handlers);

    Router::new()
        .route(
            "/",
            post({
                let handlers = handlers.clone();
                move |payload| async move { handlers.create_template(payload).await }
            }),
        )
        .route(
            "/",
            get({
                let handlers = handlers.clone();
                move |query| async move { handlers.get_templates(query).await }
            }),
        )
        .route(
            "/stats",
            get({
                let handlers = handlers.clone();
                move || async move { handlers.get_template_stats().await }
            }),
        )
        .route(
            "/:id",
            get({
                let handlers = handlers.clone();
                move |path| async move { handlers.get_template(path).await }
            }),
        )
        .route(
            "/:id",
            put({
                let handlers = handlers.clone();
                move |path, payload| async move { handlers.update_template(path, payload).await }
            }),
        )
        .route(
            "/:id",
            delete({
                let handlers = handlers.clone();
                move |path| async move { handlers.delete_template(path).await }
            }),
        )
        .route(
            "/:id/clone",
            post({
                let handlers = handlers.clone();
                move |path, payload| async move { handlers.clone_template(path, payload).await }
            }),
        )
        .route(
            "/:id/tasks",
            post({
                let handlers = handlers.clone();
                move |path, payload| async move {
                    handlers.create_task_from_template(path, payload).await
                }
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
        features::templates::repository::SqlxTemplateRepository,
        shared::database::connection::create_test_database,
    };

    async fn create_test_service() -> TemplateService<SqlxTemplateRepository> {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTemplateRepository::new(pool);
        TemplateService::new(repository)
    }

    #[tokio::test]
    async fn test_create_template_endpoint() {
        let service = create_test_service().await;
        let app = create_template_routes(service);

        let payload = CreateTemplatePayload {
            name: "Test Template".to_string(),
            title_template: "Task for {{date}}".to_string(),
            glance_note_template: None,
            detail_note_template: None,
            estimated_duration_template: Some(30),
            subtasks_template: None,
            area_id: None,
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
    async fn test_get_templates_endpoint() {
        let service = create_test_service().await;
        let app = create_template_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
