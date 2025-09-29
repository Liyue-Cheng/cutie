/// Sidecar服务器模块 - 基于新架构重写
use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use crate::config::AppConfig;
use crate::shared::core::{build_info, AppError};
use crate::startup::{AppState, HealthStatus};

/// 启动Sidecar服务器
pub async fn start_sidecar_server(app_state: AppState) -> Result<(), AppError> {
    tracing::info!("Starting Cutie Sidecar Server with new feature-sliced architecture...");

    let config = app_state.config();

    // 创建路由
    let app = create_router(app_state.clone()).await?;

    // 绑定监听器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::configuration_error(format!("Failed to bind to {}: {}", addr, e)))?;

    let actual_addr = listener.local_addr().map_err(|e| {
        AppError::configuration_error(format!("Failed to get local address: {}", e))
    })?;

    // 在日志输出之前，先输出端口号到stdout（供前端发现）
    println!("SIDECAR_PORT={}", actual_addr.port());

    tracing::info!("Sidecar server listening on {}", actual_addr);

    // 启动服务器
    axum::serve(listener, app)
        .await
        .map_err(|e| AppError::configuration_error(format!("Server failed: {}", e)))?;

    Ok(())
}

/// 创建HTTP路由器
async fn create_router(app_state: AppState) -> Result<Router, AppError> {
    let config = app_state.config().clone();

    // 创建API路由 - 使用新的功能切片架构
    let api_routes =
        crate::features::api_router::create_new_api_router(app_state.db_pool().clone());

    // 创建完整的应用路由
    let app = Router::new()
        .route(&config.server.health_check_path, get(health_check_handler))
        .route("/info", get(server_info_handler))
        .nest(&config.server.api_prefix, api_routes)
        .with_state(app_state);

    // 添加中间件
    let mut app = app;

    // 添加CORS中间件
    if config.server.cors_enabled {
        let cors = CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any);

        app = app.layer(cors);
    }

    // 添加请求日志中间件
    if config.server.request_logging {
        app = app.layer(tower_http::trace::TraceLayer::new_for_http());
    }

    // 添加压缩中间件
    if config.server.compression_enabled {
        app = app.layer(tower_http::compression::CompressionLayer::new());
    }

    // 添加请求大小限制
    app = app.layer(tower_http::limit::RequestBodyLimitLayer::new(
        config.server.max_request_size_bytes,
    ));

    tracing::info!("Router created with new feature-sliced architecture");
    Ok(app)
}

/// 健康检查处理器
async fn health_check_handler(
    State(app_state): State<AppState>,
) -> Result<Json<HealthCheckResponse>, StatusCode> {
    match app_state.health_check().await {
        Ok(HealthStatus::Healthy) => {
            let response = HealthCheckResponse {
                status: "healthy".to_string(),
                timestamp: chrono::Utc::now(),
                version: build_info::version().to_string(),
                details: Some(serde_json::json!({
                    "database": "connected",
                    "architecture": "feature_sliced",
                    "build_time": build_info::build_time(),
                    "git_commit": build_info::git_commit_hash(),
                    "rust_version": build_info::rust_version()
                })),
            };
            Ok(Json(response))
        }
        Ok(HealthStatus::Unhealthy) => Err(StatusCode::SERVICE_UNAVAILABLE),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 服务器信息处理器
async fn server_info_handler() -> Json<ServerInfoResponse> {
    Json(ServerInfoResponse {
        name: "Cutie API".to_string(),
        version: build_info::version().to_string(),
        build_time: build_info::build_time().to_string(),
        rust_version: build_info::rust_version().to_string(),
        features: vec![
            "feature_sliced_architecture".to_string(),
            "task_management".to_string(),
            "schedule_management".to_string(),
            "time_blocking".to_string(),
            "template_system".to_string(),
            "area_hierarchy".to_string(),
            "lexorank_sorting".to_string(),
        ],
    })
}

/// Sidecar进程的主入口点
pub async fn run_sidecar() -> Result<(), AppError> {
    // 加载配置（先加载配置以获取日志级别）
    let config = AppConfig::from_env()?;

    // 初始化日志系统，使用配置中的日志级别
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", config.log_level_string());
    }
    // 使用 try_init 避免重复初始化错误
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    tracing::info!("=== Cutie Sidecar Server Starting (New Architecture) ===");
    tracing::info!("Configuration loaded successfully");

    // 初始化数据库
    let db_pool = crate::startup::database::initialize_database(&config).await?;
    tracing::info!("Database initialized successfully");

    // 创建应用状态
    let app_state = AppState::new_production(config, db_pool);
    tracing::info!("Application state created");

    // 启动服务器
    start_sidecar_server(app_state).await?;

    tracing::info!("=== Cutie Sidecar Server Stopped ===");
    Ok(())
}

/// 响应结构定义

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfoResponse {
    pub name: String,
    pub version: String,
    pub build_time: String,
    pub rust_version: String,
    pub features: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_structures() {
        let health_response = HealthCheckResponse {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            version: "1.0.0".to_string(),
            details: None,
        };

        assert_eq!(health_response.status, "healthy");
        assert_eq!(health_response.version, "1.0.0");
    }

    #[test]
    fn test_ping_response() {
        let ping_response = PingResponse {
            message: "pong".to_string(),
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(ping_response.message, "pong");
    }

    #[tokio::test]
    async fn test_sidecar_components() {
        // 测试能否创建基本组件
        let config = AppConfig::from_env().unwrap_or_else(|_| {
            // 如果环境变量不存在，创建默认配置用于测试
            AppConfig::default()
        });

        // 基本的组件创建测试
        assert!(!config.server.host.is_empty());
        assert!(config.server.port > 0);
    }
}
