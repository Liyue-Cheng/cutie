/// Panic 处理器
///
/// 捕获 panic 并写入独立的日志文件
use std::panic;
use std::path::PathBuf;
use std::sync::Once;

static PANIC_HANDLER_INIT: Once = Once::new();

/// 设置全局 panic 处理器
///
/// 当程序 panic 时，会：
/// 1. 将 panic 信息写入独立的 panic 日志文件（panic-YYYYMMDD-HHMMSS.log）
/// 2. 通过 tracing 记录 error 级别日志
/// 3. 调用默认的 panic 处理器（打印到 stderr）
///
/// # Arguments
///
/// * `log_directory` - panic 日志文件存储目录
pub fn setup_panic_handler(log_directory: PathBuf) {
    PANIC_HANDLER_INIT.call_once(move || {
        // 确保目录存在
        if !log_directory.exists() {
            let _ = std::fs::create_dir_all(&log_directory);
        }

        // 克隆 log_directory 用于闭包
        let log_dir_for_hook = log_directory.clone();

        // 保存默认的 panic hook
        let default_panic = panic::take_hook();

        // 设置自定义 panic hook
        panic::set_hook(Box::new(move |panic_info| {
            let log_directory = &log_dir_for_hook;
            // 1. 提取 panic 信息
            let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic payload".to_string()
            };

            let location = if let Some(location) = panic_info.location() {
                format!(
                    "{}:{}:{}",
                    location.file(),
                    location.line(),
                    location.column()
                )
            } else {
                "Unknown location".to_string()
            };

            let thread = std::thread::current();
            let thread_name = thread.name().unwrap_or("<unnamed>");

            // 2. 格式化 panic 信息
            let panic_message = format!(
                "=== PANIC ===\n\
                 Time: {}\n\
                 Thread: {}\n\
                 Location: {}\n\
                 Message: {}\n\
                 Backtrace:\n{:?}\n\
                 =============",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                thread_name,
                location,
                payload,
                std::backtrace::Backtrace::force_capture()
            );

            // 3. 写入独立的 panic 日志文件
            let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
            let panic_log_path = log_directory.join(format!("panic-{}.log", timestamp));

            if let Err(e) = std::fs::write(&panic_log_path, &panic_message) {
                eprintln!(
                    "❌ Failed to write panic log to {:?}: {}",
                    panic_log_path, e
                );
            } else {
                eprintln!("💾 Panic log saved to: {:?}", panic_log_path);
            }

            // 4. 通过 tracing 记录 error 级别日志
            tracing::error!(
                target: "PANIC",
                thread = %thread_name,
                location = %location,
                payload = %payload,
                panic_log_file = ?panic_log_path,
                "Application panic occurred"
            );

            // 5. 调用默认的 panic 处理器
            default_panic(panic_info);
        }));

        tracing::debug!(
            target: "STARTUP:panic_handler",
            log_directory = ?log_directory,
            "Panic handler installed"
        );
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn test_panic_handler_setup() {
        let temp_dir = std::env::temp_dir().join("cutie_test_panic_logs");
        setup_panic_handler(temp_dir.clone());

        // 验证处理器已安装（不会再次安装）
        setup_panic_handler(temp_dir);
    }

    #[test]
    #[should_panic(expected = "test panic")]
    fn test_panic_capture() {
        let temp_dir = std::env::temp_dir().join("cutie_test_panic_capture");
        setup_panic_handler(temp_dir);

        // 触发 panic
        panic!("test panic");
    }
}
