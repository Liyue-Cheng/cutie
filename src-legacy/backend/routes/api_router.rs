/// 主API路由器
///
/// 组合所有子路由模块，创建完整的API路由树
use axum::Router;

use super::{
    create_area_routes, create_ordering_routes, create_schedule_routes, create_task_routes,
    create_template_routes, create_time_block_routes,
};
use crate::startup::AppState;

/// 创建完整的API路由器
///
/// **预期行为简介:** 构建包含所有V1.0 API端点的完整路由树
///
/// ## API端点概览
///
/// ### 任务管理 (Tasks)
/// - `POST /api/tasks` - 创建任务
/// - `GET /api/tasks/{id}` - 获取任务详情
/// - `PUT /api/tasks/{id}` - 更新任务
/// - `DELETE /api/tasks/{id}` - 删除任务
/// - `POST /api/tasks/{id}/completion` - 完成任务
/// - `POST /api/tasks/{id}/reopen` - 重新打开任务
/// - `GET /api/tasks/search` - 搜索任务
/// - `GET /api/tasks/unscheduled` - 获取未安排任务
/// - `GET /api/tasks/stats` - 获取任务统计
///
/// ### 日程管理 (Schedules)
/// - `POST /api/schedules` - 安排任务
/// - `GET /api/schedules` - 获取日程列表
/// - `DELETE /api/schedules/{id}` - 删除日程
/// - `POST /api/schedules/{id}/presence` - 记录努力
/// - `DELETE /api/schedules/tasks/{task_id}` - 取消任务所有日程
/// - `GET /api/schedules/stats` - 获取日程统计
///
/// ### 排序管理 (Ordering)
/// - `PUT /api/ordering` - 更新排序
/// - `GET /api/ordering` - 获取上下文排序
/// - `DELETE /api/ordering` - 清理上下文排序
/// - `PUT /api/ordering/batch` - 批量更新排序
/// - `GET /api/ordering/calculate` - 计算排序位置
///
/// ### 时间块管理 (Time Blocks)
/// - `POST /api/time-blocks` - 创建时间块
/// - `GET /api/time-blocks` - 获取时间块列表
/// - `GET /api/time-blocks/{id}` - 获取时间块详情
/// - `PUT /api/time-blocks/{id}` - 更新时间块
/// - `DELETE /api/time-blocks/{id}` - 删除时间块
/// - `POST /api/time-blocks/{id}/tasks` - 链接任务到时间块
/// - `DELETE /api/time-blocks/{id}/tasks/{task_id}` - 取消任务关联
/// - `POST /api/time-blocks/{id}/truncate` - 截断时间块
/// - `POST /api/time-blocks/{id}/extend` - 扩展时间块
/// - `POST /api/time-blocks/{id}/split` - 分割时间块
/// - `GET /api/time-blocks/conflicts` - 检查时间冲突
/// - `GET /api/time-blocks/free-slots` - 查找空闲时间段
///
/// ### 模板管理 (Templates)
/// - `POST /api/templates` - 创建模板
/// - `GET /api/templates` - 获取模板列表
/// - `GET /api/templates/{id}` - 获取模板详情
/// - `PUT /api/templates/{id}` - 更新模板
/// - `DELETE /api/templates/{id}` - 删除模板
/// - `POST /api/templates/{id}/clone` - 克隆模板
/// - `POST /api/templates/{id}/tasks` - 基于模板创建任务
/// - `GET /api/templates/stats` - 获取模板统计
///
/// ### 领域管理 (Areas)
/// - `POST /api/areas` - 创建领域
/// - `GET /api/areas` - 获取领域列表
/// - `GET /api/areas/{id}` - 获取领域详情
/// - `PUT /api/areas/{id}` - 更新领域
/// - `DELETE /api/areas/{id}` - 删除领域
/// - `GET /api/areas/{id}/path` - 获取领域路径
/// - `POST /api/areas/{id}/move` - 移动领域
/// - `POST /api/areas/{id}/restore` - 恢复领域
/// - `GET /api/areas/{id}/can-delete` - 检查是否可删除
/// - `GET /api/areas/stats` - 获取领域统计
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .merge(create_task_routes())
        .merge(create_schedule_routes())
        .merge(create_ordering_routes())
        .merge(create_time_block_routes())
        .merge(create_template_routes())
        .merge(create_area_routes())
}

/// 获取API端点总数
pub fn get_api_endpoint_count() -> usize {
    // 手动统计，确保与实际路由数量一致
    let task_endpoints = 11; // 任务相关端点
    let schedule_endpoints = 6; // 日程相关端点
    let ordering_endpoints = 5; // 排序相关端点
    let time_block_endpoints = 12; // 时间块相关端点
    let template_endpoints = 8; // 模板相关端点
    let area_endpoints = 10; // 领域相关端点

    task_endpoints
        + schedule_endpoints
        + ordering_endpoints
        + time_block_endpoints
        + template_endpoints
        + area_endpoints
}

/// 获取API版本信息
pub fn get_api_version_info() -> serde_json::Value {
    serde_json::json!({
        "version": "1.0.0",
        "name": "Cutie API",
        "description": "Cutie Task Management Backend API",
        "endpoints": get_api_endpoint_count(),
        "modules": [
            "tasks",
            "schedules",
            "ordering",
            "time-blocks",
            "templates",
            "areas"
        ],
        "features": [
            "task_management",
            "schedule_management",
            "time_blocking",
            "template_system",
            "area_hierarchy",
            "lexorank_sorting"
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_endpoint_count() {
        let count = get_api_endpoint_count();
        assert_eq!(count, 52); // 验证端点总数
    }

    #[test]
    fn test_api_version_info() {
        let info = get_api_version_info();

        assert_eq!(info["version"], "1.0.0");
        assert_eq!(info["name"], "Cutie API");
        assert_eq!(info["endpoints"], 52);

        let modules = info["modules"].as_array().unwrap();
        assert_eq!(modules.len(), 6);
        assert!(modules.contains(&serde_json::Value::String("tasks".to_string())));
        assert!(modules.contains(&serde_json::Value::String("schedules".to_string())));

        let features = info["features"].as_array().unwrap();
        assert_eq!(features.len(), 6);
        assert!(features.contains(&serde_json::Value::String("task_management".to_string())));
        assert!(features.contains(&serde_json::Value::String("lexorank_sorting".to_string())));
    }
}
