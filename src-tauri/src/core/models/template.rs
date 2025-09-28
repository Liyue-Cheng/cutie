use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Subtask;

/// Template (模板) 实体定义
///
/// 一个用于快速创建新Task的预设配置。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Template {
    /// 模板ID (主键)
    pub id: Uuid,

    /// 模板名称
    ///
    /// **前置条件:** 不能为空，用于在模板列表中展示
    pub name: String,

    /// 标题模板
    ///
    /// **后置条件:** 定义了当使用此模板创建Task时，新Task标题字段的初始值
    /// 可能包含特定的模板变量（如{{date}}），由服务层在实例化时进行解析和替换
    pub title_template: String,

    /// 快览笔记模板 (可选)
    pub glance_note_template: Option<String>,

    /// 详细笔记模板 (可选)
    pub detail_note_template: Option<String>,

    /// 预估时长模板 (可选)
    pub estimated_duration_template: Option<i32>,

    /// 子任务模板 (可选)
    pub subtasks_template: Option<Vec<Subtask>>,

    /// 领域ID (外键, 可选)
    pub area_id: Option<Uuid>,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 逻辑删除标记
    pub is_deleted: bool,
}

impl Template {
    /// 创建新的模板
    pub fn new(id: Uuid, name: String, title_template: String, created_at: DateTime<Utc>) -> Self {
        Self {
            id,
            name,
            title_template,
            glance_note_template: None,
            detail_note_template: None,
            estimated_duration_template: None,
            subtasks_template: None,
            area_id: None,
            created_at,
            updated_at: created_at,
            is_deleted: false,
        }
    }

    /// 检查模板是否包含变量
    pub fn has_variables(&self) -> bool {
        self.title_template.contains("{{")
            || self
                .glance_note_template
                .as_ref()
                .map_or(false, |s| s.contains("{{"))
            || self
                .detail_note_template
                .as_ref()
                .map_or(false, |s| s.contains("{{"))
    }

    /// 获取模板中的所有变量
    pub fn get_variables(&self) -> Vec<String> {
        let mut variables = Vec::new();

        // 简单的变量提取逻辑
        for template in [
            Some(&self.title_template),
            self.glance_note_template.as_ref(),
            self.detail_note_template.as_ref(),
        ]
        .iter()
        .flatten()
        {
            let mut start = 0;
            while let Some(start_pos) = template[start..].find("{{") {
                let abs_start = start + start_pos;
                if let Some(end_pos) = template[abs_start..].find("}}") {
                    let var_name = template[abs_start + 2..abs_start + end_pos].trim();
                    if !variables.contains(&var_name.to_string()) {
                        variables.push(var_name.to_string());
                    }
                    start = abs_start + end_pos + 2;
                } else {
                    break;
                }
            }
        }

        variables
    }
}
