use std::sync::Arc;
use uuid::Uuid;

use super::{CreateAreaData, UpdateAreaData};
use crate::common::error::{AppError, AppResult};
use crate::core::models::Area;
use crate::ports::{Clock, IdGenerator};
use crate::repositories::{AreaRepository, TaskRepository};

/// 领域服务
///
/// **预期行为简介:** 封装所有与Area相关的业务逻辑，包括领域的CRUD操作和层级关系管理
pub struct AreaService {
    /// 时钟服务
    clock: Arc<dyn Clock>,

    /// ID生成器
    id_generator: Arc<dyn IdGenerator>,

    /// 领域仓库
    area_repository: Arc<dyn AreaRepository>,

    /// 任务仓库
    task_repository: Arc<dyn TaskRepository>,
}

impl AreaService {
    /// 创建新的领域服务
    pub fn new(
        clock: Arc<dyn Clock>,
        id_generator: Arc<dyn IdGenerator>,
        area_repository: Arc<dyn AreaRepository>,
        task_repository: Arc<dyn TaskRepository>,
    ) -> Self {
        Self {
            clock,
            id_generator,
            area_repository,
            task_repository,
        }
    }

    /// 创建领域
    pub async fn create_area(&self, data: CreateAreaData) -> AppResult<Area> {
        // 验证输入
        if let Err(validation_errors) = data.validate() {
            return Err(AppError::ValidationFailed(validation_errors));
        }

        let mut tx = self.task_repository.begin_transaction().await?;

        // 验证父领域存在（如果指定了）
        if let Some(parent_id) = data.parent_area_id {
            let _parent = self
                .area_repository
                .find_by_id(parent_id)
                .await?
                .ok_or_else(|| AppError::not_found("Area", parent_id.to_string()))?;
        }

        // 创建领域
        let area_id = self.id_generator.new_uuid();
        let now = self.clock.now_utc();

        let area = Area {
            id: area_id,
            name: data.name,
            color: data.color,
            parent_area_id: data.parent_area_id,
            created_at: now,
            updated_at: now,
            is_deleted: false,
        };

        let created_area = self.area_repository.create(&mut tx, &area).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(created_area)
    }

    /// 更新领域
    pub async fn update_area(&self, area_id: Uuid, updates: UpdateAreaData) -> AppResult<Area> {
        let mut tx = self.task_repository.begin_transaction().await?;

        // 查找现有领域
        let mut current_area = self
            .area_repository
            .find_by_id(area_id)
            .await?
            .ok_or_else(|| AppError::not_found("Area", area_id.to_string()))?;

        // 验证更新数据
        let mut validation_errors = Vec::new();

        if let Some(ref name) = updates.name {
            if name.is_empty() {
                validation_errors.push(crate::common::error::ValidationError::new(
                    "name",
                    "Area name cannot be empty",
                    "NAME_EMPTY",
                ));
            } else if name.len() > 100 {
                validation_errors.push(crate::common::error::ValidationError::new(
                    "name",
                    "Area name cannot exceed 100 characters",
                    "NAME_TOO_LONG",
                ));
            }
        }

        if let Some(ref color) = updates.color {
            if !Area::validate_color(color) {
                validation_errors.push(crate::common::error::ValidationError::new(
                    "color",
                    "Invalid color format, must be #RRGGBB",
                    "COLOR_INVALID",
                ));
            }
        }

        if !validation_errors.is_empty() {
            return Err(AppError::ValidationFailed(validation_errors));
        }

        // 验证父领域（如果更新了）
        if let Some(Some(parent_id)) = updates.parent_area_id {
            // 检查父领域是否存在
            let _parent = self
                .area_repository
                .find_by_id(parent_id)
                .await?
                .ok_or_else(|| AppError::not_found("Area", parent_id.to_string()))?;

            // 检查是否会造成循环引用
            let would_cycle = self
                .area_repository
                .would_create_cycle(area_id, parent_id)
                .await?;
            if would_cycle {
                return Err(AppError::conflict("移动领域会造成循环引用"));
            }
        }

        // 应用更新
        if let Some(name) = updates.name {
            current_area.name = name;
        }
        if let Some(color) = updates.color {
            current_area.color = color;
        }
        if let Some(parent_area_id) = updates.parent_area_id {
            current_area.parent_area_id = parent_area_id;
        }

        current_area.updated_at = self.clock.now_utc();

        let updated_area = self.area_repository.update(&mut tx, &current_area).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(updated_area)
    }

    /// 删除领域（含边界检查）
    ///
    /// **函数签名:** `pub async fn delete_area(&self, area_id: Uuid) -> Result<(), AppError>`
    /// **执行过程 (Process):**
    /// 1. **启动数据库事务。**
    /// 2. **验证Area:** 检查`area_id`是否存在。
    /// 3. **边界检查:**
    ///    a. 调用 `TaskRepository::count_by_area(area_id)`检查是否有任务在使用此Area。
    ///    b. 调用 `ProjectRepository::count_by_area(area_id)`检查是否有项目在使用此Area。
    ///    c. 如果任一计数大于0，**回滚事务并返回`AppError::Conflict("无法删除尚在使用的Area")`**。
    /// 4. **核心操作:** 调用`AreaRepository::delete(area_id)`。
    /// 5. **提交事务。**
    /// 6. **返回:** `Ok(())`。
    /// **预期副作用:** 软删除`areas`表中的一条记录。
    pub async fn delete_area(&self, area_id: Uuid) -> AppResult<()> {
        // 1. 启动数据库事务
        let mut tx = self.task_repository.begin_transaction().await?;

        // 2. 验证Area
        let _area = self
            .area_repository
            .find_by_id(area_id)
            .await?
            .ok_or_else(|| AppError::not_found("Area", area_id.to_string()))?;

        // 3. 边界检查
        // a. 检查是否有任务在使用此Area
        let tasks_using_area = self.task_repository.find_by_area_id(area_id).await?;
        if !tasks_using_area.is_empty() {
            return Err(AppError::conflict("无法删除尚在使用的Area：存在关联的任务"));
        }

        // b. 检查是否有项目在使用此Area（在V1.0中项目表只建表不提供API，所以这里简化处理）
        // 检查是否有子领域
        let has_children = self.area_repository.has_children(area_id).await?;
        if has_children {
            return Err(AppError::conflict("无法删除尚在使用的Area：存在子领域"));
        }

        // 检查是否被其他实体使用
        let is_used = self.area_repository.is_used(area_id).await?;
        if is_used {
            return Err(AppError::conflict("无法删除尚在使用的Area"));
        }

        // 4. 核心操作
        self.area_repository.soft_delete(&mut tx, area_id).await?;

        // 5. 提交事务
        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        // 6. 返回
        Ok(())
    }

    /// 获取领域详情
    pub async fn get_area(&self, area_id: Uuid) -> AppResult<Option<Area>> {
        self.area_repository
            .find_by_id(area_id)
            .await
            .map_err(AppError::from)
    }

    /// 获取所有领域
    pub async fn get_all_areas(&self) -> AppResult<Vec<Area>> {
        self.area_repository
            .find_all()
            .await
            .map_err(AppError::from)
    }

    /// 获取根领域
    pub async fn get_root_areas(&self) -> AppResult<Vec<Area>> {
        self.area_repository
            .find_root_areas()
            .await
            .map_err(AppError::from)
    }

    /// 获取子领域
    pub async fn get_child_areas(&self, parent_id: Uuid) -> AppResult<Vec<Area>> {
        self.area_repository
            .find_children(parent_id)
            .await
            .map_err(AppError::from)
    }

    /// 获取所有后代领域
    pub async fn get_descendant_areas(&self, parent_id: Uuid) -> AppResult<Vec<Area>> {
        self.area_repository
            .find_descendants(parent_id)
            .await
            .map_err(AppError::from)
    }

    /// 获取领域路径
    pub async fn get_area_path(&self, area_id: Uuid) -> AppResult<Vec<Area>> {
        self.area_repository
            .find_path_to_root(area_id)
            .await
            .map_err(AppError::from)
    }

    /// 移动领域到新父级
    pub async fn move_area(&self, area_id: Uuid, new_parent_id: Option<Uuid>) -> AppResult<Area> {
        let mut tx = self.task_repository.begin_transaction().await?;

        let moved_area = self
            .area_repository
            .move_to_parent(&mut tx, area_id, new_parent_id)
            .await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(moved_area)
    }

    /// 恢复已删除的领域
    pub async fn restore_area(&self, area_id: Uuid) -> AppResult<Area> {
        let mut tx = self.task_repository.begin_transaction().await?;

        let restored_area = self.area_repository.restore(&mut tx, area_id).await?;

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(crate::common::error::DbError::TransactionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(restored_area)
    }

    /// 获取领域使用统计
    pub async fn get_usage_statistics(
        &self,
    ) -> AppResult<Vec<crate::repositories::AreaUsageCount>> {
        self.area_repository
            .count_usage()
            .await
            .map_err(AppError::from)
    }

    /// 检查领域是否可以删除
    pub async fn can_delete_area(&self, area_id: Uuid) -> AppResult<bool> {
        let has_children = self.area_repository.has_children(area_id).await?;
        let is_used = self.area_repository.is_used(area_id).await?;

        Ok(!has_children && !is_used)
    }
}
