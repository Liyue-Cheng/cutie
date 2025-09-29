/// 日程HTTP处理器
///
/// 处理日程相关的HTTP请求

use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use uuid::Uuid;

use crate::shared::{
    database::TaskScheduleRepository,
    http::{
        error_handler::{created_response, no_content_response, success_response},
        extractors::{SearchQuery, ValidatedJson},
    },
};

use super::{
    payloads::{
        BulkScheduleOperation, BulkScheduleOperationType, LogPresencePayload, ScheduleQuery,
        ScheduleTaskPayload,
    },
    service::ScheduleService,
};

/// 日程处理器
pub struct ScheduleHandlers<R: TaskScheduleRepository> {
    service: ScheduleService<R>,
}

impl<R: TaskScheduleRepository> ScheduleHandlers<R> {
    pub fn new(service: ScheduleService<R>) -> Self {
        Self { service }
    }

    /// 安排任务
    pub async fn schedule_task(
        &self,
        ValidatedJson(payload): ValidatedJson<ScheduleTaskPayload>,
    ) -> Response {
        match self.service.schedule_task(payload).await {
            Ok(schedule) => created_response(schedule).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取日程列表
    pub async fn get_schedules(&self, Query(query): Query<ScheduleQuery>) -> Response {
        match self
            .service
            .get_schedules(query.date, query.start_date, query.end_date, query.task_id)
            .await
        {
            Ok(schedules) => {
                // 如果指定了结局过滤，进一步过滤结果
                let filtered_schedules = if let Some(outcome) = query.outcome {
                    schedules
                        .into_iter()
                        .filter(|s| s.outcome == outcome)
                        .collect()
                } else {
                    schedules
                };

                success_response(filtered_schedules).into_response()
            }
            Err(err) => err.into_response(),
        }
    }

    /// 删除日程
    pub async fn delete_schedule(&self, Path(id): Path<Uuid>) -> Response {
        match self.service.delete_schedule(id).await {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 记录努力
    pub async fn log_presence(
        &self,
        Path(id): Path<Uuid>,
        ValidatedJson(payload): ValidatedJson<LogPresencePayload>,
    ) -> Response {
        match self.service.log_presence(id, payload.note).await {
            Ok(schedule) => success_response(schedule).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 取消任务所有日程
    pub async fn unschedule_task(&self, Path(task_id): Path<Uuid>) -> Response {
        match self.service.unschedule_task_completely(task_id).await {
            Ok(_) => no_content_response().into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 获取日程统计
    pub async fn get_schedule_stats(&self) -> Response {
        match self.service.get_schedule_stats().await {
            Ok(stats) => success_response(stats).into_response(),
            Err(err) => err.into_response(),
        }
    }

    /// 批量操作日程
    pub async fn bulk_operation(
        &self,
        ValidatedJson(payload): ValidatedJson<BulkScheduleOperation>,
    ) -> Response {
        let result = match payload.operation {
            BulkScheduleOperationType::Delete => {
                self.service.bulk_delete_schedules(payload.schedule_ids).await
            }
            BulkScheduleOperationType::LogPresence => {
                self.service.bulk_log_presence(payload.schedule_ids).await
            }
            BulkScheduleOperationType::MarkCarriedOver => {
                self.service
                    .bulk_mark_carried_over(payload.schedule_ids)
                    .await
            }
            BulkScheduleOperationType::MoveToDate => {
                // 从参数中提取目标日期
                if let Some(params) = &payload.parameters {
                    if let Some(target_date_value) = params.get("target_date") {
                        if let Ok(target_date_str) = serde_json::from_value::<String>(target_date_value.clone()) {
                            if let Ok(target_date) = chrono::DateTime::parse_from_rfc3339(&target_date_str) {
                                let target_date_utc = target_date.with_timezone(&chrono::Utc);
                                self.service
                                    .bulk_move_to_date(payload.schedule_ids, target_date_utc)
                                    .await
                            } else {
                                Err(crate::shared::core::AppError::validation_error(
                                    "target_date",
                                    "无效的日期格式",
                                    "INVALID_DATE_FORMAT",
                                ))
                            }
                        } else {
                            Err(crate::shared::core::AppError::validation_error(
                                "target_date",
                                "目标日期必须是字符串",
                                "TARGET_DATE_NOT_STRING",
                            ))
                        }
                    } else {
                        Err(crate::shared::core::AppError::validation_error(
                            "target_date",
                            "移动操作必须提供目标日期",
                            "TARGET_DATE_REQUIRED",
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

/// 创建日程路由
pub fn create_schedule_routes<R, S>(
    service: ScheduleService<R>,
) -> Router<S>
where
    R: TaskScheduleRepository + Clone + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
{
    let handlers = ScheduleHandlers::new(service);

    // 使用Arc包装handlers以便在多个路由中共享
    use std::sync::Arc;
    let handlers = Arc::new(handlers);

    Router::new()
        .route(
            "/",
            post({
                let handlers = handlers.clone();
                move |payload| async move { handlers.schedule_task(payload).await }
            }),
        )
        .route(
            "/",
            get({
                let handlers = handlers.clone();
                move |query| async move { handlers.get_schedules(query).await }
            }),
        )
        .route(
            "/stats",
            get({
                let handlers = handlers.clone();
                move || async move { handlers.get_schedule_stats().await }
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
            delete({
                let handlers = handlers.clone();
                move |path| async move { handlers.delete_schedule(path).await }
            }),
        )
        .route(
            "/:id/presence",
            post({
                let handlers = handlers.clone();
                move |path, payload| async move { handlers.log_presence(path, payload).await }
            }),
        )
        .route(
            "/tasks/:task_id",
            delete({
                let handlers = handlers.clone();
                move |path| async move { handlers.unschedule_task(path).await }
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
        features::schedules::repository::SqlxTaskScheduleRepository,
        shared::{core::normalize_to_day_start, database::connection::create_test_database},
    };

    async fn create_test_service() -> ScheduleService<SqlxTaskScheduleRepository> {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxTaskScheduleRepository::new(pool);
        ScheduleService::new(repository)
    }

    #[tokio::test]
    async fn test_schedule_task_endpoint() {
        let service = create_test_service().await;
        let app = create_schedule_routes(service);

        let payload = ScheduleTaskPayload {
            task_id: Uuid::new_v4(),
            target_day: normalize_to_day_start(chrono::Utc::now()),
            mode: super::payloads::ScheduleMode::Link,
            source_schedule_id: None,
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
    async fn test_get_schedules_endpoint() {
        let service = create_test_service().await;
        let app = create_schedule_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_schedule_stats_endpoint() {
        let service = create_test_service().await;
        let app = create_schedule_routes(service);

        let request = Request::builder()
            .method("GET")
            .uri("/stats")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
