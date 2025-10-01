/// 测试应用启动器
///
/// 为每个测试提供独立的 HTTP 服务器和数据库实例
use explore_lib::{
    config::AppConfig,
    startup::{AppState, Clock, IdGenerator, SystemClock, UuidV4Generator},
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::net::TcpListener;

/// 测试应用实例
pub struct TestApp {
    pub address: String,
    pub db_pool: SqlitePool,
    pub client: reqwest::Client,
}

impl TestApp {
    /// 创建新的测试应用实例
    ///
    /// 每个测试应该创建独立的 TestApp，确保测试隔离
    pub async fn new() -> Self {
        // 1. 创建内存数据库
        let db_pool = explore_lib::startup::database::create_test_database()
            .await
            .expect("Failed to create test database");

        // 2. 创建测试配置（使用默认配置）
        let config = AppConfig::default();

        // 3. 创建应用状态（使用生产环境的 Clock 和 IdGenerator）
        let app_state = AppState::new_production(config, db_pool.clone());

        // 4. 创建 HTTP 服务器（绑定到随机端口）
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to random port");
        let address = listener.local_addr().expect("Failed to get local address");

        // 5. 创建完整路由（包含 /api 前缀）
        use axum::Router;
        let api_router = explore_lib::features::create_api_router();
        let router = Router::new().nest("/api", api_router).with_state(app_state);

        // 6. 在后台启动服务器
        tokio::spawn(async move {
            axum::serve(listener, router)
                .await
                .expect("Failed to serve");
        });

        // 7. 等待服务器启动（简单延迟）
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let test_app = TestApp {
            address: format!("http://{}", address),
            db_pool,
            client: reqwest::Client::new(),
        };

        // 8. 验证服务器启动（尝试健康检查）
        eprintln!("Test server address: {}", test_app.address);

        test_app
    }

    /// GET 请求
    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.get(format!("{}/api{}", self.address, path))
    }

    /// POST 请求
    pub fn post(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.post(format!("{}/api{}", self.address, path))
    }

    /// PATCH 请求
    pub fn patch(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.patch(format!("{}/api{}", self.address, path))
    }

    /// DELETE 请求
    pub fn delete(&self, path: &str) -> reqwest::RequestBuilder {
        self.client.delete(format!("{}/api{}", self.address, path))
    }

    /// 重置数据库（删除所有数据）
    #[allow(dead_code)]
    pub async fn reset_db(&self) {
        sqlx::query("DELETE FROM task_time_block_links")
            .execute(&self.db_pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM task_schedules")
            .execute(&self.db_pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM orderings")
            .execute(&self.db_pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM time_blocks")
            .execute(&self.db_pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM tasks")
            .execute(&self.db_pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM areas")
            .execute(&self.db_pool)
            .await
            .unwrap();
    }
}

/// API 响应通用结构（匹配后端格式）
#[derive(Debug, serde::Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub timestamp: String,
    pub request_id: Option<String>,
}
