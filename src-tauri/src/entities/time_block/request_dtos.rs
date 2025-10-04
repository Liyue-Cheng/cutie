/// TimeBlock 请求 DTOs
///
/// 只包含 API 请求相关的数据传输对象
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 创建时间块的请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTimeBlockRequest {
    pub title: Option<String>,
    pub glance_note: Option<String>,
    pub detail_note: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub area_id: Option<Uuid>,
    // 🔧 REMOVED: linked_task_ids
    // 职责分离：创建纯时间块不应关联任务
    // 任务关联应使用专门的 POST /time-blocks/from-task 端点
}

/// 更新时间块的请求载荷
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTimeBlockRequest {
    pub title: Option<Option<String>>,
    pub glance_note: Option<Option<String>>,
    pub detail_note: Option<Option<String>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub area_id: Option<Option<Uuid>>,
}
