# TimeBlocks API 规范文档

本文档按照 CABC 规范 2.0 定义 TimeBlocks 功能模块的所有 API 端点。

---

## 端点清单

1. [POST /api/time-blocks/from-task](#post-apitime-blocksfrom-task) - 从任务创建时间块（拖动专用）⭐
2. [POST /api/time-blocks](#post-apitime-blocks) - 创建空时间块
3. [GET /api/time-blocks](#get-apitime-blocks) - 查询时间块列表
4. [DELETE /api/time-blocks/:id](#delete-apitime-blocksid) - 删除时间块

---

## POST /api/time-blocks/from-task

### 1. 端点签名

`POST /api/time-blocks/from-task`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要将任务从看板拖动到日历上，系统应自动创建时间块、链接任务、并更新任务的排期状态。

#### 2.2. 核心业务逻辑

**这是 Cutie 核心功能的关键端点，实现任务与时间块的多对多架构。**

- 创建时间块（继承任务的 area）
- 创建 task_time_block_links（任务 ↔ 时间块）
- **创建 task_schedules**（任务 ↔ 日期）
- 返回时间块和**更新后的任务**

#### 2.3. 预期 UI 响应

- 时间块立即出现在日历上
- 任务从 Staging 列消失
- 任务出现在 Planned 列
- **无需刷新，响应式更新**

### 3. 输入输出规范

#### 3.1. 请求

```json
{
  "task_id": "uuid (required)",
  "start_time": "ISO 8601 UTC (required)",
  "end_time": "ISO 8601 UTC (required)",
  "title": "string (nullable, 默认使用任务标题)"
}
```

#### 3.2. 响应

**201 Created:**

```json
{
  "data": {
    "time_block": {
      "id": "uuid",
      "start_time": "2025-09-30T09:00:00Z",
      "end_time": "2025-09-30T10:00:00Z",
      "area": { "color": "#4A90E2" },
      "linked_tasks": [
        { "id": "uuid", "title": "任务标题", "is_completed": false }
      ]
    },
    "updated_task": {
      "id": "uuid",
      "schedule_status": "scheduled",  // ✅ 已更新
      ...
    }
  }
}
```

**404 Not Found:** 任务不存在
**409 Conflict:** 时间冲突

### 4. 验证规则

- start_time < end_time
- 时间块不与现有时间块重叠（Cutie 核心约束）

### 5. 业务逻辑详解

1. 验证时间范围
2. 开启事务
3. 查询并验证任务存在
4. 检查时间冲突
5. 生成 UUID 和时间戳
6. 创建时间块（继承任务 area）
7. 插入 time_blocks 表
8. 插入 task_time_block_links
9. **从 start_time 提取日期**
10. 检查是否已有该日期的 schedule
11. 创建 task_schedules（outcome = 'PLANNED'）
12. 提交事务
13. 组装 TimeBlockViewDto
14. **重新组装 TaskCardDto（schedule_status = 'scheduled'）**
15. 返回 { time_block, updated_task }

### 6. 边界情况

- 任务不存在 → 404
- 时间冲突 → 409
- 该日期已有 schedule → 不重复创建

### 7. 数据访问详情

- SELECT: 3次（任务、时间冲突、schedule存在性）
- SELECT: 0-2次（Area、sort_order）
- INSERT: 1条到 time_blocks
- INSERT: 1条到 task_time_block_links
- INSERT: 0-1条到 task_schedules（如果不存在）
- 事务: 所有写操作在事务中

### 8. 预期副作用

- 创建3个表记录
- 日志: INFO "Created time block from task"
- 前端: 时间块出现 + 任务状态更新

---

## POST /api/time-blocks

### 1. 端点签名

`POST /api/time-blocks`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要在日历上直接创建一个时间块（如"深度工作"），稍后再关联具体任务。

#### 2.2. 核心业务逻辑

仅创建时间块实体，不链接任务，不创建 schedule。

#### 2.3. 预期 UI 响应

空时间块出现在日历上。

### 3. 输入输出规范

#### 3.1. 请求

```json
{
  "title": "string (nullable)",
  "start_time": "ISO 8601 UTC (required)",
  "end_time": "ISO 8601 UTC (required)",
  "area_id": "uuid (nullable)"
}
```

#### 3.2. 响应

**201 Created:** TimeBlockViewDto

### 4. 验证规则

- start_time < end_time
- 不与现有时间块重叠

### 5. 业务逻辑详解

1. 验证时间范围
2. 检查时间冲突
3. 创建并插入时间块
4. 提交事务
5. 返回 TimeBlockViewDto

### 6. 边界情况

- 时间冲突 → 409

### 7. 数据访问详情

- SELECT: 1次（时间冲突检查）
- INSERT: 1条到 time_blocks
- 事务: 写操作在事务中

### 8. 预期副作用

- 创建空时间块
- 日志: INFO "Created empty time block"

---

## GET /api/time-blocks

### 1. 端点签名

`GET /api/time-blocks?start_date=...&end_date=...`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，打开应用时，我希望看到日历上已有的时间块，以便了解今天的安排。

#### 2.2. 核心业务逻辑

按日期范围查询时间块，包含关联任务和区域信息。

#### 2.3. 预期 UI 响应

日历上显示所有时间块，使用区域颜色染色。

### 3. 输入输出规范

#### 3.1. 请求

- Query: `start_date` (ISO 8601, optional)
- Query: `end_date` (ISO 8601, optional)

#### 3.2. 响应

**200 OK:**

```json
{
  "data": [
    {
      "id": "uuid",
      "start_time": "...",
      "end_time": "...",
      "area": { "color": "#4A90E2" },
      "linked_tasks": [
        /* 关联的任务 */
      ]
    }
  ]
}
```

### 4. 验证规则

- 日期格式必须是 ISO 8601

### 5. 业务逻辑详解

1. 解析时间范围
2. 查询 time_blocks（时间重叠逻辑）
3. 为每个时间块查询关联任务
4. 为每个时间块查询 Area
5. 组装 Vec<TimeBlockViewDto>
6. 按 start_time 排序
7. 返回 200

### 6. 边界情况

- 无时间块 → 返回空数组

### 7. 数据访问详情

- SELECT: 1次（时间块）
- SELECT: N次（每个块的任务）
- SELECT: N次（每个块的 Area）
- 只读操作

### 8. 预期副作用

- 无（只读）

---

## DELETE /api/time-blocks/:id

### 1. 端点签名

`DELETE /api/time-blocks/:id`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要删除不需要的时间块，但任务应该保持"已排期"状态。

#### 2.2. 核心业务逻辑

**Cutie 哲学：删除时间块 ≠ 取消排期**

- 软删除时间块
- 删除 task_time_block_links
- **保留 task_schedules**（任务仍被排期）

#### 2.3. 预期 UI 响应

时间块从日历消失，任务仍在 Planned 列。

### 3. 输入输出规范

#### 3.1. 请求

- Path: `block_id` (UUID)

#### 3.2. 响应

**204 No Content**

**404 Not Found:** 时间块不存在

### 4. 验证规则

- block_id 必须存在

### 5. 业务逻辑详解

1. 开启事务
2. 检查时间块存在
3. 软删除时间块
4. 删除 task_time_block_links
5. **不删除 task_schedules**
6. 提交事务

### 6. 边界情况

- 时间块不存在 → 404
- 时间块已删除 → 204（幂等）

### 7. 数据访问详情

- SELECT: 1次（检查存在）
- UPDATE: 1次（软删除）
- DELETE: 1次（task_time_block_links）
- 事务: 所有操作在事务中

### 8. 预期副作用

- 软删除时间块
- 删除链接
- 保留 schedule
- 日志: INFO "Deleted time block"

---

**文件路径：** `src-tauri/src/features/time_blocks/`
**端点实现：** `endpoints/` 目录下的对应文件
