/// 日志配置
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志级别（默认：info）
    pub log_level: String,

    /// 是否启用文件日志（默认：true）
    pub file_logging_enabled: bool,

    /// 日志文件目录（默认：AppData/Local/Cutie/logs）
    pub log_directory: PathBuf,

    /// 日志文件保留天数（默认：14天）
    pub retention_days: usize,

    /// 是否启用控制台彩色输出（默认：true）
    pub console_colors_enabled: bool,

    /// 是否启用 JSON 格式日志（默认：false，开发环境用人类可读格式）
    pub json_format_enabled: bool,

    /// 是否启用 panic 捕获（默认：true）
    pub panic_capture_enabled: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            log_level: "info".to_string(),
            file_logging_enabled: true,
            log_directory: Self::default_log_directory(),
            retention_days: 14,
            console_colors_enabled: true,
            json_format_enabled: false,
            panic_capture_enabled: true,
        }
    }
}

impl LogConfig {
    /// 获取默认日志目录
    ///
    /// Windows: C:\Users\<user>\AppData\Local\Cutie\logs
    /// macOS: ~/Library/Application Support/Cutie/logs
    /// Linux: ~/.local/share/Cutie/logs
    fn default_log_directory() -> PathBuf {
        if let Some(data_dir) = dirs::data_local_dir() {
            data_dir.join("Cutie").join("logs")
        } else {
            // 如果无法获取系统目录，使用当前目录
            PathBuf::from("./logs")
        }
    }

    /// 从环境变量加载配置
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // 从 RUST_LOG 环境变量读取日志级别
        if let Ok(rust_log) = std::env::var("RUST_LOG") {
            config.log_level = rust_log;
        }

        // 从 CUTIE_LOG_DIR 环境变量读取日志目录
        if let Ok(log_dir) = std::env::var("CUTIE_LOG_DIR") {
            config.log_directory = PathBuf::from(log_dir);
        }

        // 从 CUTIE_LOG_RETENTION_DAYS 环境变量读取保留天数
        if let Ok(retention) = std::env::var("CUTIE_LOG_RETENTION_DAYS") {
            if let Ok(days) = retention.parse::<usize>() {
                config.retention_days = days;
            }
        }

        // 从 CUTIE_LOG_JSON 环境变量设置 JSON 格式
        if let Ok(json_enabled) = std::env::var("CUTIE_LOG_JSON") {
            config.json_format_enabled =
                json_enabled.to_lowercase() == "true" || json_enabled == "1";
        }

        config
    }

    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            log_level: "debug".to_string(),
            console_colors_enabled: true,
            json_format_enabled: false,
            ..Default::default()
        }
    }

    /// 创建生产环境配置
    pub fn production() -> Self {
        Self {
            log_level: "info".to_string(),
            console_colors_enabled: false,
            json_format_enabled: true,
            ..Default::default()
        }
    }

    /// 确保日志目录存在
    pub fn ensure_log_directory(&self) -> std::io::Result<()> {
        if !self.log_directory.exists() {
            std::fs::create_dir_all(&self.log_directory)?;
        }
        Ok(())
    }

    /// 清理过期日志文件
    pub fn cleanup_old_logs(&self) -> std::io::Result<usize> {
        use std::time::SystemTime;

        let mut deleted_count = 0;
        let retention_seconds = (self.retention_days as u64) * 24 * 60 * 60;

        if !self.log_directory.exists() {
            return Ok(0);
        }

        for entry in std::fs::read_dir(&self.log_directory)? {
            let entry = entry?;
            let path = entry.path();

            // 只处理日志文件
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if extension == "log" {
                        // 检查文件修改时间
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                                    if elapsed.as_secs() > retention_seconds {
                                        // 删除过期文件
                                        if std::fs::remove_file(&path).is_ok() {
                                            deleted_count += 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(deleted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LogConfig::default();
        assert_eq!(config.log_level, "info");
        assert!(config.file_logging_enabled);
        assert_eq!(config.retention_days, 14);
    }

    #[test]
    fn test_development_config() {
        let config = LogConfig::development();
        assert_eq!(config.log_level, "debug");
        assert!(config.console_colors_enabled);
        assert!(!config.json_format_enabled);
    }

    #[test]
    fn test_production_config() {
        let config = LogConfig::production();
        assert_eq!(config.log_level, "info");
        assert!(!config.console_colors_enabled);
        assert!(config.json_format_enabled);
    }
}
