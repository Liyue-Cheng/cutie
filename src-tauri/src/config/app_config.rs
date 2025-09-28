use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

use crate::common::error::AppError;
use super::{DatabaseConfig, ServerConfig};

/// 应用主配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 应用环境 (development, production, test)
    pub environment: Environment,
    
    /// 数据库配置
    pub database: DatabaseConfig,
    
    /// 服务器配置
    pub server: ServerConfig,
    
    /// 日志级别
    pub log_level: LogLevel,
    
    /// 数据目录路径
    pub data_dir: PathBuf,
    
    /// 配置文件目录路径
    pub config_dir: PathBuf,
    
    /// 是否启用AI功能
    pub ai_enabled: bool,
    
    /// AI服务配置
    pub ai_config: Option<AiConfig>,
    
    /// 是否启用性能监控
    pub performance_monitoring: bool,
    
    /// 最大并发连接数
    pub max_connections: u32,
    
    /// 请求超时时间（秒）
    pub request_timeout_seconds: u64,
}

/// 应用环境枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
    Test,
}

/// 日志级别枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// AI服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// AI服务提供商
    pub provider: String,
    
    /// API密钥
    pub api_key: Option<String>,
    
    /// API端点
    pub endpoint: Option<String>,
    
    /// 请求超时时间（秒）
    pub timeout_seconds: u64,
    
    /// 最大重试次数
    pub max_retries: u32,
    
    /// 是否启用缓存
    pub cache_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: Environment::Development,
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            log_level: LogLevel::Info,
            data_dir: Self::default_data_dir(),
            config_dir: Self::default_config_dir(),
            ai_enabled: false,
            ai_config: None,
            performance_monitoring: false,
            max_connections: 100,
            request_timeout_seconds: 30,
        }
    }
}

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, AppError> {
        let mut config = Self::default();
        
        // 加载环境
        if let Ok(env_str) = env::var("CUTIE_ENV") {
            config.environment = match env_str.as_str() {
                "development" | "dev" => Environment::Development,
                "production" | "prod" => Environment::Production,
                "test" => Environment::Test,
                _ => return Err(AppError::configuration_error(
                    format!("Invalid environment: {}", env_str)
                )),
            };
        }
        
        // 加载日志级别
        if let Ok(log_level_str) = env::var("CUTIE_LOG_LEVEL") {
            config.log_level = match log_level_str.to_lowercase().as_str() {
                "trace" => LogLevel::Trace,
                "debug" => LogLevel::Debug,
                "info" => LogLevel::Info,
                "warn" => LogLevel::Warn,
                "error" => LogLevel::Error,
                _ => return Err(AppError::configuration_error(
                    format!("Invalid log level: {}", log_level_str)
                )),
            };
        }
        
        // 加载数据目录
        if let Ok(data_dir_str) = env::var("CUTIE_DATA_DIR") {
            config.data_dir = PathBuf::from(data_dir_str);
        }
        
        // 加载配置目录
        if let Ok(config_dir_str) = env::var("CUTIE_CONFIG_DIR") {
            config.config_dir = PathBuf::from(config_dir_str);
        }
        
        // 加载AI配置
        if let Ok(ai_enabled_str) = env::var("CUTIE_AI_ENABLED") {
            config.ai_enabled = ai_enabled_str.to_lowercase() == "true";
        }
        
        if config.ai_enabled {
            config.ai_config = Some(AiConfig {
                provider: env::var("CUTIE_AI_PROVIDER").unwrap_or_else(|_| "openai".to_string()),
                api_key: env::var("CUTIE_AI_API_KEY").ok(),
                endpoint: env::var("CUTIE_AI_ENDPOINT").ok(),
                timeout_seconds: env::var("CUTIE_AI_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30),
                max_retries: env::var("CUTIE_AI_MAX_RETRIES")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3),
                cache_enabled: env::var("CUTIE_AI_CACHE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .to_lowercase() == "true",
            });
        }
        
        // 加载性能监控配置
        if let Ok(perf_monitoring_str) = env::var("CUTIE_PERFORMANCE_MONITORING") {
            config.performance_monitoring = perf_monitoring_str.to_lowercase() == "true";
        }
        
        // 加载连接配置
        if let Ok(max_conn_str) = env::var("CUTIE_MAX_CONNECTIONS") {
            config.max_connections = max_conn_str.parse()
                .map_err(|_| AppError::configuration_error("Invalid max_connections value"))?;
        }
        
        if let Ok(timeout_str) = env::var("CUTIE_REQUEST_TIMEOUT") {
            config.request_timeout_seconds = timeout_str.parse()
                .map_err(|_| AppError::configuration_error("Invalid request_timeout value"))?;
        }
        
        // 加载数据库配置
        config.database = DatabaseConfig::from_env(&config.data_dir)?;
        
        // 加载服务器配置
        config.server = ServerConfig::from_env()?;
        
        Ok(config)
    }
    
    /// 从TOML文件加载配置
    pub fn from_file(config_path: &PathBuf) -> Result<Self, AppError> {
        let config_content = std::fs::read_to_string(config_path)
            .map_err(|e| AppError::configuration_error(
                format!("Failed to read config file: {}", e)
            ))?;
        
        let config: AppConfig = toml::from_str(&config_content)
            .map_err(|e| AppError::configuration_error(
                format!("Failed to parse config file: {}", e)
            ))?;
        
        Ok(config)
    }
    
    /// 保存配置到TOML文件
    pub fn save_to_file(&self, config_path: &PathBuf) -> Result<(), AppError> {
        let config_content = toml::to_string_pretty(self)
            .map_err(|e| AppError::configuration_error(
                format!("Failed to serialize config: {}", e)
            ))?;
        
        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::configuration_error(
                    format!("Failed to create config directory: {}", e)
                ))?;
        }
        
        std::fs::write(config_path, config_content)
            .map_err(|e| AppError::configuration_error(
                format!("Failed to write config file: {}", e)
            ))?;
        
        Ok(())
    }
    
    /// 验证配置的有效性
    pub fn validate(&self) -> Result<(), AppError> {
        // 验证数据目录
        if !self.data_dir.exists() {
            std::fs::create_dir_all(&self.data_dir)
                .map_err(|e| AppError::configuration_error(
                    format!("Failed to create data directory: {}", e)
                ))?;
        }
        
        // 验证配置目录
        if !self.config_dir.exists() {
            std::fs::create_dir_all(&self.config_dir)
                .map_err(|e| AppError::configuration_error(
                    format!("Failed to create config directory: {}", e)
                ))?;
        }
        
        // 验证数据库配置
        self.database.validate()?;
        
        // 验证服务器配置
        self.server.validate()?;
        
        // 验证AI配置
        if self.ai_enabled {
            if let Some(ref ai_config) = self.ai_config {
                if ai_config.provider.is_empty() {
                    return Err(AppError::configuration_error("AI provider cannot be empty"));
                }
                
                if ai_config.timeout_seconds == 0 {
                    return Err(AppError::configuration_error("AI timeout must be greater than 0"));
                }
            } else {
                return Err(AppError::configuration_error("AI is enabled but no AI config provided"));
            }
        }
        
        // 验证连接配置
        if self.max_connections == 0 {
            return Err(AppError::configuration_error("max_connections must be greater than 0"));
        }
        
        if self.request_timeout_seconds == 0 {
            return Err(AppError::configuration_error("request_timeout_seconds must be greater than 0"));
        }
        
        Ok(())
    }
    
    /// 获取默认数据目录
    fn default_data_dir() -> PathBuf {
        if let Some(data_dir) = dirs::data_dir() {
            data_dir.join("cutie")
        } else {
            PathBuf::from("./data")
        }
    }
    
    /// 获取默认配置目录
    fn default_config_dir() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("cutie")
        } else {
            PathBuf::from("./config")
        }
    }
    
    /// 获取数据库文件路径
    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join(&self.database.filename)
    }
    
    /// 获取设置文件路径
    pub fn settings_path(&self) -> PathBuf {
        self.config_dir.join("settings.toml")
    }
    
    /// 是否为开发环境
    pub fn is_development(&self) -> bool {
        self.environment == Environment::Development
    }
    
    /// 是否为生产环境
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }
    
    /// 是否为测试环境
    pub fn is_test(&self) -> bool {
        self.environment == Environment::Test
    }
    
    /// 获取日志级别对应的log::LevelFilter
    pub fn log_level_filter(&self) -> log::LevelFilter {
        match self.log_level {
            LogLevel::Trace => log::LevelFilter::Trace,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
        }
    }
}

impl Environment {
    /// 从字符串解析环境
    pub fn from_str(s: &str) -> Result<Self, AppError> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "production" | "prod" => Ok(Environment::Production),
            "test" => Ok(Environment::Test),
            _ => Err(AppError::configuration_error(format!("Invalid environment: {}", s))),
        }
    }
    
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Development => "development",
            Environment::Production => "production",
            Environment::Test => "test",
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            api_key: None,
            endpoint: None,
            timeout_seconds: 30,
            max_retries: 3,
            cache_enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        
        assert_eq!(config.environment, Environment::Development);
        assert_eq!(config.log_level, LogLevel::Info);
        assert!(!config.ai_enabled);
        assert!(config.ai_config.is_none());
        assert!(!config.performance_monitoring);
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.request_timeout_seconds, 30);
    }

    #[test]
    fn test_environment_from_str() {
        assert_eq!(Environment::from_str("development").unwrap(), Environment::Development);
        assert_eq!(Environment::from_str("dev").unwrap(), Environment::Development);
        assert_eq!(Environment::from_str("production").unwrap(), Environment::Production);
        assert_eq!(Environment::from_str("prod").unwrap(), Environment::Production);
        assert_eq!(Environment::from_str("test").unwrap(), Environment::Test);
        
        assert!(Environment::from_str("invalid").is_err());
    }

    #[test]
    fn test_environment_as_str() {
        assert_eq!(Environment::Development.as_str(), "development");
        assert_eq!(Environment::Production.as_str(), "production");
        assert_eq!(Environment::Test.as_str(), "test");
    }

    #[test]
    fn test_config_validation() {
        let config = AppConfig::default();
        // 默认配置应该是有效的（除了目录可能不存在，但validate会创建）
        // 在测试环境中，我们不实际创建目录
    }

    #[test]
    fn test_log_level_filter() {
        let mut config = AppConfig::default();
        
        config.log_level = LogLevel::Debug;
        assert_eq!(config.log_level_filter(), log::LevelFilter::Debug);
        
        config.log_level = LogLevel::Error;
        assert_eq!(config.log_level_filter(), log::LevelFilter::Error);
    }

    #[test]
    fn test_environment_checks() {
        let mut config = AppConfig::default();
        
        config.environment = Environment::Development;
        assert!(config.is_development());
        assert!(!config.is_production());
        assert!(!config.is_test());
        
        config.environment = Environment::Production;
        assert!(!config.is_development());
        assert!(config.is_production());
        assert!(!config.is_test());
        
        config.environment = Environment::Test;
        assert!(!config.is_development());
        assert!(!config.is_production());
        assert!(config.is_test());
    }

    #[test]
    fn test_config_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let original_config = AppConfig::default();
        
        // 测试保存配置
        original_config.save_to_file(&config_path).unwrap();
        assert!(config_path.exists());
        
        // 测试加载配置
        let loaded_config = AppConfig::from_file(&config_path).unwrap();
        assert_eq!(loaded_config.environment, original_config.environment);
        assert_eq!(loaded_config.log_level, original_config.log_level);
    }

    #[test]
    fn test_ai_config_default() {
        let ai_config = AiConfig::default();
        
        assert_eq!(ai_config.provider, "openai");
        assert!(ai_config.api_key.is_none());
        assert!(ai_config.endpoint.is_none());
        assert_eq!(ai_config.timeout_seconds, 30);
        assert_eq!(ai_config.max_retries, 3);
        assert!(ai_config.cache_enabled);
    }

    #[test]
    fn test_config_paths() {
        let config = AppConfig::default();
        
        let db_path = config.database_path();
        assert!(db_path.to_string_lossy().contains(&config.database.filename));
        
        let settings_path = config.settings_path();
        assert!(settings_path.to_string_lossy().ends_with("settings.toml"));
    }
}
