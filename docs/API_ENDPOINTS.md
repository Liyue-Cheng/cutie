# Cutie API 端点列表

## 响应格式规范

### 成功响应

```json
{
  "data": {
    /* 实际数据 */
  },
  "timestamp": "2025-10-16T13:00:00Z",
  "request_id": null
}
```

### 错误响应

```json
{
  "error_type": "NotFound",
  "message": "Resource not found",
  "details": {
    /* 详细信息 */
  },
  "code": "NOT_FOUND",
  "timestamp": "2025-10-16T13:00:00Z",
  "request_id": null
}
```

---

## 一、任务管理 (Tasks)

### 1.1 创建任务

```
POST /api/tasks
```

**Request Body**:

```json
{
  "title": "任务标题",
  "glance_note": "简要备注 (可选)",
  "area_id": "uuid (可选)"
}
```

**Response**: `201 Created`

```json
{
  "data": {
    "id": "uuid",
    "title": "任务标题",
    "schedule_status": "staging"
    /* ...其他字段 */
  }
}
```

### 1.2 创建任务并添加日程

```
POST /api/tasks/with-schedule
```

**Request Body**:

```json
{
  "title": "任务标题",
  "scheduled_day": "2025-10-16"
}
```

**Response**: `201 Created`

### 1.3 获取任务详情

```
GET /api/tasks/:id
```

**Response**: `200 OK`

### 1.4 更新任务

```
PATCH /api/tasks/:id
```

**Request Body**: (所有字段可选)

```json
{
  "title": "新标题",
  "glance_note": "新备注",
  "detail_note": "详细备注",
  "estimated_duration": 60,
  "area_id": "uuid"
}
```

**Response**: `200 OK`

### 1.5 删除任务（软删除）

```
DELETE /api/tasks/:id
```

**Response**: `204 No Content`

### 1.6 永久删除任务

```
DELETE /api/tasks/:id/permanently
```

**Response**: `204 No Content`

### 1.7 完成任务

```
POST /api/tasks/:id/completion
```

**Response**: `200 OK`

### 1.8 重新打开任务

```
DELETE /api/tasks/:id/completion
```

**Response**: `200 OK`

### 1.9 归档任务

```
POST /api/tasks/:id/archive
```

**Response**: `200 OK`

### 1.10 取消归档

```
POST /api/tasks/:id/unarchive
```

**Response**: `200 OK`

### 1.11 恢复已删除任务

```
PATCH /api/tasks/:id/restore
```

**Response**: `200 OK`

### 1.12 返回暂存区

```
POST /api/tasks/:id/return-to-staging
```

**Response**: `200 OK`

---

## 二、日程管理 (Schedules)

### 2.1 添加日程

```
POST /api/tasks/:id/schedules
```

**Request Body**:

```json
{
  "scheduled_day": "2025-10-16"
}
```

**Response**: `201 Created`

### 2.2 更新日程结果

```
PATCH /api/tasks/:task_id/schedules/:date
```

**Request Body**:

```json
{
  "outcome": "COMPLETED_ON_DAY"
}
```

**Response**: `200 OK`

### 2.3 删除日程

```
DELETE /api/tasks/:task_id/schedules/:date
```

**Response**: `204 No Content`

---

## 三、模板管理 (Templates)

### 3.1 获取所有模板

```
GET /api/templates
```

**Response**: `200 OK`

```json
{
  "data": [
    {
      "id": "uuid",
      "title": "模板标题",
      "category": "GENERAL"
      /* ...其他字段 */
    }
  ]
}
```

### 3.2 创建模板

```
POST /api/templates
```

**Request Body**:

```json
{
  "title": "模板标题",
  "glance_note_template": "简要备注 (可选)",
  "category": "GENERAL"
}
```

**Response**: `201 Created`

### 3.3 更新模板

```
PATCH /api/templates/:id
```

**Request Body**: (所有字段可选)

```json
{
  "title": "新标题",
  "glance_note_template": "新备注",
  "estimated_duration_template": 60
}
```

**Response**: `200 OK`

### 3.4 删除模板

```
DELETE /api/templates/:id
```

**Response**: `204 No Content`

### 3.5 从模板创建任务

```
POST /api/templates/:id/create-task
```

**Request Body**:

```json
{
  "variables": {
    "date": "2025-10-16"
  }
}
```

**Response**: `201 Created` (返回创建的任务)

### 3.6 从任务创建模板

```
POST /api/tasks/:id/to-template
```

**Request Body**:

```json
{
  "title": "模板标题 (可选)",
  "category": "GENERAL"
}
```

**Response**: `201 Created` (返回创建的模板)

---

## 四、时间块管理 (Time Blocks)

### 4.1 获取日期范围内的时间块

```
GET /api/time-blocks?start=2025-10-01&end=2025-10-31
```

**Query Parameters**:

- `start`: 开始日期 (YYYY-MM-DD)
- `end`: 结束日期 (YYYY-MM-DD)

**Response**: `200 OK`

### 4.2 创建时间块

```
POST /api/time-blocks
```

**Request Body**:

```json
{
  "title": "时间块标题",
  "start_time": "2025-10-16T09:00:00Z",
  "end_time": "2025-10-16T10:00:00Z",
  "time_type": "FLOATING",
  "start_time_local": "09:00:00",
  "end_time_local": "10:00:00"
}
```

**Response**: `201 Created`

### 4.3 更新时间块

```
PATCH /api/time-blocks/:id
```

**Response**: `200 OK`

### 4.4 删除时间块

```
DELETE /api/time-blocks/:id
```

**Response**: `204 No Content`

### 4.5 链接任务到时间块

```
POST /api/time-blocks/:block_id/link-task
```

**Request Body**:

```json
{
  "task_id": "uuid"
}
```

**Response**: `200 OK`

### 4.6 取消任务链接

```
DELETE /api/time-blocks/:block_id/unlink-task/:task_id
```

**Response**: `204 No Content`

---

## 五、区域管理 (Areas)

### 5.1 获取所有区域

```
GET /api/areas
```

**Response**: `200 OK`

### 5.2 创建区域

```
POST /api/areas
```

**Request Body**:

```json
{
  "name": "区域名称",
  "color": "#4A90E2"
}
```

**Response**: `201 Created`

### 5.3 更新区域

```
PATCH /api/areas/:id
```

**Request Body**:

```json
{
  "name": "新名称",
  "color": "#FF5733"
}
```

**Response**: `200 OK`

### 5.4 删除区域

```
DELETE /api/areas/:id
```

**Response**: `204 No Content`

---

## 六、视图偏好 (View Preferences)

### 6.1 获取视图排序

```
GET /api/view-preferences/:context_key
```

**示例**:

```
GET /api/view-preferences/misc::staging
GET /api/view-preferences/daily::2025-10-16
```

**Response**: `200 OK`

```json
{
  "data": {
    "context_key": "misc::staging",
    "sorted_task_ids": ["id1", "id2", "id3"],
    "updated_at": "2025-10-16T13:00:00Z"
  }
}
```

### 6.2 更新视图排序

```
PUT /api/view-preferences
```

**Request Body**:

```json
{
  "context_key": "misc::staging",
  "sorted_task_ids": ["id1", "id2", "id3"]
}
```

**Response**: `200 OK`

---

## 七、回收站 (Trash)

### 7.1 获取回收站内容

```
GET /api/trash
```

**Response**: `200 OK`

### 7.2 清空回收站

```
DELETE /api/trash/clear
```

**Response**: `204 No Content`

---

## 八、系统端点

### 8.1 健康检查

```
GET /health
```

**Response**: `200 OK`

```json
{
  "status": "healthy",
  "timestamp": "2025-10-16T13:00:00Z",
  "version": "1.0.0"
}
```

### 8.2 Ping

```
GET /api/ping
```

**Response**: `200 OK`

```json
{
  "message": "pong",
  "timestamp": "2025-10-16T13:00:00Z"
}
```

### 8.3 SSE 事件流

```
GET /api/events
```

**Response**: Server-Sent Events Stream

---

## 九、错误码参考

| HTTP 状态码 | 错误类型        | 说明               |
| ----------- | --------------- | ------------------ |
| 200         | -               | 成功               |
| 201         | -               | 创建成功           |
| 204         | -               | 删除成功（无内容） |
| 400         | ValidationError | 验证失败           |
| 404         | NotFound        | 资源不存在         |
| 409         | Conflict        | 冲突（如重复创建） |
| 500         | InternalError   | 服务器内部错误     |

---

## 十、Context Key 规范

**格式**: `type::identifier`

### 杂项视图

```
misc::all          - 所有任务
misc::staging      - 暂存区
misc::planned      - 已计划
misc::incomplete   - 未完成
misc::completed    - 已完成
misc::template     - 模板
```

### 日期视图

```
daily::2025-10-16  - 特定日期
```

### 区域视图

```
area::uuid         - 特定区域
```

### 项目视图

```
project::uuid      - 特定项目
```

---

## 十一、SSE 事件类型

### 任务事件

```
task.created       - 任务创建
task.updated       - 任务更新
task.deleted       - 任务删除
task.completed     - 任务完成
task.archived      - 任务归档
```

### 日程事件

```
schedule.created   - 日程创建
schedule.updated   - 日程更新
schedule.deleted   - 日程删除
```

### 模板事件

```
template.created   - 模板创建
template.updated   - 模板更新
template.deleted   - 模板删除
```

### 时间块事件

```
time_block.created - 时间块创建
time_block.updated - 时间块更新
time_block.deleted - 时间块删除
```

### 区域事件

```
area.created       - 区域创建
area.updated       - 区域更新
area.deleted       - 区域删除
```
