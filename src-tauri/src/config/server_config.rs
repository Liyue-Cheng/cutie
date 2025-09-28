use serde::{Deserialize, Serialize};
use std::env;
use std::net::{IpAddr, Ipv4Addr};

use crate::common::error::AppError;

/// 服务器配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 监听地址
    pub host: IpAddr,

    /// 监听端口（0表示动态分配）
    pub port: u16,

    /// 是否启用CORS
    pub cors_enabled: bool,

    /// 允许的CORS源
    pub cors_origins: Vec<String>,

    /// 请求体大小限制（字节）
    pub max_request_size_bytes: usize,

    /// 是否启用请求日志
    pub request_logging: bool,

    /// 是否启用压缩
    pub compression_enabled: bool,

    /// 静态文件服务路径（可选）
    pub static_files_path: Option<String>,

    /// API路径前缀
    pub api_prefix: String,

    /// 健康检查路径
    pub health_check_path: String,

    /// 是否启用优雅关闭
    pub graceful_shutdown: bool,

    /// 优雅关闭超时时间（秒）
    pub shutdown_timeout_seconds: u64,

    /// 工作线程数（0表示使用CPU核心数）
    pub worker_threads: usize,

    /// 是否启用HTTP/2
    pub http2_enabled: bool,

    /// TLS配置（可选）
    pub tls_config: Option<TlsConfig>,
}

/// TLS配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// 证书文件路径
    pub cert_path: String,

    /// 私钥文件路径
    pub key_path: String,

    /// 是否强制HTTPS
    pub force_https: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), // localhost
            port: 3030,                                    // 固定端口，避免冲突
            cors_enabled: true,
            cors_origins: vec!["http://localhost:1420".to_string()], // Tauri默认端口
            max_request_size_bytes: 10 * 1024 * 1024,                // 10MB
            request_logging: false,
            compression_enabled: true,
            static_files_path: None,
            api_prefix: "/api".to_string(),
            health_check_path: "/health".to_string(),
            graceful_shutdown: true,
            shutdown_timeout_seconds: 30,
            worker_threads: 0, // 使用CPU核心数
            http2_enabled: false,
            tls_config: None,
        }
    }
}

impl ServerConfig {
    /// 从环境变量加载服务器配置
    pub fn from_env() -> Result<Self, AppError> {
        let mut config = Self::default();

        // 加载监听地址
        if let Ok(host_str) = env::var("CUTIE_SERVER_HOST") {
            config.host = host_str.parse().map_err(|_| {
                AppError::configuration_error(format!("Invalid server host: {}", host_str))
            })?;
        }

        // 加载监听端口
        if let Ok(port_str) = env::var("CUTIE_SERVER_PORT") {
            config.port = port_str.parse().map_err(|_| {
                AppError::configuration_error(format!("Invalid server port: {}", port_str))
            })?;
        }

        // 加载CORS配置
        if let Ok(cors_enabled_str) = env::var("CUTIE_SERVER_CORS_ENABLED") {
            config.cors_enabled = cors_enabled_str.to_lowercase() == "true";
        }

        if let Ok(cors_origins_str) = env::var("CUTIE_SERVER_CORS_ORIGINS") {
            config.cors_origins = cors_origins_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // 加载请求限制
        if let Ok(max_size_str) = env::var("CUTIE_SERVER_MAX_REQUEST_SIZE") {
            config.max_request_size_bytes = max_size_str
                .parse()
                .map_err(|_| AppError::configuration_error("Invalid max_request_size value"))?;
        }

        // 加载日志配置
        if let Ok(req_logging_str) = env::var("CUTIE_SERVER_REQUEST_LOGGING") {
            config.request_logging = req_logging_str.to_lowercase() == "true";
        }

        // 加载压缩配置
        if let Ok(compression_str) = env::var("CUTIE_SERVER_COMPRESSION") {
            config.compression_enabled = compression_str.to_lowercase() == "true";
        }

        // 加载静态文件配置
        if let Ok(static_path) = env::var("CUTIE_SERVER_STATIC_PATH") {
            if !static_path.is_empty() {
                config.static_files_path = Some(static_path);
            }
        }

        // 加载API前缀
        if let Ok(api_prefix) = env::var("CUTIE_SERVER_API_PREFIX") {
            config.api_prefix = api_prefix;
        }

        // 加载健康检查路径
        if let Ok(health_path) = env::var("CUTIE_SERVER_HEALTH_PATH") {
            config.health_check_path = health_path;
        }

        // 加载关闭配置
        if let Ok(graceful_str) = env::var("CUTIE_SERVER_GRACEFUL_SHUTDOWN") {
            config.graceful_shutdown = graceful_str.to_lowercase() == "true";
        }

        if let Ok(shutdown_timeout_str) = env::var("CUTIE_SERVER_SHUTDOWN_TIMEOUT") {
            config.shutdown_timeout_seconds = shutdown_timeout_str
                .parse()
                .map_err(|_| AppError::configuration_error("Invalid shutdown_timeout value"))?;
        }

        // 加载线程配置
        if let Ok(worker_threads_str) = env::var("CUTIE_SERVER_WORKER_THREADS") {
            config.worker_threads = worker_threads_str
                .parse()
                .map_err(|_| AppError::configuration_error("Invalid worker_threads value"))?;
        }

        // 加载HTTP/2配置
        if let Ok(http2_str) = env::var("CUTIE_SERVER_HTTP2") {
            config.http2_enabled = http2_str.to_lowercase() == "true";
        }

        // 加载TLS配置
        if let (Ok(cert_path), Ok(key_path)) = (
            env::var("CUTIE_SERVER_TLS_CERT"),
            env::var("CUTIE_SERVER_TLS_KEY"),
        ) {
            config.tls_config = Some(TlsConfig {
                cert_path,
                key_path,
                force_https: env::var("CUTIE_SERVER_FORCE_HTTPS")
                    .unwrap_or_else(|_| "false".to_string())
                    .to_lowercase()
                    == "true",
            });
        }

        Ok(config)
    }

    /// 验证服务器配置
    pub fn validate(&self) -> Result<(), AppError> {
        // 端口范围由u16类型自动保证，无需额外验证

        // 验证请求大小限制
        if self.max_request_size_bytes == 0 {
            return Err(AppError::configuration_error(
                "max_request_size_bytes must be greater than 0",
            ));
        }

        // 验证API前缀
        if !self.api_prefix.starts_with('/') {
            return Err(AppError::configuration_error(
                "api_prefix must start with '/'",
            ));
        }

        // 验证健康检查路径
        if !self.health_check_path.starts_with('/') {
            return Err(AppError::configuration_error(
                "health_check_path must start with '/'",
            ));
        }

        // 验证关闭超时
        if self.shutdown_timeout_seconds == 0 {
            return Err(AppError::configuration_error(
                "shutdown_timeout_seconds must be greater than 0",
            ));
        }

        // 验证TLS配置
        if let Some(ref tls_config) = self.tls_config {
            if tls_config.cert_path.is_empty() {
                return Err(AppError::configuration_error(
                    "TLS cert_path cannot be empty",
                ));
            }

            if tls_config.key_path.is_empty() {
                return Err(AppError::configuration_error(
                    "TLS key_path cannot be empty",
                ));
            }

            // 检查文件是否存在
            if !std::path::Path::new(&tls_config.cert_path).exists() {
                return Err(AppError::configuration_error(format!(
                    "TLS certificate file not found: {}",
                    tls_config.cert_path
                )));
            }

            if !std::path::Path::new(&tls_config.key_path).exists() {
                return Err(AppError::configuration_error(format!(
                    "TLS key file not found: {}",
                    tls_config.key_path
                )));
            }
        }

        Ok(())
    }

    /// 获取服务器监听地址
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// 获取开发环境的配置
    pub fn development() -> Self {
        Self {
            cors_enabled: true,
            cors_origins: vec![
                "http://localhost:1420".to_string(),
                "http://127.0.0.1:1420".to_string(),
                "http://localhost:3000".to_string(), // 开发服务器
            ],
            request_logging: true,
            ..Self::default()
        }
    }

    /// 获取生产环境的配置
    pub fn production() -> Self {
        Self {
            cors_enabled: false, // 生产环境通常不需要CORS
            request_logging: false,
            compression_enabled: true,
            graceful_shutdown: true,
            shutdown_timeout_seconds: 60,
            ..Self::default()
        }
    }

    /// 获取测试环境的配置
    pub fn test() -> Self {
        Self {
            port: 0, // 随机端口
            cors_enabled: false,
            request_logging: false,
            compression_enabled: false,
            graceful_shutdown: false,
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests_server_config {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();

        assert_eq!(config.host, IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
        assert_eq!(config.port, 0);
        assert!(config.cors_enabled);
        assert!(!config.cors_origins.is_empty());
        assert_eq!(config.api_prefix, "/api");
        assert_eq!(config.health_check_path, "/health");
        assert!(config.graceful_shutdown);
    }

    #[test]
    fn test_bind_address() {
        let config = ServerConfig {
            host: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            port: 3030, // 改为3030端口，避免与常见服务冲突
            ..ServerConfig::default()
        };

        assert_eq!(config.bind_address(), "0.0.0.0:3030");
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();

        // 有效配置
        assert!(config.validate().is_ok());

        // 无效端口 - 注释掉这个测试，因为u16类型本身就限制了端口范围
        // config.port = 70000;
        // assert!(config.validate().is_err());
        config.port = 3030;

        // 无效请求大小
        config.max_request_size_bytes = 0;
        assert!(config.validate().is_err());
        config.max_request_size_bytes = 1024;

        // 无效API前缀
        config.api_prefix = "api".to_string();
        assert!(config.validate().is_err());
        config.api_prefix = "/api".to_string();

        // 无效健康检查路径
        config.health_check_path = "health".to_string();
        assert!(config.validate().is_err());
        config.health_check_path = "/health".to_string();

        // 无效关闭超时
        config.shutdown_timeout_seconds = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_environment_specific_server_configs() {
        let dev_config = ServerConfig::development();
        assert!(dev_config.cors_enabled);
        assert!(dev_config.request_logging);
        assert!(dev_config.cors_origins.len() >= 2);

        let prod_config = ServerConfig::production();
        assert!(!prod_config.cors_enabled);
        assert!(!prod_config.request_logging);
        assert!(prod_config.compression_enabled);
        assert_eq!(prod_config.shutdown_timeout_seconds, 60);

        let test_config = ServerConfig::test();
        assert_eq!(test_config.port, 0);
        assert!(!test_config.cors_enabled);
        assert!(!test_config.graceful_shutdown);
    }
}
