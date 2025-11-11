-- ============================================================
-- 用户设置表 (User Settings) - V1.0
-- ============================================================
-- 存储用户的个性化配置
-- 采用 Key-Value 结构,每个设置项为一行记录

CREATE TABLE user_settings (
    -- 设置项的唯一标识符 (主键)
    setting_key TEXT PRIMARY KEY NOT NULL,
    
    -- 设置值 (JSON 格式存储,支持复杂数据类型)
    -- 示例:
    -- - 字符串: '"zh-CN"'
    -- - 数字: '100'
    -- - 布尔: 'true'
    -- - 对象: '{"format": "24h", "showSeconds": true}'
    setting_value TEXT NOT NULL,
    
    -- 设置项的数据类型 (用于前端反序列化)
    value_type TEXT NOT NULL CHECK (value_type IN ('string', 'number', 'boolean', 'object', 'array')),
    
    -- 设置项的分类 (用于UI分组显示)
    -- 'appearance' - 外观设置
    -- 'behavior' - 行为设置
    -- 'data' - 数据设置
    -- 'account' - 账号设置
    -- 'system' - 系统设置
    category TEXT NOT NULL CHECK (category IN ('appearance', 'behavior', 'data', 'account', 'system')),
    
    -- 最后更新时间 (UTC timestamp in RFC 3339 format)
    updated_at TEXT NOT NULL,
    
    -- 创建时间 (UTC timestamp in RFC 3339 format)
    created_at TEXT NOT NULL
);

-- 为常用查询创建索引
CREATE INDEX idx_user_settings_category ON user_settings(category);
CREATE INDEX idx_user_settings_updated_at ON user_settings(updated_at);

