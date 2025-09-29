// Cutie 后端模块 - 保留必要的旧模块以确保兼容性
pub mod config; // 保留配置模块
pub mod startup; // 保留启动模块（sidecar功能需要）

// 新的功能切片模块
pub mod features;
pub mod shared;

// 重新导出新架构的类型
pub use features::*;
pub use shared::*;

// 保留必要的旧模块导出
pub use config::*;
pub use startup::*;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化端口存储
    let _ = SIDECAR_PORT.set(Arc::new(Mutex::new(None)));

    // 初始化日志系统，设置默认级别
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Cutie application with new architecture");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_sidecar_port])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 带端口发现功能的启动函数
pub fn run_with_port_discovery(discovered_port: Arc<Mutex<Option<u16>>>) {
    // 初始化端口存储
    let _ = SIDECAR_PORT.set(discovered_port);

    // 初始化日志系统，设置默认级别
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Cutie application with port discovery");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_sidecar_port])
        .setup(|app| {
            // 启动端口监听器
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // 等待端口发现
                loop {
                    if let Some(port) = get_sidecar_port() {
                        tracing::info!("Port discovered: {}, notifying frontend", port);

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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
