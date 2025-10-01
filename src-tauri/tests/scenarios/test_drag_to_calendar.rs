/// 拖拽任务到日历场景测试
///
/// 验证 Cutie 核心功能：任务与时间块的多对多架构
use serde_json::json;

use crate::common::{helpers, ApiResponse, ResponseAssertions, TestApp, TestFixtures};

#[tokio::test]
async fn should_complete_drag_to_calendar_workflow() {
    let app = TestApp::new().await;

    // 1. 创建任务（自动进入 Staging）
    let task_id = TestFixtures::create_task(&app, "拖拽任务").await;

    // 验证在 staging
    let staging_res = app.get("/views/staging").send().await.unwrap();
    let staging: ApiResponse<Vec<serde_json::Value>> = staging_res.json().await.unwrap();
    let staging_data = &staging.data;
    assert_eq!(staging_data.len(), 1);
    assert_eq!(staging_data[0]["schedule_status"], "staging");

    // 2. 拖动到日历（创建时间块）
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);

    let drag_res = app
        .post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task_id,
            "start_time": start,
            "end_time": end
        }))
        .send()
        .await
        .unwrap();

    drag_res.assert_created();

    let created: ApiResponse<serde_json::Value> = drag_res.json().await.unwrap();
    let data = &created.data;

    // 3. 验证时间块创建
    let time_block = &data["time_block"];
    assert!(time_block["id"].as_str().is_some());
    assert_eq!(time_block["start_time"], start);
    assert_eq!(time_block["end_time"], end);

    // 4. 验证任务状态更新
    let updated_task = &data["updated_task"];
    assert_eq!(updated_task["schedule_status"], "scheduled");

    // 5. 验证任务从 staging 移到 planned
    let staging_res2 = app.get("/views/staging").send().await.unwrap();
    let staging2: ApiResponse<Vec<serde_json::Value>> = staging_res2.json().await.unwrap();
    assert_eq!(staging2.data.len(), 0);

    let planned_res = app.get("/views/planned").send().await.unwrap();
    let planned: ApiResponse<Vec<serde_json::Value>> = planned_res.json().await.unwrap();
    let planned_tasks = planned.data;
    assert_eq!(planned_tasks.len(), 1);
    assert_eq!(planned_tasks[0]["id"], task_id.to_string());
}

#[tokio::test]
async fn should_allow_same_task_in_multiple_time_blocks() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "分段任务").await;

    // 第一段时间
    let start1 = helpers::test_time(1);
    let end1 = helpers::test_time(2);

    app.post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task_id,
            "start_time": start1,
            "end_time": end1
        }))
        .send()
        .await
        .unwrap();

    // 第二段时间
    let start2 = helpers::test_time(3);
    let end2 = helpers::test_time(4);

    let res2 = app
        .post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task_id,
            "start_time": start2,
            "end_time": end2
        }))
        .send()
        .await
        .unwrap();

    res2.assert_created();

    // 验证有 2 个时间块
    let list_res = app.get("/time-blocks").send().await.unwrap();
    let blocks: ApiResponse<Vec<serde_json::Value>> = list_res.json().await.unwrap();
    assert_eq!(blocks.data.len(), 2);
}

#[tokio::test]
async fn should_delete_block_but_keep_task_scheduled() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "保持排期").await;

    // 拖到日历
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);

    let create_res = app
        .post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task_id,
            "start_time": start,
            "end_time": end
        }))
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

    // 验证任务仍然是 scheduled 状态（schedule 被保留）
    let planned_res = app.get("/views/planned").send().await.unwrap();
    let planned: ApiResponse<Vec<serde_json::Value>> = planned_res.json().await.unwrap();
    assert!(planned.data.iter().any(|t| t["id"] == task_id.to_string()));
}
