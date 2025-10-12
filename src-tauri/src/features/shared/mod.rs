/// 跨功能模块共享基础设施
///
/// 包含所有功能模块都可能用到的通用工具和仓库
///
/// # 架构设计
///
/// 本模块采用分层架构：
/// - `repositories`: 数据访问层，负责与数据库交互
/// - `assemblers`: 数据组装层，负责将数据库记录转换为领域对象
/// - `services`: 业务服务层，负责跨功能的业务逻辑
/// - `validators`: 验证器层，负责数据验证（预留）
///
/// # 导出策略
///
/// 1. **双层导出**：既保留命名空间（如 `repositories::`），又在顶层重新导出
/// 2. **灵活使用**：开发者可根据场景选择使用方式
///    - 简单场景：`use shared::TaskRepository`
///    - 需要语义：`use shared::repositories::TaskRepository`
/// 3. **按职责分组**：相同职责的类型放在一起
// 声明子模块（保留命名空间访问）
pub mod assemblers;
pub mod repositories;
pub mod services;
pub mod validators;

// ==================== 数据访问层（Repositories）====================
// 所有数据仓库类型，按字母顺序排列

pub use repositories::{
    AreaRepository, TaskRecurrenceLinkRepository, TaskRecurrenceRepository, TaskRepository,
    TaskScheduleRepository, TaskTimeBlockLinkRepository, TimeBlockRepository, TransactionHelper,
};

// ==================== 数据组装层（Assemblers）====================
// 负责将数据库记录组装成领域对象

pub use assemblers::{
    LinkedTaskAssembler, TaskAssembler, TimeBlockAssembler, ViewTaskCardAssembler,
};

// ==================== 业务服务层（Services）====================
// 跨功能模块的业务逻辑服务

pub use services::{
    AiClassificationService, RecurrenceInstantiationService, TimeBlockConflictChecker,
};

// ==================== 验证器层（Validators）====================
// 数据验证逻辑

pub use validators::{
    TaskValidator, TimeBlockValidator,
};

// ==================== Repository Traits ====================
// Repository 抽象接口

pub use repositories::{
    Repository, QueryableRepository, BatchRepository,
};
