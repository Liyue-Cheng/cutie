use thiserror::Error;

/// 数据库错误类型
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    ConnectionError(#[from] sqlx::Error),

    #[error("Entity not found: {entity_type} with id {entity_id}")]
    NotFound {
        entity_type: String,
        entity_id: String,
    },

    #[error("Constraint violation: {message}")]
    ConstraintViolation { message: String },

    #[error("Transaction failed: {message}")]
    TransactionFailed { message: String },

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Query error: {0}")]
    QueryError(String),
}

/// 验证错误详情
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationError {
    /// 字段名
    pub field: String,
    /// 错误消息
    pub message: String,
    /// 错误代码
    pub code: String,
}

impl ValidationError {
    pub fn new(
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: code.into(),
        }
    }
}

/// 统一应用错误类型
///
/// 这是在Cutie后端业务逻辑执行过程中，可能发生的、所有可预见的、
/// 具有明确业务含义的错误的集合。它是服务层向路由层传递失败信息的标准方式。
///
/// ## 不变量
/// AppError本身不包含任何关于HTTP状态码的知识。它只负责描述业务层面的失败。
/// 将AppError映射到具体的HTTP状态码是路由层的职责。
#[derive(Debug, Error)]
pub enum AppError {
    /// 数据库操作失败
    ///
    /// **预期行为:** 表示一个源自仓库层的、无法恢复的数据库操作失败
    /// **后置条件:** 包含原始的DbError，以便上层进行日志记录
    #[error("Database error: {0}")]
    DatabaseError(#[from] DbError),

    /// 实体未找到
    ///
    /// **预期行为:** 表示一个基于唯一标识符的查找操作没有找到对应的实体
    /// **后置条件:** entity_type和entity_id清晰地指明了哪个实体未被找到
    #[error("Not found: {entity_type} with id {entity_id}")]
    NotFound {
        entity_type: String,
        entity_id: String,
    },

    /// 验证失败
    ///
    /// **预期行为:** 表示一个或多个输入参数未能通过业务规则的验证
    /// **后置条件:** Vec<ValidationError>详细列出了所有验证失败的字段及其原因
    #[error("Validation failed: {0:?}")]
    ValidationFailed(Vec<ValidationError>),

    /// 权限被拒绝
    ///
    /// **预期行为:** 表示当前操作因权限不足而被拒绝
    #[error("Permission denied")]
    PermissionDenied,

    /// 冲突
    ///
    /// **预期行为:** 表示当前操作因与系统现有状态发生冲突而无法完成
    /// （例如，唯一性约束冲突）
    #[error("Conflict: {message}")]
    Conflict { message: String },

    /// 外部依赖失败
    ///
    /// **预期行为:** 表示一个对外部依赖（如AI服务）的调用失败
    #[error("External service '{service_name}' failed: {error_message}")]
    ExternalDependencyFailed {
        service_name: String,
        error_message: String,
    },

    /// 未指定的内部错误
    ///
    /// **预期行为:** 表示一个在业务逻辑中发生的、未被其他变体覆盖的内部错误
    #[error("Internal error occurred")]
    UnspecifiedInternalError,

    /// 配置错误
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },

    /// 序列化/反序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// 字符串错误（通用）
    #[error("Error: {0}")]
    StringError(String),
}

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::StringError(s)
    }
}

impl From<SortOrderError> for AppError {
    fn from(err: SortOrderError) -> Self {
        AppError::StringError(err.to_string())
    }
}

impl AppError {
    /// 创建未找到错误
    pub fn not_found(entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self::NotFound {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
        }
    }

    /// 创建冲突错误
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// 创建验证错误（单个）
    pub fn validation_error(
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self::ValidationFailed(vec![ValidationError::new(field, message, code)])
    }

    /// 创建外部依赖错误
    pub fn external_dependency_failed(
        service_name: impl Into<String>,
        error_message: impl Into<String>,
    ) -> Self {
        Self::ExternalDependencyFailed {
            service_name: service_name.into(),
            error_message: error_message.into(),
        }
    }

    /// 创建配置错误
    pub fn configuration_error(message: impl Into<String>) -> Self {
        Self::ConfigurationError {
            message: message.into(),
        }
    }
}

/// 应用结果类型别名
pub type AppResult<T> = Result<T, AppError>;

/// 排序相关的错误类型
#[derive(Debug, Error)]
pub enum SortOrderError {
    #[error("Invalid sort order format: {0}")]
    InvalidFormat(String),
    #[error("Cannot generate rank between identical values: {0}")]
    IdenticalValues(String),
    #[error("Invalid rank order: prev '{prev}' should be less than next '{next}'")]
    InvalidOrder { prev: String, next: String },
    #[error("LexoRank error: {0}")]
    LexoRankError(String),
}

/// 数据库结果类型别名
pub type DbResult<T> = Result<T, DbError>;

/// 排序结果类型别名
pub type SortResult<T> = Result<T, SortOrderError>;
