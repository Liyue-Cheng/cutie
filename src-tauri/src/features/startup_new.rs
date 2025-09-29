/// 新架构的启动配置
///
/// 使用功能切片架构启动服务器
use axum::{middleware, Router};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer, trace::TraceLayer,
};

use crate::shared::{
    database::{initialize_database, DatabaseConfig},
    http::middleware::{
        cors_middleware, logging_middleware, request_id_middleware, security_headers_middleware,
    },
};

use super::api_router::create_new_api_router;

/// 启动新架构的HTTP服务器
pub async fn run_new_server() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Cutie server with new feature-sliced architecture...");

    // 创建数据库连接
    let db_config = DatabaseConfig::default();
    let db_path = std::path::Path::new("cutie.db");
    let pool = initialize_database(db_path, &db_config).await?;

    tracing::info!("Database initialized successfully");

    // 创建应用路由
    let app = create_app_router(pool).await?;

    // 启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 3030));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// 创建应用路由器
async fn create_app_router(pool: SqlitePool) -> Result<Router, Box<dyn std::error::Error>> {
    let app = Router::new()
        // 使用新的功能切片API路由
        .nest("/api", create_new_api_router(pool))
        // 添加中间件层
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(cors_middleware))
        .layer(middleware::from_fn(logging_middleware))
        .layer(middleware::from_fn(request_id_middleware))
        // 添加tower中间件
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(1024 * 1024 * 16)); // 16MB limit

    tracing::info!("Application router created with new architecture");
    Ok(app)
}

/// 演示如何使用新的任务功能模块
pub async fn demo_new_task_api() -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Running demo of new task API...");

    // 创建数据库连接
    let pool = crate::shared::database::connection::create_test_database().await?;

    // 创建应用状态
    let config = crate::config::AppConfig::default();
    let app_state = crate::startup::AppState::new(config, pool);

    // 演示创建任务
    let create_payload = crate::features::tasks::shared::dtos::CreateTaskRequest {
        title: "Demo Task from New Architecture".to_string(),
        glance_note: Some(
            "This task was created using the new feature-sliced architecture".to_string(),
        ),
        detail_note: Some(
            "The new architecture provides better separation of concerns and modularity"
                .to_string(),
        ),
        estimated_duration: Some(30),
        subtasks: None,
        area_id: None,
        due_date: None,
        due_date_type: None,
        context: crate::features::tasks::shared::dtos::CreationContext {
            context_type: crate::shared::core::ContextType::Misc,
            context_id: "floating".to_string(),
        },
    };

    // 直接调用创建任务的逻辑
    let created_task =
        crate::features::tasks::endpoints::create_task::logic::execute(&app_state, create_payload)
            .await?;
    tracing::info!(
        "Created task: {} (ID: {})",
        created_task.title,
        created_task.id
    );

    // 演示获取任务
    let retrieved_task =
        crate::features::tasks::endpoints::get_task::logic::execute(&app_state, created_task.id)
            .await?;
    tracing::info!("Retrieved task: {}", retrieved_task.title);

    // 演示完成任务
    let completed_task = crate::features::tasks::endpoints::complete_task::logic::execute(
        &app_state,
        created_task.id,
    )
    .await?;
    tracing::info!("Completed task: {}", completed_task.title);

    tracing::info!("Demo completed successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_new_task_api() {
        // 运行演示
        if let Err(e) = demo_new_task_api().await {
            eprintln!("Demo failed: {}", e);
            panic!("Demo should complete successfully");
        }
    }

    #[tokio::test]
    async fn test_create_app_router() {
        let pool = crate::shared::database::connection::create_test_database()
            .await
            .unwrap();
        let router = create_app_router(pool).await.unwrap();

        // 验证路由器创建成功
        // 这里我们只检查路由器能否创建，不做具体的HTTP测试
        tracing::info!("Router created successfully in test");
    }
}
