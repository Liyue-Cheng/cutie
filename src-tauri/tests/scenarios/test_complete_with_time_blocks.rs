/// 完成任务时时间块处理场景测试
///
/// 验证 Cutie 的精确业务逻辑：过去/现在/未来时间块的不同处理
use serde_json::json;

use crate::common::{helpers, ApiResponse, ResponseAssertions, TestApp, TestFixtures};

#[tokio::test]
async fn should_preserve_past_time_blocks() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "过去任务").await;

    // 创建过去的时间块
    let past_start = helpers::test_time(-3); // 3 小时前
    let past_end = helpers::test_time(-2); // 2 小时前

    let block_res = app
        .post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task_id,
            "start_time": past_start,
            "end_time": past_end
        }))
        .send()
        .await
        .unwrap();

    let block: ApiResponse<serde_json::Value> = block_res.json().await.unwrap();
    let block_data = block.data;
    let block_id = block_data["time_block"]["id"].as_str().unwrap().to_string();

    // 完成任务
    let complete_res = app
        .post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    let completed: ApiResponse<serde_json::Value> = complete_res.json().await.unwrap();
    let response_data = completed.data;

    // 验证过去的时间块不被删除
    assert!(!response_data["deleted_time_block_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id == &block_id));

    // 验证不被截断
    assert!(!response_data["truncated_time_block_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id == &block_id));

    // 验证时间块仍存在
    let list_res = app.get("/time-blocks").send().await.unwrap();
    let blocks: ApiResponse<Vec<serde_json::Value>> = list_res.json().await.unwrap();
    assert!(blocks.data.iter().any(|b| b["id"] == block_id));
}

#[tokio::test]
async fn should_delete_future_auto_created_time_blocks() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "未来任务").await;

    // 创建未来的时间块
    let future_start = helpers::test_time(10); // 10 小时后
    let future_end = helpers::test_time(11);

    let block_res = app
        .post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task_id,
            "start_time": future_start,
            "end_time": future_end
        }))
        .send()
        .await
        .unwrap();

    let block: ApiResponse<serde_json::Value> = block_res.json().await.unwrap();
    let block_data = block.data;
    let block_id = block_data["time_block"]["id"].as_str().unwrap().to_string();

    // 完成任务
    let complete_res = app
        .post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    let completed: ApiResponse<serde_json::Value> = complete_res.json().await.unwrap();

    // 验证未来的自动创建时间块被删除
    assert!(completed.data["deleted_time_block_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id == &block_id));

    // 验证时间块确实被删除
    let list_res = app.get("/time-blocks").send().await.unwrap();
    let blocks: ApiResponse<Vec<serde_json::Value>> = list_res.json().await.unwrap();
    assert!(!blocks.data.iter().any(|b| b["id"] == block_id));
}

#[tokio::test]
async fn should_preserve_manually_created_future_blocks() {
    let app = TestApp::new().await;

    // 手动创建未来时间块
    let future_start = helpers::test_time(10);
    let future_end = helpers::test_time(11);

    let block_res = app
        .post("/time-blocks")
        .json(&json!({
            "title": "会议",  // 手动标题
            "start_time": future_start,
            "end_time": future_end
        }))
        .send()
        .await
        .unwrap();

    let block: ApiResponse<serde_json::Value> = block_res.json().await.unwrap();
    let block_id = block.data["id"].as_str().unwrap().to_string();

    // 创建任务并完成
    let task_id = TestFixtures::create_task(&app, "任务").await;

    // TODO: 链接任务到时间块

    // 完成任务
    app.post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    // 验证手动创建的未来时间块不被删除（因为标题不同）
    let list_res = app.get("/time-blocks").send().await.unwrap();
    let blocks: ApiResponse<Vec<serde_json::Value>> = list_res.json().await.unwrap();
    assert!(blocks.data.iter().any(|b| b["id"] == block_id));
}
