/// Tasks CRUD 端到端测试
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::common::{ApiResponse, ResponseAssertions, TestApp, TestData, TestFixtures};

// ==================== CREATE 测试 ====================

#[tokio::test]
async fn should_create_task_with_minimal_payload() {
    let app = TestApp::new().await;

    let response = app
        .post("/tasks")
        .json(&TestData::task_payload("新任务"))
        .send()
        .await
        .unwrap();

    response.assert_created();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let task = body.data;

    assert_eq!(task["title"], "新任务");
    assert_eq!(task["is_completed"], false);
    assert_eq!(task["schedule_status"], "staging");
    assert!(task["id"].as_str().is_some());
    assert!(task["sort_order"].as_str().is_some());
}

#[tokio::test]
async fn should_create_task_with_full_payload() {
    let app = TestApp::new().await;

    let area_id = TestFixtures::create_area(&app, "工作", "#4A90E2").await;

    let payload = json!({
        "title": "完整任务",
        "glance_note": "简要说明",
        "detail_note": "详细描述",
        "area_id": area_id,
        "project_id": null,
        "subtasks": [
            {"id": Uuid::new_v4(), "title": "子任务1", "is_completed": false, "sort_order": "0|a"}
        ]
    });

    let response = app.post("/tasks").json(&payload).send().await.unwrap();
    response.assert_created();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let task = body.data;

    assert_eq!(task["title"], "完整任务");
    assert_eq!(task["glance_note"], "简要说明");
    assert!(task["area"].is_object());
    assert_eq!(task["area"]["name"], "工作");
}

#[tokio::test]
async fn should_reject_empty_title() {
    let app = TestApp::new().await;

    let response = app
        .post("/tasks")
        .json(&json!({"title": ""}))
        .send()
        .await
        .unwrap();

    response.assert_unprocessable();
}

#[tokio::test]
async fn should_auto_assign_to_staging() {
    let app = TestApp::new().await;

    let response = app
        .post("/tasks")
        .json(&TestData::task_payload("Staging 任务"))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let task = body.data;

    assert_eq!(task["schedule_status"], "staging");
}

// ==================== READ 测试 ====================

#[tokio::test]
async fn should_get_existing_task() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "测试任务").await;

    let response = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();

    response.assert_success();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let task = body.data;

    // GET /tasks/:id 返回扁平化的 TaskDetailDto
    assert_eq!(task["id"], task_id.to_string());
    assert_eq!(task["title"], "测试任务");
}

#[tokio::test]
async fn should_return_404_for_missing_task() {
    let app = TestApp::new().await;

    let fake_id = Uuid::new_v4();
    let response = app
        .get(&format!("/tasks/{}", fake_id))
        .send()
        .await
        .unwrap();

    response.assert_not_found();
}

#[tokio::test]
async fn should_return_correct_schedule_status() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "状态测试").await;

    let response = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let task = body.data;

    // 新创建的任务应该是 staging 状态
    assert_eq!(task["schedule_status"], "staging");
}

// ==================== UPDATE 测试 ====================

#[tokio::test]
async fn should_update_single_field() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "原标题").await;

    let response = app
        .patch(&format!("/tasks/{}", task_id))
        .json(&json!({"title": "新标题"}))
        .send()
        .await
        .unwrap();

    response.assert_success();

    // 验证更新
    let get_response = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<serde_json::Value> = get_response.json().await.unwrap();
    let task = body.data;

    assert_eq!(task["title"], "新标题");
    assert_eq!(task["glance_note"], "测试笔记"); // 未修改字段保持不变
}

#[tokio::test]
async fn should_update_multiple_fields() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "原任务").await;

    let response = app
        .patch(&format!("/tasks/{}", task_id))
        .json(&json!({
            "title": "更新后的任务",
            "glance_note": "更新的笔记",
            "detail_note": "新的详细描述"
        }))
        .send()
        .await
        .unwrap();

    response.assert_success();

    // 验证所有字段都更新了
    let get_response = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<serde_json::Value> = get_response.json().await.unwrap();
    let task = body.data;

    assert_eq!(task["title"], "更新后的任务");
    assert_eq!(task["glance_note"], "更新的笔记");
    assert_eq!(task["detail_note"], "新的详细描述");
}

#[tokio::test]
async fn should_reject_empty_update() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "任务").await;

    let response = app
        .patch(&format!("/tasks/{}", task_id))
        .json(&json!({}))
        .send()
        .await
        .unwrap();

    response.assert_unprocessable();
}

// ==================== DELETE 测试 ====================

#[tokio::test]
async fn should_soft_delete_task() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "待删除").await;

    let response = app
        .delete(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();

    response.assert_success();

    // 验证任务不可见（软删除）
    let get_response = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();

    get_response.assert_not_found();
}

#[tokio::test]
async fn should_be_idempotent() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "删除测试").await;

    // 第一次删除
    let response1 = app
        .delete(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();
    response1.assert_success();

    // 第二次删除（幂等性）
    let response2 = app
        .delete(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();
    response2.assert_success();
}

// ==================== 写后读一致性测试 ====================

#[tokio::test]
async fn should_maintain_write_read_consistency() {
    let app = TestApp::new().await;

    let area_id = TestFixtures::create_area(&app, "个人", "#FF5733").await;

    let payload = json!({
        "title": "一致性测试",
        "glance_note": "简要",
        "detail_note": "详细内容",
        "area_id": area_id,
        "subtasks": [
            {"id": Uuid::new_v4(), "title": "Step 1", "is_completed": false, "sort_order": "0|a"}
        ]
    });

    // 写入
    let create_response = app.post("/tasks").json(&payload).send().await.unwrap();
    let created: ApiResponse<serde_json::Value> = create_response.json().await.unwrap();
    let created_data = created.data;
    let task_id = created_data["id"].as_str().unwrap();

    // 读取
    let get_response = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();
    let fetched: ApiResponse<serde_json::Value> = get_response.json().await.unwrap();
    let task = fetched.data;

    // 验证数据一致性（GET /tasks/:id 返回扁平化格式）
    assert_eq!(task["title"], "一致性测试");
    assert_eq!(task["glance_note"], "简要");
    assert_eq!(task["detail_note"], "详细内容");
    assert!(task["area"].is_object());
    assert_eq!(task["area"]["id"], area_id.to_string());
    assert_eq!(task["area"]["name"], "个人");
    assert!(task["subtasks"].is_array());

    let subtasks = task["subtasks"].as_array().unwrap();
    assert_eq!(subtasks.len(), 1);
}
