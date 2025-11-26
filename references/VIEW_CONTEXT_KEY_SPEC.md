# View Context Key 规范

## 📐 设计原则

Context Key 用于唯一标识一个视图上下文，作为排序配置的主键。

### 格式规范

```
{type}::{identifier}
```

---

## 📋 Context Key 类型定义

### **1. 杂项视图（Misc Views）**

无需额外标识符的固定视图

| 视图名称     | Context Key        | 说明                       |
| ------------ | ------------------ | -------------------------- |
| All 任务     | `misc::all`        | 所有任务（包括已完成）     |
| Staging 区   | `misc::staging`    | 未安排的任务（全部）       |
| Planned      | `misc::planned`    | 已安排的任务               |
| Incomplete   | `misc::incomplete` | 所有未完成任务             |
| Completed    | `misc::completed`  | 已完成任务                 |
| Deadline     | `misc::deadline`   | 即将到期的任务（7天内）    |
| Template     | `misc::template`   | 模板列表                   |
| 无项目任务池 | `misc::no-project` | 所有未分配到任何项目的任务 |

**Staging 扩展格式**（按区域筛选）：

| 视图名称              | Context Key 格式              | 说明                       |
| --------------------- | ----------------------------- | -------------------------- |
| 无区域 Staging        | `misc::staging::no-area`      | 未分配区域的 staging 任务  |
| 指定区域 Staging      | `misc::staging::{area_uuid}`  | 指定区域的 staging 任务    |

**示例**：

```javascript
// 全部 staging 任务
context_key: 'misc::staging'
sorted_task_ids: '["uuid-1", "uuid-2", "uuid-3"]'

// 无区域的 staging 任务
context_key: 'misc::staging::no-area'
sorted_task_ids: '["uuid-4", "uuid-5"]'

// 指定区域的 staging 任务
context_key: 'misc::staging::a1b2c3d4-1234-5678-90ab-cdef12345678'
sorted_task_ids: '["uuid-6", "uuid-7"]'

context_key: 'misc::deadline'
sorted_task_ids: '["uuid-8", "uuid-9"]'

context_key: 'misc::template'
sorted_task_ids: '["template-uuid-1", "template-uuid-2"]'
```

---

### **2. 日级视图（Daily Views）**

以日期为主维度的视图（可用于看板、列表或其它布局）

| 视图名称           | Context Key 格式                  | 说明                                               |
| ------------------ | --------------------------------- | -------------------------------------------------- |
| 每日全部任务       | `daily::{YYYY-MM-DD}`             | 指定日期的全部任务（含完成）                       |
| 每日未完成任务视图 | `daily::{YYYY-MM-DD}::incomplete` | 指定日期的未完成任务（如 Daily shutdown 未完成列） |
| 每日已完成任务视图 | `daily::{YYYY-MM-DD}::completed`  | 指定日期的已完成任务（如 Daily shutdown 已完成列） |

**示例**：

```javascript
// 当日全部任务
context_key: 'daily::2025-10-01'
sorted_task_ids: '["uuid-1", "uuid-2"]'

// 当日未完成任务
context_key: 'daily::2025-10-01::incomplete'
sorted_task_ids: '["uuid-3", "uuid-4"]'

// 当日已完成任务
context_key: 'daily::2025-10-01::completed'
sorted_task_ids: '["uuid-5", "uuid-6"]'
```

**日期格式**：

- 使用 ISO 8601 格式：`YYYY-MM-DD`
- UTC 时区
- 示例：`2025-10-01`, `2025-12-25`

---

### **3. 区域看板（Area Filter）**

按区域筛选的看板

| 视图名称 | Context Key 格式    | 说明           |
| -------- | ------------------- | -------------- |
| 区域筛选 | `area::{area_uuid}` | 指定区域的任务 |

**示例**：

```javascript
context_key: 'area::a1b2c3d4-1234-5678-90ab-cdef12345678'
sorted_task_ids: '["uuid-1", "uuid-2"]'
```

---

### **4. 项目看板（Project View）**

按项目筛选的看板

| 视图名称         | Context Key 格式                                | 说明                                          |
| ---------------- | ----------------------------------------------- | --------------------------------------------- |
| 项目看板（总览） | `project::{project_uuid}`                       | 指定项目的全部任务                            |
| 未分类任务列表   | `project::{project_uuid}::section::all`         | 某项目下未分配到任何章节的任务                |
| 指定章节任务列表 | `project::{project_uuid}::section::{sectionId}` | 某项目下特定章节的任务（`sectionId` 为 UUID） |

**示例**：

```javascript
context_key: 'project::proj-uuid-1234'
sorted_task_ids: '["uuid-1", "uuid-2"]'

context_key: 'project::proj-uuid-1234::section::all'
sorted_task_ids: '["uuid-3", "uuid-4"]'

context_key: 'project::proj-uuid-1234::section::sect-uuid-5678'
sorted_task_ids: '["uuid-5", "uuid-6"]'
```

---

### **5. Upcoming 视图（二维矩阵）**

按时间范围和任务类型组织的矩阵视图，用于 `/upcoming` 路由和右栏 Upcoming Panel

| 视图名称        | Context Key 格式                    | 说明                         |
| --------------- | ----------------------------------- | ---------------------------- |
| Upcoming 单元格 | `upcoming::{timeRange}::{taskType}` | 指定时间范围和任务类型的任务 |

**使用场景**：

- UpcomingView 页面（独立路由）
- UpcomingPanel 组件（右栏面板）

**时间范围（Time Range）**：

- `overdue` - 逾期
- `today` - 今日
- `thisWeek` - 本周
- `nextWeek` - 下周
- `thisMonth` - 本月
- `later` - 更远

**任务类型（Task Type）**：

- `dueDate` - 带截止日期的任务
- `recurrence` - 循环任务
- `scheduled` - 一般排期任务

**示例**：

```javascript
// 逾期的截止任务
context_key: 'upcoming::overdue::dueDate'
sorted_task_ids: '["uuid-1", "uuid-2"]'

// 今日的循环任务
context_key: 'upcoming::today::recurrence'
sorted_task_ids: '["uuid-3"]'

// 本周的排期任务
context_key: 'upcoming::thisWeek::scheduled'
sorted_task_ids: '["uuid-4", "uuid-5"]'

// 下周的截止任务
context_key: 'upcoming::nextWeek::dueDate'
sorted_task_ids: '["uuid-6"]'

// 本月的循环任务
context_key: 'upcoming::thisMonth::recurrence'
sorted_task_ids: '["uuid-7"]'

// 更远的排期任务
context_key: 'upcoming::later::scheduled'
sorted_task_ids: '["uuid-8"]'
```

**完整的 18 个单元格 Context Key**（6 时间范围 × 3 任务类型）：

```javascript
// 逾期
'upcoming::overdue::dueDate'
'upcoming::overdue::recurrence'
'upcoming::overdue::scheduled'

// 今日
'upcoming::today::dueDate'
'upcoming::today::recurrence'
'upcoming::today::scheduled'

// 本周
'upcoming::thisWeek::dueDate'
'upcoming::thisWeek::recurrence'
'upcoming::thisWeek::scheduled'

// 下周
'upcoming::nextWeek::dueDate'
'upcoming::nextWeek::recurrence'
'upcoming::nextWeek::scheduled'

// 本月
'upcoming::thisMonth::dueDate'
'upcoming::thisMonth::recurrence'
'upcoming::thisMonth::scheduled'

// 更远
'upcoming::later::dueDate'
'upcoming::later::recurrence'
'upcoming::later::scheduled'
```

---

### **6. 模板视图（Template View）**

模板列表和拖放策略

| 视图名称 | Context Key      | 说明                           |
| -------- | ---------------- | ------------------------------ |
| 模板列表 | `misc::template` | 通用模板列表（支持拖放到日历） |

**使用场景**：

- TemplateList 组件（右栏面板）
- TemplateKanbanColumn 组件
- 拖放策略：template-to-daily, daily-to-template, template-reorder

**示例**：

```javascript
context_key: 'misc::template'
sorted_task_ids: '["template-uuid-1", "template-uuid-2", "template-uuid-3"]'
```

---

### **7. 复合筛选（未来扩展）**

多个筛选条件组合

| 视图名称  | Context Key 格式                | 说明               |
| --------- | ------------------------------- | ------------------ |
| 日期+区域 | `daily::{date}::area::{uuid}`   | 某天某区域的任务   |
| 项目+区域 | `project::{uuid}::area::{uuid}` | 某项目某区域的任务 |

**示例**：

```javascript
context_key: 'daily::2025-10-01::area::a1b2c3d4'
sorted_task_ids: '["uuid-1"]'
```

---

## 🔧 前端实现

### **TypeScript 类型定义**

```typescript
// src/services/viewAdapter.ts
export type ViewContext =
  | {
      type: 'misc'
      id:
        | 'all'
        | 'staging'
        | 'planned'
        | 'incomplete'
        | 'completed'
        | 'deadline'
        | 'template'
        | 'no-project'
    }
  | { type: 'daily'; date: string } // YYYY-MM-DD
  | { type: 'area'; areaId: string }
  | { type: 'project'; projectId: string; sectionId?: 'all' | string }
  | {
      type: 'upcoming'
      timeRange: 'overdue' | 'today' | 'thisWeek' | 'nextWeek' | 'thisMonth' | 'later'
      taskType: 'dueDate' | 'recurrence' | 'scheduled'
    }
```

### **Context Key 生成函数**

```typescript
// src/stores/view.ts
function getContextKey(context: ViewContext): string {
  switch (context.type) {
    case 'misc':
      return `misc::${context.id}`
    case 'daily':
      return `daily::${context.date}`
    case 'area':
      return `area::${context.areaId}`
    case 'project':
      return context.sectionId
        ? `project::${context.projectId}::section::${context.sectionId}`
        : `project::${context.projectId}`
    case 'upcoming':
      return `upcoming::${context.timeRange}::${context.taskType}`
    default:
      throw new Error(`Unknown context type`)
  }
}
```

---

## 🗄️ 后端数据库 Schema

### **view_preferences 表**

```sql
CREATE TABLE view_preferences (
    context_key TEXT PRIMARY KEY NOT NULL,
    -- 示例：'misc::staging', 'daily::2025-10-01', 'area::uuid'

    sorted_task_ids TEXT NOT NULL,
    -- JSON 数组字符串：'["uuid1", "uuid2", "uuid3"]'

    updated_at TEXT NOT NULL
    -- UTC timestamp: '2025-10-01T10:00:00Z'
);

CREATE INDEX idx_view_prefs_updated ON view_preferences(updated_at);
```

---

## 🌐 API 端点设计

### **GET /view-preferences/:context_key**

获取指定视图的排序配置

**请求示例**：

```
GET /view-preferences/misc::staging
GET /view-preferences/daily::2025-10-01
GET /view-preferences/area::a1b2c3d4-1234-5678-90ab-cdef12345678
```

**响应**：

```json
{
  "data": {
    "context_key": "misc::staging",
    "sorted_task_ids": ["uuid-1", "uuid-2", "uuid-3"],
    "updated_at": "2025-10-01T10:00:00Z"
  }
}
```

---

### **PUT /view-preferences**

保存视图的排序配置

**请求**：

```json
{
  "context_key": "misc::staging",
  "sorted_task_ids": ["uuid-1", "uuid-2", "uuid-3"],
  "updated_at": "2025-10-01T10:00:00Z"
}
```

**响应**：

```json
{
  "data": {
    "context_key": "misc::staging",
    "sorted_task_ids": ["uuid-1", "uuid-2", "uuid-3"],
    "updated_at": "2025-10-01T10:00:00Z"
  }
}
```

---

## 📝 Context Key 示例

```javascript
// 杂项视图
'misc::all'
'misc::staging'
'misc::planned'
'misc::incomplete'
'misc::completed'
'misc::deadline'
'misc::template'
'misc::no-project'

// 日期看板
'daily::2025-10-01'
'daily::2025-10-02'
'daily::2025-12-25'

// 区域看板
'area::a1b2c3d4-1234-5678-90ab-cdef12345678'
'area::b2c3d4e5-5678-90ab-cdef-123456789abc'

// 项目看板
'project::proj-uuid-1234-5678-90ab'
'project::proj-uuid-5678-90ab-cdef'
'project::proj-uuid::section::all'
'project::proj-uuid::section::section-uuid'

// Upcoming 视图（18个单元格）
'upcoming::overdue::dueDate'
'upcoming::overdue::recurrence'
'upcoming::overdue::scheduled'
'upcoming::today::dueDate'
'upcoming::today::recurrence'
'upcoming::today::scheduled'
'upcoming::thisWeek::dueDate'
'upcoming::thisWeek::recurrence'
'upcoming::thisWeek::scheduled'
'upcoming::nextWeek::dueDate'
'upcoming::nextWeek::recurrence'
'upcoming::nextWeek::scheduled'
'upcoming::thisMonth::dueDate'
'upcoming::thisMonth::recurrence'
'upcoming::thisMonth::scheduled'
'upcoming::later::dueDate'
'upcoming::later::recurrence'
'upcoming::later::scheduled'

// 复合筛选（未来）
'daily::2025-10-01::area::a1b2c3d4'
'project::proj-uuid::area::a1b2c3d4'
```

---

## ✅ 验证规则

### **Context Key 必须满足**：

1. 只包含 ASCII 字符
2. 使用 `::` 作为分隔符
3. 第一段是类型（misc/daily/area/project）
4. UUID 使用完整格式（带连字符）
5. 日期使用 ISO 8601 格式（YYYY-MM-DD）

### **验证函数**

```typescript
function validateContextKey(key: string): boolean {
  const parts = key.split('::')
  if (parts.length < 2) return false

  const type = parts[0]
  if (!['misc', 'daily', 'area', 'project', 'upcoming'].includes(type)) {
    return false
  }

  // upcoming 类型需要 3 个部分
  if (type === 'upcoming') {
    if (parts.length !== 3) return false

    const timeRange = parts[1]
    const taskType = parts[2]

    const validTimeRanges = ['overdue', 'today', 'thisWeek', 'nextWeek', 'thisMonth', 'later']
    const validTaskTypes = ['dueDate', 'recurrence', 'scheduled']

    if (!validTimeRanges.includes(timeRange) || !validTaskTypes.includes(taskType)) {
      return false
    }
  }

  // misc 类型验证
  if (type === 'misc') {
    const validIds = [
      'all',
      'staging',
      'planned',
      'incomplete',
      'completed',
      'deadline',
      'template',
      'no-project',
    ]
    if (!validIds.includes(parts[1])) {
      return false
    }
  }

  // project 类型验证（支持章节视图）
  if (type === 'project') {
    const projectId = parts[1]
    if (!projectId) {
      return false
    }

    if (parts.length > 2) {
      if (parts[2] !== 'section' || parts.length !== 4) {
        return false
      }

      const sectionId = parts[3]
      if (sectionId !== 'all' && !isUuid(sectionId)) {
        return false
      }
    }
  }

  return true
}

function isUuid(value: string): boolean {
  return /^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$/.test(value)
}
```
