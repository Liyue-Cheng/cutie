/// Area概念的所有数据结构
///
/// 包含：
/// - Area核心模型
/// - Area相关的DTOs
pub mod model;
pub mod request_dtos;
pub mod response_dtos;

// 重新导出所有类型
pub use model::*;
pub use request_dtos::*;
pub use response_dtos::*;
