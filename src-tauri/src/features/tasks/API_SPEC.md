# Tasks API 规范文档

本文档按照 CABC 规范 2.0 定义 Tasks 功能模块的所有 API 端点。

---

## 端点清单

1. [POST /api/tasks](#post-apitasks) - 创建任务
2. [GET /api/tasks/:id](#get-apitasksid) - 获取任务详情
3. [PATCH /api/tasks/:id](#patch-apitasksid) - 更新任务
4. [DELETE /api/tasks/:id](#delete-apitasksid) - 删除任务
5. [POST /api/tasks/:id/completion](#post-apitasksidcompletion) - 完成任务
6. [DELETE /api/tasks/:id/completion](#delete-apitasksidcompletion) - 重新打开任务

---

## POST /api/tasks

### 1. 端点签名

`POST /api/tasks`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要快速创建一个新任务，以便立即捕捉想法，任务将出现在 Staging 区等待我安排。

#### 2.2. 核心业务逻辑

- 创建新的 Task 实体并插入数据库
- 自动在 staging 上下文中创建 Ordering 记录
- 使用 LexoRank 算法生成 sort_order
- 返回完整的 TaskCardDto

#### 2.3. 预期 UI 响应

任务卡片立即出现在 Staging 列，按 sort_order 排序。

### 3. 输入输出规范

#### 3.1. 请求

```json
{
  "title": "string (required, 1-255 chars)",
  "glance_note": "string (nullable)",
  "detail_note": "string (nullable)",
  "area_id": "string<uuid> (nullable)",
  "project_id": "string<uuid> (nullable)",
  "subtasks": "array (nullable, <=50 items)"
}
```

#### 3.2. 响应

**201 Created:**

```json
{
  "data": {
    "id": "uuid",
    "title": "新任务",
    "sort_order": "0|n",
    "is_completed": false,
    "schedule_status": "staging",
    "area": { "id": "uuid", "name": "工作", "color": "#4A90E2" },
    ...
  }
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "details": [{ "field": "title", "code": "TITLE_EMPTY" }]
}
```

### 4. 验证规则

- `title`: 必须存在，1-255字符
- `estimated_duration`: 0 <= x <= 10080
- `subtasks`: 数组长度 <= 50

### 5. 业务逻辑详解

1. 验证请求数据
2. 开启数据库事务
3. 生成 UUID (`id_generator.new_uuid()`)
4. 获取当前时间 (`clock.now_utc()`)
5. 构造 Task 实体
6. 插入 tasks 表
7. 使用 LexoRank 计算 sort_order
8. 插入 orderings 表（context: MISC/staging）
9. 提交事务
10. 查询 Area 信息（如果有）
11. 组装 TaskCardDto
12. 返回 201

### 6. 边界情况

- area_id 不存在 → 404 Not Found
- title 为空 → 422 Validation Failed

### 7. 数据访问详情

- SELECT: 1次（查询最大 sort_order）
- SELECT: 0-1次（查询 Area）
- INSERT: 1条到 tasks
- INSERT: 1条到 orderings
- 事务: 所有写操作在事务中

### 8. 预期副作用

- 数据库写入: tasks + orderings 各1条
- 日志: INFO "Task created"
- 前端: 任务出现在 Staging 列

---

## GET /api/tasks/:id

### 1. 端点签名

`GET /api/tasks/:id`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要查看任务的完整详情，包括历史日程记录，以便了解这个任务的完整情况。

#### 2.2. 核心业务逻辑

- 查询 Task 实体
- 查询所有 task_schedules 记录
- 组装完整的 TaskDetailDto

#### 2.3. 预期 UI 响应

任务编辑器弹窗显示完整信息，包括 schedules 数组用于调试。

### 3. 输入输出规范

#### 3.1. 请求

- Path: `task_id` (UUID)

#### 3.2. 响应

**200 OK:**

```json
{
  "data": {
    "card": {
      /* TaskCard 所有字段 */
    },
    "detail_note": "详细笔记",
    "schedules": [{ "day": "2025-09-30T00:00:00Z", "outcome": "planned" }],
    "project": null,
    "created_at": "...",
    "updated_at": "..."
  }
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task with id xxx not found"
}
```

### 4. 验证规则

- task_id 必须是有效的 UUID
- 任务必须存在且未被软删除

### 5. 业务逻辑详解

1. 查询 tasks 表
2. 查询 task_schedules 表
3. 组装 TaskCardDto（基础信息）
4. 组装 TaskDetailDto（包含 schedules）
5. 返回 200

### 6. 边界情况

- 任务不存在 → 404
- 任务已删除 → 404

### 7. 数据访问详情

- SELECT: 1次（查询 task）
- SELECT: 1次（查询 schedules）
- 只读操作，无需事务

### 8. 预期副作用

- 无（只读操作）

---

## PATCH /api/tasks/:id

### 1. 端点签名

`PATCH /api/tasks/:id`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要修改任务的信息（标题、笔记、子任务等），以便更新任务内容或调整细节。

#### 2.2. 核心业务逻辑

部分更新任务字段，只更新请求中提供的字段，其他字段保持不变。

#### 2.3. 预期 UI 响应

任务卡片立即显示更新后的内容，弹窗中的数据同步更新。

### 3. 输入输出规范

#### 3.1. 请求

```json
{
  "title": "string (optional)",
  "glance_note": "string | null (optional)",
  "detail_note": "string | null (optional)",
  "area_id": "uuid | null (optional)",
  "subtasks": "array | null (optional)"
}
```

#### 3.2. 响应

**200 OK:**

```json
{
  "data": {
    "id": "uuid",
    "title": "更新后的标题",
    "schedule_status": "...",
    ...
  }
}
```

**404 Not Found:** 任务不存在
**422 Unprocessable Entity:** 验证失败或空更新

### 4. 验证规则

- 至少提供一个字段
- 如果提供 title：1-255字符
- 如果提供 subtasks：数组长度 <= 50

### 5. 业务逻辑详解

1. 验证请求（至少一个字段）
2. 开启事务
3. 检查任务是否存在
4. 动态构建 UPDATE 语句
5. 执行更新（只更新提供的字段）
6. 更新 updated_at 时间戳
7. 提交事务
8. 重新查询任务
9. 组装 TaskCardDto（包含最新的 schedule_status）
10. 返回更新后的任务

### 6. 边界情况

- 任务不存在 → 404
- 所有字段都是 None → 422
- title 为空字符串 → 422

### 7. 数据访问详情

- SELECT: 2次（检查存在、重新查询）
- SELECT: 2-3次（schedule_status、sort_order、area）
- UPDATE: 1次（更新任务）
- 事务: 更新操作在事务中

### 8. 预期副作用

- 更新 tasks 表
- 更新 updated_at 时间戳
- 日志: INFO "Task updated"
- 前端: 任务卡片内容即时更新

---

## DELETE /api/tasks/:id

### 1. 端点签名

`DELETE /api/tasks/:id`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要删除不需要的任务，系统应智能地清理相关的时间块，避免留下无意义的空时间块。

#### 2.2. 核心业务逻辑

- 软删除任务（is_deleted = true）
- 删除所有关联记录（links, schedules）
- **智能清理孤儿时间块**：
  - 如果时间块只链接了这个任务
  - 且时间块是拖动自动创建的（title 与任务相同）
  - 则同时删除该时间块

#### 2.3. 预期 UI 响应

任务从所有列表消失，相关的孤儿时间块从日历消失。

### 3. 输入输出规范

#### 3.1. 请求

- Path: `task_id` (UUID)

#### 3.2. 响应

**200 OK:**

```json
{
  "data": {
    "deleted_time_block_ids": ["uuid1", "uuid2"]
  }
}
```

**404 Not Found:**
任务不存在

### 4. 验证规则

- task_id 必须存在

### 5. 业务逻辑详解

1. 开启事务
2. 检查任务是否存在
3. 获取任务标题（用于判断孤儿块）
4. 查询链接的所有时间块
5. 软删除任务（is_deleted = true）
6. 删除 task_time_block_links
7. 删除 task_schedules
8. 遍历每个时间块：
   - 检查是否成为孤儿（无其他链接任务）
   - 检查是否自动创建（title 相同）
   - 如果是，软删除该时间块
9. 提交事务
10. 返回被删除的时间块ID列表

### 6. 边界情况

- 任务不存在 → 404
- 任务已删除 → 204（幂等）
- 时间块有其他任务 → 保留时间块
- 时间块是手动创建 → 保留时间块

### 7. 数据访问详情

- SELECT: 2次（任务信息、链接的时间块）
- SELECT: N次（每个时间块检查剩余任务数）
- UPDATE: 1次（软删除任务）
- UPDATE: 0-N次（软删除孤儿时间块）
- DELETE: 2次（task_time_block_links, task_schedules）
- 事务: 所有操作在事务中

### 8. 预期副作用

- 数据库更新: is_deleted = true
- 删除链接和日程记录
- 可能删除孤儿时间块
- 日志: INFO "Deleted orphan time block"
- 前端: 任务和相关时间块同时消失

---

## POST /api/tasks/:id/completion

### 1. 端点签名

`POST /api/tasks/:id/completion`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要标记任务为已完成，系统应自动清理未来的日程并截断正在进行的时间块。

#### 2.2. 核心业务逻辑

- 设置 completed_at 时间戳
- 截断当前正在进行的时间块
- 清理未来的日程记录

#### 2.3. 预期 UI 响应

任务从所有列表消失，相关时间块被截断到当前时间。

### 3. 输入输出规范

#### 3.1. 请求

- Path: `task_id` (UUID)

#### 3.2. 响应

**200 OK:**

```json
{
  "data": {
    "id": "uuid",
    "is_completed": true,
    ...
  }
}
```

**404 Not Found:** 任务不存在
**409 Conflict:** 任务已完成

### 4. 验证规则

- 任务必须存在
- 任务当前必须是未完成状态

### 5. 业务逻辑详解

1. 开启事务
2. 查询任务
3. 检查是否已完成
4. 设置 completed_at = now
5. 截断相关时间块（end_time = now）
6. 清理未来日程
7. 提交事务
8. 组装返回 TaskCardDto

### 6. 边界情况

- 任务不存在 → 404
- 任务已完成 → 409

### 7. 数据访问详情

- SELECT: 1次（查询任务）
- UPDATE: 1次（设置 completed_at）
- UPDATE: 0-N次（截断时间块）
- DELETE: 0-N次（清理未来日程）
- 事务: 所有操作在事务中

### 8. 预期副作用

- 设置 completed_at
- 截断时间块
- 清理未来日程
- 日志: INFO "Task completed"

---

## DELETE /api/tasks/:id/completion

### 1. 端点签名

`DELETE /api/tasks/:id/completion`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要重新打开已完成的任务，使其回到未完成状态，以便继续处理或重新安排。

#### 2.2. 核心业务逻辑

- 将已完成的任务重新打开
- 设置 completed_at 为 NULL
- 返回更新后的任务信息
- **不恢复已删除或截断的时间块**

#### 2.3. 预期 UI 响应

任务等待用户重新完成。

### 3. 输入输出规范

#### 3.1. 请求

- Path: `task_id` (UUID)

#### 3.2. 响应

**200 OK:**

```json
{
  "data": {
    "task": {
      "id": "uuid",
      "title": "任务标题",
      "is_completed": false,
      "completed_at": null,
      "schedule_status": "staging",
      ...
    }
  }
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task with id xxx not found"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "任务尚未完成"
}
```

### 4. 验证规则

- task_id 必须存在
- 任务必须处于已完成状态（completed_at IS NOT NULL）

### 5. 业务逻辑详解

1. 开启数据库事务
2. 查询任务是否存在
3. 验证任务是否已完成
4. 设置 completed_at = NULL
5. 更新 updated_at = now
6. 提交事务
7. 重新查询任务
8. 组装 TaskCardDto
9. 返回 200

### 6. 边界情况

- 任务不存在 → 404 Not Found
- 任务未完成（completed_at IS NULL）→ 409 Conflict
- 任务已删除（is_deleted = true）→ 404 Not Found

### 7. 数据访问详情

- SELECT: 2次（查询任务、重新查询）
- UPDATE: 1次（设置 completed_at = NULL）
- 事务: 所有写操作在事务中

### 8. 预期副作用

- **数据库写入**: 更新 tasks 表的 completed_at 和 updated_at 字段
- **日程状态**: 不修改已有的日程记录（outcome 保持历史状态）
- **时间块**: 不恢复已删除或截断的时间块
- **日志**: INFO "Task reopened"
- **前端**: 任务出现在 Staging 区

---

**文件路径：** `src-tauri/src/features/tasks/`
**端点实现：** `endpoints/` 目录下的对应文件
**装配器：** `shared/assembler.rs`
