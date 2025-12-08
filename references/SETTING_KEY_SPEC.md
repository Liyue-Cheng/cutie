# Setting Key 规范

## 📐 设计原则

Setting Key 用于唯一标识一个用户设置项，作为 `user_settings` 表的主键。

### 格式规范

```
{category}.{group?}.{name}
```

- **category**: 设置分类（必需）
- **group**: 设置分组（可选，用于逻辑分组）
- **name**: 设置名称（必需）

**分隔符**: `.`（点号）

---

## 📋 Setting Key 类型定义

### **1. 外观设置（Appearance）**

用户界面外观相关配置

| Setting Key | 默认值 | 类型 | 说明 |
| --- | --- | --- | --- |
| `appearance.theme` | `"business"` | string | 主题名称 |
| `appearance.language` | `"en"` | string | 界面语言 |
| `appearance.display_scale` | `100` | number | 显示缩放比例 (%) |

**示例**：

```javascript
// 主题设置
setting_key: 'appearance.theme'
setting_value: '"rose-pine"'
value_type: 'string'

// 语言设置
setting_key: 'appearance.language'
setting_value: '"zh-CN"'
value_type: 'string'

// 缩放比例
setting_key: 'appearance.display_scale'
setting_value: '125'
value_type: 'number'
```

---

### **2. AI 设置（AI）**

AI 功能相关配置，按模型用途分组

| Setting Key | 默认值 | 类型 | 说明 |
| --- | --- | --- | --- |
| `ai.conversation.api_base_url` | `""` | string | 对话模型 API 地址 |
| `ai.conversation.api_key` | `""` | string | 对话模型 API 密钥 |
| `ai.conversation.model` | `""` | string | 对话模型名称 |
| `ai.quick.api_base_url` | `""` | string | 快速模型 API 地址 |
| `ai.quick.api_key` | `""` | string | 快速模型 API 密钥 |
| `ai.quick.model` | `""` | string | 快速模型名称 |

**分组说明**：

- `conversation`: 用于长对话、复杂推理的模型
- `quick`: 用于快速响应、简单任务的模型

**示例**：

```javascript
// 对话模型配置
setting_key: 'ai.conversation.api_base_url'
setting_value: '"https://api.openai.com/v1"'
value_type: 'string'

setting_key: 'ai.conversation.api_key'
setting_value: '"sk-xxx..."'
value_type: 'string'

setting_key: 'ai.conversation.model'
setting_value: '"gpt-4"'
value_type: 'string'

// 快速模型配置
setting_key: 'ai.quick.model'
setting_value: '"gpt-3.5-turbo"'
value_type: 'string'
```

---

### **3. 行为设置（Behavior）**

应用行为相关配置

| Setting Key | 默认值 | 类型 | 说明 |
| --- | --- | --- | --- |
| `behavior.default_task_duration` | `30` | number | 默认任务时长（分钟） |
| `behavior.work_hours_start` | `"09:00"` | string | 工作时间开始 |
| `behavior.work_hours_end` | `"18:00"` | string | 工作时间结束 |

**示例**：

```javascript
// 默认任务时长
setting_key: 'behavior.default_task_duration'
setting_value: '45'
value_type: 'number'

// 工作时间
setting_key: 'behavior.work_hours_start'
setting_value: '"08:30"'
value_type: 'string'
```

---

### **4. 数据设置（Data）**

数据管理相关配置

| Setting Key | 默认值 | 类型 | 说明 |
| --- | --- | --- | --- |
| `data.auto_archive_days` | `30` | number | 自动归档天数 |

**示例**：

```javascript
setting_key: 'data.auto_archive_days'
setting_value: '60'
value_type: 'number'
```

---

### **5. 账户设置（Account）**

用户账户相关配置

| Setting Key | 默认值 | 类型 | 说明 |
| --- | --- | --- | --- |
| `account.user_name` | `""` | string | 用户名称 |
| `account.user_email` | `""` | string | 用户邮箱 |

**示例**：

```javascript
setting_key: 'account.user_name'
setting_value: '"Alice"'
value_type: 'string'
```

---

### **6. 调试设置（Debug）**

开发调试相关配置

| Setting Key | 默认值 | 类型 | 说明 |
| --- | --- | --- | --- |
| `debug.show_logs` | `false` | boolean | 显示日志面板 |
| `debug.log_level` | `"info"` | string | 日志级别 |
| `debug.test_string` | `"Hello World"` | string | 测试字符串 |
| `debug.test_number` | `42` | number | 测试整数 |
| `debug.test_float` | `3.14` | number | 测试浮点数 |
| `debug.test_boolean` | `false` | boolean | 测试布尔值 |
| `debug.test_toggle` | `true` | boolean | 测试开关 |

**示例**：

```javascript
setting_key: 'debug.show_logs'
setting_value: 'true'
value_type: 'boolean'

setting_key: 'debug.log_level'
setting_value: '"debug"'
value_type: 'string'
```

---

### **7. Internal 设置（Internal）**

以 `internal.` 开头的设置是**隐藏设置**，具有以下特点：

1. **不在设置面板显示** - 设置面板应过滤 `internal.*` 的设置
2. **通过 UI 交互自动保存** - 用户操作时自动持久化
3. **记住用户偏好** - 下次打开时恢复上次状态

#### **CalendarPanel 设置**（被 HomeView 和 CalendarView 共享）

| Setting Key | 默认值 | 类型 | 说明 |
| --- | --- | --- | --- |
| `internal.calendar.default_view_type` | `"month"` | string | 日历模式默认视图（week/month） |
| `internal.calendar.default_zoom` | `1` | number | 日历默认缩放（1/2/3） |
| `internal.calendar.month_filter.recurring` | `true` | boolean | 月视图显示循环任务 |
| `internal.calendar.month_filter.scheduled` | `true` | boolean | 月视图显示已排期任务 |
| `internal.calendar.month_filter.due_dates` | `true` | boolean | 月视图显示截止日期 |
| `internal.calendar.month_filter.all_day` | `true` | boolean | 月视图显示全天事件 |

#### **Home - RecentTaskPanel 设置**

| Setting Key | 默认值 | 类型 | 说明 |
| --- | --- | --- | --- |
| `internal.home.recent.default_days` | `3` | number | Recent 默认显示天数（1/3/5） |
| `internal.home.recent.show_completed` | `true` | boolean | 默认显示已完成任务 |
| `internal.home.recent.show_daily_recurring` | `true` | boolean | 默认显示每日循环任务 |

**示例**：

```javascript
// 日历视图类型
setting_key: 'internal.calendar.default_view_type'
setting_value: '"week"'
value_type: 'string'

// 月视图筛选
setting_key: 'internal.calendar.month_filter.recurring'
setting_value: 'false'
value_type: 'boolean'

// Recent 天数
setting_key: 'internal.home.recent.default_days'
setting_value: '5'
value_type: 'number'
```

---

## 🔧 前端实现

### **TypeScript 类型定义**

```typescript
// src/types/user-settings.ts

/**
 * 设置值类型
 */
export type ValueType = 'string' | 'number' | 'boolean' | 'object' | 'array'

/**
 * 用户设置 DTO
 */
export interface UserSettingDto {
  setting_key: string       // Setting Key
  setting_value: string     // JSON 字符串
  value_type: ValueType     // 值类型
  updated_at: string        // ISO 8601 UTC
  created_at: string        // ISO 8601 UTC
}

/**
 * 设置分类
 */
export type SettingCategory =
  | 'appearance'
  | 'ai'
  | 'behavior'
  | 'data'
  | 'account'
  | 'debug'
  | 'internal'  // 隐藏设置

/**
 * AI 设置分组
 */
export type AiSettingGroup = 'conversation' | 'quick'
```

### **Setting Key 解析函数**

```typescript
// src/utils/setting-key.ts

interface ParsedSettingKey {
  category: string
  group?: string
  name: string
}

/**
 * 解析 Setting Key
 */
function parseSettingKey(key: string): ParsedSettingKey {
  const parts = key.split('.')

  if (parts.length === 2) {
    return {
      category: parts[0],
      name: parts[1],
    }
  }

  if (parts.length === 3) {
    return {
      category: parts[0],
      group: parts[1],
      name: parts[2],
    }
  }

  throw new Error(`Invalid setting key format: ${key}`)
}

/**
 * 构建 Setting Key
 */
function buildSettingKey(category: string, name: string, group?: string): string {
  if (group) {
    return `${category}.${group}.${name}`
  }
  return `${category}.${name}`
}

/**
 * 获取分类下所有设置
 */
function getSettingsByCategory(
  settings: Map<string, UserSettingDto>,
  category: string
): UserSettingDto[] {
  return Array.from(settings.values())
    .filter(s => s.setting_key.startsWith(category + '.'))
}
```

---

## 🗄️ 后端数据库 Schema

### **user_settings 表**

```sql
CREATE TABLE user_settings (
    -- 设置项的唯一标识符 (主键)
    -- 格式: {category}.{group?}.{name}
    -- 示例: appearance.theme, ai.conversation.api_key
    setting_key TEXT PRIMARY KEY NOT NULL,

    -- 设置值 (JSON 格式存储)
    -- 字符串: '"value"'
    -- 数字: '42' 或 '3.14'
    -- 布尔: 'true' 或 'false'
    -- 对象: '{"key": "value"}'
    -- 数组: '["a", "b", "c"]'
    setting_value TEXT NOT NULL,

    -- 值类型标识
    -- 可选值: string, number, boolean, object, array
    value_type TEXT NOT NULL CHECK (value_type IN ('string', 'number', 'boolean', 'object', 'array')),

    -- 最后更新时间 (UTC timestamp in RFC 3339 format)
    updated_at TEXT NOT NULL,

    -- 创建时间 (UTC timestamp in RFC 3339 format)
    created_at TEXT NOT NULL
);

-- 为常用查询创建索引
CREATE INDEX idx_user_settings_updated_at ON user_settings(updated_at);
```

---

## 🌐 API 端点设计

### **GET /api/user-settings**

获取所有用户设置

**响应**：

```json
[
  {
    "setting_key": "appearance.theme",
    "setting_value": "\"business\"",
    "value_type": "string",
    "updated_at": "2025-01-11T12:00:00Z",
    "created_at": "2025-01-11T12:00:00Z"
  },
  {
    "setting_key": "ai.conversation.model",
    "setting_value": "\"gpt-4\"",
    "value_type": "string",
    "updated_at": "2025-01-11T12:00:00Z",
    "created_at": "2025-01-11T12:00:00Z"
  }
]
```

---

### **GET /api/user-settings/:key**

获取单个设置

**请求示例**：

```
GET /api/user-settings/appearance.theme
GET /api/user-settings/ai.conversation.api_key
```

**响应**：

```json
{
  "setting_key": "appearance.theme",
  "setting_value": "\"rose-pine\"",
  "value_type": "string",
  "updated_at": "2025-01-11T12:00:00Z",
  "created_at": "2025-01-11T12:00:00Z"
}
```

---

### **PUT /api/user-settings/:key**

更新单个设置

**请求**：

```json
{
  "value": "rose-pine",
  "value_type": "string"
}
```

**响应**：

```json
{
  "setting_key": "appearance.theme",
  "setting_value": "\"rose-pine\"",
  "value_type": "string",
  "updated_at": "2025-01-11T12:30:00Z",
  "created_at": "2025-01-11T12:00:00Z"
}
```

**SSE 事件**: `user_settings.updated`

---

### **PUT /api/user-settings/batch**

批量更新设置

**请求**：

```json
{
  "settings": [
    { "key": "appearance.theme", "value": "dark", "value_type": "string" },
    { "key": "appearance.display_scale", "value": 125, "value_type": "number" }
  ]
}
```

**响应**：

```json
{
  "updated_count": 2,
  "settings": [
    {
      "setting_key": "appearance.theme",
      "setting_value": "\"dark\"",
      "value_type": "string",
      "updated_at": "2025-01-11T12:30:00Z",
      "created_at": "2025-01-11T12:00:00Z"
    },
    {
      "setting_key": "appearance.display_scale",
      "setting_value": "125",
      "value_type": "number",
      "updated_at": "2025-01-11T12:30:00Z",
      "created_at": "2025-01-11T12:00:00Z"
    }
  ]
}
```

**SSE 事件**: `user_settings.batch_updated`

---

### **POST /api/user-settings/reset**

重置所有设置为默认值

**响应**：

```json
{
  "reset_count": 11,
  "settings": [
    // 所有默认设置...
  ]
}
```

**SSE 事件**: `user_settings.reset`

---

## 📝 Setting Key 完整列表

```javascript
// 外观设置
'appearance.theme'
'appearance.language'
'appearance.display_scale'

// AI 设置 - 对话模型
'ai.conversation.api_base_url'
'ai.conversation.api_key'
'ai.conversation.model'

// AI 设置 - 快速模型
'ai.quick.api_base_url'
'ai.quick.api_key'
'ai.quick.model'

// 行为设置
'behavior.default_task_duration'
'behavior.work_hours_start'
'behavior.work_hours_end'

// 数据设置
'data.auto_archive_days'

// 账户设置
'account.user_name'
'account.user_email'

// 调试设置
'debug.show_logs'
'debug.log_level'
'debug.test_string'
'debug.test_number'
'debug.test_float'
'debug.test_boolean'
'debug.test_toggle'

// Internal 设置（隐藏设置）
// CalendarPanel
'internal.calendar.default_view_type'
'internal.calendar.default_zoom'
'internal.calendar.month_filter.recurring'
'internal.calendar.month_filter.scheduled'
'internal.calendar.month_filter.due_dates'
'internal.calendar.month_filter.all_day'
// Home - RecentTaskPanel
'internal.home.recent.default_days'
'internal.home.recent.show_completed'
'internal.home.recent.show_daily_recurring'
```

---

## ✅ 验证规则

### **Setting Key 必须满足**：

1. 只包含小写字母、数字和下划线
2. 使用 `.` 作为分隔符
3. 至少包含 2 段（category.name）
4. 普通设置最多 3 段，internal 设置最多 5 段
5. 每段不能为空
6. category 必须是预定义的分类

### **验证函数**

```typescript
const VALID_CATEGORIES = [
  'appearance',
  'ai',
  'behavior',
  'data',
  'account',
  'debug',
  'internal',  // 隐藏设置
]

function validateSettingKey(key: string): boolean {
  // 基础格式检查
  if (!key || typeof key !== 'string') {
    return false
  }

  // 只允许小写字母、数字、下划线和点
  if (!/^[a-z0-9_.]+$/.test(key)) {
    return false
  }

  const parts = key.split('.')

  // 至少 2 段
  if (parts.length < 2) {
    return false
  }

  // 验证 category
  const category = parts[0]
  if (!VALID_CATEGORIES.includes(category)) {
    return false
  }

  // internal 设置最多 5 段，其他最多 3 段
  const maxParts = category === 'internal' ? 5 : 3
  if (parts.length > maxParts) {
    return false
  }

  // 每段不能为空
  if (parts.some(p => p.length === 0)) {
    return false
  }

  return true
}

function validateValueType(type: string): boolean {
  return ['string', 'number', 'boolean', 'object', 'array'].includes(type)
}

function validateSettingValue(value: string, type: ValueType): boolean {
  try {
    const parsed = JSON.parse(value)

    switch (type) {
      case 'string':
        return typeof parsed === 'string'
      case 'number':
        return typeof parsed === 'number' && !isNaN(parsed)
      case 'boolean':
        return typeof parsed === 'boolean'
      case 'object':
        return typeof parsed === 'object' && parsed !== null && !Array.isArray(parsed)
      case 'array':
        return Array.isArray(parsed)
      default:
        return false
    }
  } catch {
    return false
  }
}
```

---

## 🔄 SSE 事件

### **事件类型**

| 事件名 | 触发时机 | Payload |
| --- | --- | --- |
| `user_settings.updated` | 单个设置更新 | `UserSettingDto` |
| `user_settings.batch_updated` | 批量设置更新 | `{ settings: UserSettingDto[] }` |
| `user_settings.reset` | 设置重置 | `{ settings: UserSettingDto[] }` |

### **前端事件处理**

```typescript
// src/stores/user-settings/event-handlers.ts

interruptHandler.on('user_settings.updated', (event) => {
  const setting: UserSettingDto = event.payload
  core.addOrUpdateSetting_mut(setting)
})

interruptHandler.on('user_settings.batch_updated', (event) => {
  const settings: UserSettingDto[] = event.payload.settings
  core.addOrUpdateBatch_mut(settings)
})

interruptHandler.on('user_settings.reset', (event) => {
  const settings: UserSettingDto[] = event.payload.settings
  core.replaceAll_mut(settings)
})
```

---

## 🚀 扩展指南

### **添加新设置项**

1. **后端**: 在 `defaults.rs` 添加默认值定义
2. **前端**: 在 `core.ts` 添加快捷访问器（可选）
3. **文档**: 更新本规范文档

**示例 - 添加通知设置**:

```rust
// src-tauri/src/features/user_settings/shared/defaults.rs
DefaultSetting {
    key: "notification.enabled",
    value: "true",
    value_type: ValueType::Boolean,
},
DefaultSetting {
    key: "notification.sound",
    value: "\"default\"",
    value_type: ValueType::String,
},
```

```typescript
// src/stores/user-settings/core.ts
const notificationEnabled = computed(() =>
  getSettingValue('notification.enabled', true)
)
const notificationSound = computed(() =>
  getSettingValue('notification.sound', 'default')
)
```

### **添加新分类**

1. 更新 `VALID_CATEGORIES` 常量
2. 在本规范添加分类说明
3. 定义该分类下的设置项
