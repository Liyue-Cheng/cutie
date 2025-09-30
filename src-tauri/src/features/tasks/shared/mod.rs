/// Tasks 功能模块的共享基础设施
///
/// 包含：
/// - assembler: 装配器，负责将实体转换为 DTO
/// - 其他共享工具（按需添加）
///
/// 注意：
/// - DTOs 在 entities 模块中定义
/// - 验证逻辑在各个 endpoint 的 validation 模块中
pub mod assembler;

// 重新导出常用类型
pub use assembler::TaskAssembler;
