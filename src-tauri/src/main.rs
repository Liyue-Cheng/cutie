// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 定义常量以避免魔法字符串和数字
const SIDECAR_ARG: &str = "--sidecar";

/// 应用程序主入口。
///
/// 根据命令行参数，此函数会以两种模式之一启动应用：
/// 1. **正常模式**: 同时启动 Tauri GUI 应用和后端的 HTTP Sidecar 服务器。
/// 2. **Sidecar 模式**: 如果提供了 `--sidecar` 参数，则只启动 HTTP Sidecar 服务器。
///    这在开发或需要独立运行后端服务时非常有用。
fn main() {
    // 检查是否以Sidecar模式启动
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            SIDECAR_ARG => {
                // Sidecar模式 - 启动HTTP服务器（使用新架构）
                println!("🔧 Starting Cutie sidecar with new architecture...");
                let rt = tokio::runtime::Runtime::new()
                    .expect("Failed to create Tokio runtime for sidecar");
                rt.block_on(async {
                    if let Err(e) = explore_lib::startup::sidecar::run_sidecar().await {
                        eprintln!("❌ Sidecar server failed to start: {}", e);
                        std::process::exit(1);
                    }
                });
            }
            _ => {
                eprintln!("❌ Unknown argument: {}", args[1]);
                print_usage();
                std::process::exit(1);
            }
        }
    } else {
        // 正常Tauri模式 - 同时启动Sidecar服务器
        println!("🖥️  Starting Tauri GUI with new architecture sidecar...");
        run_tauri_with_sidecar()
    }
}

/// 使用动态端口发现机制启动 Tauri 应用和 Sidecar 服务器
fn run_tauri_with_sidecar() {
    use std::io::{BufRead, BufReader};
    use std::process::{Command, Stdio};
    use std::sync::{Arc, Mutex};

    // 使用Arc<Mutex<Option<u16>>>来安全地在线程间共享端口号
    let discovered_port = Arc::new(Mutex::new(None::<u16>));
    let port_clone = Arc::clone(&discovered_port);

    // 使用Arc<Mutex<Option<u32>>>来存储子进程PID
    let sidecar_pid = Arc::new(Mutex::new(None::<u32>));
    let pid_clone = Arc::clone(&sidecar_pid);

    // 启动sidecar子进程
    std::thread::spawn(move || {
        let current_pid = std::process::id();

        let mut child = Command::new(std::env::current_exe().unwrap())
            .arg(SIDECAR_ARG)
            .env("CUTIE_PARENT_PID", current_pid.to_string()) // 传递父进程 PID
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start sidecar process");

        let child_pid = child.id();
        println!("🚀 Sidecar process started with PID: {}", child_pid);

        // 存储子进程PID
        if let Ok(mut pid_guard) = pid_clone.lock() {
            *pid_guard = Some(child_pid);
        }

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);

            for line in reader.lines() {
                if let Ok(line) = line {
                    // 检查是否是端口发现行
                    if line.starts_with("SIDECAR_PORT=") {
                        if let Ok(port) = line.strip_prefix("SIDECAR_PORT=").unwrap().parse::<u16>()
                        {
                            println!("🔍 Discovered sidecar port: {}", port);

                            // 安全地设置发现的端口
                            if let Ok(mut port_guard) = port_clone.lock() {
                                *port_guard = Some(port);
                            }

                            // 端口发现后继续读取其他日志，不要break
                        }
                    } else {
                        // 转发其他输出到控制台
                        println!("[Sidecar] {}", line);
                    }
                }
            }
        }

        // 等待子进程结束
        let status = child.wait();
        println!("🛑 Sidecar process exited with status: {:?}", status);
    });

    // 启动Tauri应用，并传递端口发现回调和子进程PID
    explore_lib::run_with_port_discovery_and_cleanup(discovered_port, sidecar_pid);
}

/// 打印使用说明
fn print_usage() {
    println!("Cutie 启动选项:");
    println!("  (无参数)    - 启动Tauri GUI应用 + 后台新架构Sidecar服务器");
    println!("  --sidecar   - 只启动新架构Sidecar HTTP服务器");
    println!();
    println!("示例:");
    println!("  cargo run                # 启动GUI应用");
    println!("  cargo run -- --sidecar  # 启动sidecar服务器");
}
