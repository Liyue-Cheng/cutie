/// Time Blocks 端到端测试
use serde_json::json;

use crate::common::{helpers, ApiResponse, ResponseAssertions, TestApp, TestData, TestFixtures};

// ==================== CREATE 测试 ====================

#[tokio::test]
async fn should_create_empty_time_block() {
    let app = TestApp::new().await;

    let start = helpers::test_time(1);
    let end = helpers::test_time(2);

    let response = app
        .post("/time-blocks")
        .json(&TestData::time_block_payload(
            &start,
            &end,
            Some("深度工作"),
        ))
        .send()
        .await
        .unwrap();

    response.assert_created();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let block = body.data;

    assert_eq!(block["title"], "深度工作");
    assert_eq!(block["start_time"], start);
    assert_eq!(block["end_time"], end);
    assert!(block["linked_tasks"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn should_reject_invalid_time_range() {
    let app = TestApp::new().await;

    let start = helpers::test_time(2);
    let end = helpers::test_time(1); // end < start

    let response = app
        .post("/time-blocks")
        .json(&TestData::time_block_payload(&start, &end, None))
        .send()
        .await
        .unwrap();

    response.assert_unprocessable();
}

// ==================== FROM-TASK 创建测试 ====================

#[tokio::test]
async fn should_create_time_block_from_task() {
    let app = TestApp::new().await;
    let task_id = TestFixtures::create_task(&app, "拖动任务").await;

    let start = helpers::test_time(1);
    let end = helpers::test_time(2);

    let response = app
        .post("/time-blocks/from-task")
        .json(&TestData::time_block_from_task_payload(
            task_id, &start, &end,
        ))
        .send()
        .await
        .unwrap();

    response.assert_created();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let data = body.data;

    // 验证时间块
    let time_block = &data["time_block"];
    assert!(time_block["id"].as_str().is_some());

    let linked_tasks = time_block["linked_tasks"].as_array().unwrap();
    assert_eq!(linked_tasks.len(), 1);
    assert_eq!(linked_tasks[0]["id"], task_id.to_string());

    // 验证任务状态更新
    let updated_task = &data["updated_task"];
    assert_eq!(updated_task["id"], task_id.to_string());
    assert_eq!(updated_task["schedule_status"], "scheduled");
}

#[tokio::test]
async fn should_inherit_area_from_task() {
    let app = TestApp::new().await;

    // 创建带 area 的任务
    let area_id = TestFixtures::create_area(&app, "工作", "#4A90E2").await;

    let task_res = app
        .post("/tasks")
        .json(&json!({
            "title": "带区域任务",
            "area_id": area_id
        }))
        .send()
        .await
        .unwrap();
    let task: ApiResponse<serde_json::Value> = task_res.json().await.unwrap();
    let task_data = task.data;
    let task_id = task_data["id"].as_str().unwrap();

    // 拖动到日历
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);

    let response = app
        .post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task_id,
            "start_time": start,
            "end_time": end
        }))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let time_block = &body.data["time_block"];

    // 验证继承了 area
    assert!(time_block["area"].is_object());
    assert_eq!(time_block["area"]["id"], area_id.to_string());
}

#[tokio::test]
async fn should_create_task_schedule_when_dragging() {
    let app = TestApp::new().await;
    let task_id = TestFixtures::create_task(&app, "排期任务").await;

    // 验证初始在 staging
    let staging_res = app.get("/views/staging").send().await.unwrap();
    let staging: ApiResponse<Vec<serde_json::Value>> = staging_res.json().await.unwrap();
    assert!(staging.data.iter().any(|t| t["id"] == task_id.to_string()));

    // 拖动到日历
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);

    app.post("/time-blocks/from-task")
        .json(&TestData::time_block_from_task_payload(
            task_id, &start, &end,
        ))
        .send()
        .await
        .unwrap();

    // 验证从 staging 消失
    let staging_res2 = app.get("/views/staging").send().await.unwrap();
    let staging2: ApiResponse<Vec<serde_json::Value>> = staging_res2.json().await.unwrap();
    assert!(!staging2.data.iter().any(|t| t["id"] == task_id.to_string()));

    // 验证出现在 planned
    let planned_res = app.get("/views/planned").send().await.unwrap();
    let planned: ApiResponse<Vec<serde_json::Value>> = planned_res.json().await.unwrap();
    assert!(planned.data.iter().any(|t| t["id"] == task_id.to_string()));
}

// ==================== LIST 测试 ====================

#[tokio::test]
async fn should_list_time_blocks() {
    let app = TestApp::new().await;

    let start1 = helpers::test_time(1);
    let end1 = helpers::test_time(2);
    let start2 = helpers::test_time(3);
    let end2 = helpers::test_time(4);

    TestFixtures::create_time_block(&app, &start1, &end1).await;
    TestFixtures::create_time_block(&app, &start2, &end2).await;

    let response = app.get("/time-blocks").send().await.unwrap();
    response.assert_success();

    let body: ApiResponse<Vec<serde_json::Value>> = response.json().await.unwrap();
    let blocks = &body.data;

    assert_eq!(blocks.len(), 2);
}

#[tokio::test]
async fn should_return_empty_array_when_no_blocks() {
    let app = TestApp::new().await;

    let response = app.get("/time-blocks").send().await.unwrap();
    response.assert_success();

    let body: ApiResponse<Vec<serde_json::Value>> = response.json().await.unwrap();
    let blocks = &body.data;

    assert_eq!(blocks.len(), 0);
}

// ==================== DELETE 测试 ====================

#[tokio::test]
async fn should_delete_time_block() {
    let app = TestApp::new().await;

    let start = helpers::test_time(1);
    let end = helpers::test_time(2);
    let block_id = TestFixtures::create_time_block(&app, &start, &end).await;

    let response = app
        .delete(&format!("/time-blocks/{}", block_id))
        .send()
        .await
        .unwrap();

    response.assert_success();

    // 验证删除
    let list_res = app.get("/time-blocks").send().await.unwrap();
    let body: ApiResponse<Vec<serde_json::Value>> = list_res.json().await.unwrap();
    let blocks = &body.data;

    assert!(!blocks.iter().any(|b| b["id"] == block_id.to_string()));
}

#[tokio::test]
async fn should_preserve_task_schedule_when_deleting_block() {
    let app = TestApp::new().await;
    let task_id = TestFixtures::create_task(&app, "排期任务").await;

    // 拖动到日历
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);

    let create_res = app
        .post("/time-blocks/from-task")
        .json(&TestData::time_block_from_task_payload(
            task_id, &start, &end,
        ))
        .send()
        .await
        .unwrap();
    let created: ApiResponse<serde_json::Value> = create_res.json().await.unwrap();
    let created_data = created.data;
    let block_id = created_data["time_block"]["id"].as_str().unwrap();

    // 删除时间块
    app.delete(&format!("/time-blocks/{}", block_id))
        .send()
        .await
        .unwrap();

    // 验证任务仍在 planned（schedule 保留）
    let planned_res = app.get("/views/planned").send().await.unwrap();
    let planned: ApiResponse<Vec<serde_json::Value>> = planned_res.json().await.unwrap();
    let planned_tasks = planned.data;
    assert!(planned_tasks.iter().any(|t| t["id"] == task_id.to_string()));
}
