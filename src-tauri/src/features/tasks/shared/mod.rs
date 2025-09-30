/// Tasks功能模块的共享基础设施
///
/// 包含所有API端点共享的组件：
/// - validation: 验证逻辑
///
/// 注意：DTOs已移至entities模块，repository层已移动到legacy目录
pub mod validation;

pub use validation::*;
