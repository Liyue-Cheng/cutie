use crate::entities::user_setting::{SettingCategory, UserSetting};
use crate::infra::core::{AppError, AppResult, DbError};
use sqlx::SqlitePool;

/// 用户设置 Repository
pub struct UserSettingRepository;

impl UserSettingRepository {
    /// 查询所有设置
    pub async fn find_all(pool: &SqlitePool) -> AppResult<Vec<UserSetting>> {
        let settings = sqlx::query_as::<_, UserSetting>(
            r#"
            SELECT setting_key, setting_value, value_type, category, updated_at, created_at
            FROM user_settings
            ORDER BY category, setting_key
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(DbError::QueryError(format!(
                "Failed to fetch all user settings: {}",
                e
            )))
        })?;

        Ok(settings)
    }

    /// 按 key 查询单个设置
    pub async fn find_by_key(pool: &SqlitePool, key: &str) -> AppResult<Option<UserSetting>> {
        let setting = sqlx::query_as::<_, UserSetting>(
            r#"
            SELECT setting_key, setting_value, value_type, category, updated_at, created_at
            FROM user_settings
            WHERE setting_key = ?
            "#,
        )
        .bind(key)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(DbError::QueryError(format!(
                "Failed to fetch user setting by key: {}",
                e
            )))
        })?;

        Ok(setting)
    }

    /// 按 category 查询设置
    pub async fn find_by_category(
        pool: &SqlitePool,
        category: SettingCategory,
    ) -> AppResult<Vec<UserSetting>> {
        let category_str = match category {
            SettingCategory::Appearance => "appearance",
            SettingCategory::Behavior => "behavior",
            SettingCategory::Data => "data",
            SettingCategory::Account => "account",
            SettingCategory::Debug => "debug",
            SettingCategory::System => "system",
        };

        let settings = sqlx::query_as::<_, UserSetting>(
            r#"
            SELECT setting_key, setting_value, value_type, category, updated_at, created_at
            FROM user_settings
            WHERE category = ?
            ORDER BY setting_key
            "#,
        )
        .bind(category_str)
        .fetch_all(pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(DbError::QueryError(format!(
                "Failed to fetch user settings by category: {}",
                e
            )))
        })?;

        Ok(settings)
    }

    /// UPSERT 单个设置 (插入或更新)
    pub async fn upsert(pool: &SqlitePool, setting: &UserSetting) -> AppResult<UserSetting> {
        let category_str = match setting.category {
            SettingCategory::Appearance => "appearance",
            SettingCategory::Behavior => "behavior",
            SettingCategory::Data => "data",
            SettingCategory::Account => "account",
            SettingCategory::Debug => "debug",
            SettingCategory::System => "system",
        };

        let value_type_str = match setting.value_type {
            crate::entities::user_setting::ValueType::String => "string",
            crate::entities::user_setting::ValueType::Number => "number",
            crate::entities::user_setting::ValueType::Boolean => "boolean",
            crate::entities::user_setting::ValueType::Object => "object",
            crate::entities::user_setting::ValueType::Array => "array",
        };

        sqlx::query(
            r#"
            INSERT INTO user_settings 
                (setting_key, setting_value, value_type, category, updated_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(setting_key) DO UPDATE SET
                setting_value = excluded.setting_value,
                value_type = excluded.value_type,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(&setting.setting_key)
        .bind(&setting.setting_value)
        .bind(value_type_str)
        .bind(category_str)
        .bind(setting.updated_at)
        .bind(setting.created_at)
        .execute(pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(DbError::QueryError(format!(
                "Failed to upsert user setting: {}",
                e
            )))
        })?;

        // 返回更新后的设置
        Self::find_by_key(pool, &setting.setting_key)
            .await?
            .ok_or_else(|| {
                AppError::DatabaseError(DbError::QueryError(
                    "Failed to fetch setting after upsert".to_string(),
                ))
            })
    }

    /// 批量 UPSERT 设置 (事务内)
    pub async fn upsert_batch(
        pool: &SqlitePool,
        settings: &[UserSetting],
    ) -> AppResult<Vec<UserSetting>> {
        let mut tx = pool.begin().await.map_err(|e| {
            AppError::DatabaseError(DbError::TransactionFailed {
                message: format!("Failed to start transaction: {}", e),
            })
        })?;

        for setting in settings {
            let category_str = match setting.category {
                SettingCategory::Appearance => "appearance",
                SettingCategory::Behavior => "behavior",
                SettingCategory::Data => "data",
                SettingCategory::Account => "account",
                SettingCategory::Debug => "debug",
                SettingCategory::System => "system",
            };

            let value_type_str = match setting.value_type {
                crate::entities::user_setting::ValueType::String => "string",
                crate::entities::user_setting::ValueType::Number => "number",
                crate::entities::user_setting::ValueType::Boolean => "boolean",
                crate::entities::user_setting::ValueType::Object => "object",
                crate::entities::user_setting::ValueType::Array => "array",
            };

            sqlx::query(
                r#"
                INSERT INTO user_settings 
                    (setting_key, setting_value, value_type, category, updated_at, created_at)
                VALUES (?, ?, ?, ?, ?, ?)
                ON CONFLICT(setting_key) DO UPDATE SET
                    setting_value = excluded.setting_value,
                    value_type = excluded.value_type,
                    updated_at = excluded.updated_at
                "#,
            )
            .bind(&setting.setting_key)
            .bind(&setting.setting_value)
            .bind(value_type_str)
            .bind(category_str)
            .bind(setting.updated_at)
            .bind(setting.created_at)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                AppError::DatabaseError(DbError::QueryError(format!(
                    "Failed to upsert setting in batch: {}",
                    e
                )))
            })?;
        }

        tx.commit().await.map_err(|e| {
            AppError::DatabaseError(DbError::TransactionFailed {
                message: format!("Failed to commit transaction: {}", e),
            })
        })?;

        // 返回所有更新后的设置
        let keys: Vec<String> = settings.iter().map(|s| s.setting_key.clone()).collect();
        let placeholders = keys.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            r#"
            SELECT setting_key, setting_value, value_type, category, updated_at, created_at
            FROM user_settings
            WHERE setting_key IN ({})
            "#,
            placeholders
        );

        let mut query_builder = sqlx::query_as::<_, UserSetting>(&query);
        for key in keys {
            query_builder = query_builder.bind(key);
        }

        let updated_settings = query_builder.fetch_all(pool).await.map_err(|e| {
            AppError::DatabaseError(DbError::QueryError(format!(
                "Failed to fetch settings after batch upsert: {}",
                e
            )))
        })?;

        Ok(updated_settings)
    }

    /// 删除所有设置 (用于重置)
    pub async fn delete_all(pool: &SqlitePool) -> AppResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_settings
            "#,
        )
        .execute(pool)
        .await
        .map_err(|e| {
            AppError::DatabaseError(DbError::QueryError(format!(
                "Failed to delete all user settings: {}",
                e
            )))
        })?;

        Ok(result.rows_affected())
    }
}
