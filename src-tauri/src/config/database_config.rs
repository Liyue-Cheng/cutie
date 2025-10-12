use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

use crate::infra::core::AppError;

/// 数据库配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库文件名
    pub filename: String,

    /// 连接池最大连接数
    pub max_connections: u32,

    /// 连接池最小连接数
    pub min_connections: u32,

    /// 连接超时时间（秒）
    pub connect_timeout_seconds: u64,

    /// 空闲超时时间（秒）
    pub idle_timeout_seconds: u64,

    /// 是否启用WAL模式
    pub wal_mode: bool,

    /// 是否启用外键约束
    pub foreign_keys: bool,

    /// 同步模式
    pub synchronous: SynchronousMode,

    /// 缓存大小（KB）
    pub cache_size_kb: i32,

    /// 是否启用查询日志
    pub query_logging: bool,

    /// 慢查询阈值（毫秒）
    pub slow_query_threshold_ms: u64,

    /// 是否自动运行迁移
    pub auto_migrate: bool,
}

/// SQLite同步模式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SynchronousMode {
    /// 完全同步（最安全，最慢）
    Full,
    /// 正常同步（平衡）
    Normal,
    /// 关闭同步（最快，有风险）
    Off,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            filename: "cutie.db".to_string(),
            max_connections: 5, // ✅ 降低到 5，配合应用层写串行化，减少无意义并发
            min_connections: 1,
            connect_timeout_seconds: 30,
            idle_timeout_seconds: 600, // 10分钟
            wal_mode: true,
            foreign_keys: true,
            synchronous: SynchronousMode::Normal,
            cache_size_kb: 10240, // 10MB
            query_logging: false,
            slow_query_threshold_ms: 1000, // 1秒
            auto_migrate: true,
        }
    }
}

impl DatabaseConfig {
    /// 从环境变量加载数据库配置
    pub fn from_env(_data_dir: &PathBuf) -> Result<Self, AppError> {
        let mut config = Self::default();

        // 加载数据库文件名
        if let Ok(filename) = env::var("CUTIE_DB_FILENAME") {
            config.filename = filename;
        }

        // 加载连接池配置
        if let Ok(max_conn_str) = env::var("CUTIE_DB_MAX_CONNECTIONS") {
            config.max_connections = max_conn_str
                .parse()
                .map_err(|_| AppError::configuration_error("Invalid DB max_connections value"))?;
        }

        if let Ok(min_conn_str) = env::var("CUTIE_DB_MIN_CONNECTIONS") {
            config.min_connections = min_conn_str
                .parse()
                .map_err(|_| AppError::configuration_error("Invalid DB min_connections value"))?;
        }

        // 加载超时配置
        if let Ok(connect_timeout_str) = env::var("CUTIE_DB_CONNECT_TIMEOUT") {
            config.connect_timeout_seconds = connect_timeout_str
                .parse()
                .map_err(|_| AppError::configuration_error("Invalid DB connect_timeout value"))?;
        }

        if let Ok(idle_timeout_str) = env::var("CUTIE_DB_IDLE_TIMEOUT") {
            config.idle_timeout_seconds = idle_timeout_str
                .parse()
                .map_err(|_| AppError::configuration_error("Invalid DB idle_timeout value"))?;
        }

        // 加载SQLite特定配置
        if let Ok(wal_mode_str) = env::var("CUTIE_DB_WAL_MODE") {
            config.wal_mode = wal_mode_str.to_lowercase() == "true";
        }

        if let Ok(foreign_keys_str) = env::var("CUTIE_DB_FOREIGN_KEYS") {
            config.foreign_keys = foreign_keys_str.to_lowercase() == "true";
        }

        if let Ok(sync_mode_str) = env::var("CUTIE_DB_SYNCHRONOUS") {
            config.synchronous = match sync_mode_str.to_uppercase().as_str() {
                "FULL" => SynchronousMode::Full,
                "NORMAL" => SynchronousMode::Normal,
                "OFF" => SynchronousMode::Off,
                _ => {
                    return Err(AppError::configuration_error(format!(
                        "Invalid synchronous mode: {}",
                        sync_mode_str
                    )))
                }
            };
        }

        if let Ok(cache_size_str) = env::var("CUTIE_DB_CACHE_SIZE_KB") {
            config.cache_size_kb = cache_size_str
                .parse()
                .map_err(|_| AppError::configuration_error("Invalid DB cache_size value"))?;
        }

        // 加载日志配置
        if let Ok(query_logging_str) = env::var("CUTIE_DB_QUERY_LOGGING") {
            config.query_logging = query_logging_str.to_lowercase() == "true";
        }

        if let Ok(slow_query_str) = env::var("CUTIE_DB_SLOW_QUERY_THRESHOLD_MS") {
            config.slow_query_threshold_ms = slow_query_str.parse().map_err(|_| {
                AppError::configuration_error("Invalid DB slow_query_threshold value")
            })?;
        }

        // 加载迁移配置
        if let Ok(auto_migrate_str) = env::var("CUTIE_DB_AUTO_MIGRATE") {
            config.auto_migrate = auto_migrate_str.to_lowercase() == "true";
        }

        Ok(config)
    }

    /// 构建SQLite连接字符串
    pub fn connection_string(&self, db_path: &PathBuf) -> String {
        let mut params = Vec::new();

        // 添加WAL模式
        if self.wal_mode {
            params.push("journal_mode=WAL".to_string());
        }

        // 添加外键约束
        if self.foreign_keys {
            params.push("foreign_keys=ON".to_string());
        } else {
            params.push("foreign_keys=OFF".to_string());
        }

        // 添加同步模式
        params.push(format!(
            "synchronous={}",
            self.synchronous.as_pragma_value()
        ));

        // 添加缓存大小
        params.push(format!("cache_size={}", self.cache_size_kb));

        // 构建完整的连接字符串
        let query_string = params.join("&");
        format!("sqlite://{}?{}", db_path.to_string_lossy(), query_string)
    }

    /// 验证数据库配置
    pub fn validate(&self) -> Result<(), AppError> {
        if self.filename.is_empty() {
            return Err(AppError::configuration_error(
                "Database filename cannot be empty",
            ));
        }

        if self.max_connections == 0 {
            return Err(AppError::configuration_error(
                "Database max_connections must be greater than 0",
            ));
        }

        if self.min_connections > self.max_connections {
            return Err(AppError::configuration_error(
                "Database min_connections cannot be greater than max_connections",
            ));
        }

        if self.connect_timeout_seconds == 0 {
            return Err(AppError::configuration_error(
                "Database connect_timeout_seconds must be greater than 0",
            ));
        }

        if self.cache_size_kb <= 0 {
            return Err(AppError::configuration_error(
                "Database cache_size_kb must be greater than 0",
            ));
        }

        Ok(())
    }

    /// 获取开发环境的配置
    pub fn development() -> Self {
        Self {
            query_logging: true,
            slow_query_threshold_ms: 100,
            auto_migrate: true,
            ..Self::default()
        }
    }

    /// 获取生产环境的配置
    pub fn production() -> Self {
        Self {
            max_connections: 20,
            cache_size_kb: 20480, // 20MB
            query_logging: false,
            slow_query_threshold_ms: 2000,
            synchronous: SynchronousMode::Full,
            auto_migrate: false, // 生产环境手动控制迁移
            ..Self::default()
        }
    }

    /// 获取测试环境的配置
    pub fn test() -> Self {
        Self {
            filename: ":memory:".to_string(), // 内存数据库
            max_connections: 1,
            min_connections: 1,
            query_logging: true,
            auto_migrate: true,
            ..Self::default()
        }
    }
}

impl SynchronousMode {
    /// 转换为SQLite pragma值
    pub fn as_pragma_value(&self) -> &'static str {
        match self {
            SynchronousMode::Full => "FULL",
            SynchronousMode::Normal => "NORMAL",
            SynchronousMode::Off => "OFF",
        }
    }
}
