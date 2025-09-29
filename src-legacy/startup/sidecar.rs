use axum::ServiceExt;
use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
// use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::shared::core::AppError;
use crate::config::AppConfig;
use crate::startup::AppState;

/// Sidecar服务器启动和管理
///
/// **预期行为简介:** 启动独立的HTTP服务器作为Tauri应用的Sidecar进程

/// 启动Sidecar服务器
///
/// **预期行为简介:** 启动HTTP服务器，监听动态分配的端口，通过stdout输出端口号
/// **输入输出规范:**
/// - **前置条件:** app_state必须是完全初始化的依赖注入容器
/// - **后置条件:** HTTP服务器开始监听，端口号通过stdout输出
/// **边界情况:** 如果端口绑定失败，返回详细错误信息
/// **预期副作用:** 启动HTTP服务器，占用系统端口，输出到stdout
pub async fn start_sidecar_server(app_state: AppState) -> Result<(), AppError> {
    log::info!("Starting Cutie Sidecar Server...");

    let config = app_state.config();

    // 创建路由
    let app = create_router(app_state.clone()).await?;

    // 绑定到指定地址
    let bind_addr = "127.0.0.1:3030";

    let listener = TcpListener::bind(bind_addr).await.map_err(|e| {
        AppError::configuration_error(format!("Failed to bind to address {}: {}", bind_addr, e))
    })?;

    let actual_addr = listener.local_addr().map_err(|e| {
        AppError::configuration_error(format!("Failed to get local address: {}", e))
    })?;

    // 输出端口号到stdout，供Tauri主进程读取
    println!("CUTIE_SIDECAR_PORT={}", actual_addr.port());

    log::info!("Sidecar server listening on {}", actual_addr);
    log::info!(
        "API endpoints available at http://{}{}",
        actual_addr,
        config.server.api_prefix
    );
    log::info!(
        "Health check available at http://{}{}",
        actual_addr,
        config.server.health_check_path
    );

    // 启动服务器
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::configuration_error(format!("Server error: {}", e)))?;

    log::info!("Sidecar server shutdown completed");
    Ok(())
}

/// 创建Axum路由器
///
/// **预期行为简介:** 构建完整的HTTP路由配置，包括中间件和端点
async fn create_router(app_state: AppState) -> Result<Router, AppError> {
    let config = app_state.config().clone();

    // 创建基础路由
    let mut app = Router::new()
        .route(&config.server.health_check_path, get(health_check_handler))
        .route("/info", get(server_info_handler));

    // 添加完整的API路由 - 使用新的功能切片架构
    let api_routes = Router::new()
        .route("/ping", get(ping_handler))
        .merge(crate::features::api_router::create_new_api_router(app_state.db_pool().clone()));

    app = app.nest(&config.server.api_prefix, api_routes);

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

    let middleware_count = if config.server.cors_enabled { 1 } else { 0 }
        + if config.server.request_logging { 1 } else { 0 }
        + if config.server.compression_enabled {
            1
        } else {
            0
        }
        + 1;

    log::debug!("Router created with {} middleware layers", middleware_count);

    Ok(app.with_state(app_state))
}

/// 健康检查处理器
async fn health_check_handler(
    State(app_state): State<AppState>,
) -> Result<Json<HealthCheckResponse>, StatusCode> {
    match app_state.health_check().await {
        Ok(health_status) => {
            let response = HealthCheckResponse {
                status: health_status.overall.as_str().to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                details: health_status.details,
                database_status: health_status.database.as_str().to_string(),
                repositories_status: health_status.repositories.as_str().to_string(),
            };

            match health_status.overall {
                crate::startup::HealthStatus::Healthy => Ok(Json(response)),
                crate::startup::HealthStatus::Degraded => {
                    // 返回206 Partial Content表示部分功能可用
                    Err(StatusCode::PARTIAL_CONTENT)
                }
                crate::startup::HealthStatus::Unhealthy => Err(StatusCode::SERVICE_UNAVAILABLE),
            }
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// 服务器信息处理器
async fn server_info_handler(State(app_state): State<AppState>) -> Json<ServerInfoResponse> {
    let config = app_state.config();

    Json(ServerInfoResponse {
        name: "Cutie Sidecar Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: config.environment.as_str().to_string(),
        api_prefix: config.server.api_prefix.clone(),
        features: ServerFeatures {
            ai_enabled: config.ai_enabled,
            performance_monitoring: config.performance_monitoring,
            cors_enabled: config.server.cors_enabled,
            compression_enabled: config.server.compression_enabled,
            tls_enabled: config.server.tls_config.is_some(),
        },
        uptime_seconds: 0, // TODO: 实现真实的运行时间统计
    })
}

/// Ping处理器
async fn ping_handler() -> Json<PingResponse> {
    Json(PingResponse {
        message: "pong".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

/// 优雅关闭信号
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            log::info!("Received Ctrl+C signal, shutting down...");
        },
        _ = terminate => {
            log::info!("Received terminate signal, shutting down...");
        },
    }
}

/// 健康检查响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub details: Vec<String>,
    pub database_status: String,
    pub repositories_status: String,
}

/// 服务器信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfoResponse {
    pub name: String,
    pub version: String,
    pub environment: String,
    pub api_prefix: String,
    pub features: ServerFeatures,
    pub uptime_seconds: u64,
}

/// 服务器功能特性
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerFeatures {
    pub ai_enabled: bool,
    pub performance_monitoring: bool,
    pub cors_enabled: bool,
    pub compression_enabled: bool,
    pub tls_enabled: bool,
}

/// Ping响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PingResponse {
    pub message: String,
    pub timestamp: String,
}

/// 启动Sidecar进程的主入口点
///
/// **预期行为简介:** 这是Sidecar进程的main函数，负责完整的启动流程
pub async fn run_sidecar() -> Result<(), AppError> {
    // 初始化日志
    env_logger::try_init().map_err(|e| {
        AppError::configuration_error(format!("Failed to initialize logger: {}", e))
    })?;

    log::info!("=== Cutie Sidecar Server Starting ===");

    // 加载配置
    let config = AppConfig::from_env()?;
    log::info!(
        "Configuration loaded for environment: {}",
        config.environment.as_str()
    );

    // 验证配置
    config.validate()?;
    log::info!("Configuration validated successfully");

    // 初始化数据库
    let db_pool = crate::startup::database::initialize_database(&config).await?;
    log::info!("Database initialized successfully");

    // 构建应用状态
    let app_state = AppState::new_production(config, db_pool);
    log::info!("Application state initialized");

    // 启动服务器
    start_sidecar_server(app_state).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_check_response_serialization() {
        let response = HealthCheckResponse {
            status: "healthy".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            version: "1.0.0".to_string(),
            details: vec!["All systems operational".to_string()],
            database_status: "healthy".to_string(),
            repositories_status: "healthy".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("healthy"));
        assert!(json.contains("1.0.0"));

        let deserialized: HealthCheckResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.status, response.status);
        assert_eq!(deserialized.version, response.version);
    }

    #[test]
    fn test_server_info_response_serialization() {
        let response = ServerInfoResponse {
            name: "Cutie Sidecar Server".to_string(),
            version: "1.0.0".to_string(),
            environment: "test".to_string(),
            api_prefix: "/api".to_string(),
            features: ServerFeatures {
                ai_enabled: true,
                performance_monitoring: false,
                cors_enabled: true,
                compression_enabled: true,
                tls_enabled: false,
            },
            uptime_seconds: 3600,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Cutie Sidecar Server"));
        assert!(json.contains("test"));

        let deserialized: ServerInfoResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, response.name);
        assert_eq!(
            deserialized.features.ai_enabled,
            response.features.ai_enabled
        );
    }

    #[test]
    fn test_ping_response_serialization() {
        let response = PingResponse {
            message: "pong".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("pong"));

        let deserialized: PingResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.message, response.message);
    }
}
