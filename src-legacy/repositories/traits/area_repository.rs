/// AreaRepository trait定义
///
/// 提供对Area实体的所有持久化操作接口

use async_trait::async_trait;
use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::shared::core::AppResult;
use crate::entities::Area;

/// 区域仓库接口
/// 
/// 注意：事务管理在服务层进行，本层的方法接受事务句柄作为参数
#[async_trait]
pub trait AreaRepository: Send + Sync {
    // --- 写操作 ---
    /// 创建新区域
    async fn create(&self, tx: &mut Transaction<'_, Sqlite>, area: &Area) -> AppResult<Area>;

    /// 更新区域
    async fn update(&self, tx: &mut Transaction<'_, Sqlite>, area: &Area) -> AppResult<Area>;

    /// 软删除区域
    async fn delete(&self, tx: &mut Transaction<'_, Sqlite>, id: Uuid) -> AppResult<()>;

    // --- 读操作 ---
    /// 根据ID查找区域
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Area>>;

    /// 查找所有区域
    async fn find_all(&self) -> AppResult<Vec<Area>>;
}
