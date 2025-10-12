/// 共享组装器层
/// 
/// 包含所有功能模块共享的数据组装逻辑

pub mod task_assembler;
pub mod linked_task_assembler;
pub mod task_card_assembler;
pub mod time_block_assembler;

// 重新导出常用类型 - 明确导出，避免通配符
pub use task_assembler::TaskAssembler;
pub use linked_task_assembler::LinkedTaskAssembler;
pub use task_card_assembler::ViewTaskCardAssembler;
pub use time_block_assembler::TimeBlockAssembler;
