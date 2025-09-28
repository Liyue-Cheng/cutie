use async_trait::async_trait;
use uuid::Uuid;

use super::Transaction;
use crate::common::error::DbError;
use crate::core::models::Area;

/// 领域仓库接口定义
///
/// **预期行为简介:** 提供对Area实体的所有持久化操作
#[async_trait]
pub trait AreaRepository: Send + Sync {
    /// 创建新领域
    ///
    /// **预期行为简介:** 在数据库中插入一个新的领域记录
    /// **输入输出规范:**
    /// - **前置条件:** area对象必须完整且有效，name不能为空
    /// - **后置条件:** 成功时返回插入的Area对象
    /// **边界情况:** 如果parent_area_id指向不存在的领域，返回DbError::ConstraintViolation
    /// **预期副作用:** 向areas表插入一条新记录
    async fn create(&self, tx: &mut Transaction<'_>, area: &Area) -> Result<Area, DbError>;

    /// 更新领域
    ///
    /// **预期行为简介:** 更新指定领域的信息
    /// **输入输出规范:**
    /// - **前置条件:** area.id必须存在，area对象必须完整
    /// - **后置条件:** 返回更新后的Area对象
    /// **边界情况:** 如果area.id不存在，返回DbError::NotFound
    /// **预期副作用:** 修改areas表中的一条记录，updated_at字段被更新
    async fn update(&self, tx: &mut Transaction<'_>, area: &Area) -> Result<Area, DbError>;

    /// 根据ID查找领域
    ///
    /// **预期行为简介:** 根据UUID查找单个领域
    /// **输入输出规范:**
    /// - **前置条件:** area_id必须是有效的UUID
    /// - **后置条件:** 如果找到返回Some(Area)，否则返回None
    /// **边界情况:** 如果领域被逻辑删除，返回None
    /// **预期副作用:** 无
    async fn find_by_id(&self, area_id: Uuid) -> Result<Option<Area>, DbError>;

    /// 查找所有领域
    ///
    /// **预期行为简介:** 查找所有未被删除的领域
    /// **输入输出规范:**
    /// - **前置条件:** 无
    /// - **后置条件:** 返回所有Area记录的列表
    /// **边界情况:** 如果没有领域，返回空列表
    /// **预期副作用:** 无
    async fn find_all(&self) -> Result<Vec<Area>, DbError>;

    /// 查找根领域
    ///
    /// **预期行为简介:** 查找所有没有父领域的顶级领域
    /// **输入输出规范:**
    /// - **前置条件:** 无
    /// - **后置条件:** 返回所有parent_area_id为NULL的Area记录
    /// **边界情况:** 如果没有根领域，返回空列表
    /// **预期副作用:** 无
    async fn find_root_areas(&self) -> Result<Vec<Area>, DbError>;

    /// 查找子领域
    ///
    /// **预期行为简介:** 查找指定领域的所有直接子领域
    /// **输入输出规范:**
    /// - **前置条件:** parent_id必须是有效的UUID
    /// - **后置条件:** 返回所有parent_area_id等于parent_id的Area记录
    /// **边界情况:** 如果没有子领域，返回空列表
    /// **预期副作用:** 无
    async fn find_children(&self, parent_id: Uuid) -> Result<Vec<Area>, DbError>;

    /// 查找领域的所有后代
    ///
    /// **预期行为简介:** 递归查找指定领域的所有后代领域
    /// **输入输出规范:**
    /// - **前置条件:** parent_id必须是有效的UUID
    /// - **后置条件:** 返回指定领域下的所有后代Area记录
    /// **边界情况:** 如果没有后代，返回空列表
    /// **预期副作用:** 无
    async fn find_descendants(&self, parent_id: Uuid) -> Result<Vec<Area>, DbError>;

    /// 查找领域路径
    ///
    /// **预期行为简介:** 查找从根领域到指定领域的完整路径
    /// **输入输出规范:**
    /// - **前置条件:** area_id必须是有效的UUID
    /// - **后置条件:** 返回从根到指定领域的Area记录列表，按层级顺序排列
    /// **边界情况:** 如果是根领域，返回只包含自己的列表
    /// **预期副作用:** 无
    async fn find_path_to_root(&self, area_id: Uuid) -> Result<Vec<Area>, DbError>;

    /// 软删除领域
    ///
    /// **预期行为简介:** 将领域标记为已删除
    /// **输入输出规范:**
    /// - **前置条件:** area_id必须存在
    /// - **后置条件:** 领域的is_deleted字段被设置为true
    /// **边界情况:** 如果领域有子领域或关联的任务，返回DbError::Conflict
    /// **预期副作用:** 更新areas表中的is_deleted和updated_at字段
    async fn soft_delete(&self, tx: &mut Transaction<'_>, area_id: Uuid) -> Result<(), DbError>;

    /// 恢复已删除的领域
    ///
    /// **预期行为简介:** 将软删除的领域恢复为可见状态
    async fn restore(&self, tx: &mut Transaction<'_>, area_id: Uuid) -> Result<Area, DbError>;

    /// 检查领域是否有子领域
    ///
    /// **预期行为简介:** 检查指定领域是否有直接子领域
    async fn has_children(&self, area_id: Uuid) -> Result<bool, DbError>;

    /// 检查领域是否被使用
    ///
    /// **预期行为简介:** 检查领域是否被任务、项目或时间块使用
    async fn is_used(&self, area_id: Uuid) -> Result<bool, DbError>;

    /// 移动领域到新父级
    ///
    /// **预期行为简介:** 将领域移动到新的父领域下
    /// **输入输出规范:**
    /// - **前置条件:** area_id必须存在，new_parent_id如果非空必须存在且不能造成循环
    /// - **后置条件:** 领域的parent_area_id被更新
    /// **边界情况:** 如果会造成循环引用，返回DbError::Conflict
    /// **预期副作用:** 更新areas表中的parent_area_id和updated_at字段
    async fn move_to_parent(
        &self,
        tx: &mut Transaction<'_>,
        area_id: Uuid,
        new_parent_id: Option<Uuid>,
    ) -> Result<Area, DbError>;

    /// 验证领域层级结构
    ///
    /// **预期行为简介:** 验证将area_id设置为parent_id的子级是否会造成循环引用
    async fn would_create_cycle(
        &self,
        area_id: Uuid,
        potential_parent_id: Uuid,
    ) -> Result<bool, DbError>;

    /// 统计领域使用情况
    ///
    /// **预期行为简介:** 统计每个领域被使用的次数
    async fn count_usage(&self) -> Result<Vec<AreaUsageCount>, DbError>;
}

/// 领域使用统计结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct AreaUsageCount {
    pub area_id: Uuid,
    pub area_name: String,
    pub task_count: i64,
    pub time_block_count: i64,
    pub project_count: i64,
    pub total_usage: i64,
}
