-- Add AI category to user_settings table

-- 1. 创建新表，包含新的 CHECK 约束（加入 'ai'）
CREATE TABLE user_settings_new (
    setting_key TEXT PRIMARY KEY NOT NULL,
    setting_value TEXT NOT NULL,
    value_type TEXT NOT NULL CHECK (value_type IN ('string', 'number', 'boolean', 'object', 'array')),
    category TEXT NOT NULL CHECK (category IN ('appearance', 'behavior', 'data', 'account', 'debug', 'system', 'ai')),
    updated_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- 2. 将旧表数据复制到新表
INSERT INTO user_settings_new (setting_key, setting_value, value_type, category, updated_at, created_at)
SELECT setting_key, setting_value, value_type, category, updated_at, created_at
FROM user_settings;

-- 3. 删除旧表
DROP TABLE user_settings;

-- 4. 将新表重命名为原表名
ALTER TABLE user_settings_new RENAME TO user_settings;

-- 5. 重新创建索引
CREATE INDEX idx_user_settings_category ON user_settings(category);
CREATE INDEX idx_user_settings_updated_at ON user_settings(updated_at);

