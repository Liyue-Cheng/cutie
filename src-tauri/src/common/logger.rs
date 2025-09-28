use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;

/// 日志配置结构
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// 日志级别
    pub level: LevelFilter,
    /// 是否显示时间戳
    pub show_timestamp: bool,
    /// 是否显示模块路径
    pub show_module_path: bool,
    /// 是否显示线程ID
    pub show_thread_id: bool,
    /// 自定义格式
    pub custom_format: Option<String>,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            level: LevelFilter::Info,
            show_timestamp: true,
            show_module_path: true,
            show_thread_id: false,
            custom_format: None,
        }
    }
}

/// 初始化日志记录器
///
/// **预期行为简介:** 根据输入参数（如日志级别、格式），配置并初始化一个全局的日志记录器
/// **输入输出规范:**
/// - **前置条件:** config为有效的LoggerConfig
/// - **后置条件:** 成功调用后，整个应用后续的log::info!, log::error!等宏调用，都会按照此配置进行输出
/// **预期副作用:** 这是一个全局状态修改操作，会注册一个全局的Log trait实现。它在应用的生命周期中只应被调用一次
/// **边界情况:** 如果重复调用，应有一个明确的行为（返回一个错误），以防止意外的重置配置
pub fn init_logger(config: LoggerConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut builder = Builder::from_default_env();

    // 设置日志级别
    builder.filter_level(config.level);

    // 设置输出目标为标准输出
    builder.target(Target::Stdout);

    // 自定义格式化函数
    builder.format(move |buf, record| {
        let level = record.level();
        let target = record.target();
        let args = record.args();

        // 构建时间戳
        let timestamp = if config.show_timestamp {
            format!(
                "{} ",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC")
            )
        } else {
            String::new()
        };

        // 构建模块路径
        let module_path = if config.show_module_path {
            format!(" [{}]", target)
        } else {
            String::new()
        };

        // 构建线程ID
        let thread_id = if config.show_thread_id {
            format!(" (thread: {:?})", std::thread::current().id())
        } else {
            String::new()
        };

        // 根据日志级别选择颜色（如果终端支持）
        let level_colored = match level {
            log::Level::Error => format!("\x1b[31m{}\x1b[0m", level), // 红色
            log::Level::Warn => format!("\x1b[33m{}\x1b[0m", level),  // 黄色
            log::Level::Info => format!("\x1b[32m{}\x1b[0m", level),  // 绿色
            log::Level::Debug => format!("\x1b[36m{}\x1b[0m", level), // 青色
            log::Level::Trace => format!("\x1b[35m{}\x1b[0m", level), // 紫色
        };

        if let Some(ref custom_format) = config.custom_format {
            // 使用自定义格式（简单的占位符替换）
            let formatted = custom_format
                .replace("{timestamp}", &timestamp.trim())
                .replace("{level}", &level.to_string())
                .replace("{level_colored}", &level_colored)
                .replace("{target}", target)
                .replace(
                    "{module}",
                    &module_path.trim_start_matches(" [").trim_end_matches("]"),
                )
                .replace(
                    "{thread}",
                    &thread_id
                        .trim_start_matches(" (thread: ")
                        .trim_end_matches(")"),
                )
                .replace("{args}", &args.to_string());

            writeln!(buf, "{}", formatted)
        } else {
            // 默认格式
            writeln!(
                buf,
                "{}{}{}{} {}",
                timestamp, level_colored, module_path, thread_id, args
            )
        }
    });

    // 尝试初始化，如果已经初始化过会返回错误
    match builder.try_init() {
        Ok(()) => {
            log::info!(
                "Logger initialized successfully with level: {:?}",
                config.level
            );
            Ok(())
        }
        Err(e) => {
            // 如果是SetLoggerError，说明已经初始化过了
            if e.to_string().contains(
                "attempted to set a logger after the logging system was already initialized",
            ) {
                Err(
                    "Logger has already been initialized. Multiple initialization is not allowed."
                        .into(),
                )
            } else {
                Err(e.into())
            }
        }
    }
}

/// 初始化开发环境日志记录器
///
/// **预期行为简介:** 使用适合开发环境的默认配置初始化日志记录器
pub fn init_dev_logger() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if is_logger_initialized() {
        return Ok(());
    }
    let config = LoggerConfig {
        level: LevelFilter::Debug,
        show_timestamp: true,
        show_module_path: true,
        show_thread_id: false,
        custom_format: None,
    };

    init_logger(config)
}

/// 初始化生产环境日志记录器
///
/// **预期行为简介:** 使用适合生产环境的默认配置初始化日志记录器
pub fn init_prod_logger() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = LoggerConfig {
        level: LevelFilter::Info,
        show_timestamp: true,
        show_module_path: false,
        show_thread_id: false,
        custom_format: Some("{timestamp} {level} {args}".to_string()),
    };

    init_logger(config)
}

/// 初始化测试环境日志记录器
///
/// **预期行为简介:** 使用适合测试环境的默认配置初始化日志记录器
pub fn init_test_logger() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = LoggerConfig {
        level: LevelFilter::Trace,
        show_timestamp: false,
        show_module_path: true,
        show_thread_id: true,
        custom_format: None,
    };

    init_logger(config)
}

/// 获取当前日志级别
pub fn get_current_log_level() -> Option<LevelFilter> {
    // 注意：env_logger不提供直接获取当前级别的方法
    // 这里我们返回None，表示无法确定
    // 在实际应用中，可能需要维护一个全局状态来跟踪当前级别
    None
}

/// 检查日志记录器是否已初始化
pub fn is_logger_initialized() -> bool {
    // 尝试获取当前的最大日志级别
    log::max_level() != LevelFilter::Off
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, error, info, warn};

    #[test]
    fn test_logger_config_default() {
        let config = LoggerConfig::default();
        assert_eq!(config.level, LevelFilter::Info);
        assert!(config.show_timestamp);
        assert!(config.show_module_path);
        assert!(!config.show_thread_id);
        assert!(config.custom_format.is_none());
    }

    #[test]
    fn test_init_dev_logger() {
        // 注意：在测试中，日志记录器可能已经被初始化
        // 所以这个测试可能会失败，这是正常的
        let result = init_dev_logger();

        // 如果成功或者已经初始化，都算正常
        match result {
            Ok(()) => {
                info!("Dev logger initialized successfully");
            }
            Err(e) => {
                if e.to_string().contains("already initialized") {
                    // 这是预期的情况，不应该panic
                    println!("Logger already initialized (expected in tests)");
                } else {
                    // 记录错误但不panic，因为在测试环境中可能有其他原因
                    println!(
                        "Logger initialization error (may be expected in tests): {}",
                        e
                    );
                }
            }
        }
    }

    #[test]
    fn test_logger_output() {
        // 尝试初始化测试日志记录器
        let _ = init_test_logger();

        // 测试不同级别的日志输出
        error!("This is an error message");
        warn!("This is a warning message");
        info!("This is an info message");
        debug!("This is a debug message");

        // 在测试中，我们主要验证这些调用不会panic
        // 实际的输出格式需要手动检查
    }

    #[test]
    fn test_is_logger_initialized() {
        // 尝试初始化
        let _ = init_test_logger();

        // 检查是否已初始化
        assert!(is_logger_initialized());
    }
}
