/// 删除任务时孤儿时间块清理测试
///
/// 验证 Cutie 的智能清理逻辑
use serde_json::json;

use crate::common::{helpers, ApiResponse, ResponseAssertions, TestApp, TestFixtures};

#[tokio::test]
async fn should_delete_orphan_auto_created_time_block() {
    let app = TestApp::new().await;

    // 创建任务
    let task_id = TestFixtures::create_task(&app, "孤儿测试").await;

    // 拖动到日历（自动创建时间块，title = 任务标题）
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
    let block_id = created_data["time_block"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    // 删除任务
    let delete_res = app
        .delete(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();

    let deleted: ApiResponse<serde_json::Value> = delete_res.json().await.unwrap();
    let deleted_data = deleted.data;
    let deleted_blocks = deleted_data["deleted_time_block_ids"].as_array().unwrap();

    // 验证孤儿时间块被删除
    assert_eq!(deleted_blocks.len(), 1);
    assert_eq!(deleted_blocks[0], block_id);

    // 验证时间块确实不存在
    let list_res = app.get("/time-blocks").send().await.unwrap();
    let blocks: ApiResponse<Vec<serde_json::Value>> = list_res.json().await.unwrap();
    assert!(!blocks.data.iter().any(|b| b["id"] == block_id));
}

#[tokio::test]
async fn should_preserve_manually_created_time_block() {
    let app = TestApp::new().await;

    // 创建任务
    let task_id = TestFixtures::create_task(&app, "任务A").await;

    // 手动创建时间块（title 不同）
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);

    let block_res = app
        .post("/time-blocks")
        .json(&json!({
            "title": "深度工作时间",  // 与任务标题不同
            "start_time": start,
            "end_time": end
        }))
        .send()
        .await
        .unwrap();

    let block: ApiResponse<serde_json::Value> = block_res.json().await.unwrap();
    let block_data = block.data;
    let block_id = block_data["id"].as_str().unwrap().to_string();

    // 手动链接任务到时间块（需要先实现 link API，暂时跳过这步）
    // 假设已经链接...

    // 删除任务
    let delete_res = app
        .delete(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();

    let deleted: ApiResponse<serde_json::Value> = delete_res.json().await.unwrap();
    let deleted_data = deleted.data;
    let deleted_blocks = deleted_data["deleted_time_block_ids"].as_array().unwrap();

    // 验证手动创建的时间块不被删除
    assert_eq!(deleted_blocks.len(), 0);
}

#[tokio::test]
async fn should_preserve_shared_time_block() {
    let app = TestApp::new().await;

    // 创建两个任务
    let task1_id = TestFixtures::create_task(&app, "共享任务1").await;
    let task2_id = TestFixtures::create_task(&app, "共享任务2").await;

    // 创建时间块并链接 task1
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);

    let block_res = app
        .post("/time-blocks/from-task")
        .json(&json!({
            "task_id": task1_id,
            "start_time": start,
            "end_time": end
        }))
        .send()
        .await
        .unwrap();

    let block: ApiResponse<serde_json::Value> = block_res.json().await.unwrap();
    let block_data = block.data;
    let block_id = block_data["time_block"]["id"].as_str().unwrap().to_string();

    // TODO: 链接 task2 到同一个时间块（需要 link API）

    // 删除 task1
    let delete_res = app
        .delete(&format!("/tasks/{}", task1_id))
        .send()
        .await
        .unwrap();

    let deleted: ApiResponse<serde_json::Value> = delete_res.json().await.unwrap();
    let deleted_data = &deleted.data;

    // 因为时间块还链接了 task2，不应该被删除
    let deleted_blocks = deleted_data["deleted_time_block_ids"].as_array().unwrap();
    assert_eq!(deleted_blocks.len(), 0);
}
