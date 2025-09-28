pub mod area;
pub mod enums;
pub mod ordering;
/// 核心领域模型层
///
/// 本层定义了Cutie业务世界中所有核心实体的静态数据结构。
/// 它们是系统中"通用语言"的基础，是所有上层逻辑操作的对象。
/// 本层代码不包含任何业务逻辑，只有纯粹的数据定义。
pub mod task;
pub mod task_schedule;
pub mod template;
pub mod time_block;

pub use area::Area;
pub use enums::*;
pub use ordering::Ordering;
pub use task::Task;
pub use task_schedule::TaskSchedule;
pub use template::Template;
pub use time_block::TimeBlock;
