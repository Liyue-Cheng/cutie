/// HTTP处理器模块
/// 
/// 本层负责处理HTTP请求，解析参数，调用服务层，并返回HTTP响应。
/// 严格遵循"不包含业务逻辑"的原则，只做HTTP <-> 服务层的翻译工作。

pub mod payloads;
pub mod responses;
pub mod error_handler;
pub mod task_handlers;
pub mod schedule_handlers;
pub mod ordering_handlers;
pub mod time_block_handlers;
pub mod template_handlers;
pub mod area_handlers;

pub use payloads::*;
pub use responses::*;
pub use error_handler::*;
pub use task_handlers::*;
pub use schedule_handlers::*;
pub use ordering_handlers::*;
pub use time_block_handlers::*;
pub use template_handlers::*;
pub use area_handlers::*;
