/// TemplateRepository trait定义
///
/// 提供对Template实体的所有持久化操作接口
use async_trait::async_trait;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::infra::core::AppResult;
use crate::entities::Template;

/// 模板仓库接口
///
/// 注意：事务管理在服务层进行，本层的方法接受事务句柄作为参数
#[async_trait]
pub trait TemplateRepository: Send + Sync {
    // --- 写操作 ---
    /// 创建新模板
    async fn create(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        template: &Template,
    ) -> AppResult<Template>;

    /// 更新模板
    async fn update(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        template: &Template,
    ) -> AppResult<Template>;

    /// 软删除模板
    async fn delete(&self, tx: &mut Transaction<'_, Sqlite>, id: Uuid) -> AppResult<()>;

    // --- 读操作 ---
    /// 根据ID查找模板
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Template>>;

    /// 查找所有模板
    async fn find_all(&self) -> AppResult<Vec<Template>>;
}
