/// 测试数据库工具
use crate::shared::database::DatabaseConfig;
use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use tempfile::TempDir;

/// 测试数据库包装器
///
/// 自动管理临时数据库文件的生命周期
pub struct TestDb {
    pub pool: Pool<Sqlite>,
    #[allow(dead_code)]
    temp_dir: Arc<TempDir>, // 保持临时目录不被删除
}

impl TestDb {
    /// 获取数据库连接池
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// 清空所有表（用于测试间隔离）
    pub async fn clean(&self) -> Result<(), sqlx::Error> {
        // 删除所有数据，保留表结构
        sqlx::query("DELETE FROM task_time_block_links")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM task_recurrence_links")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM task_recurrences")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM templates")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM time_blocks")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM task_schedules")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM tasks").execute(&self.pool).await?;
        sqlx::query("DELETE FROM areas").execute(&self.pool).await?;
        sqlx::query("DELETE FROM view_preferences")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM event_outbox")
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/// 创建测试数据库
///
/// 自动执行 migrations 并返回 TestDb 实例
pub async fn create_test_db() -> Result<TestDb, Box<dyn std::error::Error>> {
    // 创建临时目录
    let temp_dir = Arc::new(tempfile::tempdir()?);
    let db_path = temp_dir.path().join("test.db");

    // 创建数据库连接
    let database_url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = SqlitePool::connect(&database_url).await?;

    // 运行 migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(TestDb { pool, temp_dir })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_db() {
        let test_db = create_test_db().await.unwrap();
        assert!(test_db.pool().acquire().await.is_ok());
    }

    #[tokio::test]
    async fn test_clean_db() {
        let test_db = create_test_db().await.unwrap();

        // 插入测试数据
        sqlx::query(
            "INSERT INTO areas (id, name, color, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind("test-area-id")
        .bind("Test Area")
        .bind("#FF0000")
        .bind(chrono::Utc::now())
        .bind(chrono::Utc::now())
        .execute(test_db.pool())
        .await
        .unwrap();

        // 验证数据存在
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM areas")
            .fetch_one(test_db.pool())
            .await
            .unwrap();
        assert_eq!(count.0, 1);

        // 清空数据库
        test_db.clean().await.unwrap();

        // 验证数据已清空
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM areas")
            .fetch_one(test_db.pool())
            .await
            .unwrap();
        assert_eq!(count.0, 0);
    }
}
