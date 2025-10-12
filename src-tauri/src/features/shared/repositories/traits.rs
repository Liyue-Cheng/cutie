/// Repository trait 抽象
/// 
/// 定义了所有 Repository 的通用接口，遵循 Rust 最佳实践

use async_trait::async_trait;
use sqlx::{Sqlite, SqlitePool, Transaction};
use uuid::Uuid;

use crate::infra::core::AppResult;

/// 基础 Repository trait
/// 
/// 所有具体的 Repository 都应该实现这个 trait
#[async_trait]
pub trait Repository<Entity, ID = Uuid> {
    /// 根据 ID 查找实体
    async fn find_by_id(pool: &SqlitePool, id: ID) -> AppResult<Option<Entity>>;
    
    /// 在事务中根据 ID 查找实体
    async fn find_by_id_in_tx(
        tx: &mut Transaction<'_, Sqlite>, 
        id: ID
    ) -> AppResult<Option<Entity>>;
    
    /// 保存实体（插入或更新）
    async fn save(pool: &SqlitePool, entity: &Entity) -> AppResult<()>;
    
    /// 在事务中保存实体
    async fn save_in_tx(
        tx: &mut Transaction<'_, Sqlite>, 
        entity: &Entity
    ) -> AppResult<()>;
    
    /// 软删除实体
    async fn soft_delete(pool: &SqlitePool, id: ID) -> AppResult<()>;
    
    /// 在事务中软删除实体
    async fn soft_delete_in_tx(
        tx: &mut Transaction<'_, Sqlite>, 
        id: ID
    ) -> AppResult<()>;
}

/// 可查询的 Repository trait
/// 
/// 为支持复杂查询的 Repository 提供额外的查询方法
#[async_trait]
pub trait QueryableRepository<Entity, ID = Uuid>: Repository<Entity, ID> {
    /// 查找所有未删除的实体
    async fn find_all(pool: &SqlitePool) -> AppResult<Vec<Entity>>;
    
    /// 根据条件查找实体
    async fn find_by_condition(
        pool: &SqlitePool, 
        condition: &str, 
        params: &[&dyn sqlx::Encode<'_, Sqlite>]
    ) -> AppResult<Vec<Entity>>;
    
    /// 统计实体数量
    async fn count(pool: &SqlitePool) -> AppResult<i64>;
}

/// 可批量操作的 Repository trait
#[async_trait]
pub trait BatchRepository<Entity, ID = Uuid>: Repository<Entity, ID> {
    /// 批量插入
    async fn batch_insert(pool: &SqlitePool, entities: &[Entity]) -> AppResult<()>;
    
    /// 在事务中批量插入
    async fn batch_insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>, 
        entities: &[Entity]
    ) -> AppResult<()>;
    
    /// 批量软删除
    async fn batch_soft_delete(pool: &SqlitePool, ids: &[ID]) -> AppResult<()>;
    
    /// 在事务中批量软删除
    async fn batch_soft_delete_in_tx(
        tx: &mut Transaction<'_, Sqlite>, 
        ids: &[ID]
    ) -> AppResult<()>;
}
