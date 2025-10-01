/// Tasks 生命周期测试（Complete/Reopen）
use reqwest::StatusCode;
use serde_json::json;

use crate::common::{ApiResponse, ResponseAssertions, TestApp, TestFixtures};

// ==================== COMPLETE 测试 ====================

#[tokio::test]
async fn should_complete_task() {
    let app = TestApp::new().await;
    let task_id = TestFixtures::create_task(&app, "待完成任务").await;

    let response = app
        .post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    response.assert_success();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let response_data = &body.data;
    let task = &response_data["task"];

    // TaskCardDto 不包含 completed_at，只验证 is_completed
    assert_eq!(task["is_completed"], true);
}

#[tokio::test]
async fn should_reject_already_completed_task() {
    let app = TestApp::new().await;
    let task_id = TestFixtures::create_task(&app, "任务").await;

    // 第一次完成
    app.post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    // 第二次完成应该失败
    let response = app
        .post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    response.assert_conflict();
}

#[tokio::test]
async fn should_return_404_for_nonexistent_task() {
    let app = TestApp::new().await;
    let fake_id = uuid::Uuid::new_v4();

    let response = app
        .post(&format!("/tasks/{}/completion", fake_id))
        .send()
        .await
        .unwrap();

    response.assert_not_found();
}

// ==================== REOPEN 测试 ====================

#[tokio::test]
async fn should_reopen_completed_task() {
    let app = TestApp::new().await;
    let task_id = TestFixtures::create_task(&app, "可重开任务").await;

    // 先完成
    app.post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    // 重新打开
    let response = app
        .delete(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    response.assert_success();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let response_data = &body.data;
    let task = &response_data["task"];

    // TaskCardDto 不包含 completed_at，只验证 is_completed
    assert_eq!(task["is_completed"], false);
}

#[tokio::test]
async fn should_reject_reopen_uncompleted_task() {
    let app = TestApp::new().await;
    let task_id = TestFixtures::create_task(&app, "未完成任务").await;

    let response = app
        .delete(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    response.assert_conflict();
}

#[tokio::test]
async fn should_return_task_to_staging_after_reopen() {
    let app = TestApp::new().await;
    let task_id = TestFixtures::create_task(&app, "回到 Staging").await;

    // 完成
    app.post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    // 验证不在 staging
    let staging_res = app.get("/views/staging").send().await.unwrap();
    let staging: ApiResponse<Vec<serde_json::Value>> = staging_res.json().await.unwrap();
    assert!(!staging.data.iter().any(|t| t["id"] == task_id.to_string()));

    // 重开
    app.delete(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    // 验证回到 staging
    let staging_res2 = app.get("/views/staging").send().await.unwrap();
    let staging2: ApiResponse<Vec<serde_json::Value>> = staging_res2.json().await.unwrap();
    assert!(staging2.data.iter().any(|t| t["id"] == task_id.to_string()));
}

// ==================== 完整生命周期测试 ====================

#[tokio::test]
async fn should_handle_complete_reopen_complete_cycle() {
    let app = TestApp::new().await;
    let task_id = TestFixtures::create_task(&app, "循环任务").await;

    // 第一次完成
    let res1 = app
        .post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();
    res1.assert_success();

    // 重开
    let res2 = app
        .delete(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();
    res2.assert_success();

    // 第二次完成
    let res3 = app
        .post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();
    res3.assert_success();

    // 验证最终状态
    let get_res = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<serde_json::Value> = get_res.json().await.unwrap();
    let task = body.data;

    assert_eq!(task["is_completed"], true);
}
