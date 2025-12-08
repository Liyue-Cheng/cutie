/// Sidecar服务器模块 - 基于新架构重写
use axum::{extract::State, http::StatusCode, response::Json, routing::get, routing::post, Router};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use crate::config::AppConfig;
use crate::infra::core::{build_info, AppError};
use crate::startup::{AppState, HealthStatus};

/// 全局 shutdown 信号发送器
static SHUTDOWN_TX: std::sync::OnceLock<broadcast::Sender<()>> = std::sync::OnceLock::new();

/// 初始化 shutdown channel
fn init_shutdown_channel() -> broadcast::Receiver<()> {
    let (tx, rx) = broadcast::channel(1);
    let _ = SHUTDOWN_TX.set(tx);
    rx
}

/// 触发 shutdown 信号
pub fn trigger_shutdown() {
    if let Some(tx) = SHUTDOWN_TX.get() {
        let _ = tx.send(());
    }
}

/// 启动Sidecar服务器（带优雅关闭）
pub async fn start_sidecar_server(app_state: AppState) -> Result<(), AppError> {
    tracing::info!("Starting Cutie Sidecar Server with new feature-sliced architecture...");

    // 初始化 shutdown channel
    let shutdown_rx = init_shutdown_channel();

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

    // 设置优雅关闭信号（同时监听系统信号和 HTTP shutdown 请求）
    let shutdown_signal = setup_shutdown_signal(shutdown_rx);

    // 启动服务器（带优雅关闭）
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .map_err(|e| AppError::configuration_error(format!("Server failed: {}", e)))?;

    tracing::info!("Sidecar server shut down gracefully");
    Ok(())
}

/// 设置关闭信号监听
///
/// 监听 SIGTERM、SIGINT 信号以及 HTTP shutdown 请求
async fn setup_shutdown_signal(mut shutdown_rx: broadcast::Receiver<()>) {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    let http_shutdown = async move {
        let _ = shutdown_rx.recv().await;
    };

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, shutting down...");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM signal, shutting down...");
        }
        _ = http_shutdown => {
            tracing::info!("Received HTTP shutdown request, shutting down...");
        }
    }
}

// ❌ 父进程监控功能已移除
//
// 原因：每 2 秒执行一次同步阻塞的 tasklist/kill 命令会严重影响性能
// 在 Windows 上，tasklist 可能需要 100-300ms，阻塞整个 Tokio runtime
//
// 替代方案：
// 1. Tauri 会自动管理 sidecar 进程的生命周期
// 2. 如果需要更精细的控制，可以使用：
//    - Windows: 使用 WinAPI (OpenProcess + WaitForSingleObject) 的异步封装
//    - Unix: 使用 pidfd (Linux 5.3+) 或 kqueue (BSD/macOS)
// 3. 或者从 Tauri 前端定期发送心跳请求

/// 创建HTTP路由器
async fn create_router(app_state: AppState) -> Result<Router, AppError> {
    let config = app_state.config().clone();

    // 创建API路由 - 使用新的功能切片架构
    let api_routes = crate::features::create_api_router();

    // 创建完整的应用路由
    let app = Router::new()
        .route(&config.server.health_check_path, get(health_check_handler))
        .route("/info", get(server_info_handler))
        .route("/admin/shutdown", post(shutdown_handler))
        .nest(&config.server.api_prefix, api_routes)
        .with_state(app_state);

    // 添加中间件
    let mut app = app;

    // 添加请求追踪中间件（包含 req_id 生成与标准字段）
    if config.server.request_logging {
        use axum::middleware;
        app = app.layer(middleware::from_fn(
            crate::infra::logging::request_tracing_middleware,
        ));
    }

    // 添加CORS中间件
    if config.server.cors_enabled {
        let cors = CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
            .max_age(std::time::Duration::from_secs(3600));

        app = app.layer(cors);
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
        ],
    })
}

/// Shutdown 处理器 - 触发优雅关闭
async fn shutdown_handler() -> Json<ShutdownResponse> {
    tracing::info!("Shutdown endpoint called, initiating graceful shutdown...");
    trigger_shutdown();
    Json(ShutdownResponse {
        message: "Shutdown initiated".to_string(),
        timestamp: chrono::Utc::now(),
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

    // 使用统一日志系统初始化
    if let Err(e) = crate::infra::logging::init_logging() {
        eprintln!("⚠️  Failed to initialize logging system: {}", e);
        // 降级到简单的控制台日志
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
    }

    tracing::info!(
        target: "STARTUP:sidecar",
        version = env!("CARGO_PKG_VERSION"),
        "=== Cutie Sidecar Server Starting (New Architecture) ==="
    );
    tracing::info!(
        target: "STARTUP:sidecar",
        "Configuration loaded successfully"
    );

    // 输出数据库路径信息
    let db_path = config.database_path();
    tracing::info!(
        target: "STARTUP:database",
        database_path = %db_path.display(),
        build_mode = if cfg!(debug_assertions) { "debug" } else { "release" },
        "Database path configured"
    );

    // 初始化数据库
    let db_pool = crate::startup::database::initialize_database(&config).await?;
    tracing::info!("Database initialized successfully");

    // 创建应用状态
    let app_state = AppState::new_production(config, db_pool.clone());
    tracing::info!("Application state created");

    // 启动事件分发器（后台任务）
    {
        use crate::infra::events::{
            dispatcher::EventDispatcher, outbox::SqlxEventOutboxRepository,
        };
        use std::sync::Arc;

        let outbox_repo = Arc::new(SqlxEventOutboxRepository::new(db_pool.clone()));
        let sse_state = app_state.sse_state().clone();
        let write_semaphore = app_state.write_semaphore(); // ✅ 注入写入信号量
        let dispatcher = Arc::new(EventDispatcher::new(
            outbox_repo,
            sse_state,
            20, // 20ms 间隔，更快的 SSE 响应
            write_semaphore,
        ));

        tokio::spawn(async move {
            dispatcher.start().await;
        });

        tracing::info!("Event dispatcher started");
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownResponse {
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
