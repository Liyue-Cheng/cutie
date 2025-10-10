/// 日志标签定义
///
/// 采用三层标签体系：LAYER:FEATURE:COMPONENT
///
/// ## 第一层：架构层（LAYER）
/// - ENDPOINT: HTTP 端点层
/// - SERVICE: 业务服务层
/// - REPOSITORY: 数据仓储层
/// - ASSEMBLER: 装配器层
/// - MIDDLEWARE: 中间件层
/// - INFRA: 基础设施层
/// - STARTUP: 启动层
///
/// ## 第二层：功能模块（FEATURE）
/// - TASKS, TEMPLATES, RECURRENCES, TIME_BLOCKS, AREAS, VIEWS, TRASH, EVENTS
///
/// ## 第三层：具体组件（COMPONENT）
/// - 具体的文件名或操作名

// ==================== 第一层：架构层 ====================

/// HTTP 端点层（endpoints/）
pub const LAYER_ENDPOINT: &str = "ENDPOINT";

/// 业务服务层（shared/services/, *_service.rs）
pub const LAYER_SERVICE: &str = "SERVICE";

/// 数据仓储层（shared/repositories/）
pub const LAYER_REPOSITORY: &str = "REPOSITORY";

/// 装配器层（shared/assembler.rs, shared/assemblers/）
pub const LAYER_ASSEMBLER: &str = "ASSEMBLER";

/// 中间件层（HTTP middleware, tracing layer）
pub const LAYER_MIDDLEWARE: &str = "MIDDLEWARE";

/// 基础设施层（database, config, events）
pub const LAYER_INFRA: &str = "INFRA";

/// 启动层（sidecar, app initialization）
pub const LAYER_STARTUP: &str = "STARTUP";

// ==================== 第二层：功能模块 ====================

/// 任务管理功能
pub const FEATURE_TASKS: &str = "TASKS";

/// 模板系统功能
pub const FEATURE_TEMPLATES: &str = "TEMPLATES";

/// 周期任务功能
pub const FEATURE_RECURRENCES: &str = "RECURRENCES";

/// 时间块功能
pub const FEATURE_TIME_BLOCKS: &str = "TIME_BLOCKS";

/// 区域层级功能
pub const FEATURE_AREAS: &str = "AREAS";

/// 视图查询功能
pub const FEATURE_VIEWS: &str = "VIEWS";

/// 回收站功能
pub const FEATURE_TRASH: &str = "TRASH";

/// 事件系统（SSE）
pub const FEATURE_EVENTS: &str = "EVENTS";

/// 视图偏好设置
pub const FEATURE_VIEW_PREFERENCES: &str = "VIEW_PREFERENCES";

// ==================== 辅助函数 ====================

/// 构建完整的日志标签
///
/// # Examples
///
/// ```
/// use crate::shared::logging::tags::*;
///
/// let tag = build_tag(LAYER_ENDPOINT, FEATURE_TASKS, "create_task");
/// assert_eq!(tag, "ENDPOINT:TASKS:create_task");
/// ```
pub fn build_tag(layer: &str, feature: &str, component: &str) -> String {
    format!("{}:{}:{}", layer, feature, component)
}

/// 构建端点层标签
pub fn endpoint_tag(feature: &str, component: &str) -> String {
    build_tag(LAYER_ENDPOINT, feature, component)
}

/// 构建服务层标签
pub fn service_tag(feature: &str, component: &str) -> String {
    build_tag(LAYER_SERVICE, feature, component)
}

/// 构建仓储层标签
pub fn repository_tag(feature: &str, component: &str) -> String {
    build_tag(LAYER_REPOSITORY, feature, component)
}

/// 构建装配器层标签
pub fn assembler_tag(feature: &str, component: &str) -> String {
    build_tag(LAYER_ASSEMBLER, feature, component)
}

/// 构建中间件层标签
pub fn middleware_tag(component: &str) -> String {
    format!("{}:{}", LAYER_MIDDLEWARE, component)
}

/// 构建基础设施层标签
pub fn infra_tag(component: &str) -> String {
    format!("{}:{}", LAYER_INFRA, component)
}

/// 构建启动层标签
pub fn startup_tag(component: &str) -> String {
    format!("{}:{}", LAYER_STARTUP, component)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_tag() {
        let tag = build_tag(LAYER_ENDPOINT, FEATURE_TASKS, "create_task");
        assert_eq!(tag, "ENDPOINT:TASKS:create_task");
    }

    #[test]
    fn test_endpoint_tag() {
        assert_eq!(
            endpoint_tag(FEATURE_TASKS, "create_task"),
            "ENDPOINT:TASKS:create_task"
        );
    }

    #[test]
    fn test_repository_tag() {
        assert_eq!(
            repository_tag(FEATURE_TASKS, "TaskRepository"),
            "REPOSITORY:TASKS:TaskRepository"
        );
    }

    #[test]
    fn test_middleware_tag() {
        assert_eq!(middleware_tag("request_id"), "MIDDLEWARE:request_id");
    }
}
