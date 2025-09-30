/// TimeBlockRepository trait定义
///
/// 提供对TimeBlock实体的所有持久化操作接口
use async_trait::async_trait;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::entities::TimeBlock;
use crate::shared::core::AppResult;

/// 时间块仓库接口
///
/// 注意：事务管理在服务层进行，本层的方法接受事务句柄作为参数
#[async_trait]
pub trait TimeBlockRepository: Send + Sync {
    // --- 写操作 ---
    /// 创建新时间块
    async fn create(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        time_block: &TimeBlock,
    ) -> AppResult<TimeBlock>;

    /// 更新时间块
    async fn update(
        &self,
        tx: &mut Transaction<'_, Sqlite>,
        time_block: &TimeBlock,
    ) -> AppResult<TimeBlock>;

    /// 软删除时间块
    async fn delete(&self, tx: &mut Transaction<'_, Sqlite>, id: Uuid) -> AppResult<()>;

    // --- 读操作 ---
    /// 根据ID查找时间块
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<TimeBlock>>;

    /// 查找所有时间块
    async fn find_all(&self) -> AppResult<Vec<TimeBlock>>;
}
