pub mod area;
pub mod ordering;
pub mod schedule;
/// 实体层 (Entities Layer)
///
/// 本层统一管理所有数据结构，按纯业务概念组织：
/// - 每个概念包含其核心模型、DTOs、枚举和值对象
/// - 不包含HTTP响应结构（应在shared/http中）
/// - 不使用shared等通用命名
// 按业务概念组织的实体
pub mod task;
pub mod template;
pub mod time_block;

// 重新导出所有公共类型
pub use area::*;
pub use ordering::*;
pub use schedule::*;
pub use task::*;
pub use template::*;
pub use time_block::*;
