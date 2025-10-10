// Cutie 后端模块 - 保留必要的旧模块以确保兼容性
pub mod config; // 保留配置模块
pub mod startup; // 保留启动模块（sidecar功能需要）

// 新的功能切片模块
pub mod entities;
pub mod features;
pub mod shared;

// 显式导出最常用的核心类型，避免 ambiguous glob re-exports 警告

// Features - 导出路由创建函数
pub use features::create_api_router;

// Startup - 导出核心应用状态
pub use startup::AppState;

// 注意：以下类型在多个模块中都有定义，不在顶层导出以避免歧义：
// - DatabaseConfig: 在 config::database_config 和 shared::database::connection 中都有
// - SynchronousMode: 在 config::database_config 和 shared::database::connection 中都有
// - HealthCheckResponse, PingResponse, ServerInfoResponse: 在 startup::sidecar 和 shared::http::responses 中都有
//
// 使用时请指定完整路径，例如：
// - use crate::config::DatabaseConfig;
// - use crate::shared::database::DatabaseConfig;

use std::sync::{Arc, Mutex};
use tauri::Emitter;

// 全局端口存储
static SIDECAR_PORT: std::sync::OnceLock<Arc<Mutex<Option<u16>>>> = std::sync::OnceLock::new();

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// 获取发现的sidecar端口号
#[tauri::command]
fn get_sidecar_port() -> Option<u16> {
    if let Some(port_mutex) = SIDECAR_PORT.get() {
        if let Ok(port_guard) = port_mutex.lock() {
            return *port_guard;
        }
    }
    None
}

/// 设置sidecar端口号（内部使用）
pub fn set_sidecar_port(port: u16) {
    if let Some(port_mutex) = SIDECAR_PORT.get() {
        if let Ok(mut port_guard) = port_mutex.lock() {
            *port_guard = Some(port);
        }
    }
}

/// 初始化日志系统
/// 使用新的统一日志模块，支持文件落盘、轮转、panic捕获
fn init_logging() {
    // 设置默认日志级别（如果未设置）
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    
    // 使用统一日志系统初始化
    if let Err(e) = shared::logging::init_logging() {
        eprintln!("⚠️  Failed to initialize logging system: {}", e);
        // 降级到简单的控制台日志
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
    }
}

/// 构建基础的 Tauri 应用程序构建器
/// 在这里统一处理日志初始化和应用构建
fn build_tauri_app() -> tauri::Builder<tauri::Wry> {
    // 首先初始化日志系统
    init_logging();

    // 记录应用构建日志
    tracing::info!(
        target: "STARTUP:tauri",
        version = env!("CARGO_PKG_VERSION"),
        "Building Cutie application with Tauri"
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_sidecar_port])
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化端口存储
    let _ = SIDECAR_PORT.set(Arc::new(Mutex::new(None)));

    // 构建并运行应用（日志初始化在 build_tauri_app 中完成）
    build_tauri_app()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 带端口发现功能的启动函数
pub fn run_with_port_discovery(discovered_port: Arc<Mutex<Option<u16>>>) {
    run_with_port_discovery_and_cleanup(discovered_port, Arc::new(Mutex::new(None)));
}

/// 带端口发现和清理功能的启动函数
pub fn run_with_port_discovery_and_cleanup(
    discovered_port: Arc<Mutex<Option<u16>>>,
    sidecar_pid: Arc<Mutex<Option<u32>>>,
) {
    // 初始化端口存储
    let _ = SIDECAR_PORT.set(discovered_port);

    // 构建应用并添加端口发现和清理功能
    let pid_for_cleanup = sidecar_pid.clone();

    build_tauri_app()
        .setup(move |app| {
            // 记录端口发现模式启动
            tracing::info!(
                target: "STARTUP:tauri",
                "Starting Cutie application with port discovery mode"
            );

            // 启动端口监听器
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // 等待端口发现
                loop {
                    if let Some(port) = get_sidecar_port() {
                        tracing::info!(
                            target: "STARTUP:tauri",
                            port = %port,
                            "Port discovered, notifying frontend"
                        );

                        // 通知前端端口已发现
                        if let Err(e) = app_handle.emit("sidecar-port-discovered", port) {
                            tracing::error!("Failed to emit port discovery event: {}", e);
                        }
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            });

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |_app_handle, event| {
            // 全局事件处理器 - 捕获应用退出
            match event {
                tauri::RunEvent::ExitRequested { .. } => {
                    tracing::info!("Application exit requested, killing sidecar process...");
                    cleanup_sidecar_process_by_pid(&pid_for_cleanup);
                    // 清理完成，允许退出
                    tracing::info!("Cleanup completed, allowing exit");
                }
                _ => {}
            }
        });
}

/// 通过 PID 清理 sidecar 子进程
fn cleanup_sidecar_process_by_pid(pid: &Arc<Mutex<Option<u32>>>) {
    if let Ok(pid_guard) = pid.lock() {
        if let Some(process_pid) = *pid_guard {
            tracing::info!("Attempting to kill sidecar process (PID: {})", process_pid);

            #[cfg(target_os = "windows")]
            {
                use std::process::Command;
                match Command::new("taskkill")
                    .args(&["/F", "/PID", &process_pid.to_string()])
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            tracing::info!("Sidecar process killed successfully");
                        } else {
                            tracing::error!(
                                "Failed to kill sidecar process: {}",
                                String::from_utf8_lossy(&output.stderr)
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to execute taskkill: {}", e);
                    }
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                use std::process::Command;
                match Command::new("kill")
                    .args(&["-9", &process_pid.to_string()])
                    .output()
                {
                    Ok(output) => {
                        if output.status.success() {
                            tracing::info!("Sidecar process killed successfully");
                        } else {
                            tracing::error!(
                                "Failed to kill sidecar process: {}",
                                String::from_utf8_lossy(&output.stderr)
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to execute kill: {}", e);
                    }
                }
            }
        } else {
            tracing::warn!("Sidecar process PID not available");
        }
    }
}
