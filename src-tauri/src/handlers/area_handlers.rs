/// 领域相关的HTTP处理器
///
/// 实现所有与领域相关的API端点处理逻辑
use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;

use super::{
    error_handler::{created_response, no_content_response, success_response},
    payloads::{CreateAreaPayload, UpdateAreaPayload},
};
use crate::common::error::AppError;
use crate::startup::AppState;

/// 查询参数：获取领域
#[derive(Debug, Deserialize)]
pub struct GetAreasQuery {
    /// 父领域ID（获取子领域）
    pub parent_id: Option<Uuid>,

    /// 是否只获取根领域
    pub roots_only: Option<bool>,

    /// 是否包含后代
    pub include_descendants: Option<bool>,
}

/// 创建领域处理器
///
/// **端点:** `POST /areas`
pub async fn create_area_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateAreaPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Creating area with name: {}", payload.name);

    // 转换载荷为服务层数据结构
    let create_data = crate::services::CreateAreaData::from(payload);

    // 调用服务层
    let created_area = app_state.area_service.create_area(create_data).await?;

    log::info!("Area created successfully: {}", created_area.id);

    Ok(created_response(created_area))
}

/// 获取领域详情处理器
///
/// **端点:** `GET /areas/{id}`
pub async fn get_area_handler(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting area: {}", area_id);

    let area = app_state
        .area_service
        .get_area(area_id)
        .await?
        .ok_or_else(|| AppError::not_found("Area", area_id.to_string()))?;

    Ok(success_response(area))
}

/// 更新领域处理器
///
/// **端点:** `PUT /areas/{id}`
pub async fn update_area_handler(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
    Json(payload): Json<UpdateAreaPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Updating area: {}", area_id);

    // 转换载荷为服务层数据结构
    let update_data = crate::services::UpdateAreaData::from(payload);

    // 调用服务层
    let updated_area = app_state
        .area_service
        .update_area(area_id, update_data)
        .await?;

    log::info!("Area updated successfully: {}", area_id);

    Ok(success_response(updated_area))
}

/// 删除领域处理器
///
/// **端点:** `DELETE /areas/{id}`
pub async fn delete_area_handler(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Deleting area: {}", area_id);

    // 调用服务层
    app_state.area_service.delete_area(area_id).await?;

    log::info!("Area deleted successfully: {}", area_id);

    Ok(no_content_response())
}

/// 获取领域列表处理器
///
/// **端点:** `GET /areas`
pub async fn get_areas_handler(
    State(app_state): State<AppState>,
    Query(query): Query<GetAreasQuery>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let areas = if query.roots_only.unwrap_or(false) {
        // 获取根领域
        log::debug!("Getting root areas");
        app_state.area_service.get_root_areas().await?
    } else if let Some(parent_id) = query.parent_id {
        if query.include_descendants.unwrap_or(false) {
            // 获取所有后代领域
            log::debug!("Getting descendant areas for: {}", parent_id);
            app_state
                .area_service
                .get_descendant_areas(parent_id)
                .await?
        } else {
            // 获取直接子领域
            log::debug!("Getting child areas for: {}", parent_id);
            app_state.area_service.get_child_areas(parent_id).await?
        }
    } else {
        // 获取所有领域
        log::debug!("Getting all areas");
        app_state.area_service.get_all_areas().await?
    };

    Ok(success_response(areas))
}

/// 获取领域路径处理器
///
/// **端点:** `GET /areas/{id}/path`
pub async fn get_area_path_handler(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting path for area: {}", area_id);

    let path = app_state.area_service.get_area_path(area_id).await?;

    Ok(success_response(path))
}

/// 移动领域处理器
///
/// **端点:** `POST /areas/{id}/move`
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct MoveAreaPayload {
    /// 新的父领域ID
    pub new_parent_id: Option<Uuid>,
}

pub async fn move_area_handler(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
    Json(payload): Json<MoveAreaPayload>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!(
        "Moving area {} to parent: {:?}",
        area_id,
        payload.new_parent_id
    );

    // 调用服务层
    let moved_area = app_state
        .area_service
        .move_area(area_id, payload.new_parent_id)
        .await?;

    log::info!("Area {} moved successfully", area_id);

    Ok(success_response(moved_area))
}

/// 恢复已删除领域处理器
///
/// **端点:** `POST /areas/{id}/restore`
pub async fn restore_area_handler(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Restoring area: {}", area_id);

    // 调用服务层
    let restored_area = app_state.area_service.restore_area(area_id).await?;

    log::info!("Area {} restored successfully", area_id);

    Ok(success_response(restored_area))
}

/// 获取领域使用统计处理器
///
/// **端点:** `GET /areas/stats`
pub async fn get_area_stats_handler(
    State(app_state): State<AppState>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Getting area usage statistics");

    let stats = app_state.area_service.get_usage_statistics().await?;

    let stats_response = super::responses::StatsResponse {
        stats: serde_json::to_value(stats).unwrap_or(serde_json::Value::Null),
        generated_at: chrono::Utc::now(),
        range: None,
    };

    Ok(success_response(stats_response))
}

/// 检查领域是否可删除处理器
///
/// **端点:** `GET /areas/{id}/can-delete`
pub async fn check_area_can_delete_handler(
    State(app_state): State<AppState>,
    Path(area_id): Path<Uuid>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    log::debug!("Checking if area {} can be deleted", area_id);

    let can_delete = app_state.area_service.can_delete_area(area_id).await?;

    #[derive(serde::Serialize)]
    struct CanDeleteResponse {
        can_delete: bool,
        area_id: Uuid,
    }

    Ok(success_response(CanDeleteResponse {
        can_delete,
        area_id,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_area_payload_serialization() {
        let payload = CreateAreaPayload {
            name: "Work".to_string(),
            color: "#FF5722".to_string(),
            parent_area_id: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Work"));
        assert!(json.contains("#FF5722"));

        let deserialized: CreateAreaPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, payload.name);
        assert_eq!(deserialized.color, payload.color);
    }

    #[test]
    fn test_get_areas_query_parsing() {
        let query_str = "parent_id=550e8400-e29b-41d4-a716-446655440000&include_descendants=true";
        let query: GetAreasQuery = serde_urlencoded::from_str(query_str).unwrap();

        assert!(query.parent_id.is_some());
        assert_eq!(query.include_descendants, Some(true));
        assert_eq!(query.roots_only, None);
    }

    #[test]
    fn test_move_area_payload_serialization() {
        let payload = MoveAreaPayload {
            new_parent_id: Some(Uuid::new_v4()),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("new_parent_id"));

        let deserialized: MoveAreaPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.new_parent_id, payload.new_parent_id);
    }
}
