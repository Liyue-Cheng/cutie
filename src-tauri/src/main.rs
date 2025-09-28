// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // 检查是否以Sidecar模式启动
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--sidecar" {
        // Sidecar模式 - 启动HTTP服务器
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            if let Err(e) = explore_lib::startup::sidecar::run_sidecar().await {
                eprintln!("Sidecar server failed: {}", e);
                std::process::exit(1);
            }
        });
    } else {
        // 正常Tauri模式 - 同时启动Sidecar服务器
        run_tauri_with_sidecar()
    }
}

fn run_tauri_with_sidecar() {
    // 在后台线程启动Sidecar服务器
    std::thread::spawn(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            // 等待一下让Tauri先启动
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

            if let Err(e) = explore_lib::startup::sidecar::run_sidecar().await {
                eprintln!("Sidecar server failed: {}", e);
            }
        });
    });

    // 启动Tauri应用
    explore_lib::run()
}
