/// 日志便利宏
///
/// 提供简化的日志宏，自动注入分层标签

/// 端点层日志宏
///
/// # Examples
///
/// ```ignore
/// log_endpoint!(info, feature = FEATURE_TASKS, component = "create_task",
///     task_id = %task_id,
///     "Task created successfully"
/// );
/// ```
#[macro_export]
macro_rules! log_endpoint {
    // info 级别
    (info, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::info!(
            target: &$crate::infra::logging::tags::endpoint_tag($feature, $component),
            $($arg)*
        )
    };
    // debug 级别
    (debug, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::debug!(
            target: &$crate::infra::logging::tags::endpoint_tag($feature, $component),
            $($arg)*
        )
    };
    // warn 级别
    (warn, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::warn!(
            target: &$crate::infra::logging::tags::endpoint_tag($feature, $component),
            $($arg)*
        )
    };
    // error 级别
    (error, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::error!(
            target: &$crate::infra::logging::tags::endpoint_tag($feature, $component),
            $($arg)*
        )
    };
    // trace 级别
    (trace, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::trace!(
            target: &$crate::infra::logging::tags::endpoint_tag($feature, $component),
            $($arg)*
        )
    };
}

/// 服务层日志宏
#[macro_export]
macro_rules! log_service {
    (info, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::info!(
            target: &$crate::infra::logging::tags::service_tag($feature, $component),
            $($arg)*
        )
    };
    (debug, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::debug!(
            target: &$crate::infra::logging::tags::service_tag($feature, $component),
            $($arg)*
        )
    };
    (warn, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::warn!(
            target: &$crate::infra::logging::tags::service_tag($feature, $component),
            $($arg)*
        )
    };
    (error, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::error!(
            target: &$crate::infra::logging::tags::service_tag($feature, $component),
            $($arg)*
        )
    };
    (trace, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::trace!(
            target: &$crate::infra::logging::tags::service_tag($feature, $component),
            $($arg)*
        )
    };
}

/// 仓储层日志宏
#[macro_export]
macro_rules! log_repository {
    (info, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::info!(
            target: &$crate::infra::logging::tags::repository_tag($feature, $component),
            $($arg)*
        )
    };
    (debug, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::debug!(
            target: &$crate::infra::logging::tags::repository_tag($feature, $component),
            $($arg)*
        )
    };
    (warn, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::warn!(
            target: &$crate::infra::logging::tags::repository_tag($feature, $component),
            $($arg)*
        )
    };
    (error, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::error!(
            target: &$crate::infra::logging::tags::repository_tag($feature, $component),
            $($arg)*
        )
    };
    (trace, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::trace!(
            target: &$crate::infra::logging::tags::repository_tag($feature, $component),
            $($arg)*
        )
    };
}

/// 装配器层日志宏
#[macro_export]
macro_rules! log_assembler {
    (info, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::info!(
            target: &$crate::infra::logging::tags::assembler_tag($feature, $component),
            $($arg)*
        )
    };
    (debug, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::debug!(
            target: &$crate::infra::logging::tags::assembler_tag($feature, $component),
            $($arg)*
        )
    };
    (warn, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::warn!(
            target: &$crate::infra::logging::tags::assembler_tag($feature, $component),
            $($arg)*
        )
    };
    (error, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::error!(
            target: &$crate::infra::logging::tags::assembler_tag($feature, $component),
            $($arg)*
        )
    };
    (trace, feature = $feature:expr, component = $component:expr, $($arg:tt)*) => {
        tracing::trace!(
            target: &$crate::infra::logging::tags::assembler_tag($feature, $component),
            $($arg)*
        )
    };
}

/// 中间件层日志宏
#[macro_export]
macro_rules! log_middleware {
    (info, component = $component:expr, $($arg:tt)*) => {
        tracing::info!(
            target: &$crate::infra::logging::tags::middleware_tag($component),
            $($arg)*
        )
    };
    (debug, component = $component:expr, $($arg:tt)*) => {
        tracing::debug!(
            target: &$crate::infra::logging::tags::middleware_tag($component),
            $($arg)*
        )
    };
    (warn, component = $component:expr, $($arg:tt)*) => {
        tracing::warn!(
            target: &$crate::infra::logging::tags::middleware_tag($component),
            $($arg)*
        )
    };
    (error, component = $component:expr, $($arg:tt)*) => {
        tracing::error!(
            target: &$crate::infra::logging::tags::middleware_tag($component),
            $($arg)*
        )
    };
}

/// 基础设施层日志宏
#[macro_export]
macro_rules! log_infra {
    (info, component = $component:expr, $($arg:tt)*) => {
        tracing::info!(
            target: &$crate::infra::logging::tags::infra_tag($component),
            $($arg)*
        )
    };
    (debug, component = $component:expr, $($arg:tt)*) => {
        tracing::debug!(
            target: &$crate::infra::logging::tags::infra_tag($component),
            $($arg)*
        )
    };
    (warn, component = $component:expr, $($arg:tt)*) => {
        tracing::warn!(
            target: &$crate::infra::logging::tags::infra_tag($component),
            $($arg)*
        )
    };
    (error, component = $component:expr, $($arg:tt)*) => {
        tracing::error!(
            target: &$crate::infra::logging::tags::infra_tag($component),
            $($arg)*
        )
    };
}

/// 启动层日志宏
#[macro_export]
macro_rules! log_startup {
    (info, component = $component:expr, $($arg:tt)*) => {
        tracing::info!(
            target: &$crate::infra::logging::tags::startup_tag($component),
            $($arg)*
        )
    };
    (debug, component = $component:expr, $($arg:tt)*) => {
        tracing::debug!(
            target: &$crate::infra::logging::tags::startup_tag($component),
            $($arg)*
        )
    };
    (warn, component = $component:expr, $($arg:tt)*) => {
        tracing::warn!(
            target: &$crate::infra::logging::tags::startup_tag($component),
            $($arg)*
        )
    };
    (error, component = $component:expr, $($arg:tt)*) => {
        tracing::error!(
            target: &$crate::infra::logging::tags::startup_tag($component),
            $($arg)*
        )
    };
}
