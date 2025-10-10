/// 日志系统初始化
use super::config::LogConfig;
use std::sync::Once;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

static INIT: Once = Once::new();

/// 全局 WorkerGuard，防止非阻塞日志写入器过早释放
static mut GUARDS: Option<Vec<WorkerGuard>> = None;

/// 初始化日志系统（使用默认配置）
///
/// 这个函数只会执行一次，多次调用是安全的
pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let config = LogConfig::from_env();
    init_logging_with_config(config)
}

/// 使用自定义配置初始化日志系统
///
/// 这个函数只会执行一次，多次调用是安全的
///
/// # Features
///
/// - 控制台输出（支持彩色）
/// - 文件输出（按天轮转）
/// - 过期日志清理
/// - 环境变量过滤器（RUST_LOG）
/// - Panic 捕获（可选）
pub fn init_logging_with_config(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    let mut result = Ok(());
    let mut guards = Vec::new();

    INIT.call_once(|| {
        // 1. 确保日志目录存在
        if config.file_logging_enabled {
            if let Err(e) = config.ensure_log_directory() {
                eprintln!("⚠️  Failed to create log directory: {}", e);
                result = Err(e.into());
                return;
            }

            // 2. 清理过期日志
            match config.cleanup_old_logs() {
                Ok(count) => {
                    if count > 0 {
                        println!("🗑️  Cleaned up {} old log file(s)", count);
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to cleanup old logs: {}", e);
                }
            }
        }

        // 3. 创建环境过滤器
        let env_filter = EnvFilter::try_from_default_env()
            .or_else(|_| EnvFilter::try_new(&config.log_level))
            .unwrap_or_else(|_| EnvFilter::new("info"));

        // 4. 创建控制台输出层
        let console_layer = fmt::layer()
            .with_target(true) // 显示 target（我们的分层标签）
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_ansi(config.console_colors_enabled)
            .compact()
            .with_filter(env_filter.clone());

        // 5. 创建文件输出层（如果启用）
        let file_layer = if config.file_logging_enabled {
            // 按天轮转的日志文件
            let file_appender = tracing_appender::rolling::daily(
                &config.log_directory,
                "cutie.log", // 文件名格式：cutie.log.YYYY-MM-DD
            );

            // 使用非阻塞写入器，避免 I/O 阻塞
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            guards.push(guard);

            // 文件日志层
            let layer = fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_file(true)
                .with_line_number(true)
                .with_ansi(false) // 文件中不使用 ANSI 颜色
                .with_writer(non_blocking);

            // 根据配置选择格式
            if config.json_format_enabled {
                // JSON 格式（生产环境）
                Some(layer.json().with_filter(env_filter.clone()).boxed())
            } else {
                // 人类可读格式（开发环境）
                Some(layer.with_filter(env_filter.clone()).boxed())
            }
        } else {
            None
        };

        // 6. 组合所有层并初始化
        let registry = tracing_subscriber::registry().with(console_layer);

        if let Some(file_layer) = file_layer {
            registry.with(file_layer).init();
        } else {
            registry.init();
        }

        // 7. 设置 panic 处理器（如果启用）
        if config.panic_capture_enabled {
            super::panic_handler::setup_panic_handler(config.log_directory.clone());
        }

        // 8. 保存 guards 到全局静态变量
        unsafe {
            GUARDS = Some(guards);
        }

        // 9. 记录初始化成功
        tracing::info!(
            target: "STARTUP:logging",
            log_level = %config.log_level,
            log_directory = ?config.log_directory,
            file_logging = config.file_logging_enabled,
            json_format = config.json_format_enabled,
            "Logging system initialized successfully"
        );

        result = Ok(());
    });

    result
}

/// 获取当前日志配置信息（用于诊断）
pub fn get_log_info() -> String {
    let config = LogConfig::from_env();
    format!(
        "Log Level: {}\nLog Directory: {:?}\nFile Logging: {}\nJSON Format: {}\nRetention Days: {}",
        config.log_level,
        config.log_directory,
        config.file_logging_enabled,
        config.json_format_enabled,
        config.retention_days
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_logging_is_idempotent() {
        // 多次调用不应 panic
        let _ = init_logging();
        let _ = init_logging();
        let _ = init_logging();
    }

    #[test]
    fn test_get_log_info() {
        let info = get_log_info();
        assert!(info.contains("Log Level"));
        assert!(info.contains("Log Directory"));
    }
}
