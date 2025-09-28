use async_trait::async_trait;
use uuid::Uuid;

use super::Transaction;
use crate::common::error::DbError;
use crate::core::models::Template;

/// 模板仓库接口定义
///
/// **预期行为简介:** 提供对Template实体的所有持久化操作
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    /// 创建新模板
    ///
    /// **预期行为简介:** 在数据库中插入一个新的模板记录
    /// **输入输出规范:**
    /// - **前置条件:** template对象必须完整且有效，name和title_template不能为空
    /// - **后置条件:** 成功时返回插入的Template对象
    /// **边界情况:** 如果template.id已存在，返回DbError::ConstraintViolation
    /// **预期副作用:** 向templates表插入一条新记录
    async fn create(
        &self,
        tx: &mut Transaction<'_>,
        template: &Template,
    ) -> Result<Template, DbError>;

    /// 更新模板
    ///
    /// **预期行为简介:** 更新指定模板的信息
    /// **输入输出规范:**
    /// - **前置条件:** template.id必须存在，template对象必须完整
    /// - **后置条件:** 返回更新后的Template对象
    /// **边界情况:** 如果template.id不存在，返回DbError::NotFound
    /// **预期副作用:** 修改templates表中的一条记录，updated_at字段被更新
    async fn update(
        &self,
        tx: &mut Transaction<'_>,
        template: &Template,
    ) -> Result<Template, DbError>;

    /// 根据ID查找模板
    ///
    /// **预期行为简介:** 根据UUID查找单个模板
    /// **输入输出规范:**
    /// - **前置条件:** template_id必须是有效的UUID
    /// - **后置条件:** 如果找到返回Some(Template)，否则返回None
    /// **边界情况:** 如果模板被逻辑删除，返回None
    /// **预期副作用:** 无
    async fn find_by_id(&self, template_id: Uuid) -> Result<Option<Template>, DbError>;

    /// 查找所有模板
    ///
    /// **预期行为简介:** 查找所有未被删除的模板
    /// **输入输出规范:**
    /// - **前置条件:** 无
    /// - **后置条件:** 返回所有Template记录的列表，按名称排序
    /// **边界情况:** 如果没有模板，返回空列表
    /// **预期副作用:** 无
    async fn find_all(&self) -> Result<Vec<Template>, DbError>;

    /// 根据名称搜索模板
    ///
    /// **预期行为简介:** 根据模板名称进行模糊搜索
    /// **输入输出规范:**
    /// - **前置条件:** name_query不能为空
    /// - **后置条件:** 返回名称包含查询字符串的Template记录列表
    /// **边界情况:** 如果没有匹配的模板，返回空列表
    /// **预期副作用:** 无
    async fn search_by_name(&self, name_query: &str) -> Result<Vec<Template>, DbError>;

    /// 根据领域查找模板
    ///
    /// **预期行为简介:** 查找属于指定领域的所有模板
    /// **输入输出规范:**
    /// - **前置条件:** area_id必须是有效的UUID
    /// - **后置条件:** 返回area_id匹配的Template记录列表
    /// **边界情况:** 如果没有该领域的模板，返回空列表
    /// **预期副作用:** 无
    async fn find_by_area_id(&self, area_id: Uuid) -> Result<Vec<Template>, DbError>;

    /// 查找包含特定变量的模板
    ///
    /// **预期行为简介:** 查找模板内容中包含指定变量的模板
    /// **输入输出规范:**
    /// - **前置条件:** variable_name不能为空
    /// - **后置条件:** 返回包含{{variable_name}}的Template记录列表
    /// **边界情况:** 如果没有包含该变量的模板，返回空列表
    /// **预期副作用:** 无
    async fn find_containing_variable(&self, variable_name: &str)
        -> Result<Vec<Template>, DbError>;

    /// 软删除模板
    ///
    /// **预期行为简介:** 将模板标记为已删除
    /// **输入输出规范:**
    /// - **前置条件:** template_id必须存在
    /// - **后置条件:** 模板的is_deleted字段被设置为true
    /// **边界情况:** 如果模板已被删除，幂等地返回成功
    /// **预期副作用:** 更新templates表中的is_deleted和updated_at字段
    async fn soft_delete(&self, tx: &mut Transaction<'_>, template_id: Uuid)
        -> Result<(), DbError>;

    /// 恢复已删除的模板
    ///
    /// **预期行为简介:** 将软删除的模板恢复为可见状态
    async fn restore(
        &self,
        tx: &mut Transaction<'_>,
        template_id: Uuid,
    ) -> Result<Template, DbError>;

    /// 克隆模板
    ///
    /// **预期行为简介:** 基于现有模板创建一个副本
    /// **输入输出规范:**
    /// - **前置条件:** template_id必须存在，new_name不能为空且不能与现有模板重名
    /// - **后置条件:** 创建一个新的模板，内容与原模板相同但名称不同
    /// **预期副作用:** 向templates表插入一条新记录
    async fn clone_template(
        &self,
        tx: &mut Transaction<'_>,
        template_id: Uuid,
        new_name: String,
    ) -> Result<Template, DbError>;

    /// 检查模板名称是否可用
    ///
    /// **预期行为简介:** 检查指定名称是否已被其他模板使用
    async fn is_name_available(
        &self,
        name: &str,
        exclude_id: Option<Uuid>,
    ) -> Result<bool, DbError>;

    /// 获取模板使用统计
    ///
    /// **预期行为简介:** 统计每个模板被用于创建任务的次数
    async fn get_usage_stats(&self) -> Result<Vec<TemplateUsageStats>, DbError>;

    /// 批量删除模板
    ///
    /// **预期行为简介:** 批量软删除多个模板
    async fn batch_soft_delete(
        &self,
        tx: &mut Transaction<'_>,
        template_ids: &[Uuid],
    ) -> Result<(), DbError>;

    /// 导出模板数据
    ///
    /// **预期行为简介:** 导出模板为可序列化的格式，用于备份或迁移
    async fn export_templates(
        &self,
        template_ids: Option<&[Uuid]>,
    ) -> Result<Vec<Template>, DbError>;

    /// 统计模板数量
    ///
    /// **预期行为简介:** 统计各种状态的模板数量
    async fn count_templates(&self) -> Result<TemplateCount, DbError>;
}

/// 模板使用统计结果
#[derive(Debug, Clone)]
pub struct TemplateUsageStats {
    pub template_id: Uuid,
    pub template_name: String,
    pub usage_count: i64,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 模板数量统计结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateCount {
    pub total: i64,
    pub active: i64,
    pub deleted: i64,
}
