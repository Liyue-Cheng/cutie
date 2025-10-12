/// 共享验证器层
/// 
/// 包含所有功能模块共享的验证逻辑
/// 
/// # 设计原则
/// 
/// 1. **单一职责**：每个验证器只负责一种实体的验证
/// 2. **可复用**：验证逻辑可以在不同的 endpoint 中复用
/// 3. **一致性**：所有验证器都返回统一的错误格式
/// 4. **可测试**：验证逻辑独立，易于单元测试

pub mod task_validator;
pub mod time_block_validator;

// 单元测试模块
#[cfg(test)]
mod task_validator_tests;
#[cfg(test)]
mod time_block_validator_tests;

// 重新导出常用类型
pub use task_validator::TaskValidator;
pub use time_block_validator::TimeBlockValidator;
