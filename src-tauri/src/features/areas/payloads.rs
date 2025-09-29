/// 领域功能模块的HTTP载荷定义
///
/// 定义领域相关API端点的请求体和响应体结构

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::{
    core::{Area, ValidationError},
    http::extractors::Validate,
};

/// 创建领域请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAreaPayload {
    /// 领域名称
    pub name: String,

    /// 颜色代码
    pub color: String,

    /// 父领域ID
    pub parent_area_id: Option<Uuid>,
}

impl Validate for CreateAreaPayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证名称
        if self.name.trim().is_empty() {
            errors.push(ValidationError::new(
                "name",
                "领域名称不能为空",
                "NAME_EMPTY",
            ));
        }

        if self.name.len() > 100 {
            errors.push(ValidationError::new(
                "name",
                "领域名称不能超过100个字符",
                "NAME_TOO_LONG",
            ));
        }

        // 验证颜色代码
        if !Area::validate_color(&self.color) {
            errors.push(ValidationError::new(
                "color",
                "无效的颜色代码格式，必须是#RRGGBB格式",
                "INVALID_COLOR_FORMAT",
            ));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 更新领域请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAreaPayload {
    /// 领域名称
    pub name: Option<String>,

    /// 颜色代码
    pub color: Option<String>,

    /// 父领域ID
    pub parent_area_id: Option<Option<Uuid>>,
}

impl Validate for UpdateAreaPayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // 验证名称
        if let Some(name) = &self.name {
            if name.trim().is_empty() {
                errors.push(ValidationError::new(
                    "name",
                    "领域名称不能为空",
                    "NAME_EMPTY",
                ));
            }

            if name.len() > 100 {
                errors.push(ValidationError::new(
                    "name",
                    "领域名称不能超过100个字符",
                    "NAME_TOO_LONG",
                ));
            }
        }

        // 验证颜色代码
        if let Some(color) = &self.color {
            if !Area::validate_color(color) {
                errors.push(ValidationError::new(
                    "color",
                    "无效的颜色代码格式，必须是#RRGGBB格式",
                    "INVALID_COLOR_FORMAT",
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 移动领域请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveAreaPayload {
    /// 新的父领域ID，null表示移动到根级别
    pub new_parent_id: Option<Uuid>,
}

impl Validate for MoveAreaPayload {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        // 基本验证，更复杂的循环检测在服务层进行
        Ok(())
    }
}

/// 领域查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaQuery {
    /// 父领域ID
    pub parent_id: Option<Uuid>,

    /// 是否只获取根领域
    pub roots_only: Option<bool>,

    /// 是否包含后代领域
    pub include_descendants: Option<bool>,

    /// 搜索关键词
    pub q: Option<String>,
}

impl Default for AreaQuery {
    fn default() -> Self {
        Self {
            parent_id: None,
            roots_only: None,
            include_descendants: None,
            q: None,
        }
    }
}

/// 领域统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaStatsResponse {
    /// 总领域数
    pub total_count: i64,

    /// 根领域数
    pub root_count: i64,

    /// 最大层级深度
    pub max_depth: i32,

    /// 平均子领域数
    pub avg_children_count: f64,

    /// 使用中的领域数（有任务或项目的领域）
    pub used_count: i64,

    /// 未使用的领域数
    pub unused_count: i64,
}

/// 领域路径响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaPathResponse {
    /// 从根到目标领域的完整路径
    pub path: Vec<Area>,

    /// 路径深度
    pub depth: i32,

    /// 是否为根领域
    pub is_root: bool,
}

/// 领域删除检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaCanDeleteResponse {
    /// 是否可以删除
    pub can_delete: bool,

    /// 领域ID
    pub area_id: Uuid,

    /// 阻止删除的原因（如果不能删除）
    pub blocking_reason: Option<String>,

    /// 相关的任务数量
    pub related_tasks_count: Option<i64>,

    /// 相关的项目数量
    pub related_projects_count: Option<i64>,

    /// 子领域数量
    pub children_count: Option<i64>,
}

/// 批量领域操作载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAreaOperation {
    /// 领域ID列表
    pub area_ids: Vec<Uuid>,

    /// 操作类型
    pub operation: BulkAreaOperationType,

    /// 操作参数（可选）
    pub parameters: Option<serde_json::Value>,
}

/// 批量领域操作类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BulkAreaOperationType {
    /// 批量删除
    Delete,

    /// 批量移动到新父领域
    MoveToParent,

    /// 批量更新颜色
    UpdateColor,
}

impl Validate for BulkAreaOperation {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.area_ids.is_empty() {
            errors.push(ValidationError::new(
                "area_ids",
                "领域ID列表不能为空",
                "AREA_IDS_EMPTY",
            ));
        }

        if self.area_ids.len() > 50 {
            errors.push(ValidationError::new(
                "area_ids",
                "批量操作不能超过50个领域",
                "TOO_MANY_AREAS",
            ));
        }

        // 验证移动操作的参数
        if self.operation == BulkAreaOperationType::MoveToParent {
            if let Some(params) = &self.parameters {
                if !params.get("new_parent_id").is_some() {
                    errors.push(ValidationError::new(
                        "parameters.new_parent_id",
                        "移动操作必须提供新的父领域ID",
                        "NEW_PARENT_ID_REQUIRED",
                    ));
                }
            } else {
                errors.push(ValidationError::new(
                    "parameters",
                    "移动操作必须提供参数",
                    "PARAMETERS_REQUIRED",
                ));
            }
        }

        // 验证更新颜色操作的参数
        if self.operation == BulkAreaOperationType::UpdateColor {
            if let Some(params) = &self.parameters {
                if let Some(color_value) = params.get("color") {
                    if let Ok(color_str) = serde_json::from_value::<String>(color_value.clone()) {
                        if !Area::validate_color(&color_str) {
                            errors.push(ValidationError::new(
                                "parameters.color",
                                "无效的颜色代码格式",
                                "INVALID_COLOR_FORMAT",
                            ));
                        }
                    } else {
                        errors.push(ValidationError::new(
                            "parameters.color",
                            "颜色必须是字符串",
                            "COLOR_NOT_STRING",
                        ));
                    }
                } else {
                    errors.push(ValidationError::new(
                        "parameters.color",
                        "更新颜色操作必须提供颜色值",
                        "COLOR_REQUIRED",
                    ));
                }
            } else {
                errors.push(ValidationError::new(
                    "parameters",
                    "更新颜色操作必须提供参数",
                    "PARAMETERS_REQUIRED",
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_area_payload_validation() {
        let valid_payload = CreateAreaPayload {
            name: "Work".to_string(),
            color: "#FF0000".to_string(),
            parent_area_id: None,
        };

        assert!(valid_payload.validate().is_ok());

        // 测试无效颜色
        let invalid_payload = CreateAreaPayload {
            name: "Work".to_string(),
            color: "red".to_string(), // 无效格式
            parent_area_id: None,
        };

        assert!(invalid_payload.validate().is_err());

        // 测试空名称
        let empty_name_payload = CreateAreaPayload {
            name: "".to_string(),
            color: "#FF0000".to_string(),
            parent_area_id: None,
        };

        assert!(empty_name_payload.validate().is_err());
    }

    #[test]
    fn test_update_area_payload_validation() {
        let valid_payload = UpdateAreaPayload {
            name: Some("Updated Work".to_string()),
            color: Some("#00FF00".to_string()),
            parent_area_id: Some(Some(Uuid::new_v4())),
        };

        assert!(valid_payload.validate().is_ok());

        // 测试无效颜色
        let invalid_payload = UpdateAreaPayload {
            name: None,
            color: Some("invalid-color".to_string()),
            parent_area_id: None,
        };

        assert!(invalid_payload.validate().is_err());
    }

    #[test]
    fn test_bulk_area_operation_validation() {
        let valid_operation = BulkAreaOperation {
            area_ids: vec![Uuid::new_v4(), Uuid::new_v4()],
            operation: BulkAreaOperationType::Delete,
            parameters: None,
        };

        assert!(valid_operation.validate().is_ok());

        // 测试空ID列表
        let invalid_operation = BulkAreaOperation {
            area_ids: vec![],
            operation: BulkAreaOperationType::Delete,
            parameters: None,
        };

        assert!(invalid_operation.validate().is_err());
    }
}
