/// Views 聚合端点测试
use serde_json::json;

use crate::common::{helpers, ApiResponse, ResponseAssertions, TestApp, TestFixtures};

// ==================== STAGING 视图测试 ====================

#[tokio::test]
async fn should_return_only_unscheduled_tasks() {
    let app = TestApp::new().await;

    // 创建任务
    let task1_id = TestFixtures::create_task(&app, "Staging 任务").await;
    let task2_id = TestFixtures::create_task(&app, "待排期任务").await;

    // 将 task2 拖到日历
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);
    app.post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task2_id,
            "start_time": start,
            "end_time": end
        }))
        .send()
        .await
        .unwrap();

    // 查询 staging 视图
    let response = app.get("/views/staging").send().await.unwrap();
    response.assert_success();

    let body: ApiResponse<Vec<serde_json::Value>> = response.json().await.unwrap();
    let tasks = &body.data;

    // 只应该有 task1
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"], task1_id.to_string());
    assert_eq!(tasks[0]["schedule_status"], "staging");
}

#[tokio::test]
async fn should_return_empty_when_all_scheduled() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "已排期").await;

    // 排期
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);
    app.post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task_id,
            "start_time": start,
            "end_time": end
        }))
        .send()
        .await
        .unwrap();

    // Staging 应该为空
    let response = app.get("/views/staging").send().await.unwrap();
    let body: ApiResponse<Vec<serde_json::Value>> = response.json().await.unwrap();

    assert_eq!(body.data.len(), 0);
}

// ==================== PLANNED 视图测试 ====================

#[tokio::test]
async fn should_return_only_scheduled_tasks() {
    let app = TestApp::new().await;

    let task1_id = TestFixtures::create_task(&app, "Staging 任务").await;
    let task2_id = TestFixtures::create_task(&app, "已排期任务").await;

    // 排期 task2
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);
    app.post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task2_id,
            "start_time": start,
            "end_time": end
        }))
        .send()
        .await
        .unwrap();

    // 查询 planned 视图
    let response = app.get("/views/planned").send().await.unwrap();
    response.assert_success();

    let body: ApiResponse<Vec<serde_json::Value>> = response.json().await.unwrap();
    let tasks = &body.data;

    // 只应该有 task2
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0]["id"], task2_id.to_string());
    assert_eq!(tasks[0]["schedule_status"], "scheduled");
}

// ==================== ALL-INCOMPLETE 视图测试 ====================

#[tokio::test]
async fn should_return_all_incomplete_tasks() {
    let app = TestApp::new().await;

    let task1_id = TestFixtures::create_task(&app, "未完成1").await;
    let task2_id = TestFixtures::create_task(&app, "未完成2").await;
    let task3_id = TestFixtures::create_task(&app, "待完成").await;

    // 完成 task3
    app.post(&format!("/tasks/{}/completion", task3_id))
        .send()
        .await
        .unwrap();

    // 查询 all-incomplete
    let response = app.get("/views/all-incomplete").send().await.unwrap();
    response.assert_success();

    let body: ApiResponse<Vec<serde_json::Value>> = response.json().await.unwrap();
    let tasks = &body.data;

    // 应该只有 task1 和 task2
    assert_eq!(tasks.len(), 2);
    assert!(tasks.iter().any(|t| t["id"] == task1_id.to_string()));
    assert!(tasks.iter().any(|t| t["id"] == task2_id.to_string()));
    assert!(!tasks.iter().any(|t| t["id"] == task3_id.to_string()));
}

// ==================== ALL 视图测试 ====================

#[tokio::test]
async fn should_return_all_tasks_including_completed() {
    let app = TestApp::new().await;

    let task1_id = TestFixtures::create_task(&app, "未完成").await;
    let task2_id = TestFixtures::create_task(&app, "已完成").await;

    // 完成 task2
    app.post(&format!("/tasks/{}/completion", task2_id))
        .send()
        .await
        .unwrap();

    // 查询 all
    let response = app.get("/views/all").send().await.unwrap();
    response.assert_success();

    let body: ApiResponse<Vec<serde_json::Value>> = response.json().await.unwrap();
    let tasks = &body.data;

    // 应该都存在
    assert_eq!(tasks.len(), 2);
    assert!(tasks.iter().any(|t| t["id"] == task1_id.to_string()));
    assert!(tasks.iter().any(|t| t["id"] == task2_id.to_string()));
}

#[tokio::test]
async fn should_return_empty_when_no_tasks() {
    let app = TestApp::new().await;

    let response = app.get("/views/all").send().await.unwrap();
    let body: ApiResponse<Vec<serde_json::Value>> = response.json().await.unwrap();

    assert_eq!(body.data.len(), 0);
}

// ==================== 视图一致性测试 ====================

#[tokio::test]
async fn should_maintain_consistency_across_views() {
    let app = TestApp::new().await;

    // 创建 3 个任务
    let staging_id = TestFixtures::create_task(&app, "Staging").await;
    let planned_id = TestFixtures::create_task(&app, "Planned").await;
    let completed_id = TestFixtures::create_task(&app, "Completed").await;

    // 排期 planned
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);
    app.post("/time-blocks/from-task")
        .json(&json!({
            "task_id": planned_id,
            "start_time": start,
            "end_time": end
        }))
        .send()
        .await
        .unwrap();

    // 完成 completed
    app.post(&format!("/tasks/{}/completion", completed_id))
        .send()
        .await
        .unwrap();

    // 验证各视图
    let all_res = app.get("/views/all").send().await.unwrap();
    let all: ApiResponse<Vec<serde_json::Value>> = all_res.json().await.unwrap();
    assert_eq!(all.data.len(), 3);

    let incomplete_res = app.get("/views/all-incomplete").send().await.unwrap();
    let incomplete: ApiResponse<Vec<serde_json::Value>> = incomplete_res.json().await.unwrap();
    assert_eq!(incomplete.data.len(), 2);

    let staging_res = app.get("/views/staging").send().await.unwrap();
    let staging: ApiResponse<Vec<serde_json::Value>> = staging_res.json().await.unwrap();
    assert_eq!(staging.data.len(), 1);
    assert_eq!(staging.data[0]["id"], staging_id.to_string());

    let planned_res = app.get("/views/planned").send().await.unwrap();
    let planned: ApiResponse<Vec<serde_json::Value>> = planned_res.json().await.unwrap();
    assert_eq!(planned.data.len(), 1);
    assert_eq!(planned.data[0]["id"], planned_id.to_string());
}
