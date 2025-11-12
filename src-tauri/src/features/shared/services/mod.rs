/// 共享业务服务层
///
/// 包含所有功能模块共享的业务逻辑
pub mod ai_classification_service;
pub mod conflict_checker;
pub mod recurrence_instantiation_service;

// 重新导出常用类型 - 明确导出，避免通配符
pub use ai_classification_service::AiClassificationService;
pub use conflict_checker::TimeBlockConflictChecker;
pub use recurrence_instantiation_service::RecurrenceInstantiationService;
