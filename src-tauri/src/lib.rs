// Cutie 后端模块 - 保留必要的旧模块以确保兼容性
pub mod config; // 保留配置模块
pub mod startup; // 保留启动模块（sidecar功能需要）

// 新的功能切片模块
pub mod entities;
pub mod features;
pub mod infra;

// 显式导出最常用的核心类型，避免 ambiguous glob re-exports 警告

// Features - 导出路由创建函数
pub use features::create_api_router;

// Startup - 导出核心应用状态
pub use startup::AppState;

// 注意：以下类型在多个模块中都有定义，不在顶层导出以避免歧义：
// - DatabaseConfig: 在 config::database_config 和 crate::infra::database::connection 中都有
// - SynchronousMode: 在 config::database_config 和 crate::infra::database::connection 中都有
// - HealthCheckResponse, PingResponse, ServerInfoResponse: 在 startup::sidecar 和 crate::infra::http::responses 中都有
//
// 使用时请指定完整路径，例如：
// - use crate::config::DatabaseConfig;
// - use crate::infra::database::DatabaseConfig;

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
    if let Err(e) = infra::logging::init_logging() {
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
                    tracing::info!("Application exit requested, initiating graceful shutdown...");
                    graceful_shutdown_sidecar(&pid_for_cleanup);
                    tracing::info!("Cleanup completed, allowing exit");
                }
                _ => {}
            }
        });
}

/// 优雅关闭 sidecar 进程
///
/// 1. 先尝试通过 HTTP shutdown endpoint 请求优雅关闭
/// 2. 等待最多 3 秒让进程自行退出
/// 3. 如果超时，强制杀死进程
fn graceful_shutdown_sidecar(pid: &Arc<Mutex<Option<u32>>>) {
    // 获取端口号
    let port = get_sidecar_port();

    if let Some(port) = port {
        tracing::info!("Sending shutdown request to sidecar on port {}", port);

        // 发送 HTTP shutdown 请求
        let shutdown_url = format!("http://127.0.0.1:{}/admin/shutdown", port);

        #[cfg(target_os = "windows")]
        let client_result = {
            use std::os::windows::process::CommandExt;
            use std::process::Command;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            // 使用 curl 发送 POST 请求（Windows 10+ 自带 curl）
            Command::new("curl")
                .args(&["-X", "POST", "-s", "-o", "NUL", &shutdown_url])
                .creation_flags(CREATE_NO_WINDOW)
                .output()
        };

        #[cfg(not(target_os = "windows"))]
        let client_result = {
            use std::process::Command;
            Command::new("curl")
                .args(&["-X", "POST", "-s", "-o", "/dev/null", &shutdown_url])
                .output()
        };

        match client_result {
            Ok(output) if output.status.success() => {
                tracing::info!("Shutdown request sent successfully, waiting for graceful shutdown...");

                // 等待进程退出（最多 3 秒）
                if wait_for_process_exit(pid, std::time::Duration::from_secs(3)) {
                    tracing::info!("Sidecar process exited gracefully");
                    return;
                }

                tracing::warn!("Graceful shutdown timeout, forcing termination...");
            }
            Ok(_) => {
                tracing::warn!("Shutdown request failed, forcing termination...");
            }
            Err(e) => {
                tracing::warn!("Failed to send shutdown request: {}, forcing termination...", e);
            }
        }
    } else {
        tracing::warn!("Sidecar port not available, forcing termination...");
    }

    // 强制杀死进程
    force_kill_sidecar_process(pid);
}

/// 等待进程退出
fn wait_for_process_exit(pid: &Arc<Mutex<Option<u32>>>, timeout: std::time::Duration) -> bool {
    let start = std::time::Instant::now();
    let check_interval = std::time::Duration::from_millis(100);

    while start.elapsed() < timeout {
        if !is_process_running(pid) {
            return true;
        }
        std::thread::sleep(check_interval);
    }

    false
}

/// 检查进程是否还在运行
fn is_process_running(pid: &Arc<Mutex<Option<u32>>>) -> bool {
    if let Ok(pid_guard) = pid.lock() {
        if let Some(process_pid) = *pid_guard {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                use std::process::Command;
                const CREATE_NO_WINDOW: u32 = 0x08000000;

                // 使用 tasklist 检查进程是否存在
                if let Ok(output) = Command::new("tasklist")
                    .args(&["/FI", &format!("PID eq {}", process_pid), "/NH"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output()
                {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    // 如果输出包含 PID，说明进程还在运行
                    return stdout.contains(&process_pid.to_string());
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                use std::process::Command;
                // 使用 kill -0 检查进程是否存在（不发送信号）
                if let Ok(output) = Command::new("kill")
                    .args(&["-0", &process_pid.to_string()])
                    .output()
                {
                    return output.status.success();
                }
            }
        }
    }
    false
}

/// 强制杀死 sidecar 进程
fn force_kill_sidecar_process(pid: &Arc<Mutex<Option<u32>>>) {
    if let Ok(pid_guard) = pid.lock() {
        if let Some(process_pid) = *pid_guard {
            tracing::info!("Attempting to kill sidecar process (PID: {})", process_pid);

            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                use std::process::Command;

                // CREATE_NO_WINDOW flag to prevent console window from appearing
                const CREATE_NO_WINDOW: u32 = 0x08000000;

                match Command::new("taskkill")
                    .args(&["/F", "/PID", &process_pid.to_string()])
                    .creation_flags(CREATE_NO_WINDOW)
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
