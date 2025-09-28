/// 通用工具与错误处理层
///
/// 本层提供被整个应用程序复用的基础工具和统一的错误处理模型。
/// 它旨在减少代码重复，并为所有其他层级提供一套标准的、可预测的、
/// 用于表示失败和执行通用计算的机制。
pub mod error;
pub mod logger;
pub mod utils;

pub use error::*;
pub use logger::*;
pub use utils::*;
