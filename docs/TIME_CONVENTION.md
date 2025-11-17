# Cutie 项目时间处理规范

## 📋 目录

- [核心原则](#核心原则)
- [时间模型](#时间模型)
- [API 规范](#api-规范)
- [前端实现指南](#前端实现指南)
- [常见错误](#常见错误)
- [迁移指南](#迁移指南)

---

## 核心原则

### 1. 后端使用"本地时间模型"

**后端存储和处理所有用户意图相关的时间时，均使用本地时间（无时区偏移）。**

- ✅ **用户意图时间**：任务截止日期、日程安排、时间块开始/结束时间
- ❌ **系统元数据**：记录创建时间（`created_at`）、更新时间（`updated_at`）

### 2. UTC 仅用于系统内部

**UTC 时间仅用于系统内部元数据，不涉及用户业务逻辑。**

- ✅ 数据库审计字段（`created_at`, `updated_at`）
- ✅ 日志时间戳
- ✅ 性能监控、错误追踪
- ❌ 任务时间、日程时间、时间块时间

### 3. 前端与后端保持一致

**前端在与后端通信时，必须使用本地时间格式，避免时区转换。**

---

## 时间模型

### 用户意图时间（本地时间）

| 场景     | 格式                  | 示例                                                       | 说明                 |
| -------- | --------------------- | ---------------------------------------------------------- | -------------------- |
| 日期     | `YYYY-MM-DD`          | `2025-10-08`                                               | 纯日期，无时间部分   |
| 日期时间 | `YYYY-MM-DDTHH:mm:ss` | `2025-10-08T14:30:00`                                      | 本地时间，无时区后缀 |
| 时间范围 | 两个日期时间          | `start: 2025-10-08T09:00:00`<br>`end: 2025-10-08T10:00:00` | 开始和结束时间       |

**特点**：

- 无时区信息（无 `Z` 后缀，无 `+08:00` 等偏移）
- 表示用户所在时区的"墙上时钟时间"
- 后端按字面值存储和处理

### 系统元数据时间（UTC）

| 场景       | 格式                       | 示例                       | 说明                |
| ---------- | -------------------------- | -------------------------- | ------------------- |
| 创建时间   | `YYYY-MM-DDTHH:mm:ss.sssZ` | `2025-10-08T06:30:00.000Z` | UTC 时间，带 Z 后缀 |
| 更新时间   | `YYYY-MM-DDTHH:mm:ss.sssZ` | `2025-10-08T06:30:00.000Z` | UTC 时间，带 Z 后缀 |
| 日志时间戳 | `YYYY-MM-DDTHH:mm:ss.sssZ` | `2025-10-08T06:30:00.000Z` | UTC 时间，带 Z 后缀 |

**特点**：

- 带 `Z` 后缀，表示 UTC 时区
- 用于系统内部，不直接展示给用户
- 前端显示时需转换为本地时间

---

## API 规范

### 请求格式

#### ✅ 正确示例

```typescript
// 创建任务
POST /api/tasks
{
  "title": "开会",
  "due_date": {
    "date": "2025-10-08",           // 纯日期
    "time": "14:30:00"              // 可选的时间部分
  }
}

// 创建时间块
POST /api/timeblocks
{
  "title": "写代码",
  "start_time": "2025-10-08T09:00:00",  // 本地时间，无 Z
  "end_time": "2025-10-08T11:00:00",    // 本地时间，无 Z
  "is_all_day": false
}

// 创建日程
POST /api/schedules
{
  "task_id": "uuid",
  "scheduled_day": "2025-10-08"    // 纯日期
}
```

#### ❌ 错误示例

```typescript
// ❌ 不要使用 UTC 时间
POST /api/timeblocks
{
  "start_time": "2025-10-08T01:00:00.000Z",  // UTC，会导致时间偏移
  "end_time": "2025-10-08T03:00:00.000Z"
}

// ❌ 不要使用带时区偏移的时间
POST /api/timeblocks
{
  "start_time": "2025-10-08T09:00:00+08:00",  // 带时区，后端可能不支持
  "end_time": "2025-10-08T11:00:00+08:00"
}
```

### 响应格式

后端返回的数据中：

- **用户意图时间**：本地时间格式（无时区）
- **系统元数据**：UTC 时间格式（带 Z）

```typescript
// 响应示例
{
  "id": "uuid",
  "title": "开会",
  "start_time": "2025-10-08T14:30:00",     // 本地时间
  "end_time": "2025-10-08T15:30:00",       // 本地时间
  "created_at": "2025-10-08T06:30:00.000Z", // UTC 时间
  "updated_at": "2025-10-08T06:30:00.000Z"  // UTC 时间
}
```

---

## 前端实现指南

### 工具函数

**位置**：`src/infra/utils/dateUtils.ts`

#### 日期相关

```typescript
import {
  getTodayDateString, // 获取今天的日期（YYYY-MM-DD）
  toDateString, // Date 转 YYYY-MM-DD
  parseDateString, // YYYY-MM-DD 转 Date
  getYesterdayDateString, // 获取昨天的日期
  getTomorrowDateString, // 获取明天的日期
} from '@/infra/utils/dateUtils'

// ✅ 正确：获取今天的日期
const today = getTodayDateString() // "2025-10-08"

// ❌ 错误：使用 UTC 日期
const today = new Date().toISOString().split('T')[0] // 可能是昨天或明天
```

#### 时间相关

```typescript
import {
  toLocalISOString, // Date 转本地 ISO 字符串
  getNowLocalISOString, // 获取当前时间的本地 ISO 字符串
  parseLocalISOString, // 本地 ISO 字符串转 Date
  formatTime, // 格式化为 12 小时制（9:30 AM）
  formatTime24, // 格式化为 24 小时制（09:30）
  formatDateTime, // 格式化为可读日期时间
} from '@/infra/utils/dateUtils'

// ✅ 正确：获取本地时间字符串
const now = getNowLocalISOString() // "2025-10-08T14:30:00"

// ✅ 正确：Date 转本地时间字符串
const date = new Date()
const localTime = toLocalISOString(date) // "2025-10-08T14:30:00"

// ❌ 错误：使用 toISOString（会转换为 UTC）
const utcTime = date.toISOString() // "2025-10-08T06:30:00.000Z"（错误！）
```

#### 显示相关

```typescript
import {
  formatTime, // 12 小时制
  formatTime24, // 24 小时制
  formatDateTime, // 完整日期时间
  formatUtcToLocal, // UTC 转本地显示
  formatRelativeTime, // 相对时间（几分钟前）
} from '@/infra/utils/dateUtils'

// ✅ 显示时间
formatTime(new Date()) // "2:30 PM"
formatTime24(new Date()) // "14:30"
formatDateTime(new Date()) // "2025年10月8日 14:30"

// ✅ 显示系统元数据（UTC）
formatUtcToLocal('2025-10-08T06:30:00.000Z') // "2025年10月8日 14:30"
formatRelativeTime('2025-10-08T06:30:00.000Z') // "2 小时前"
```

### 常见场景

#### 1. 创建时间块

```typescript
// ✅ 正确
import { toLocalISOString } from '@/infra/utils/dateUtils'

const startTime = new Date() // 用户选择的开始时间
const endTime = new Date(startTime.getTime() + 60 * 60 * 1000) // 1小时后

await api.post('/timeblocks', {
  title: '开会',
  start_time: toLocalISOString(startTime), // "2025-10-08T14:30:00"
  end_time: toLocalISOString(endTime), // "2025-10-08T15:30:00"
  is_all_day: false,
})

// ❌ 错误
await api.post('/timeblocks', {
  start_time: startTime.toISOString(), // UTC 时间，会偏移！
  end_time: endTime.toISOString(),
})
```

#### 2. 日历事件拖拽

```typescript
// ✅ 正确
import { toLocalISOString } from '@/infra/utils/dateUtils'

function handleEventDrop(info: EventDropArg) {
  const newStart = info.event.start // Date 对象
  const newEnd = info.event.end

  await api.patch(`/timeblocks/${info.event.id}`, {
    start_time: toLocalISOString(newStart), // 本地时间
    end_time: toLocalISOString(newEnd),
  })
}

// ❌ 错误
function handleEventDrop(info: EventDropArg) {
  await api.patch(`/timeblocks/${info.event.id}`, {
    start_time: info.event.start.toISOString(), // UTC，错误！
    end_time: info.event.end.toISOString(),
  })
}
```

#### 3. 判断"今天"

```typescript
// ✅ 正确
import { getTodayDateString } from '@/infra/utils/dateUtils'

const today = getTodayDateString() // "2025-10-08"
const isToday = task.due_date?.date === today

// ❌ 错误
const today = new Date().toISOString().split('T')[0] // UTC 日期，可能错误
```

#### 4. FullCalendar 事件源

```typescript
// ✅ 正确：直接使用 Date 对象或本地时间字符串
import { toLocalISOString, parseLocalISOString } from '@/infra/utils/dateUtils'

const events: EventInput[] = timeBlocks.map((block) => ({
  id: block.id,
  title: block.title,
  start: parseLocalISOString(block.start_time), // 转为 Date 对象
  end: parseLocalISOString(block.end_time),
  allDay: block.is_all_day,
}))

// 或者，如果 FullCalendar 接受字符串（会自动解析为本地时间）
const events: EventInput[] = timeBlocks.map((block) => ({
  id: block.id,
  title: block.title,
  start: block.start_time, // "2025-10-08T14:30:00"（本地时间）
  end: block.end_time,
  allDay: block.is_all_day,
}))

// ❌ 错误：使用 toISOString
const events: EventInput[] = timeBlocks.map((block) => ({
  start: new Date(block.start_time).toISOString(), // 转成 UTC，错误！
  end: new Date(block.end_time).toISOString(),
}))
```

---

## 常见错误

### 1. 使用 `toISOString()` 处理用户时间

```typescript
// ❌ 错误
const now = new Date()
const timeString = now.toISOString() // "2025-10-08T06:30:00.000Z"（UTC）
// 发送给后端后，后端会理解为 UTC 时间，实际偏移 8 小时

// ✅ 正确
import { toLocalISOString } from '@/infra/utils/dateUtils'
const timeString = toLocalISOString(now) // "2025-10-08T14:30:00"（本地）
```

### 2. 使用 `toISOString().split('T')[0]` 判断日期

```typescript
// ❌ 错误
const today = new Date().toISOString().split('T')[0]
// 在 UTC+8 时区，凌晨 0:00-7:59 会得到昨天的日期

// ✅ 正确
import { getTodayDateString } from '@/infra/utils/dateUtils'
const today = getTodayDateString()
```

### 3. 直接解析本地时间字符串

```typescript
// ❌ 错误
const date = new Date('2025-10-08T14:30:00')
// 某些浏览器会当作 UTC 解析，某些会当作本地时间

// ✅ 正确
import { parseLocalISOString } from '@/infra/utils/dateUtils'
const date = parseLocalISOString('2025-10-08T14:30:00')
```

### 4. 混用 UTC 和本地时间

```typescript
// ❌ 错误
const start = new Date().toISOString() // UTC
const end = toLocalISOString(new Date()) // 本地
// 两个时间格式不一致，会导致混乱

// ✅ 正确：统一使用本地时间
const start = toLocalISOString(new Date())
const end = toLocalISOString(new Date())
```

---

## 迁移指南

### 第一步：识别问题代码

使用以下命令查找所有使用 `toISOString()` 的地方：

```bash
grep -r "toISOString()" src/
```

### 第二步：分类处理

#### A. 用户意图时间（需要修改）

- 任务、日程、时间块的时间字段
- 日历事件的开始/结束时间
- 日期判断（今天、昨天、明天）

**替换方案**：

```typescript
// 替换前
date.toISOString()
new Date().toISOString().split('T')[0]

// 替换后
import { toLocalISOString, getTodayDateString } from '@/infra/utils/dateUtils'
toLocalISOString(date)
getTodayDateString()
```

#### B. 系统元数据（保持不变）

- `created_at`, `updated_at` 字段
- 日志时间戳
- 性能监控数据

**保持使用 `toISOString()`**，但在显示时转换：

```typescript
// 存储/传输：保持 UTC
const timestamp = new Date().toISOString()

// 显示：转换为本地时间
import { formatUtcToLocal } from '@/infra/utils/dateUtils'
const displayTime = formatUtcToLocal(timestamp)
```

### 第三步：测试验证

1. **时区测试**：在不同时区测试（UTC+8, UTC+0, UTC-5）
2. **边界测试**：测试午夜前后的日期判断
3. **跨天测试**：测试跨天的时间块和任务

---

## 参考资料

- [MDN - Date](https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Global_Objects/Date)
- [ISO 8601](https://en.wikipedia.org/wiki/ISO_8601)
- [FullCalendar - Date Parsing](https://fullcalendar.io/docs/date-parsing)

---

## 更新日志

- **2025-11-15**：初始版本，定义项目时间处理规范



