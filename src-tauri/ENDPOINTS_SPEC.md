# 后端 API 端点规格

本文档定义前端当前需要的后端 API 端点。

## 优先级说明

- 🔴 P0: 必须实现（基本功能）
- 🟡 P1: 重要功能
- 🟢 P2: 增强功能

---

## 🔴 P0: 必须实现

### 1. GET /views/staging

获取未排期的任务列表

**用途：** HomeView 加载 Staging 列

**响应：** `200 OK`

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "未排期的任务",
    "glance_note": "快速笔记",
    "sort_order": "aaa",
    "is_completed": false,
    "schedule_status": "staging",
    "subtasks": [
      {
        "id": "...",
        "title": "子任务1",
        "is_completed": false,
        "sort_order": "aaa"
      }
    ],
    "area": {
      "id": "...",
      "name": "工作",
      "color": "#4a90e2"
    },
    "project_id": null,
    "schedule_info": null,
    "due_date": {
      "date": "2024-12-31T00:00:00Z",
      "type": "soft",
      "is_overdue": false
    },
    "has_detail_note": true
  }
]
```

**返回类型：** `Vec<TaskCardDto>`

**业务逻辑：**

- 查询 `tasks` 表中所有 `is_deleted = false` 且 `completed_at IS NULL` 的任务
- 排除已在 `task_schedules` 表中有记录的任务
- 从 `areas` 表获取区域信息
- 使用 `TaskAssembler::task_to_card_full()` 组装

---

### 2. POST /tasks

创建新任务

**用途：** Staging 列添加任务

**请求体：**

```json
{
  "title": "新任务标题",
  "glance_note": null,
  "detail_note": null,
  "area_id": null,
  "due_date": null,
  "due_date_type": null,
  "project_id": null,
  "subtasks": null
}
```

**响应：** `201 Created`

```json
{
  "id": "...",
  "title": "新任务标题",
  "glance_note": null,
  "sort_order": "aaa",
  "is_completed": false,
  "schedule_status": "staging",
  ...
}
```

**返回类型：** `TaskCardDto`

**业务逻辑：**

1. 验证 `title` 不为空且长度 ≤ 255
2. 生成 UUID 和时间戳
3. 插入 `tasks` 表
4. 使用 `TaskAssembler::task_to_card_basic()` 组装返回

---

## 🟡 P1: 重要功能

### 3. POST /tasks/:id/completion ✅

完成任务（已实现）

**文件：** `src-tauri/src/features/tasks/endpoints/legacy.rs`

---

### 4. DELETE /tasks/:id/completion

重新打开已完成的任务

**用途：** 取消任务完成状态

**响应：** `200 OK`

```json
{
  "id": "...",
  "title": "任务标题",
  "is_completed": false,
  ...
}
```

**返回类型：** `TaskCardDto`

**业务逻辑：**

1. 检查任务是否存在
2. 检查任务是否已完成（completed_at IS NOT NULL）
3. 设置 `completed_at = NULL`
4. 更新 `updated_at`

---

### 5. GET /tasks/:id

获取任务详情

**用途：** 任务编辑器打开时加载完整信息

**响应：** `200 OK`

```json
{
  "id": "...",
  "title": "任务标题",
  "detail_note": "详细笔记（Markdown）",
  "schedules": [
    {
      "day": "2024-10-28T00:00:00Z",
      "outcome": "completed"
    }
  ],
  "project": {
    "id": "...",
    "name": "项目名"
  },
  "created_at": "2024-10-28T10:00:00Z",
  "updated_at": "2024-10-28T11:00:00Z",
  ...
}
```

**返回类型：** `TaskDetailDto`

**业务逻辑：**

- 查询任务基本信息
- 查询所有日程记录（task_schedules）
- 查询项目信息（如果有）
- 使用 `TaskAssembler::card_to_detail_full()` 组装

---

### 6. PATCH /tasks/:id

更新任务

**用途：** 任务编辑器保存修改

**请求体：**

```json
{
  "title": "修改后的标题",
  "glance_note": "修改后的笔记",
  "area_id": "new-area-id",
  "subtasks": [...]
}
```

**响应：** `200 OK` - 返回 `TaskCardDto`

---

### 7. DELETE /tasks/:id

删除任务（软删除）

**用途：** 删除任务

**响应：** `204 No Content`

**业务逻辑：**

- 设置 `is_deleted = true`
- 更新 `updated_at`

---

## 🟢 P2: 增强功能

### 8. GET /tasks/search

搜索任务

**查询参数：**

- `q`: 搜索关键词
- `limit`: 结果数量限制（默认50）

---

### 9. POST /schedules

安排任务到指定日期

**请求体：**

```json
{
  "task_id": "...",
  "scheduled_day": "2024-10-28"
}
```

---

### 10. GET /views/daily-schedule

获取指定日期的任务列表

**查询参数：**

- `day`: YYYY-MM-DD

**响应：** `Vec<TaskCardDto>`

---

## 实现建议

### SFC 文件结构

```
src-tauri/src/features/tasks/endpoints/
├── legacy.rs              ✅ (complete_task)
├── get_staging_view.rs    🔴 P0
├── create_task.rs         🔴 P0
├── reopen_task.rs         🟡 P1
├── get_task.rs            🟡 P1
├── update_task.rs         🟡 P1
└── delete_task.rs         🟡 P1
```

### 使用装配器

所有端点都应使用 `TaskAssembler` 来转换数据：

```rust
use crate::features::shared::TaskAssembler;

let task_card = TaskAssembler::task_to_card_basic(&task);
```
