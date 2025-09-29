/// 领域业务逻辑层
///
/// 实现领域的业务逻辑和规则
use chrono::Utc;
use uuid::Uuid;

use crate::shared::{
    core::{AppError, AppResult, Area, DbError},
    database::AreaRepository,
};

use sqlx::Row;

use super::payloads::{
    AreaCanDeleteResponse, AreaPathResponse, AreaStatsResponse, CreateAreaPayload, MoveAreaPayload,
    UpdateAreaPayload,
};

/// 领域服务
pub struct AreaService<R: AreaRepository> {
    repository: R,
}

impl<R: AreaRepository> AreaService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 创建新领域
    pub async fn create_area(&self, payload: CreateAreaPayload) -> AppResult<Area> {
        let now = Utc::now();
        let area_id = Uuid::new_v4();

        // 验证父领域存在（如果指定了）
        if let Some(parent_id) = payload.parent_area_id {
            if self.repository.find_by_id(parent_id).await?.is_none() {
                return Err(AppError::not_found("Area", parent_id.to_string()));
            }
        }

        let mut area = Area::new(area_id, payload.name, payload.color, now);
        area.parent_area_id = payload.parent_area_id;

        self.repository.create(&area).await
    }

    /// 根据ID获取领域
    pub async fn get_area(&self, id: Uuid) -> AppResult<Area> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::not_found("Area", id.to_string()))
    }

    /// 更新领域
    pub async fn update_area(&self, id: Uuid, payload: UpdateAreaPayload) -> AppResult<Area> {
        let mut area = self.get_area(id).await?;
        let now = Utc::now();

        // 更新字段
        if let Some(name) = payload.name {
            area.name = name;
        }
        if let Some(color) = payload.color {
            area.color = color;
        }
        if let Some(parent_area_id) = payload.parent_area_id {
            // 验证新父领域存在（如果指定了）
            if let Some(parent_id) = parent_area_id {
                if self.repository.find_by_id(parent_id).await?.is_none() {
                    return Err(AppError::not_found("Area", parent_id.to_string()));
                }

                // 防止循环引用
                if parent_id == area.id {
                    return Err(AppError::conflict("领域不能将自己设为父领域"));
                }

                // 检查是否会造成循环引用
                if self.would_create_cycle(area.id, parent_id).await? {
                    return Err(AppError::conflict("此操作会造成循环引用"));
                }
            }

            area.parent_area_id = parent_area_id;
        }

        area.updated_at = now;
        self.repository.update(&area).await
    }

    /// 删除领域
    pub async fn delete_area(&self, id: Uuid) -> AppResult<()> {
        // 检查领域是否存在
        self.get_area(id).await?;

        // 检查是否可以删除
        if !self.repository.can_delete(id).await? {
            return Err(AppError::conflict("领域正在被使用，无法删除"));
        }

        self.repository.delete(id).await
    }

    /// 移动领域
    pub async fn move_area(&self, id: Uuid, payload: MoveAreaPayload) -> AppResult<Area> {
        let mut area = self.get_area(id).await?;
        let now = Utc::now();

        // 验证新父领域存在（如果指定了）
        if let Some(new_parent_id) = payload.new_parent_id {
            if self.repository.find_by_id(new_parent_id).await?.is_none() {
                return Err(AppError::not_found("Area", new_parent_id.to_string()));
            }

            // 防止循环引用
            if new_parent_id == area.id {
                return Err(AppError::conflict("领域不能将自己设为父领域"));
            }

            // 检查是否会造成循环引用
            if self.would_create_cycle(area.id, new_parent_id).await? {
                return Err(AppError::conflict("此操作会造成循环引用"));
            }
        }

        area.set_parent(payload.new_parent_id, now);
        self.repository.update(&area).await
    }

    /// 获取领域路径
    pub async fn get_area_path(&self, id: Uuid) -> AppResult<AreaPathResponse> {
        let path = self.repository.get_path(id).await?;

        if path.is_empty() {
            return Err(AppError::not_found("Area", id.to_string()));
        }

        Ok(AreaPathResponse {
            depth: path.len() as i32,
            is_root: path.len() == 1,
            path,
        })
    }

    /// 获取领域列表
    pub async fn get_areas(
        &self,
        parent_id: Option<Uuid>,
        roots_only: Option<bool>,
        include_descendants: Option<bool>,
        search_query: Option<String>,
    ) -> AppResult<Vec<Area>> {
        let mut areas = if roots_only == Some(true) {
            // 只获取根领域
            self.repository.find_roots().await?
        } else if let Some(parent_id) = parent_id {
            if include_descendants == Some(true) {
                // 获取所有后代领域
                self.repository.get_descendants(parent_id).await?
            } else {
                // 获取直接子领域
                self.repository.find_by_parent_id(Some(parent_id)).await?
            }
        } else {
            // 获取所有领域
            self.repository.find_all().await?
        };

        // 如果有搜索查询，进行过滤
        if let Some(query) = search_query {
            let query_lower = query.to_lowercase();
            areas.retain(|area| area.name.to_lowercase().contains(&query_lower));
        }

        Ok(areas)
    }

    /// 检查领域是否可删除
    pub async fn check_area_can_delete(&self, id: Uuid) -> AppResult<AreaCanDeleteResponse> {
        // 检查领域是否存在
        self.get_area(id).await?;

        // 检查子领域数量
        let children = self.repository.find_by_parent_id(Some(id)).await?;
        let children_count = children.len() as i64;

        // 检查相关任务数量
        // TODO: 这里应该调用任务服务来检查任务数量
        // 为了避免模块间依赖，这里简化实现
        let related_tasks_count = 0i64;

        // TODO: 检查相关项目数量
        let related_projects_count = 0i64;

        let can_delete =
            children_count == 0 && related_tasks_count == 0 && related_projects_count == 0;

        let blocking_reason = if !can_delete {
            if children_count > 0 {
                Some("领域包含子领域".to_string())
            } else if related_tasks_count > 0 {
                Some("领域正在被任务使用".to_string())
            } else if related_projects_count > 0 {
                Some("领域正在被项目使用".to_string())
            } else {
                Some("未知原因".to_string())
            }
        } else {
            None
        };

        Ok(AreaCanDeleteResponse {
            can_delete,
            area_id: id,
            blocking_reason,
            related_tasks_count: Some(related_tasks_count),
            related_projects_count: Some(related_projects_count),
            children_count: Some(children_count),
        })
    }

    /// 获取领域统计
    pub async fn get_area_stats(&self) -> AppResult<AreaStatsResponse> {
        let all_areas = self.repository.find_all().await?;
        let roots = self.repository.find_roots().await?;

        let total_count = all_areas.len() as i64;
        let root_count = roots.len() as i64;

        // 计算最大深度
        let mut max_depth = 0;
        for area in &all_areas {
            let path = self.repository.get_path(area.id).await?;
            max_depth = max_depth.max(path.len() as i32);
        }

        // 计算平均子领域数
        let mut total_children = 0;
        for area in &all_areas {
            let children = self.repository.find_by_parent_id(Some(area.id)).await?;
            total_children += children.len();
        }

        let avg_children_count = if total_count > 0 {
            total_children as f64 / total_count as f64
        } else {
            0.0
        };

        // 计算使用中的领域数
        // TODO: 这里应该调用任务服务来检查领域使用情况
        // 为了避免模块间依赖，这里简化实现
        let used_count = 0i64;

        let unused_count = total_count - used_count;

        Ok(AreaStatsResponse {
            total_count,
            root_count,
            max_depth,
            avg_children_count,
            used_count,
            unused_count,
        })
    }

    /// 检查是否会造成循环引用
    async fn would_create_cycle(&self, area_id: Uuid, new_parent_id: Uuid) -> AppResult<bool> {
        // 获取新父领域的完整路径
        let parent_path = self.repository.get_path(new_parent_id).await?;

        // 检查当前领域是否在新父领域的路径中
        Ok(parent_path.iter().any(|ancestor| ancestor.id == area_id))
    }

    /// 批量删除领域
    pub async fn bulk_delete_areas(&self, area_ids: Vec<Uuid>) -> AppResult<usize> {
        let mut deleted_count = 0;

        for area_id in area_ids {
            if self.repository.can_delete(area_id).await? {
                self.repository.delete(area_id).await?;
                deleted_count += 1;
            }
        }

        Ok(deleted_count)
    }

    /// 批量移动领域到新父领域
    pub async fn bulk_move_to_parent(
        &self,
        area_ids: Vec<Uuid>,
        new_parent_id: Option<Uuid>,
    ) -> AppResult<usize> {
        let mut moved_count = 0;
        let now = Utc::now();

        // 验证新父领域存在（如果指定了）
        if let Some(parent_id) = new_parent_id {
            if self.repository.find_by_id(parent_id).await?.is_none() {
                return Err(AppError::not_found("Area", parent_id.to_string()));
            }
        }

        for area_id in area_ids {
            if let Some(mut area) = self.repository.find_by_id(area_id).await? {
                // 检查循环引用
                if let Some(parent_id) = new_parent_id {
                    if area_id == parent_id || self.would_create_cycle(area_id, parent_id).await? {
                        continue; // 跳过会造成循环引用的操作
                    }
                }

                area.set_parent(new_parent_id, now);
                self.repository.update(&area).await?;
                moved_count += 1;
            }
        }

        Ok(moved_count)
    }

    /// 批量更新颜色
    pub async fn bulk_update_color(&self, area_ids: Vec<Uuid>, color: String) -> AppResult<usize> {
        // 验证颜色格式
        if !Area::validate_color(&color) {
            return Err(AppError::validation_error(
                "color",
                "无效的颜色代码格式",
                "INVALID_COLOR_FORMAT",
            ));
        }

        let mut updated_count = 0;
        let now = Utc::now();

        for area_id in area_ids {
            if let Some(mut area) = self.repository.find_by_id(area_id).await? {
                area.color = color.clone();
                area.updated_at = now;
                self.repository.update(&area).await?;
                updated_count += 1;
            }
        }

        Ok(updated_count)
    }
}

// 为了访问repository.pool，我们需要为AreaService添加一个方法
impl AreaService<crate::features::areas::repository::SqlxAreaRepository> {
    /// 获取数据库连接池（仅用于SqlxAreaRepository）
    fn get_repository_pool(&self) -> &sqlx::SqlitePool {
        &self.repository.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        features::areas::repository::SqlxAreaRepository,
        shared::database::connection::create_test_database,
    };

    #[tokio::test]
    async fn test_create_area() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxAreaRepository::new(pool);
        let service = AreaService::new(repository);

        let payload = CreateAreaPayload {
            name: "Work".to_string(),
            color: "#FF0000".to_string(),
            parent_area_id: None,
        };

        let area = service.create_area(payload).await.unwrap();
        assert_eq!(area.name, "Work");
        assert_eq!(area.color, "#FF0000");
        assert!(area.parent_area_id.is_none());
    }

    #[tokio::test]
    async fn test_area_hierarchy_operations() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxAreaRepository::new(pool);
        let service = AreaService::new(repository);

        // 创建根领域
        let root_payload = CreateAreaPayload {
            name: "Root".to_string(),
            color: "#FF0000".to_string(),
            parent_area_id: None,
        };
        let root_area = service.create_area(root_payload).await.unwrap();

        // 创建子领域
        let child_payload = CreateAreaPayload {
            name: "Child".to_string(),
            color: "#00FF00".to_string(),
            parent_area_id: Some(root_area.id),
        };
        let child_area = service.create_area(child_payload).await.unwrap();

        // 测试获取路径
        let path_response = service.get_area_path(child_area.id).await.unwrap();
        assert_eq!(path_response.depth, 2);
        assert!(!path_response.is_root);
        assert_eq!(path_response.path.len(), 2);

        // 测试移动领域
        let move_payload = MoveAreaPayload {
            new_parent_id: None, // 移动到根级别
        };
        let moved_area = service
            .move_area(child_area.id, move_payload)
            .await
            .unwrap();
        assert!(moved_area.parent_area_id.is_none());
    }

    #[tokio::test]
    async fn test_cycle_prevention() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxAreaRepository::new(pool);
        let service = AreaService::new(repository);

        // 创建两个领域
        let area1 = service
            .create_area(CreateAreaPayload {
                name: "Area1".to_string(),
                color: "#FF0000".to_string(),
                parent_area_id: None,
            })
            .await
            .unwrap();

        let area2 = service
            .create_area(CreateAreaPayload {
                name: "Area2".to_string(),
                color: "#00FF00".to_string(),
                parent_area_id: Some(area1.id),
            })
            .await
            .unwrap();

        // 尝试将area1设为area2的子领域（会造成循环）
        let move_payload = MoveAreaPayload {
            new_parent_id: Some(area2.id),
        };

        let result = service.move_area(area1.id, move_payload).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("循环引用"));
    }

    #[tokio::test]
    async fn test_bulk_operations() {
        let pool = create_test_database().await.unwrap();
        let repository = SqlxAreaRepository::new(pool);
        let service = AreaService::new(repository);

        // 创建多个领域
        let mut area_ids = Vec::new();
        for i in 0..3 {
            let area = service
                .create_area(CreateAreaPayload {
                    name: format!("Area {}", i),
                    color: "#FF0000".to_string(),
                    parent_area_id: None,
                })
                .await
                .unwrap();
            area_ids.push(area.id);
        }

        // 批量更新颜色
        let updated_count = service
            .bulk_update_color(area_ids.clone(), "#00FF00".to_string())
            .await
            .unwrap();
        assert_eq!(updated_count, 3);

        // 批量删除
        let deleted_count = service.bulk_delete_areas(area_ids).await.unwrap();
        assert_eq!(deleted_count, 3);
    }
}
