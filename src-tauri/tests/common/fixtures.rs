/// 测试数据固件
///
/// 提供标准的测试数据构建器
use serde_json::json;
use uuid::Uuid;

use crate::common::{ApiResponse, TestApp};

/// 测试数据构建器
pub struct TestData;

impl TestData {
    /// 创建标准测试区域 payload
    pub fn area_payload(name: &str, color: &str) -> serde_json::Value {
        json!({
            "name": name,
            "color": color
        })
    }

    /// 创建标准测试任务 payload
    pub fn task_payload(title: &str) -> serde_json::Value {
        json!({
            "title": title,
            "glance_note": "测试笔记",
            "detail_note": null,
            "area_id": null,
            "project_id": null,
            "subtasks": null
        })
    }

    /// 创建完整任务 payload
    pub fn task_payload_full(
        title: &str,
        glance: Option<&str>,
        detail: Option<&str>,
        area_id: Option<Uuid>,
    ) -> serde_json::Value {
        json!({
            "title": title,
            "glance_note": glance,
            "detail_note": detail,
            "area_id": area_id,
            "project_id": null,
            "subtasks": null
        })
    }

    /// 创建时间块 payload
    pub fn time_block_payload(start: &str, end: &str, title: Option<&str>) -> serde_json::Value {
        json!({
            "start_time": start,
            "end_time": end,
            "title": title,
            "glance_note": null,
            "detail_note": null,
            "area_id": null
        })
    }

    /// 从任务创建时间块 payload
    pub fn time_block_from_task_payload(
        task_id: Uuid,
        start: &str,
        end: &str,
    ) -> serde_json::Value {
        json!({
            "task_id": task_id,
            "start_time": start,
            "end_time": end
        })
    }
}

/// 测试数据创建辅助函数
pub struct TestFixtures;

impl TestFixtures {
    /// 创建测试区域并返回 ID
    pub async fn create_area(app: &TestApp, name: &str, color: &str) -> Uuid {
        let response = app
            .post("/areas")
            .json(&TestData::area_payload(name, color))
            .send()
            .await
            .expect("Failed to create area");

        let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
        let area_id = body.data["id"].as_str().unwrap();
        Uuid::parse_str(area_id).unwrap()
    }

    /// 创建测试任务并返回 ID
    pub async fn create_task(app: &TestApp, title: &str) -> Uuid {
        let response = app
            .post("/tasks")
            .json(&TestData::task_payload(title))
            .send()
            .await
            .expect("Failed to create task");

        let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
        let task_id = body.data["id"].as_str().unwrap();
        Uuid::parse_str(task_id).unwrap()
    }

    /// 批量创建任务
    pub async fn create_tasks(app: &TestApp, count: usize) -> Vec<Uuid> {
        let mut ids = Vec::new();
        for i in 0..count {
            let id = Self::create_task(app, &format!("Task {}", i + 1)).await;
            ids.push(id);
        }
        ids
    }

    /// 创建时间块并返回 ID
    pub async fn create_time_block(app: &TestApp, start: &str, end: &str) -> Uuid {
        let response = app
            .post("/time-blocks")
            .json(&TestData::time_block_payload(
                start,
                end,
                Some("测试时间块"),
            ))
            .send()
            .await
            .expect("Failed to create time block");

        let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
        let block_id = body.data["id"].as_str().unwrap();
        Uuid::parse_str(block_id).unwrap()
    }
}
