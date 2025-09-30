/// 外部依赖抽象层（端口）
///
/// 本层定义了一系列抽象接口（Rust中的Trait），用于隔离核心业务逻辑与所有不确定的、
/// 具有副作用的外部依赖（如系统时钟、文件系统、网络API、随机数生成等）。
/// 本层只包含接口定义和基础适配器实现。

pub mod clock;
pub mod id_generator;

pub use clock::*;
pub use id_generator::*;
