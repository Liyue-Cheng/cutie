use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Area (领域) 实体定义
///
/// 代表一个用户自定义的、用于分类和染色的结构化标签。
///
/// ## 不变量
/// - 一个Area不能将自己或自己的子孙Area设为父节点，以防止循环依赖
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Area {
    /// 领域ID (主键)
    pub id: Uuid,

    /// 领域名称
    ///
    /// **前置条件:** 不能为空。建议在同一层级下保持唯一
    pub name: String,

    /// 颜色代码
    ///
    /// **前置条件:** 必须是有效的十六进制颜色码字符串 (e.g., #RRGGBB)
    pub color: String,

    /// 父领域ID (自关联, 可选)
    ///
    /// **前置条件:** 如果非NULL，必须指向一个存在的Area.id
    /// **不变量:** 一个Area不能将自己或自己的子孙Area设为父节点，以防止循环依赖
    pub parent_area_id: Option<Uuid>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 逻辑删除标记
    pub is_deleted: bool,
}

impl Area {
    /// 创建新的领域
    pub fn new(id: Uuid, name: String, color: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            name,
            color,
            parent_area_id: None,
            created_at,
            updated_at: created_at,
            is_deleted: false,
        }
    }

    /// 验证颜色代码格式
    pub fn validate_color(color: &str) -> bool {
        // 简单的十六进制颜色验证
        if !color.starts_with('#') || color.len() != 7 {
            return false;
        }

        color[1..].chars().all(|c| c.is_ascii_hexdigit())
    }

    /// 设置父领域
    pub fn set_parent(&mut self, parent_id: Option<Uuid>, updated_at: DateTime<Utc>) {
        self.parent_area_id = parent_id;
        self.updated_at = updated_at;
    }

    /// 检查是否为根领域
    pub fn is_root(&self) -> bool {
        self.parent_area_id.is_none()
    }
}

