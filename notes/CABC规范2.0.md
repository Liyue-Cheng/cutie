# CABC V2.1: POST /api/tasks

## 1. 端点签名 (Endpoint Signature)

`POST /api/tasks`

---

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要在任何看板（如Staging区或今日看板）上快速创建一个新任务，以便我能立即捕捉我的想法，而不需要复杂的步骤。

### 2.2. 核心业务逻辑 (Core Business Logic)

为了实现快速创建，后端将在数据库中创建一个新的`Task`实体。为了让它能被立即看到和排序，系统会同时为这个新任务在它所属的上下文（由请求中的`context`字段决定，若无则默认为Staging区）中，创建一个`Ordering`记录。

### 2.3. 预期的UI响应 (Expected UI Response)

调用成功后，一个新的任务卡片应该**立即**（通过乐观更新或WebSocket推送）出现在用户创建它时所在的看板的列表顶部或底部。这张卡片应显示正确的标题，并处于未完成状态。

---

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "title": "string (required, 1-255 chars)",
  "glance_note": "string (nullable, <=140 chars)",
  "detail_note": "string (nullable)",
  "area_id": "string<uuid> (nullable)",
  "context": {
    "type": "'DAILY_KANBAN' | 'PROJECT_LIST' | 'MISC'",
    "id": "string"
  }
}
```

### 3.2. 响应 (Responses)

**201 Created:**

```json
// content: application/json
// schema: TaskCardDto
{
  "id": "a1b2c3d4-...",
  "title": "新任务",
  "glance_note": "快速笔记",
  "sort_order": "a0b1c2...",
  "is_completed": false,
  "schedule_status": "staging",
  "subtasks": null,
  "area": {
    "id": "f9e8d7c6-...",
    "name": "工作",
    "color": "#4A90E2"
  },
  "project_id": null,
  "schedule_info": null,
  "due_date": null,
  "has_detail_note": false
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "title", "code": "TITLE_EMPTY", "message": "任务标题不能为空" }
  ]
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "关联的实体（如Area或Project）不存在"
}
```

---

## 4. 验证规则 (Validation Rules)

- `title`:
  - **必须**存在。
  - **必须**为非空字符串 (trim后)。
  - 长度**必须**小于等于 255 个字符。
- `estimated_duration` (如果未来添加):
  - 如果提供，**必须**是大于等于 0 的整数。
  - 如果提供，**必须**小于等于 10080 (7天)。
- `subtasks` (如果未来添加):
  - 如果提供，其数组长度**必须**小于等于 50。

---

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用`validation`模块对请求体进行验证。
2.  启动数据库事务。
3.  通过`Clock`服务获取当前时间`now`。
4.  通过`IdGenerator`服务生成新的`task_id`。
5.  根据请求数据，构造一个`Task`领域实体对象。
6.  调用`database`模块的`insert_task`函数持久化`Task`。
7.  根据请求中的`context`，确定新任务的排序上下文（默认为`MISC::staging_all`）。
8.  调用`database`模块的`calculate_new_sort_order`函数获取该上下文中的新排序值。
9.  调用`database`模块的`insert_ordering`函数持久化排序信息。
10. 提交数据库事务。
11. 从数据库中重新或附加查询`Area`等关联信息。
12. 调用`Assembler`将`Task`实体和所有上下文信息组装成`TaskCardDto`。
13. 返回`201 Created`和组装好的`TaskCardDto`。

---

## 6. 边界情况 (Edge Cases)

- **`area_id`不存在:** 如果请求中提供了`area_id`，但在`areas`表中找不到对应的记录，**必须**返回`404 Not Found`错误，并回滚事务。
- **并发创建:** （暂不考虑V1.0）在高并发下，`calculate_new_sort_order`的逻辑需要能处理可能的竞态条件，以保证`sort_order`的唯一性和正确性。

---

## 7. 数据访问详情 (Data Access Details)

- **`SELECT`:** 1次，查询`areas`表以验证`area_id`是否存在（如果提供）。
- **`SELECT`:** 1次，查询`orderings`表以获取当前上下文的最大`sort_order`。
- **`INSERT`:** 1条记录到 `tasks` 表。
- **`INSERT`:** 1条记录到 `orderings` 表。
- **(事务):** 以上所有数据库写操作**必须**包含在一个数据库事务内。

---

## 8. 预期副作用 (Expected Side Effects)

- **数据库写入:**
  - `tasks`表: 插入1条新记录。
  - `orderings`表: 插入1条新记录。
- **日志记录:**
  - 成功时，以`INFO`级别记录"Task created successfully"及任务ID。
  - 失败时（如验证失败或数据库错误），以`WARN`或`ERROR`级别记录详细错误信息。
- **WebSocket推送:**
  - 成功创建后，**必须**通过WebSocket事件总线广播一个`TaskCreated`事件，其负载为新创建的`TaskCardDto`。