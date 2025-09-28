// Cutie 后端模块
pub mod adapters;
pub mod common;
pub mod config;
pub mod core;
pub mod handlers;
pub mod middleware;
pub mod ports;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod startup;

// 重新导出常用类型
pub use adapters::*;
pub use common::*;
pub use config::*;
pub use core::*;
pub use handlers::*;
pub use middleware::*;
pub use ports::*;
pub use repositories::*;
pub use routes::*;
pub use services::*;
pub use startup::*;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志
    if let Err(e) = init_dev_logger() {
        eprintln!("Failed to initialize logger: {}", e);
    }

    log::info!("Starting Cutie application");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
