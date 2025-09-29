use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

use crate::common::error::DbError;
use crate::ports::{Setting, SettingRepository};

/// TOML设置文件结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlSettingsFile {
    /// 设置项映射
    #[serde(flatten)]
    pub settings: HashMap<String, TomlSetting>,
}

/// TOML设置项结构
#[derive(Debug, Clone, Serialize, Deserialize)]
struct TomlSetting {
    /// 设置值
    pub value: String,

    /// 设置描述
    pub description: Option<String>,

    /// 创建时间
    pub created_at: String,

    /// 更新时间
    pub updated_at: String,
}

/// TOML设置仓库适配器
///
/// **预期行为简介:** SettingRepository的TOML文件实现，用于V1.0的设置存储
///
/// ## 实现细节
/// - 使用TOML文件格式存储设置
/// - 支持并发读写（通过RwLock）
/// - 自动处理文件创建和目录创建
/// - 保持与SettingRepository接口的完全兼容
#[derive(Debug)]
pub struct TomlSettingRepository {
    /// TOML文件路径
    file_path: PathBuf,

    /// 内存缓存
    cache: RwLock<Option<TomlSettingsFile>>,
}

impl TomlSettingRepository {
    /// 创建新的TOML设置仓库
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            cache: RwLock::new(None),
        }
    }

    /// 加载TOML文件到内存
    async fn load_file(&self) -> Result<TomlSettingsFile, DbError> {
        // 检查缓存
        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache {
                return Ok(cached.clone());
            }
        }

        // 如果文件不存在，创建空的设置文件
        if !self.file_path.exists() {
            let empty_settings = TomlSettingsFile {
                settings: HashMap::new(),
            };

            self.save_file(&empty_settings).await?;

            // 更新缓存
            {
                let mut cache = self.cache.write().await;
                *cache = Some(empty_settings.clone());
            }

            return Ok(empty_settings);
        }

        // 读取文件内容
        let content = tokio::fs::read_to_string(&self.file_path)
            .await
            .map_err(|e| DbError::ConnectionError(sqlx::Error::Io(e)))?;

        // 解析TOML
        let settings: TomlSettingsFile = toml::from_str(&content).map_err(|e| {
            DbError::ConnectionError(sqlx::Error::Configuration(
                format!("Invalid TOML format: {}", e).into(),
            ))
        })?;

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            *cache = Some(settings.clone());
        }

        Ok(settings)
    }

    /// 保存设置到TOML文件
    async fn save_file(&self, settings: &TomlSettingsFile) -> Result<(), DbError> {
        // 确保目录存在
        if let Some(parent) = self.file_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| DbError::ConnectionError(sqlx::Error::Io(e)))?;
        }

        // 序列化为TOML
        let content = toml::to_string_pretty(settings).map_err(|e| {
            DbError::ConnectionError(sqlx::Error::Configuration(
                format!("Failed to serialize TOML: {}", e).into(),
            ))
        })?;

        // 写入文件
        tokio::fs::write(&self.file_path, content)
            .await
            .map_err(|e| DbError::ConnectionError(sqlx::Error::Io(e)))?;

        // 更新缓存
        {
            let mut cache = self.cache.write().await;
            *cache = Some(settings.clone());
        }

        Ok(())
    }

    /// 将TomlSetting转换为Setting
    fn toml_setting_to_setting(
        key: String,
        toml_setting: &TomlSetting,
    ) -> Result<Setting, DbError> {
        let created_at = DateTime::parse_from_rfc3339(&toml_setting.created_at)
            .map_err(|_| {
                DbError::ConnectionError(sqlx::Error::Configuration(
                    "Invalid created_at format".into(),
                ))
            })?
            .with_timezone(&Utc);

        let updated_at = DateTime::parse_from_rfc3339(&toml_setting.updated_at)
            .map_err(|_| {
                DbError::ConnectionError(sqlx::Error::Configuration(
                    "Invalid updated_at format".into(),
                ))
            })?
            .with_timezone(&Utc);

        Ok(Setting {
            key,
            value: toml_setting.value.clone(),
            description: toml_setting.description.clone(),
            created_at,
            updated_at,
        })
    }

    /// 将Setting转换为TomlSetting
    fn setting_to_toml_setting(setting: &Setting) -> TomlSetting {
        TomlSetting {
            value: setting.value.clone(),
            description: setting.description.clone(),
            created_at: setting.created_at.to_rfc3339(),
            updated_at: setting.updated_at.to_rfc3339(),
        }
    }

    /// 清除缓存
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
    }

    /// 强制重新加载文件
    pub async fn reload(&self) -> Result<(), DbError> {
        self.clear_cache().await;
        self.load_file().await?;
        Ok(())
    }
}

#[async_trait]
impl SettingRepository for TomlSettingRepository {
    async fn get_setting(&self, key: &str) -> Result<Option<Setting>, DbError> {
        let settings_file = self.load_file().await?;

        if let Some(toml_setting) = settings_file.settings.get(key) {
            let setting = Self::toml_setting_to_setting(key.to_string(), toml_setting)?;
            Ok(Some(setting))
        } else {
            Ok(None)
        }
    }

    async fn get_all_settings(&self) -> Result<Vec<Setting>, DbError> {
        let settings_file = self.load_file().await?;
        let mut settings = Vec::new();

        for (key, toml_setting) in &settings_file.settings {
            let setting = Self::toml_setting_to_setting(key.clone(), toml_setting)?;
            settings.push(setting);
        }

        // 按键名排序
        settings.sort_by(|a, b| a.key.cmp(&b.key));

        Ok(settings)
    }

    async fn upsert_setting(&self, setting: &Setting) -> Result<Setting, DbError> {
        if setting.key.is_empty() {
            return Err(DbError::ConstraintViolation {
                message: "Setting key cannot be empty".to_string(),
            });
        }

        let mut settings_file = self.load_file().await?;

        // 创建或更新设置
        let now = Utc::now();
        let updated_setting = if settings_file.settings.contains_key(&setting.key) {
            // 更新现有设置
            Setting {
                key: setting.key.clone(),
                value: setting.value.clone(),
                description: setting.description.clone(),
                created_at: settings_file.settings[&setting.key]
                    .created_at
                    .parse::<DateTime<chrono::FixedOffset>>()
                    .map_err(|_| {
                        DbError::ConnectionError(sqlx::Error::Configuration(
                            "Invalid created_at format".into(),
                        ))
                    })?
                    .with_timezone(&Utc),
                updated_at: now,
            }
        } else {
            // 创建新设置
            Setting {
                key: setting.key.clone(),
                value: setting.value.clone(),
                description: setting.description.clone(),
                created_at: now,
                updated_at: now,
            }
        };

        // 转换并保存
        let toml_setting = Self::setting_to_toml_setting(&updated_setting);
        settings_file
            .settings
            .insert(setting.key.clone(), toml_setting);

        self.save_file(&settings_file).await?;

        Ok(updated_setting)
    }

    async fn delete_setting(&self, key: &str) -> Result<(), DbError> {
        let mut settings_file = self.load_file().await?;

        settings_file.settings.remove(key);

        self.save_file(&settings_file).await?;

        Ok(())
    }

    async fn get_settings_by_keys(&self, keys: &[String]) -> Result<Vec<Setting>, DbError> {
        let settings_file = self.load_file().await?;
        let mut result = Vec::new();

        for key in keys {
            if let Some(toml_setting) = settings_file.settings.get(key) {
                let setting = Self::toml_setting_to_setting(key.clone(), toml_setting)?;
                result.push(setting);
            }
        }

        Ok(result)
    }

    async fn exists(&self, key: &str) -> Result<bool, DbError> {
        let settings_file = self.load_file().await?;
        Ok(settings_file.settings.contains_key(key))
    }

    async fn count(&self) -> Result<usize, DbError> {
        let settings_file = self.load_file().await?;
        Ok(settings_file.settings.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_toml_setting_repository_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("test_settings.toml");

        let repo = TomlSettingRepository::new(settings_path);
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
    async fn test_toml_setting_repository_update() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("test_settings.toml");

        let repo = TomlSettingRepository::new(settings_path);
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
    async fn test_toml_setting_repository_delete() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("test_settings.toml");

        let repo = TomlSettingRepository::new(settings_path);
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
    async fn test_toml_setting_repository_get_all() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("test_settings.toml");

        let repo = TomlSettingRepository::new(settings_path);
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
    async fn test_toml_setting_repository_batch_get() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("test_settings.toml");

        let repo = TomlSettingRepository::new(settings_path);
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
    async fn test_toml_setting_repository_empty_key_error() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("test_settings.toml");

        let repo = TomlSettingRepository::new(settings_path);
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
    async fn test_toml_setting_repository_file_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("test_settings.toml");

        let now = Utc::now();

        // 第一个仓库实例
        {
            let repo1 = TomlSettingRepository::new(settings_path.clone());
            let setting = Setting::new(
                "persistent.key".to_string(),
                "persistent_value".to_string(),
                now,
            );
            repo1.upsert_setting(&setting).await.unwrap();
        }

        // 第二个仓库实例（模拟重启）
        {
            let repo2 = TomlSettingRepository::new(settings_path.clone());
            let retrieved = repo2.get_setting("persistent.key").await.unwrap();

            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().value, "persistent_value");
        }

        // 验证文件确实存在
        assert!(settings_path.exists());
    }

    #[tokio::test]
    async fn test_toml_setting_repository_cache_invalidation() {
        let temp_dir = TempDir::new().unwrap();
        let settings_path = temp_dir.path().join("test_settings.toml");

        let repo = TomlSettingRepository::new(settings_path);
        let now = Utc::now();

        // 添加设置
        let setting = Setting::new("cache.test".to_string(), "initial".to_string(), now);
        repo.upsert_setting(&setting).await.unwrap();

        // 验证缓存工作
        let count1 = repo.count().await.unwrap();
        assert_eq!(count1, 1);

        // 清除缓存并重新加载
        repo.clear_cache().await;
        repo.reload().await.unwrap();

        // 验证数据仍然存在
        let count2 = repo.count().await.unwrap();
        assert_eq!(count2, 1);

        let retrieved = repo.get_setting("cache.test").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, "initial");
    }
}
