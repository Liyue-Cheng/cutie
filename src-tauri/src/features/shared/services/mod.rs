/// 共享业务服务层
/// 
/// 包含所有功能模块共享的业务逻辑

pub mod ai_classification_service;
pub mod conflict_checker;
pub mod recurrence_instantiation_service;

// 重新导出常用类型
pub use ai_classification_service::*;
pub use conflict_checker::*;
pub use recurrence_instantiation_service::*;
