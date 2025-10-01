/// 任务排期状态转换场景测试
///
/// 验证 schedule_status 的正确转换
use serde_json::json;

use crate::common::{helpers, ApiResponse, ResponseAssertions, TestApp, TestFixtures};

#[tokio::test]
async fn should_transition_from_staging_to_scheduled() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "状态转换").await;

    // 初始状态：staging
    let get_res1 = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();
    let task1: ApiResponse<serde_json::Value> = get_res1.json().await.unwrap();
    assert_eq!(task1.data["schedule_status"], "staging");

    // 拖到日历
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

    // 状态变为：scheduled
    let get_res2 = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();
    let task2: ApiResponse<serde_json::Value> = get_res2.json().await.unwrap();
    assert_eq!(task2.data["schedule_status"], "scheduled");
}

#[tokio::test]
async fn should_remain_scheduled_after_deleting_time_block() {
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

    // 验证任务仍然是 scheduled（Cutie 哲学：删除时间块 ≠ 取消排期）
    let get_res = app
        .get(&format!("/tasks/{}", task_id))
        .send()
        .await
        .unwrap();
    let task: ApiResponse<serde_json::Value> = get_res.json().await.unwrap();
    assert_eq!(task.data["schedule_status"], "scheduled");
}

#[tokio::test]
async fn should_return_to_staging_after_reopen() {
    let app = TestApp::new().await;

    let task_id = TestFixtures::create_task(&app, "重开回流").await;

    // 拖到日历
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

    // 完成任务
    app.post(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    // 重新打开
    app.delete(&format!("/tasks/{}/completion", task_id))
        .send()
        .await
        .unwrap();

    // 验证回到 staging
    let staging_res = app.get("/views/staging").send().await.unwrap();
    let staging: ApiResponse<Vec<serde_json::Value>> = staging_res.json().await.unwrap();
    let staging_tasks = &staging.data;
    assert!(staging_tasks.iter().any(|t| t["id"] == task_id.to_string()));
}

#[tokio::test]
async fn should_correctly_calculate_schedule_status_in_all_views() {
    let app = TestApp::new().await;

    // 创建 staging 任务
    let staging_id = TestFixtures::create_task(&app, "Staging").await;

    // 创建并排期任务
    let scheduled_id = TestFixtures::create_task(&app, "Scheduled").await;
    let start = helpers::test_time(1);
    let end = helpers::test_time(2);
    app.post("/time-blocks/from-task")
        .json(&json!({
            "task_id": scheduled_id,
            "start_time": start,
            "end_time": end
        }))
        .send()
        .await
        .unwrap();

    // 验证各个视图的 schedule_status
    let all_res = app.get("/views/all").send().await.unwrap();
    let all: ApiResponse<Vec<serde_json::Value>> = all_res.json().await.unwrap();

    let staging_task = all
        .data
        .iter()
        .find(|t| t["id"] == staging_id.to_string())
        .unwrap();
    assert_eq!(staging_task["schedule_status"], "staging");

    let scheduled_task = all
        .data
        .iter()
        .find(|t| t["id"] == scheduled_id.to_string())
        .unwrap();
    assert_eq!(scheduled_task["schedule_status"], "scheduled");
}
