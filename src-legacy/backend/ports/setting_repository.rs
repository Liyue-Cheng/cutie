use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// use uuid::Uuid; // 暂时不需要

use crate::common::error::DbError;

/// 设置项结构
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Setting {
    /// 设置键
    pub key: String,
    /// 设置值
    pub value: String,
    /// 设置描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Setting {
    /// 创建新的设置项
    pub fn new(key: String, value: String, created_at: DateTime<Utc>) -> Self {
        Self {
            key,
            value,
            description: None,
            created_at,
            updated_at: created_at,
        }
    }

    /// 创建带描述的设置项
    pub fn with_description(
        key: String,
        value: String,
        description: String,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            key,
            value,
            description: Some(description),
            created_at,
            updated_at: created_at,
        }
    }

    /// 更新值
    pub fn update_value(&mut self, new_value: String, updated_at: DateTime<Utc>) {
        self.value = new_value;
        self.updated_at = updated_at;
    }
}

/// 设置仓库接口定义
///
/// **预期行为简介:** 抽象化对用户设置的持久化存储和读取操作。
/// 这是对"设置"这一特殊外部依赖的封装。
///
/// ## 已知适配器
/// - TomlSettingRepository: V1.0生产适配器，其内部实现对settings.toml文件的读写操作
/// - (未来) SqlxSettingRepository: 未来云同步版本的适配器，实现对数据库settings表的读写
/// - InMemorySettingRepository: 测试适配器，使用HashMap在内存中模拟设置的存储
#[async_trait]
pub trait SettingRepository: Send + Sync {
    /// 获取单个设置项
    ///
    /// **预期行为简介:** 根据键获取对应的设置项
    /// **输入输出规范:**
    /// - **前置条件:** key不能为空
    /// - **后置条件:** 如果设置项存在，返回Some(Setting)；否则返回None
    /// **边界情况:** key不存在时返回None而不是错误
    async fn get_setting(&self, key: &str) -> Result<Option<Setting>, DbError>;

    /// 获取所有设置项
    ///
    /// **预期行为简介:** 获取所有存储的设置项
    /// **输入输出规范:**
    /// - **前置条件:** 无
    /// - **后置条件:** 返回所有设置项的向量，如果没有设置项则返回空向量
    async fn get_all_settings(&self) -> Result<Vec<Setting>, DbError>;

    /// 创建或更新设置项
    ///
    /// **预期行为简介:** 创建或更新一个设置项。如果具有相同key的设置已存在，
    /// 则更新其value和updated_at；如果不存在，则创建
    /// **输入输出规范:**
    /// - **前置条件:** 输入的Setting对象的key不能为空
    /// - **后置条件:** 操作成功后，必须返回一个包含最终写入状态（包括更新后的updated_at）的Setting对象。
    ///   对该key的后续get_setting调用必须能返回这个新状态
    /// **边界情况:**
    /// - 并发写入：当两个操作同时upsert同一个key时，最终结果应是"最后写入者获胜"，且存储状态不能损坏
    async fn upsert_setting(&self, setting: &Setting) -> Result<Setting, DbError>;

    /// 删除设置项
    ///
    /// **预期行为简介:** 删除指定键的设置项
    /// **输入输出规范:**
    /// - **前置条件:** key不能为空
    /// - **后置条件:** 成功时不返回内容。该key的后续get_setting调用必须返回None
    /// **边界情况:** 如果key不存在，操作应直接成功返回，不产生任何影响
    async fn delete_setting(&self, key: &str) -> Result<(), DbError>;

    /// 批量获取设置项
    ///
    /// **预期行为简介:** 根据键列表批量获取设置项
    async fn get_settings_by_keys(&self, keys: &[String]) -> Result<Vec<Setting>, DbError>;

    /// 检查设置项是否存在
    ///
    /// **预期行为简介:** 检查指定键的设置项是否存在
    async fn exists(&self, key: &str) -> Result<bool, DbError>;

    /// 获取设置项数量
    ///
    /// **预期行为简介:** 获取存储的设置项总数
    async fn count(&self) -> Result<usize, DbError>;
}

/// 内存设置仓库适配器（测试用）
#[derive(Debug)]
pub struct InMemorySettingRepository {
    settings: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, Setting>>>,
}

impl InMemorySettingRepository {
    pub fn new() -> Self {
        Self {
            settings: std::sync::Arc::new(tokio::sync::RwLock::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// 预设一些测试设置
    pub async fn with_test_data() -> Self {
        let repo = Self::new();
        let now = Utc::now();

        let settings = vec![
            Setting::with_description(
                "app.theme".to_string(),
                "dark".to_string(),
                "应用主题设置".to_string(),
                now,
            ),
            Setting::with_description(
                "app.language".to_string(),
                "zh-CN".to_string(),
                "应用语言设置".to_string(),
                now,
            ),
            Setting::new("app.auto_save".to_string(), "true".to_string(), now),
        ];

        for setting in settings {
            let _ = repo.upsert_setting(&setting).await;
        }

        repo
    }
}

impl Default for InMemorySettingRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SettingRepository for InMemorySettingRepository {
    async fn get_setting(&self, key: &str) -> Result<Option<Setting>, DbError> {
        let settings = self.settings.read().await;
        Ok(settings.get(key).cloned())
    }

    async fn get_all_settings(&self) -> Result<Vec<Setting>, DbError> {
        let settings = self.settings.read().await;
        Ok(settings.values().cloned().collect())
    }

    async fn upsert_setting(&self, setting: &Setting) -> Result<Setting, DbError> {
        if setting.key.is_empty() {
            return Err(DbError::ConstraintViolation {
                message: "Setting key cannot be empty".to_string(),
            });
        }

        let mut settings = self.settings.write().await;
        let updated_setting = if let Some(existing) = settings.get(&setting.key) {
            // 更新现有设置
            let mut updated = existing.clone();
            updated.value = setting.value.clone();
            updated.updated_at = Utc::now();
            if setting.description.is_some() {
                updated.description = setting.description.clone();
            }
            updated
        } else {
            // 创建新设置
            setting.clone()
        };

        settings.insert(setting.key.clone(), updated_setting.clone());
        Ok(updated_setting)
    }

    async fn delete_setting(&self, key: &str) -> Result<(), DbError> {
        let mut settings = self.settings.write().await;
        settings.remove(key);
        Ok(())
    }

    async fn get_settings_by_keys(&self, keys: &[String]) -> Result<Vec<Setting>, DbError> {
        let settings = self.settings.read().await;
        let mut result = Vec::new();

        for key in keys {
            if let Some(setting) = settings.get(key) {
                result.push(setting.clone());
            }
        }

        Ok(result)
    }

    async fn exists(&self, key: &str) -> Result<bool, DbError> {
        let settings = self.settings.read().await;
        Ok(settings.contains_key(key))
    }

    async fn count(&self) -> Result<usize, DbError> {
        let settings = self.settings.read().await;
        Ok(settings.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_setting_repository_basic_operations() {
        let repo = InMemorySettingRepository::new();
        let now = Utc::now();

        let setting = Setting::new("test.key".to_string(), "test_value".to_string(), now);

        // 测试创建
        let created = repo.upsert_setting(&setting).await.unwrap();
        assert_eq!(created.key, "test.key");
        assert_eq!(created.value, "test_value");

        // 测试获取
        let retrieved = repo.get_setting("test.key").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "test_value");

        // 测试存在性检查
        let exists = repo.exists("test.key").await.unwrap();
        assert!(exists);

        let not_exists = repo.exists("nonexistent.key").await.unwrap();
        assert!(!not_exists);
    }

    #[tokio::test]
    async fn test_in_memory_setting_repository_update() {
        let repo = InMemorySettingRepository::new();
        let now = Utc::now();

        let setting = Setting::new("test.key".to_string(), "initial_value".to_string(), now);
        repo.upsert_setting(&setting).await.unwrap();

        // 更新值
        let updated_setting =
            Setting::new("test.key".to_string(), "updated_value".to_string(), now);
        let result = repo.upsert_setting(&updated_setting).await.unwrap();

        assert_eq!(result.value, "updated_value");
        assert!(result.updated_at >= now);

        // 验证更新
        let retrieved = repo.get_setting("test.key").await.unwrap().unwrap();
        assert_eq!(retrieved.value, "updated_value");
    }

    #[tokio::test]
    async fn test_in_memory_setting_repository_delete() {
        let repo = InMemorySettingRepository::new();
        let now = Utc::now();

        let setting = Setting::new("test.key".to_string(), "test_value".to_string(), now);
        repo.upsert_setting(&setting).await.unwrap();

        // 确认存在
        let exists_before = repo.exists("test.key").await.unwrap();
        assert!(exists_before);

        // 删除
        repo.delete_setting("test.key").await.unwrap();

        // 确认已删除
        let exists_after = repo.exists("test.key").await.unwrap();
        assert!(!exists_after);

        let retrieved = repo.get_setting("test.key").await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_setting_repository_get_all() {
        let repo = InMemorySettingRepository::new();
        let now = Utc::now();

        let settings = vec![
            Setting::new("key1".to_string(), "value1".to_string(), now),
            Setting::new("key2".to_string(), "value2".to_string(), now),
            Setting::new("key3".to_string(), "value3".to_string(), now),
        ];

        for setting in &settings {
            repo.upsert_setting(setting).await.unwrap();
        }

        let all_settings = repo.get_all_settings().await.unwrap();
        assert_eq!(all_settings.len(), 3);

        let count = repo.count().await.unwrap();
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn test_in_memory_setting_repository_batch_get() {
        let repo = InMemorySettingRepository::new();
        let now = Utc::now();

        let settings = vec![
            Setting::new("key1".to_string(), "value1".to_string(), now),
            Setting::new("key2".to_string(), "value2".to_string(), now),
            Setting::new("key3".to_string(), "value3".to_string(), now),
        ];

        for setting in &settings {
            repo.upsert_setting(setting).await.unwrap();
        }

        let keys = vec![
            "key1".to_string(),
            "key3".to_string(),
            "nonexistent".to_string(),
        ];
        let batch_result = repo.get_settings_by_keys(&keys).await.unwrap();

        assert_eq!(batch_result.len(), 2); // 只有key1和key3存在
        assert!(batch_result.iter().any(|s| s.key == "key1"));
        assert!(batch_result.iter().any(|s| s.key == "key3"));
        assert!(!batch_result.iter().any(|s| s.key == "nonexistent"));
    }

    #[tokio::test]
    async fn test_in_memory_setting_repository_empty_key_error() {
        let repo = InMemorySettingRepository::new();
        let now = Utc::now();

        let setting = Setting::new("".to_string(), "value".to_string(), now);
        let result = repo.upsert_setting(&setting).await;

        assert!(result.is_err());
        if let Err(DbError::ConstraintViolation { message }) = result {
            assert!(message.contains("empty"));
        } else {
            panic!("Expected ConstraintViolation error");
        }
    }

    #[tokio::test]
    async fn test_in_memory_setting_repository_with_test_data() {
        let repo = InMemorySettingRepository::with_test_data().await;

        let count = repo.count().await.unwrap();
        assert_eq!(count, 3);

        let theme = repo.get_setting("app.theme").await.unwrap();
        assert!(theme.is_some());
        assert_eq!(theme.unwrap().value, "dark");
    }
}
