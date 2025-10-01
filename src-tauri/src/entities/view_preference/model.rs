use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 视图排序偏好实体
///
/// 用于存储用户在不同视图中的任务排序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewPreference {
    /// 视图上下文唯一标识
    /// 格式: misc::staging, daily::2025-10-01, area::{uuid}, project::{uuid}
    pub context_key: String,

    /// 排序后的任务ID数组
    /// 数组顺序即为任务在该视图中的显示顺序
    pub sorted_task_ids: Vec<String>,

    /// 最后更新时间
    pub updated_at: DateTime<Utc>,
}

/// 数据库行结构（用于 sqlx 查询）
#[derive(Debug, sqlx::FromRow)]
pub struct ViewPreferenceRow {
    pub context_key: String,
    pub sorted_task_ids: String, // JSON 字符串
    pub updated_at: String,      // RFC 3339 字符串
}

impl TryFrom<ViewPreferenceRow> for ViewPreference {
    type Error = String;

    fn try_from(row: ViewPreferenceRow) -> Result<Self, Self::Error> {
        // 解析 JSON 数组
        let sorted_task_ids: Vec<String> = serde_json::from_str(&row.sorted_task_ids)
            .map_err(|e| format!("Failed to parse sorted_task_ids: {}", e))?;

        // 解析时间
        let updated_at = DateTime::parse_from_rfc3339(&row.updated_at)
            .map_err(|e| format!("Failed to parse updated_at: {}", e))?
            .with_timezone(&Utc);

        Ok(ViewPreference {
            context_key: row.context_key,
            sorted_task_ids,
            updated_at,
        })
    }
}

