/// Areas CRUD 端到端测试
use reqwest::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::common::{ApiResponse, ResponseAssertions, TestApp, TestData, TestFixtures};

// ==================== CREATE 测试 ====================

#[tokio::test]
async fn should_create_area_with_valid_payload() {
    let app = TestApp::new().await;

    let response = app
        .post("/areas")
        .json(&TestData::area_payload("工作", "#4A90E2"))
        .send()
        .await
        .unwrap();

    response.assert_created();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let area = body.data;

    assert_eq!(area["name"], "工作");
    assert_eq!(area["color"], "#4A90E2");
    assert!(area["id"].as_str().is_some());
}

#[tokio::test]
async fn should_reject_duplicate_area_name() {
    let app = TestApp::new().await;

    // 创建第一个
    app.post("/areas")
        .json(&TestData::area_payload("工作", "#4A90E2"))
        .send()
        .await
        .unwrap();

    // 尝试创建重名的
    let response = app
        .post("/areas")
        .json(&TestData::area_payload("工作", "#FF0000"))
        .send()
        .await
        .unwrap();

    response.assert_conflict();
}

#[tokio::test]
async fn should_reject_empty_name() {
    let app = TestApp::new().await;

    let response = app
        .post("/areas")
        .json(&json!({"name": "", "color": "#4A90E2"}))
        .send()
        .await
        .unwrap();

    response.assert_unprocessable();
}

#[tokio::test]
async fn should_reject_invalid_color_format() {
    let app = TestApp::new().await;

    let response = app
        .post("/areas")
        .json(&json!({"name": "测试", "color": "invalid"}))
        .send()
        .await
        .unwrap();

    response.assert_unprocessable();
}

#[tokio::test]
async fn should_create_area_with_parent() {
    let app = TestApp::new().await;

    let parent_id = TestFixtures::create_area(&app, "父区域", "#FF0000").await;

    let response = app
        .post("/areas")
        .json(&json!({
            "name": "子区域",
            "color": "#00FF00",
            "parent_area_id": parent_id
        }))
        .send()
        .await
        .unwrap();

    response.assert_created();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let area = body.data;

    assert_eq!(area["parent_area_id"], parent_id.to_string());
}

// ==================== READ 测试 ====================

#[tokio::test]
async fn should_list_all_areas() {
    let app = TestApp::new().await;

    TestFixtures::create_area(&app, "工作", "#4A90E2").await;
    TestFixtures::create_area(&app, "个人", "#FF5733").await;

    let response = app.get("/areas").send().await.unwrap();
    response.assert_success();

    let body: ApiResponse<Vec<serde_json::Value>> = response.json().await.unwrap();
    let areas = &body.data;

    assert_eq!(areas.len(), 2);
}

#[tokio::test]
async fn should_get_single_area() {
    let app = TestApp::new().await;
    let area_id = TestFixtures::create_area(&app, "测试区域", "#4A90E2").await;

    let response = app
        .get(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();

    response.assert_success();

    let body: ApiResponse<serde_json::Value> = response.json().await.unwrap();
    let area = body.data;

    assert_eq!(area["id"], area_id.to_string());
    assert_eq!(area["name"], "测试区域");
}

// ==================== UPDATE 测试 ====================

#[tokio::test]
async fn should_update_area_name() {
    let app = TestApp::new().await;
    let area_id = TestFixtures::create_area(&app, "旧名称", "#4A90E2").await;

    let response = app
        .patch(&format!("/areas/{}", area_id))
        .json(&json!({"name": "新名称"}))
        .send()
        .await
        .unwrap();

    response.assert_success();

    // 验证更新
    let get_res = app
        .get(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<serde_json::Value> = get_res.json().await.unwrap();
    let area = body.data;

    assert_eq!(area["name"], "新名称");
    assert_eq!(area["color"], "#4A90E2"); // 未修改字段保持不变
}

#[tokio::test]
async fn should_update_area_color() {
    let app = TestApp::new().await;
    let area_id = TestFixtures::create_area(&app, "区域", "#4A90E2").await;

    let response = app
        .patch(&format!("/areas/{}", area_id))
        .json(&json!({"color": "#FF5733"}))
        .send()
        .await
        .unwrap();

    response.assert_success();

    let get_res = app
        .get(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<serde_json::Value> = get_res.json().await.unwrap();
    let area = body.data;

    assert_eq!(area["color"], "#FF5733");
}

// ==================== DELETE 测试 ====================

#[tokio::test]
async fn should_soft_delete_area() {
    let app = TestApp::new().await;
    let area_id = TestFixtures::create_area(&app, "待删除", "#4A90E2").await;

    let response = app
        .delete(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();

    response.assert_success();

    // 验证软删除
    let get_res = app
        .get(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();
    get_res.assert_not_found();

    // 验证不在列表中
    let list_res = app.get("/areas").send().await.unwrap();
    let body: ApiResponse<Vec<serde_json::Value>> = list_res.json().await.unwrap();
    let areas = &body.data;

    assert!(!areas.iter().any(|a| a["id"] == area_id.to_string()));
}

#[tokio::test]
async fn should_be_idempotent_delete() {
    let app = TestApp::new().await;
    let area_id = TestFixtures::create_area(&app, "删除测试", "#4A90E2").await;

    // 第一次删除
    let res1 = app
        .delete(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();
    res1.assert_success();

    // 第二次删除（返回404是正常的，资源已不存在）
    let res2 = app
        .delete(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();
    // 幂等性的两种实现都可接受：200（成功）或 404（资源不存在）
    assert!(res2.status().is_success() || res2.status() == reqwest::StatusCode::NOT_FOUND);
}

// ==================== 完整 CRUD 流程测试 ====================

#[tokio::test]
async fn should_handle_complete_crud_lifecycle() {
    let app = TestApp::new().await;

    // 1. Create
    let create_res = app
        .post("/areas")
        .json(&TestData::area_payload("生命周期", "#4A90E2"))
        .send()
        .await
        .unwrap();
    create_res.assert_created();

    let created: ApiResponse<serde_json::Value> = create_res.json().await.unwrap();
    let area_id = created.data["id"].as_str().unwrap().to_string();

    // 2. Read
    let get_res = app
        .get(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();
    get_res.assert_success();

    // 3. Update
    let update_res = app
        .patch(&format!("/areas/{}", area_id))
        .json(&json!({"name": "更新后", "color": "#FF0000"}))
        .send()
        .await
        .unwrap();
    update_res.assert_success();

    // 4. Verify Update
    let verify_res = app
        .get(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();
    let body: ApiResponse<serde_json::Value> = verify_res.json().await.unwrap();
    assert_eq!(body.data["name"], "更新后");

    // 5. Delete
    let delete_res = app
        .delete(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();
    delete_res.assert_success();

    // 6. Verify Delete
    let final_res = app
        .get(&format!("/areas/{}", area_id))
        .send()
        .await
        .unwrap();
    final_res.assert_not_found();
}
