# Cutie API Reference

> 本文档由 `doc-composer` 工具自动生成，请勿手动编辑。
> 源文件位置：`src-tauri/src/features/*/endpoints/*.rs`

## Table of Contents

- [Areas (领域管理)](#areas)
  - [POST /api/areas](#post-apiareas)
  - [DELETE /api/areas/{id}](#delete-apiareasid)
  - [GET /api/areas/{id}](#get-apiareasid)
  - [GET /api/areas](#get-apiareas)
  - [PUT /api/areas/{id}](#put-apiareasid)
- [Tasks (任务管理)](#tasks)
  - [POST /api/tasks/{id}/schedules](#post-apitasksidschedules)
  - [POST /api/tasks/{id}/completion](#post-apitasksidcompletion)
  - [POST /api/tasks](#post-apitasks)
  - [DELETE /api/tasks/{id}/schedules/{date}](#delete-apitasksidschedulesdate)
  - [DELETE /api/tasks/{id}](#delete-apitasksid)
  - [GET /api/tasks/{id}](#get-apitasksid)
  - [DELETE /api/tasks/{id}/completion](#delete-apitasksidcompletion)
  - [POST /api/tasks/{id}/return-to-staging](#post-apitasksidreturn-to-staging)
  - [PATCH /api/tasks/{id}/schedules/{date}](#patch-apitasksidschedulesdate)
  - [PATCH /api/tasks/{id}](#patch-apitasksid)
- [Time Blocks (时间块管理)](#time_blocks)
  - [POST /api/time-blocks/from-task](#post-apitime-blocksfrom-task)
  - [POST /api/time-blocks](#post-apitime-blocks)
  - [DELETE /api/time-blocks/{id}](#delete-apitime-blocksid)
  - [GET /api/time-blocks?start_date={start_date}&end_date={end_date}](#get-apitime-blocks?start_date=start_date&end_date=end_date)
  - [PATCH /api/time-blocks/{id}](#patch-apitime-blocksid)
- [View Preferences (视图偏好)](#view_preferences)
  - [GET /api/view-preferences/:context_key](#get-apiview-preferences:context_key)
  - [PUT /api/view-preferences](#put-apiview-preferences)
- [Views (视图查询)](#views)
  - [GET /api/views/all](#get-apiviewsall)
  - [GET /api/views/all-incomplete](#get-apiviewsall-incomplete)
  - [GET /api/views/daily/:date](#get-apiviewsdaily:date)
  - [GET /api/views/planned](#get-apiviewsplanned)
  - [GET /api/views/staging](#get-apiviewsstaging)

---

## Areas (领域管理)

### GET /api/areas

<details>
<summary>源文件: <code>src\features\areas\endpoints\list_areas.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/areas

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有可用的领域列表，
> 以便了解我的任务分类体系并选择合适的领域来分类任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有未删除的领域（`is_deleted = false`），
按名称字母序升序排列，并将结果组装成 `AreaDto` 列表返回。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- 无

**Query Parameters:**
- 无（当前版本不支持分页、过滤、排序参数）

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `AreaDto[]`

```json
[
  {
    "id": "uuid",
    "name": "string",
    "color": "string (#RRGGBB)",
    "parent_area_id": "uuid | null",
    "created_at": "ISO8601 timestamp",
    "updated_at": "ISO8601 timestamp"
  },
  ...
]
```

**注意：** 空列表返回 `[]`，而不是错误。

## 4. 验证规则 (Validation Rules)

- 无输入参数，无需验证。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `database::find_all_areas` 查询数据库：
    - 查询 `areas` 表
    - 过滤条件：`is_deleted = false`
    - 排序：按 `name` 字段升序
2.  将查询结果（`Vec<Area>`）映射为 `Vec<AreaDto>`。
3.  返回 `200 OK` 和领域列表。

## 6. 边界情况 (Edge Cases)

- **数据库中没有领域:** 返回空数组 `[]`（200 OK）。
- **所有领域都已删除:** 返回空数组 `[]`（200 OK）。
- **领域数量很大:** 当前无分页机制，可能返回大量数据（性能问题）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询 `areas` 表所有未删除的领域（带 `is_deleted = false` 过滤，按 `name` 排序）。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无数据库写入，无 SSE 事件，无其他已知副作用）*

**性能考虑：**
1. 当前实现会一次性返回所有领域，没有分页机制。
2. 如果领域数量超过数百个，建议添加分页参数（limit/offset 或 cursor-based）。
3. 考虑添加客户端缓存或 SSE 订阅机制，减少重复查询。

### POST /api/areas

<details>
<summary>源文件: <code>src\features\areas\endpoints\create_area.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

POST /api/areas

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要创建一个新的领域（Area），以便我能将任务分类到不同的项目或上下文中，
> 并通过颜色标记快速识别不同领域的任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

在数据库中创建一个新的 `Area` 实体。系统将验证名称唯一性（在未删除的领域中），
并确保颜色格式符合十六进制颜色码规范（#RRGGBB）。新领域可以是根领域，也可以指定父领域。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "name": "string (required, 非空)",
  "color": "string (required, 格式: #RRGGBB)",
  "parent_area_id": "string (UUID) | null (optional)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`
*   **Schema:** `AreaDto`

```json
{
  "id": "uuid",
  "name": "string",
  "color": "string (#RRGGBB)",
  "parent_area_id": "uuid | null",
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "name", "code": "NAME_EMPTY", "message": "名称不能为空" }
  ]
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "Area 名称已存在"
}
```

## 4. 验证规则 (Validation Rules)

- `name`:
    - **必须**存在。
    - **必须**为非空字符串 (trim后)。
    - 在未删除的领域中**必须**唯一。
    - 违反时返回错误码：`NAME_EMPTY` 或 `CONFLICT`
- `color`:
    - **必须**存在。
    - **必须**符合格式 `#RRGGBB`（7个字符，以#开头，后跟6个十六进制字符）。
    - 违反时返回错误码：`INVALID_COLOR`
- `parent_area_id`:
    - 如果提供，**必须**是有效的 UUID 格式。
    - 如果提供，**应该**指向一个存在的 Area（当前未强制验证）。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation` 验证 `name` 非空。
2.  调用 `Area::validate_color` 验证 `color` 格式。
3.  启动数据库事务（`db_pool().begin()`）。
4.  调用 `database::check_name_exists_in_tx` 检查名称是否已存在。
5.  如果名称已存在，返回 `409 Conflict` 错误并回滚事务。
6.  通过 `IdGenerator` 生成新的 `area_id`（UUID）。
7.  通过 `Clock` 服务获取当前时间 `now`。
8.  构造 `Area` 领域实体对象：
    - 设置 `id = area_id`, `name`, `color`, `parent_area_id`
    - 设置 `created_at = now`, `updated_at = now`
    - 设置 `is_deleted = false`
9.  调用 `database::insert_area_in_tx` 持久化领域到 `areas` 表。
10. 提交数据库事务（`tx.commit()`）。
11. 组装 `AreaDto` 并返回 `201 Created`。

## 6. 边界情况 (Edge Cases)

- **`name` 为空或全空格:** 返回 `422` 错误，错误码 `NAME_EMPTY`。
- **`name` 已存在（未删除的领域中）:** 返回 `409` 错误，消息 "Area 名称已存在"。
- **`color` 格式无效（非 #RRGGBB）:** 返回 `422` 错误，错误码 `INVALID_COLOR`。
- **`parent_area_id` 无效（不存在或格式错误）:** 当前实现中正常返回，未验证父领域存在性。
- **并发创建相同名称:** 可能导致两个检查都通过，但由于唯一约束（如果有）会导致数据库错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询 `areas` 表以检查名称唯一性。
    - **`INSERT`:** 1条记录到 `areas` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 成功时，可能以 `INFO` 级别记录 "Area created successfully"（如有）。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无 SSE 事件，无写入许可，无其他已知副作用）*

### DELETE /api/areas/{id}

<details>
<summary>源文件: <code>src\features\areas\endpoints\delete_area.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

DELETE /api/areas/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要删除一个不再使用的领域，
> 以便清理我的领域列表并保持任务分类体系的整洁。

### 2.2. 核心业务逻辑 (Core Business Logic)

软删除数据库中指定 ID 的领域（设置 `is_deleted = true`，更新 `updated_at`）。
领域的删除是软删除，不会物理删除数据库记录，也不会影响已关联该领域的任务。
已关联该领域的任务仍然保留其 `area_id` 引用，但前端可以选择性地隐藏或显示已删除领域的任务。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 领域ID

### 3.2. 响应 (Responses)

**204 No Content:**

*   成功删除，无响应体。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Area not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `area_id`:
    - **必须**是有效的 UUID 格式（由 Axum 路径提取器自动验证）。
    - **必须**存在于数据库中。
    - **必须**未被删除（`is_deleted = false`）。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  从路径参数中提取 `area_id`。
2.  启动数据库事务（`db_pool().begin()`）。
3.  调用 `database::check_area_exists_in_tx` 检查领域是否存在。
4.  如果领域不存在或已删除，返回 `404 Not Found` 错误并回滚事务。
5.  调用 `database::soft_delete_area_in_tx` 软删除领域：
    - 设置 `is_deleted = true`
    - 更新 `updated_at` 为当前时间
6.  提交数据库事务（`tx.commit()`）。
7.  返回 `204 No Content`。

## 6. 边界情况 (Edge Cases)

- **领域不存在:** 返回 `404` 错误。
- **领域已删除:** 幂等，返回 `404` 错误（因为 `check_area_exists_in_tx` 不返回已删除的领域）。
- **领域有子领域:** 当前实现中仍然允许删除，子领域的 `parent_area_id` 仍然指向被删除的领域。
- **领域有关联任务:** 当前实现中允许删除，任务的 `area_id` 仍然保留该引用。
- **领域有时间块关联:** 不影响时间块，时间块不直接关联领域。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询 `areas` 表以检查领域存在性（带 `is_deleted = false` 过滤）。
    - **`UPDATE`:** 1条记录在 `areas` 表（设置 `is_deleted = true`, `updated_at`）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **级联影响:**
    - **已关联任务:** 任务的 `area_id` 字段保持不变，前端需要处理已删除领域的显示逻辑。
    - **子领域:** 子领域的 `parent_area_id` 保持不变，可能成为"孤儿"领域（父领域已删除）。
- **日志记录:**
    - 失败时（如领域不存在），可能以 `WARN` 级别记录详细错误信息。

*（无 SSE 事件，无写入许可，无其他已知副作用）*

**注意事项：**
1. 当前实现不会自动清理或调整关联数据（任务、子领域）。
2. 如果需要防止删除有子领域的领域，需要在业务逻辑中添加检查。
3. 如果需要级联删除子领域或任务，需要在业务逻辑中添加相应处理。

### GET /api/areas/{id}

<details>
<summary>源文件: <code>src\features\areas\endpoints\get_area.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/areas/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查询单个领域的详细信息，
> 以便查看该领域的名称、颜色、父领域关系和创建/更新时间等元数据。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询指定 ID 的领域，仅返回未删除的领域（`is_deleted = false`）。
将查询结果组装成 `AreaDto` 并返回。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 领域ID

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `AreaDto`

```json
{
  "id": "uuid",
  "name": "string",
  "color": "string (#RRGGBB)",
  "parent_area_id": "uuid | null",
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Area not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `area_id`:
    - **必须**是有效的 UUID 格式（由 Axum 路径提取器自动验证）。
    - **必须**存在于数据库中。
    - **必须**未被删除（`is_deleted = false`）。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  从路径参数中提取 `area_id`（Axum 自动解析 UUID）。
2.  调用 `database::find_area_by_id` 查询数据库。
3.  如果领域不存在或已删除，返回 `404 Not Found` 错误。
4.  组装 `AreaDto` 并返回 `200 OK`。

## 6. 边界情况 (Edge Cases)

- **`area_id` 格式无效（非 UUID）:** Axum 路径提取器自动返回 `400 Bad Request`。
- **领域不存在:** 返回 `404` 错误。
- **领域已删除:** 返回 `404` 错误（查询条件中排除了已删除的领域）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询 `areas` 表以获取领域详情（带 `is_deleted = false` 过滤）。
- **日志记录:**
    - 失败时（如领域不存在），可能以 `WARN` 级别记录详细错误信息。

*（无数据库写入，无 SSE 事件，无其他已知副作用）*

### PUT /api/areas/{id}

<details>
<summary>源文件: <code>src\features\areas\endpoints\update_area.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

PUT /api/areas/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要更新一个领域的名称、颜色或父领域关系，
> 以便调整我的任务分类体系和视觉标记。

### 2.2. 核心业务逻辑 (Core Business Logic)

更新数据库中指定 ID 的领域的可选字段（`name`、`color`、`parent_area_id`）。
系统将验证新值的有效性，更新 `updated_at` 时间戳，并返回更新后的完整领域信息。
所有字段都是可选的，只更新请求中提供的字段。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 领域ID

**请求体 (Request Body):** `application/json`

```json
{
  "name": "string | null (optional)",
  "color": "string (#RRGGBB) | null (optional)",
  "parent_area_id": "uuid | null | null (optional, 双重 Option)"
}
```

**注意：** `parent_area_id` 使用 `Option<Option<Uuid>>` 结构：
- 未提供字段：不更新父领域
- 提供 `null`：清除父领域（设为根领域）
- 提供 UUID：设置新的父领域

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `AreaDto`

```json
{
  "id": "uuid",
  "name": "string",
  "color": "string (#RRGGBB)",
  "parent_area_id": "uuid | null",
  "created_at": "ISO8601 timestamp",
  "updated_at": "ISO8601 timestamp (更新后的时间)"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Area not found: {id}"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "name", "code": "NAME_EMPTY", "message": "名称不能为空" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `area_id`:
    - **必须**是有效的 UUID 格式（由 Axum 路径提取器自动验证）。
    - **必须**存在于数据库中。
    - **必须**未被删除（`is_deleted = false`）。
    - 违反时返回 `404 NOT_FOUND`
- `name`:
    - 如果提供，**必须**为非空字符串 (trim后)。
    - 违反时返回错误码：`NAME_EMPTY`
- `color`:
    - 如果提供，**必须**符合格式 `#RRGGBB`。
    - 违反时返回错误码：`INVALID_COLOR`
- `parent_area_id`:
    - 如果提供 UUID，**应该**指向一个存在的 Area（当前未强制验证）。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  从路径参数中提取 `area_id`。
2.  启动数据库事务（`db_pool().begin()`）。
3.  调用 `database::check_area_exists_in_tx` 检查领域是否存在。
4.  如果领域不存在，返回 `404 Not Found` 错误并回滚事务。
5.  验证请求字段：
    - 如果提供 `name`，验证非空。
    - 如果提供 `color`，调用 `Area::validate_color` 验证格式。
6.  调用 `database::update_area_in_tx` 更新领域：
    - 动态构建 SQL UPDATE 语句（仅更新提供的字段）
    - 自动更新 `updated_at` 为当前时间
7.  提交数据库事务（`tx.commit()`）。
8.  重新查询更新后的领域（`database::find_area_by_id`）。
9.  组装 `AreaDto` 并返回 `200 OK`。

## 6. 边界情况 (Edge Cases)

- **领域不存在:** 返回 `404` 错误。
- **领域已删除:** 返回 `404` 错误（`check_area_exists_in_tx` 排除已删除的领域）。
- **`name` 为空或全空格:** 返回 `422` 错误，错误码 `NAME_EMPTY`。
- **`color` 格式无效:** 返回 `422` 错误，错误码 `INVALID_COLOR`。
- **请求体所有字段都为 null/未提供:** 仍然成功返回，但只更新 `updated_at` 时间戳。
- **`parent_area_id` 设为自己或循环引用:** 当前实现中未验证，可能导致循环依赖。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 2次（检查存在性 + 重新查询更新后的领域）。
    - **`UPDATE`:** 1条记录在 `areas` 表（更新提供的字段 + `updated_at`）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无 SSE 事件，无写入许可，无其他已知副作用）*

---

## Tasks (任务管理)

### POST /api/tasks

<details>
<summary>源文件: <code>src\features\tasks\endpoints\create_task.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要快速创建一个新任务并放入 Staging 区，
> 以便我能立即捕捉我的想法，而不需要复杂的步骤。

### 2.2. 核心业务逻辑 (Core Business Logic)

在数据库中创建一个新的 `Task` 实体，默认进入 Staging 区（未安排到具体日期）。
新任务的初始状态为未完成（`completed_at = NULL`），无日程安排记录。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "title": "string (required, 1-255 chars)",
  "glance_note": "string | null (optional)",
  "detail_note": "string | null (optional)",
  "estimated_duration": "number | null (optional, 分钟数，0-10080)",
  "area_id": "string (UUID) | null (optional)",
  "due_date": "string (YYYY-MM-DD) | null (optional)",
  "due_date_type": "'soft' | 'hard' | null (optional)",
  "subtasks": "array | null (optional, 最多50个)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskCardDto`

```json
{
  "id": "uuid",
  "title": "string",
  "glance_note": "string | null",
  "schedule_status": "staging",
  "is_completed": false,
  "area": { "id": "uuid", "name": "string", "color": "string" } | null,
  "project_id": null,
  "subtasks": [...] | null,
  "schedules": null,
  "due_date": {...} | null,
  "has_detail_note": boolean
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

## 4. 验证规则 (Validation Rules)

- `title`:
    - **必须**存在。
    - **必须**为非空字符串 (trim后)。
    - 长度**必须**小于等于 255 个字符。
    - 违反时返回错误码：`TITLE_EMPTY` 或 `TITLE_TOO_LONG`
- `estimated_duration`:
    - 如果提供，**必须**是大于等于 0 的整数。
    - 如果提供，**必须**小于等于 10080 (7天 = 7*24*60 分钟)。
    - 违反时返回错误码：`DURATION_NEGATIVE` 或 `DURATION_TOO_LONG`
- `subtasks`:
    - 如果提供，数组长度**必须**小于等于 50。
    - 违反时返回错误码：`TOO_MANY_SUBTASKS`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_create_request` 验证请求体。
2.  获取写入许可（`app_state.acquire_write_permit()`），确保写操作串行执行。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  通过 `IdGenerator` 生成新的 `task_id`（UUID）。
5.  通过 `Clock` 服务获取当前时间 `now`。
6.  构造 `Task` 领域实体对象：
    - 设置 `id`, `title`, `glance_note`, `detail_note` 等字段
    - 设置 `completed_at = None`（未完成）
    - 设置 `created_at = now`, `updated_at = now`
    - 设置 `is_deleted = false`
7.  调用 `TaskRepository::insert_in_tx` 持久化任务到 `tasks` 表。
8.  提交数据库事务（`TransactionHelper::commit`）。
9.  调用 `TaskAssembler::task_to_card_basic` 组装 `TaskCardDto`。
10. 设置 `task_card.schedule_status = Staging`（因为新任务无日程）。
11. 填充 `task_card.schedules` 字段（应为 `None`，因为无日程）。
12. 返回 `201 Created` 和组装好的 `TaskCardDto`。

## 6. 边界情况 (Edge Cases)

- **`title` 为空或全空格:** 返回 `422` 错误，错误码 `TITLE_EMPTY`。
- **`title` 超过 255 字符:** 返回 `422` 错误，错误码 `TITLE_TOO_LONG`。
- **`estimated_duration` 为负数:** 返回 `422` 错误，错误码 `DURATION_NEGATIVE`。
- **`estimated_duration` 超过 10080:** 返回 `422` 错误，错误码 `DURATION_TOO_LONG`。
- **`subtasks` 超过 50 个:** 返回 `422` 错误，错误码 `TOO_MANY_SUBTASKS`。
- **`area_id` 不存在:** 当前实现中正常返回（area 字段为 null），未来可能需要验证。
- **并发创建:** 使用写入许可确保写操作串行执行，避免并发问题。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`INSERT`:** 1条记录到 `tasks` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **日志记录:**
    - 成功时，以 `INFO` 级别记录 "Task created successfully" 及任务ID（如有）。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*

### DELETE /api/tasks/{id}

<details>
<summary>源文件: <code>src\features\tasks\endpoints\delete_task.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

DELETE /api/tasks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我删除一个任务时，我希望系统能够：
> 1. 软删除任务（标记为已删除，而非物理删除）
> 2. 清理所有相关的日程和链接记录
> 3. 智能清理"孤儿"时间块（只关联这一个任务且是自动创建的）

### 2.2. 核心业务逻辑 (Core Business Logic)

软删除任务（设置 `is_deleted = true`），并清理相关数据：
1. 删除所有 `task_time_block_links` 记录
2. 删除所有 `task_schedules` 记录
3. 检查时间块是否变成"孤儿"，如果是且为自动创建的，则删除该时间块

**孤儿时间块定义：**
- 该时间块只链接了这一个任务
- 删除这个任务后，时间块没有任何关联任务
- 时间块的 `title` 与任务 `title` 相同（自动创建的标志）

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "success": true
}
```

**注意：** 副作用（删除的时间块）通过 SSE 事件推送。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取写入许可（`app_state.acquire_write_permit()`）。
2.  启动数据库事务（`TransactionHelper::begin`）。
3.  查询任务的完整数据（`TaskRepository::find_by_id_in_tx`）。
4.  如果任务不存在，返回 404 错误。
5.  组装基础 `TaskCardDto`（用于事件载荷）。
6.  查询任务链接的所有时间块（`TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx`）。
7.  软删除任务（`TaskRepository::soft_delete_in_tx`，设置 `is_deleted = true`）。
8.  删除任务的所有链接记录（`TaskTimeBlockLinkRepository::delete_all_for_task_in_tx`）。
9.  删除任务的所有日程记录（`TaskScheduleRepository::delete_all_in_tx`）。
10. 对每个链接的时间块，调用 `should_delete_orphan_block` 判断是否应该删除：
    - 检查时间块是否还有其他任务（`count_remaining_tasks_in_block_in_tx`）
    - 检查时间块是否是自动创建的（标题与任务标题一致）
11. 在执行删除之前，先查询被删除的时间块的完整数据（用于 SSE 事件）。
12. 删除孤儿时间块（`TimeBlockRepository::soft_delete_in_tx`）。
13. 写入领域事件到 outbox（包含删除的任务和副作用的时间块）。
14. 提交事务（`TransactionHelper::commit`）。
15. 返回成功响应。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已删除:** 幂等，返回 `404` 错误（因为 find_by_id_in_tx 不返回已删除的任务）。
- **时间块还有其他任务:** 不删除时间块（避免影响其他任务）。
- **时间块是手动创建的（标题与任务不一致）:** 不删除时间块（保留用户的手动数据）。
- **时间块是孤儿且自动创建:** 删除时间块。
- **无关联时间块和日程的任务:** 只软删除任务本身。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询任务、链接的时间块、剩余任务数量。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `is_deleted = true`）。
    - **`DELETE`:** 0-N 条记录在 `task_time_block_links` 表。
    - **`DELETE`:** 0-N 条记录在 `task_schedules` 表。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除孤儿时间块）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.deleted` 事件，包含：
        - 删除的任务（`TaskCardDto`）
        - 删除时间（`deleted_at`）
        - 副作用：删除的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 记录被删除的孤儿时间块 ID。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*

### GET /api/tasks/{id}

<details>
<summary>源文件: <code>src\features\tasks\endpoints\get_task.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/tasks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看任务的详细信息（包括完整笔记、日程、时间块等），
> 以便我能全面了解任务的状态和相关安排。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据任务 ID 查询任务的完整详情，包括基础信息、详细笔记、所有日程（包含关联的时间块）。
返回 `TaskDetailDto`，其中包含比 `TaskCardDto` 更详细的信息（如 detail_note）。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskDetailDto`

```json
{
  "card": {
    "id": "uuid",
    "title": "string",
    "glance_note": "string | null",
    "schedule_status": "staging" | "scheduled",
    "is_completed": boolean,
    "area": {...} | null,
    "schedules": [...] | null,
    "due_date": {...} | null,
    "has_detail_note": boolean
  },
  "detail_note": "string | null",
  "project": null,
  "created_at": "2025-10-05T12:00:00Z",
  "updated_at": "2025-10-05T12:00:00Z"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中且未删除（`is_deleted = false`）。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  查询任务实体（`TaskRepository::find_by_id`）。
2.  如果任务不存在或已删除，返回 404 错误。
3.  组装基础 `TaskCardDto`（`TaskAssembler::task_to_card_basic`）。
4.  查询并组装完整的 schedules 数据（`TaskAssembler::assemble_schedules`）。
5.  根据 schedules 判断并设置正确的 `schedule_status`:
    - 如果今天或未来有日程：`Scheduled`
    - 否则：`Staging`
6.  组装 `TaskDetailDto`，包含：
    - `card`: 完整的任务卡片
    - `detail_note`: 详细笔记
    - `project`: 项目信息（暂未实现，返回 null）
    - `created_at`, `updated_at`: 时间戳
7.  返回 `TaskDetailDto`。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已删除 (`is_deleted = true`):** 返回 `404` 错误（视为不存在）。
- **任务无 schedules:** `schedules` 字段为 `None`，`schedule_status` 为 `Staging`。
- **任务无 detail_note:** `detail_note` 字段为 `null`。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次查询 `tasks` 表（获取任务基础信息）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（获取所有日程）。
    - **`SELECT`:** 0-N 次查询 `time_blocks` 表（获取每个日程关联的时间块）。
    - **`SELECT`:** 0-1 次查询 `areas` 表（获取 area 信息，如果有）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（如任务不存在），以 `WARN` 级别记录错误信息。

*（无其他已知副作用）*

### PATCH /api/tasks/{id}

<details>
<summary>源文件: <code>src\features\tasks\endpoints\update_task.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

PATCH /api/tasks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要修改任务的标题、笔记、子任务等信息，
> 并且当我修改任务标题或 area 时，系统能自动同步更新相关的时间块，
> 以保持数据一致性。

### 2.2. 核心业务逻辑 (Core Business Logic)

更新任务的可变字段（标题、笔记、子任务、area 等）。
特殊业务逻辑：当标题或 area 有变更时，自动更新所有"唯一关联且自动创建"的时间块，
确保时间块的标题和 area 与任务保持一致。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

所有字段都是可选的（部分更新）：

```json
{
  "title": "string | null (optional, 1-255 chars)",
  "glance_note": "string | null (optional, 支持置空)",
  "detail_note": "string | null (optional, 支持置空)",
  "estimated_duration": "number | null (optional, 0-10080)",
  "area_id": "UUID | null (optional, 支持置空)",
  "due_date": "string (YYYY-MM-DD) | null (optional)",
  "due_date_type": "'soft' | 'hard' | null (optional)",
  "subtasks": "array | null (optional, 最多50个, 支持置空)"
}
```

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "task": {
    "id": "uuid",
    "title": "string",
    "glance_note": "string | null",
    "schedule_status": "staging" | "scheduled",
    "is_completed": false,
    "area": {...} | null,
    "project_id": null,
    "subtasks": [...] | null,
    "schedules": [...] | null,
    "due_date": {...} | null,
    "has_detail_note": boolean
  }
}
```

**注意：** 副作用（更新的时间块）通过 SSE 事件推送，不在 HTTP 响应中包含。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "任务不存在"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [...]
}
```

## 4. 验证规则 (Validation Rules)

- `title`:
    - 如果提供，**必须**为非空字符串 (trim后)。
    - 如果提供，长度**必须**小于等于 255 个字符。
    - 违反时返回错误码：`TITLE_EMPTY` 或 `TITLE_TOO_LONG`
- `subtasks`:
    - 如果提供，数组长度**必须**小于等于 50。
    - 违反时返回错误码：`TOO_MANY_SUBTASKS`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_update_request` 验证请求体。
2.  获取当前时间 `now`。
3.  获取写入许可（`app_state.acquire_write_permit()`）。
4.  启动数据库事务（`TransactionHelper::begin`）。
5.  查询旧任务数据（`TaskRepository::find_by_id_in_tx`）。
6.  如果任务不存在，返回 404 错误。
7.  更新任务（`TaskRepository::update_in_tx`）。
8.  检查标题或 area 是否有变更。
9.  如果有变更，查询所有链接的时间块（`TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx`）。
10. 对每个时间块：
    - 检查是否是唯一关联（`is_exclusive_link_in_tx`）
    - 检查是否是自动创建的（标题与旧任务标题一致）
    - 如果是唯一关联且自动创建，更新时间块的标题和 area
11. 查询更新后的完整时间块数据（`TimeBlockAssembler::assemble_for_event_in_tx`）。
12. 重新查询任务并组装 `TaskCardDto`。
13. 在事务内填充 `schedules` 字段。
14. 根据 schedules 设置正确的 `schedule_status`。
15. 写入领域事件到 outbox（包含更新的任务和副作用的时间块）。
16. 提交事务（`TransactionHelper::commit`）。
17. 返回更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **`title` 为空或全空格:** 返回 `422` 错误，错误码 `TITLE_EMPTY`。
- **`title` 超过 255 字符:** 返回 `422` 错误，错误码 `TITLE_TOO_LONG`。
- **`subtasks` 超过 50 个:** 返回 `422` 错误，错误码 `TOO_MANY_SUBTASKS`。
- **时间块是手动创建的（标题与任务不一致）:** 不自动更新。
- **时间块关联多个任务:** 不自动更新（避免影响其他任务）。
- **幂等性:** 相同参数重复调用，结果一致，副作用只执行一次（通过 correlation_id 实现）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询旧任务、链接的时间块、排他性检查。
    - **`UPDATE`:** 1条记录在 `tasks` 表。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（仅更新唯一关联且自动创建的时间块）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.updated` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 副作用：更新的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 成功时，记录更新的时间块 ID。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*

### DELETE /api/tasks/{id}/completion

<details>
<summary>源文件: <code>src\features\tasks\endpoints\reopen_task.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

DELETE /api/tasks/{id}/completion

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我误标记了一个任务为已完成，或者需要重新开始一个已完成的任务时，
> 我希望能够将其重新打开，使其回到未完成状态。

### 2.2. 核心业务逻辑 (Core Business Logic)

将已完成的任务重新打开，设置 `completed_at = NULL`，使任务回到未完成状态。
这是 `complete_task` 的逆操作，但不会恢复已删除或截断的时间块（这些是完成任务时的副作用，不可逆）。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "task": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "staging" | "scheduled",
    "is_completed": false,
    "completed_at": null,
    "schedules": [...] | null,
    ...
  }
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "任务尚未完成"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中且未删除。
    - 违反时返回 `404 NOT_FOUND`
- **业务规则验证:**
    - 任务**必须**已完成（`completed_at IS NOT NULL`）。
    - 违反时返回 `409 CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取当前时间 `now`。
2.  获取写入许可（`app_state.acquire_write_permit()`）。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  查询任务（`TaskRepository::find_by_id_in_tx`）。
5.  如果任务不存在，返回 404 错误。
6.  检查任务是否已完成，如果未完成，返回 409 冲突。
7.  设置任务为未完成（`TaskRepository::set_reopened_in_tx`，设置 `completed_at = NULL`, `updated_at = now`）。
8.  提交事务（`TransactionHelper::commit`）。
9.  重新查询任务（`TaskRepository::find_by_id`）。
10. 组装 `TaskCardDto`（`TaskAssembler::task_to_card_basic`）。
11. 填充 `schedules` 字段（`TaskAssembler::assemble_schedules`）。
12. 根据 schedules 设置正确的 `schedule_status`：
    - 如果今天或未来有日程：`Scheduled`
    - 否则：`Staging`
13. 返回重新打开后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已删除 (`is_deleted = true`):** 返回 `404` 错误（视为不存在）。
- **任务未完成:** 返回 `409` 冲突。
- **日程记录:** 不影响已有的日程记录（包括 outcome 状态），只改变任务的完成状态。
- **时间块:** 不恢复完成任务时已删除或截断的时间块（这些副作用不可逆）。
- **幂等性:** 对已未完成的任务调用会返回 409 错误（不具有幂等性）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（事务内）。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `completed_at = NULL`, `updated_at = now`）。
    - **`SELECT`:** 1次查询 `tasks` 表（事务后，重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **无 SSE 事件:** 当前实现不发送 SSE 事件（可能需要补充 `task.reopened` 事件）。
- **日志记录:**
    - 成功时，记录性能指标（各阶段耗时）。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*

### POST /api/tasks/{id}/completion

<details>
<summary>源文件: <code>src\features\tasks\endpoints\complete_task.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/{id}/completion

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我完成一个任务时，我希望系统能够：
> 1. 标记任务为已完成
> 2. 保留当天的日程记录（记录我的努力）
> 3. 清理未来的日程和时间块（因为任务已完成，不需要未来的安排）
> 4. 智能处理正在进行的时间块（截断到当前时间）

### 2.2. 核心业务逻辑 (Core Business Logic)

完成任务，并根据 Cutie 的业务规则智能处理相关的日程和时间块：
1. **当天日程**:
   - 如果今天已有日程：设置为已完成（`outcome = 'COMPLETED_ON_DAY'`）
   - 如果今天没有日程：创建一条新日程并设置为已完成（确保任务保留在今天的看板中）
2. **未来日程**: 删除
3. **时间块处理**（仅针对唯一关联且自动创建的时间块）:
   - 在过去：保留
   - 正在进行（start_time <= now < end_time）：截断到当前时间
   - 在未来：删除

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "task": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "staging" | "scheduled",
    "is_completed": true,
    "completed_at": "2025-10-05T12:00:00Z",
    ...
  }
}
```

**注意：** 副作用（删除/截断的时间块）通过 SSE 事件推送。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "任务已经完成"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中。
    - 违反时返回 `404 NOT_FOUND`
- **业务规则验证:**
    - 任务**不能**已经完成（`completed_at IS NOT NULL`）。
    - 违反时返回 `409 CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取当前时间 `now`。
2.  获取写入许可（`app_state.acquire_write_permit()`）。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  查询任务（`TaskRepository::find_by_id_in_tx`）。
5.  如果任务不存在，返回 404 错误。
6.  检查任务是否已完成，如果是，返回 409 冲突。
7.  设置任务为已完成（`TaskRepository::set_completed_in_tx`）。
8.  处理日程:
    - 检查今天是否有日程（`TaskScheduleRepository::has_schedule_for_day_in_tx`）
    - 如果有：更新当天日程为已完成（`TaskScheduleRepository::update_today_to_completed_in_tx`）
    - 如果没有：创建今天的日程（`TaskScheduleRepository::create_in_tx`），然后更新为已完成
    - 删除未来日程（`TaskScheduleRepository::delete_future_schedules_in_tx`）
9.  查询所有链接的时间块（`TaskTimeBlockLinkRepository::find_linked_time_blocks_in_tx`）。
10. 对每个时间块，调用 `classify_time_block_action` 分类处理动作：
    - 检查是否是唯一关联（`is_exclusive_link_in_tx`）
    - 检查是否是自动创建的（标题与任务标题一致）
    - 根据时间判断动作：保留/截断/删除
11. 在执行删除/截断之前，先查询完整的时间块数据（用于 SSE 事件）。
12. 执行时间块的删除和截断操作：
    - 删除未来的时间块（`TimeBlockRepository::soft_delete_in_tx`）
    - 截断正在进行的时间块（`TimeBlockRepository::truncate_to_in_tx`）
13. 查询被截断的时间块的完整数据。
14. 重新查询任务并组装 `TaskCardDto`。
15. 在事务内填充 `schedules` 字段。
16. 根据 schedules 设置正确的 `schedule_status`。
17. 写入领域事件到 outbox（包含完成的任务和副作用的时间块）。
18. 提交事务（`TransactionHelper::commit`）。
19. 返回完成后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务已完成:** 返回 `409` 冲突（幂等性保护）。
- **今天没有日程:** 自动创建一条日程并标记为已完成，确保任务保留在今天的看板中。
- **今天已有日程:** 直接更新为已完成。
- **时间块是手动创建的（标题与任务不一致）:** 保留，不删除也不截断。
- **时间块关联多个任务:** 保留，不删除也不截断（避免影响其他任务）。
- **时间块在过去:** 保留（记录已完成的工作）。
- **时间块正在进行:** 截断到当前时间（记录部分努力）。
- **时间块在未来:** 删除（因为任务已完成，不需要未来的时间安排）。
- **无日程和时间块的任务:** 创建今天的日程并标记为已完成，然后更新 `completed_at` 字段。
- **幂等性:** 通过 `completed_at` 检查和 correlation_id 实现。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 查询任务、链接的时间块、排他性检查、检查今天是否有日程。
    - **`UPDATE`:** 1条记录在 `tasks` 表（设置 `completed_at`）。
    - **`INSERT`:** 0-1 条记录在 `task_schedules` 表（如果今天没有日程，创建一条）。
    - **`UPDATE`:** 1 条记录在 `task_schedules` 表（今天的日程设为完成）。
    - **`DELETE`:** 0-N 条记录在 `task_schedules` 表（删除未来日程）。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除或截断）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.completed` 事件，包含：
        - 完成的任务（`TaskCardDto`）
        - 副作用：删除的时间块列表（`TimeBlockViewDto[]`）
        - 副作用：截断的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 记录删除和截断的时间块 ID。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*

### POST /api/tasks/{id}/return-to-staging

<details>
<summary>源文件: <code>src\features\tasks\endpoints\return_to_staging.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/{id}/return-to-staging

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我想要重置一个任务的所有未来安排时，我希望系统能够：
> 1. 删除所有今天及未来的日程和时间块
> 2. 保留过去的历史记录（记录我的努力）
> 3. 如果任务已完成，自动重新打开它
> 4. 将任务返回到 Staging 区

### 2.2. 核心业务逻辑 (Core Business Logic)

将任务返回 Staging 区，清理所有今天及未来的安排，但保留过去的历史记录。
具体操作：
1. 删除今天及未来的所有 `task_schedules` 记录
2. 删除今天及未来的所有 `task_time_block_links` 记录
3. 软删除"孤儿"时间块
4. 如果任务已完成，自动重新打开（设置 `completed_at = NULL`）
5. `schedule_status` 变为 `Staging`

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "task_card": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "staging",
    "is_completed": false,
    "schedules": null,
    ...
  }
}
```

**注意：** 副作用（删除的时间块）通过 SSE 事件推送。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中且未删除。
    - 违反时返回 `404 NOT_FOUND`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  获取当前时间 `now`。
2.  计算"今天"的本地日期（UTC零点表示）（`utc_time_to_local_date_utc_midnight`）。
3.  获取写入许可（`app_state.acquire_write_permit()`）。
4.  启动数据库事务（`TransactionHelper::begin`）。
5.  查询任务（`TaskRepository::find_by_id_in_tx`）。
6.  如果任务不存在，返回 404 错误。
7.  查找今天及未来的所有时间块（`database::find_future_time_blocks`）。
8.  对每个时间块，删除任务到时间块的链接（`database::delete_task_time_block_link`）。
9.  对每个时间块，检查是否变成"孤儿"（`TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx`）。
10. 如果时间块没有剩余任务，软删除该时间块（`TimeBlockRepository::soft_delete_in_tx`）。
11. 在删除之前，查询被删除的时间块的完整数据（用于 SSE 事件）。
12. 删除今天及未来的所有日程记录（`database::delete_future_schedules`）。
13. 如果任务已完成，重新打开它（`TaskRepository::set_reopened_in_tx`）。
14. 重新查询任务并组装 `TaskCardDto`。
15. 在事务内填充 `schedules` 字段。
16. 设置 `schedule_status` 为 `Staging`（因为已删除所有未来日程）。
17. 写入领域事件到 outbox（`task.returned_to_staging` 事件）。
18. 提交事务（`TransactionHelper::commit`）。
19. 返回更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **任务没有任何日程和时间块:** 返回成功（幂等操作）。
- **任务已在 Staging 区:** 返回成功（幂等操作）。
- **任务已完成:** 自动重新打开（`completed_at` 设为 NULL）。
- **只有过去的日程:** 保留过去的日程，只删除今天及未来的。
- **时间块还有其他任务:** 不删除时间块（避免影响其他任务）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（验证任务存在）。
    - **`SELECT`:** 1次查询 `time_blocks` 表（查找今天及未来的时间块）。
    - **`DELETE`:** 0-N 条记录在 `task_time_block_links` 表。
    - **`SELECT`:** 0-N 次查询 `task_time_block_links` 表（检查孤儿状态）。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除孤儿时间块）。
    - **`DELETE`:** 0-N 条记录在 `task_schedules` 表（删除今天及未来的日程）。
    - **`UPDATE`:** 0-1 条记录在 `tasks` 表（如果已完成，重新打开）。
    - **`SELECT`:** 1次查询 `tasks` 表（重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.returned_to_staging` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 副作用：删除的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 记录删除的孤儿时间块 ID。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*

### POST /api/tasks/{id}/schedules

<details>
<summary>源文件: <code>src\features\tasks\endpoints\add_schedule.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

POST /api/tasks/{id}/schedules

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要为任务添加日程安排，指定任务在某天需要完成，
> 以便我能更好地规划我的每日工作。

### 2.2. 核心业务逻辑 (Core Business Logic)

为任务添加日程记录到 `task_schedules` 表，初始 `outcome` 为 `PLANNED`。
如果这是任务的第一个日程，任务的 `schedule_status` 会从 `Staging` 变为 `Scheduled`。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID

**请求体 (Request Body):** `application/json`

```json
{
  "scheduled_day": "string (YYYY-MM-DD, required)"
}
```

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`

```json
{
  "task_card": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "scheduled",
    "schedules": [
      {
        "id": "uuid",
        "scheduled_day": "2025-10-05",
        "outcome": "PLANNED",
        "time_blocks": []
      }
    ],
    ...
  }
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "该日期已有日程安排"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "scheduled_day", "code": "INVALID_DATE_FORMAT", "message": "日期格式错误，请使用 YYYY-MM-DD 格式" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `scheduled_day`:
    - **必须**存在。
    - **必须**符合 `YYYY-MM-DD` 格式。
    - 违反时返回错误码：`INVALID_DATE_FORMAT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  解析日期字符串为 `DateTime<Utc>`（`validation::parse_date`）。
2.  获取写入许可（`app_state.acquire_write_permit()`）。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  查询任务（`TaskRepository::find_by_id_in_tx`）。
5.  如果任务不存在，返回 404 错误。
6.  检查该日期是否已有日程（`TaskScheduleRepository::has_schedule_for_day_in_tx`）。
7.  如果已有日程，返回 409 冲突。
8.  创建日程记录（`TaskScheduleRepository::create_in_tx`，初始 `outcome = PLANNED`）。
9.  重新查询任务（`TaskRepository::find_by_id_in_tx`）。
10. 组装 `TaskCardDto`（`TaskAssembler::task_to_card_basic`）。
11. 在事务内填充 `schedules` 字段（`TaskAssembler::assemble_schedules_in_tx`）。
12. 根据 schedules 设置正确的 `schedule_status`（应为 `Scheduled`，因为刚添加了日程）。
13. 写入领域事件到 outbox（`task.scheduled` 事件）。
14. 提交事务（`TransactionHelper::commit`）。
15. 返回 `201 Created` 和更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **该日期已有日程:** 返回 `409` 冲突。
- **日期格式错误:** 返回 `422` 验证错误。
- **添加过去的日期:** 允许（系统不限制日期范围）。
- **添加未来很远的日期:** 允许（系统不限制日期范围）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（验证任务存在）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（检查日期冲突）。
    - **`INSERT`:** 1条记录到 `task_schedules` 表。
    - **`SELECT`:** 1次查询 `tasks` 表（重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.scheduled` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 新增的日期（`scheduled_day`）
- **日志记录:**
    - 成功时，记录日程创建信息。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*

### DELETE /api/tasks/{id}/schedules/{date}

<details>
<summary>源文件: <code>src\features\tasks\endpoints\delete_schedule.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

DELETE /api/tasks/{id}/schedules/{date}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我取消某天的任务安排时，我希望系统能够：
> 1. 删除该日期的日程记录
> 2. 清理该日期关联的时间块链接
> 3. 智能清理"孤儿"时间块（只关联该任务且没有其他用途的时间块）

### 2.2. 核心业务逻辑 (Core Business Logic)

删除任务在指定日期的日程记录，并清理相关数据：
1. 删除 `task_schedules` 记录
2. 删除该日期所有时间块的 `task_time_block_links` 记录
3. 软删除"孤儿"时间块（删除链接后没有任何关联任务的时间块）
4. 如果任务没有剩余日程，`schedule_status` 会变回 `Staging`

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID
- `date` (YYYY-MM-DD, required): 日程日期

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "task_card": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "staging" | "scheduled",
    "schedules": [...] | null,
    ...
  }
}
```

**注意：** 副作用（删除的时间块）通过 SSE 事件推送。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}" | "Schedule not found: Task {id} on {date}"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**是有效的 UUID 格式。
    - **必须**存在于数据库中且未删除。
    - 违反时返回 `404 NOT_FOUND`
- `date`:
    - **必须**符合 `YYYY-MM-DD` 格式。
    - 该日期**必须**有日程记录。
    - 违反时返回 `404 NOT_FOUND` 或 `422 VALIDATION_FAILED`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  解析日期字符串为 `DateTime<Utc>`（`validation::parse_date`）。
2.  获取写入许可（`app_state.acquire_write_permit()`）。
3.  启动数据库事务（`TransactionHelper::begin`）。
4.  查询任务（`TaskRepository::find_by_id_in_tx`）。
5.  如果任务不存在，返回 404 错误。
6.  检查该日期是否有日程（`TaskScheduleRepository::has_schedule_for_day_in_tx`）。
7.  如果该日期没有日程，返回 404 错误。
8.  查找该日期的所有时间块（`database::find_time_blocks_for_day`）。
9.  对每个时间块，删除任务到时间块的链接（`database::delete_task_time_block_link`）。
10. 对每个时间块，检查是否变成"孤儿"（`TaskTimeBlockLinkRepository::count_remaining_tasks_in_block_in_tx`）。
11. 如果时间块没有剩余任务，软删除该时间块（`TimeBlockRepository::soft_delete_in_tx`）。
12. 在删除之前，查询被删除的时间块的完整数据（用于 SSE 事件）。
13. 删除日程记录（`database::delete_schedule`）。
14. 重新查询任务并组装 `TaskCardDto`。
15. 在事务内填充 `schedules` 字段。
16. 根据 schedules 设置正确的 `schedule_status`（如果没有剩余日程，应为 `Staging`）。
17. 写入领域事件到 outbox（`task.schedule_deleted` 事件）。
18. 提交事务（`TransactionHelper::commit`）。
19. 返回更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **该日期没有日程:** 返回 `404` 错误。
- **时间块还有其他任务:** 不删除时间块（避免影响其他任务）。
- **该日期没有时间块:** 只删除日程记录。
- **删除最后一个日程:** `schedule_status` 变为 `Staging`。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（验证任务存在）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（检查日程是否存在）。
    - **`SELECT`:** 1次查询 `time_blocks` 表（查找该日期的时间块）。
    - **`DELETE`:** 0-N 条记录在 `task_time_block_links` 表。
    - **`SELECT`:** 0-N 次查询 `task_time_block_links` 表（检查孤儿状态）。
    - **`UPDATE`:** 0-N 条记录在 `time_blocks` 表（软删除孤儿时间块）。
    - **`DELETE`:** 1条记录在 `task_schedules` 表。
    - **`SELECT`:** 1次查询 `tasks` 表（重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.schedule_deleted` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 删除的日期（`deleted_date`）
        - 副作用：删除的时间块列表（`TimeBlockViewDto[]`）
- **日志记录:**
    - 记录删除的孤儿时间块 ID。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*

### PATCH /api/tasks/{id}/schedules/{date}

<details>
<summary>源文件: <code>src\features\tasks\endpoints\update_schedule.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

PATCH /api/tasks/{id}/schedules/{date}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要修改任务的日程安排，可以更改日期或更新完成状态，
> 以便我能灵活调整我的任务计划。

### 2.2. 核心业务逻辑 (Core Business Logic)

更新任务在指定日期的日程记录。支持两种更新：
1. 更改日期（`new_date`）：将日程从原日期移动到新日期
2. 更新结果状态（`outcome`）：标记日程的完成情况（PLANNED/PRESENCE_LOGGED/COMPLETED_ON_DAY/CARRIED_OVER）

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 任务ID
- `date` (YYYY-MM-DD, required): 原日期

**请求体 (Request Body):** `application/json`

```json
{
  "new_date": "string (YYYY-MM-DD) | null (optional)",
  "outcome": "string ('PLANNED' | 'PRESENCE_LOGGED' | 'COMPLETED_ON_DAY' | 'CARRIED_OVER') | null (optional)"
}
```

**请求头 (Request Headers):**
- `X-Correlation-ID` (optional): 用于前端去重和请求追踪

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`

```json
{
  "task_card": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "scheduled",
    "schedules": [...],
    ...
  }
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {id}" | "Schedule not found: Task {id} on {date}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "目标日期已有日程安排"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "request", "code": "EMPTY_REQUEST", "message": "必须提供 new_date 或 outcome 至少一个字段" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- **请求完整性:**
    - `new_date` 和 `outcome` **至少提供一个**。
    - 违反时返回错误码：`EMPTY_REQUEST`
- `new_date`:
    - 如果提供，**必须**符合 `YYYY-MM-DD` 格式。
    - 违反时返回错误码：`INVALID_DATE_FORMAT`
- `outcome`:
    - 如果提供，**必须**是有效值之一：`PLANNED`, `PRESENCE_LOGGED`, `COMPLETED_ON_DAY`, `CARRIED_OVER`。
    - 违反时返回错误码：`INVALID_OUTCOME`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  验证请求（`validation::validate_request`，确保至少提供一个字段）。
2.  解析原始日期（`validation::parse_date`）。
3.  获取写入许可（`app_state.acquire_write_permit()`）。
4.  启动数据库事务（`TransactionHelper::begin`）。
5.  查询任务（`TaskRepository::find_by_id_in_tx`）。
6.  如果任务不存在，返回 404 错误。
7.  检查原始日期是否有日程（`TaskScheduleRepository::has_schedule_for_day_in_tx`）。
8.  如果原始日期没有日程，返回 404 错误。
9.  如果提供了 `new_date`：
    - 解析新日期
    - 如果新日期与原日期不同，检查新日期是否已有日程
    - 如果新日期已有日程，返回 409 冲突
    - 更新日程的日期（`database::update_schedule_date`）
10. 如果提供了 `outcome`：
    - 解析 outcome 枚举值（`validation::parse_outcome`）
    - 确定目标日期（如果更改了日期，使用新日期；否则使用原日期）
    - 更新日程的 outcome（`database::update_schedule_outcome`）
11. 重新查询任务并组装 `TaskCardDto`。
12. 在事务内填充 `schedules` 字段。
13. 根据 schedules 设置正确的 `schedule_status`。
14. 写入领域事件到 outbox（`task.schedule_updated` 事件）。
15. 提交事务（`TransactionHelper::commit`）。
16. 返回更新后的任务。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **原日期没有日程:** 返回 `404` 错误。
- **新日期已有日程:** 返回 `409` 冲突。
- **新日期与原日期相同:** 允许（仅视为 outcome 更新）。
- **两个字段都不提供:** 返回 `422` 验证错误。
- **outcome 值无效:** 返回 `422` 验证错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次查询 `tasks` 表（验证任务存在）。
    - **`SELECT`:** 1-2次查询 `task_schedules` 表（检查原日期和新日期）。
    - **`UPDATE`:** 1条记录在 `task_schedules` 表（更新日期和/或 outcome）。
    - **`SELECT`:** 1次查询 `tasks` 表（重新获取数据）。
    - **`SELECT`:** 1次查询 `task_schedules` 表（填充 schedules）。
    - **`INSERT`:** 1条记录到 `event_outbox` 表（领域事件）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **写入许可:**
    - 获取应用级写入许可，确保 SQLite 写操作串行执行。
- **SSE 事件:**
    - 发送 `task.schedule_updated` 事件，包含：
        - 更新后的任务（`TaskCardDto`）
        - 原日期（`original_date`）
        - 新日期（`new_date`，如果有）
        - 新 outcome（`outcome`，如果有）
- **日志记录:**
    - 成功时，记录日程更新信息。
    - 失败时，记录详细错误信息。

*（无其他已知副作用）*

---

## Time Blocks (时间块管理)

### POST /api/time-blocks

<details>
<summary>源文件: <code>src\features\time_blocks\endpoints\create_time_block.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

POST /api/time-blocks

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要在日历上创建一个纯时间块（会议、约会、独立事件），
> 以便我能够管理我的日程安排，而不必关联到具体的任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

创建一个独立的时间块，不关联任何任务。此端点专注于纯时间块的创建（会议、约会等）。
关键业务规则：时间块不允许重叠，系统会自动检测并拒绝重叠的时间段。
如果需要创建与任务关联的时间块，应使用专门的 `POST /api/time-blocks/from-task` 端点。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "start_time": "string (ISO 8601 UTC, required)",
  "end_time": "string (ISO 8601 UTC, required)",
  "title": "string | null (optional, 最多255字符)",
  "glance_note": "string | null (optional)",
  "detail_note": "string | null (optional)",
  "area_id": "UUID | null (optional)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`
*   **Schema:** `TimeBlockViewDto`

```json
{
  "id": "uuid",
  "start_time": "2025-10-05T14:00:00Z",
  "end_time": "2025-10-05T15:00:00Z",
  "title": "string | null",
  "glance_note": "string | null",
  "detail_note": "string | null",
  "area_id": "uuid | null",
  "linked_tasks": [],
  "is_recurring": false
}
```

**400 Bad Request:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "开始时间必须早于结束时间",
  "details": [
    { "field": "time_range", "code": "INVALID_TIME_RANGE", "message": "开始时间必须早于结束时间" }
  ]
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "该时间段与现有时间块重叠，时间块不允许重叠"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "title", "code": "TITLE_TOO_LONG", "message": "标题不能超过255个字符" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `start_time`:
    - **必须**存在。
    - **必须**是有效的 ISO 8601 UTC 时间格式。
    - **必须**早于 `end_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- `end_time`:
    - **必须**存在。
    - **必须**是有效的 ISO 8601 UTC 时间格式。
    - **必须**晚于 `start_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- `title`:
    - 如果提供，长度**必须**小于等于 255 个字符。
    - 违反时返回错误码：`TITLE_TOO_LONG`
- **时间冲突验证**:
    - 新时间块的时间范围**不能**与现有时间块重叠。
    - 违反时返回错误码：`CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_create_request` 验证请求体。
2.  启动数据库事务（`app_state.db_pool().begin()`）。
3.  调用 `TimeBlockConflictChecker::check_in_tx` 检查时间冲突：
    - 查询时间范围重叠的现有时间块
    - 如果存在重叠，返回 409 冲突错误
4.  通过 `IdGenerator` 生成新的 `block_id`（UUID）。
5.  通过 `Clock` 服务获取当前时间 `now`。
6.  构造 `TimeBlock` 领域实体对象：
    - 设置 `id`, `title`, `glance_note`, `detail_note`, `area_id`
    - 设置 `start_time`, `end_time`
    - 设置 `created_at = now`, `updated_at = now`
    - 设置 `is_deleted = false`
    - 设置循环相关字段为 `None`（当前版本不支持循环）
7.  调用 `TimeBlockRepository::insert_in_tx` 持久化时间块到 `time_blocks` 表。
8.  提交数据库事务。
9.  组装返回的 `TimeBlockViewDto`：
    - 填充所有基础字段
    - 设置 `linked_tasks = []`（纯时间块无关联任务）
    - 设置 `is_recurring = false`
10. 返回 `201 Created` 和组装好的 `TimeBlockViewDto`。

## 6. 边界情况 (Edge Cases)

- **`start_time >= end_time`:** 返回 `400` 错误，错误码 `INVALID_TIME_RANGE`。
- **时间范围与现有时间块重叠:** 返回 `409` 错误，错误码 `CONFLICT`。
- **`title` 超过 255 字符:** 返回 `422` 错误，错误码 `TITLE_TOO_LONG`。
- **`area_id` 不存在:** 当前实现中正常返回（area_id 字段为提供的值），未来可能需要验证。
- **无标题的时间块:** 允许创建，`title` 为 `null`。
- **并发创建重叠时间块:** 事务隔离保证只有一个会成功，其他会收到冲突错误。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询重叠的时间块（冲突检测）。
    - **`INSERT`:** 1条记录到 `time_blocks` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 成功时，可能记录时间块创建信息（如有配置）。
    - 失败时（验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*

### POST /api/time-blocks/from-task

<details>
<summary>源文件: <code>src\features\time_blocks\endpoints\create_from_task.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

POST /api/time-blocks/from-task

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我将一个任务拖动到日历的特定时间段时，
> 我希望系统能够：
> 1. 为这个任务创建一个时间块（分配具体的执行时间）
> 2. 自动创建任务的日程记录（标记任务在该日期有安排）
> 3. 更新任务的状态为"已排期"
> 4. 返回完整的任务信息，以便我能看到更新后的状态

### 2.2. 核心业务逻辑 (Core Business Logic)

这是专门为"拖动任务到日历"场景设计的端点，执行一系列原子操作：
1. 创建时间块（记录具体的执行时间段）
2. 建立任务与时间块的链接关系
3. 创建或更新任务的日程记录（task_schedules），标记任务在该日期有安排
4. 时间块的标题默认使用任务标题（可自定义）
5. 时间块的 area 继承任务的 area
6. 返回完整的时间块视图和更新后的任务卡片

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "task_id": "UUID (required)",
  "start_time": "string (ISO 8601 UTC, required)",
  "end_time": "string (ISO 8601 UTC, required)",
  "title": "string | null (optional, 默认使用任务标题)"
}
```

### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`

```json
{
  "time_block": {
    "id": "uuid",
    "start_time": "2025-10-05T14:00:00Z",
    "end_time": "2025-10-05T15:00:00Z",
    "title": "string",
    "glance_note": null,
    "detail_note": null,
    "area_id": "uuid | null",
    "linked_tasks": [
      {
        "id": "uuid",
        "title": "string",
        "is_completed": false
      }
    ],
    "is_recurring": false
  },
  "updated_task": {
    "id": "uuid",
    "title": "string",
    "schedule_status": "scheduled",
    "is_completed": false,
    "area": {...} | null,
    "schedules": [
      {
        "scheduled_day": "2025-10-05",
        "outcome": null
      }
    ],
    ...
  }
}
```

**400 Bad Request:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "开始时间必须早于结束时间",
  "details": [
    { "field": "time_range", "code": "INVALID_TIME_RANGE", "message": "开始时间必须早于结束时间" }
  ]
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "Task not found: {task_id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "该时间段与现有时间块重叠"
}
```

## 4. 验证规则 (Validation Rules)

- `task_id`:
    - **必须**存在。
    - **必须**是有效的 UUID 格式。
    - 对应的任务**必须**存在于数据库中。
    - 违反时返回错误码：`NOT_FOUND`
- `start_time`:
    - **必须**存在。
    - **必须**是有效的 ISO 8601 UTC 时间格式。
    - **必须**早于 `end_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- `end_time`:
    - **必须**存在。
    - **必须**是有效的 ISO 8601 UTC 时间格式。
    - **必须**晚于 `start_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- **时间冲突验证**:
    - 新时间块的时间范围**不能**与现有时间块重叠。
    - 违反时返回错误码：`CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_request` 验证请求体。
2.  启动数据库事务（`app_state.db_pool().begin()`）。
3.  调用 `TaskRepository::find_by_id_in_tx` 查询任务：
    - 如果任务不存在，返回 404 错误
4.  调用 `TimeBlockConflictChecker::check_in_tx` 检查时间冲突：
    - 查询时间范围重叠的现有时间块
    - 如果存在重叠，返回 409 冲突错误
5.  通过 `IdGenerator` 生成新的 `block_id`（UUID）。
6.  通过 `Clock` 服务获取当前时间 `now`。
7.  确定时间块标题：使用请求中的自定义标题，如果没有则使用任务标题。
8.  构造 `TimeBlock` 领域实体对象：
    - 设置 `id`, `title`（来自请求或任务）
    - 设置 `start_time`, `end_time`
    - 设置 `area_id`（继承任务的 area）
    - 设置 `created_at = now`, `updated_at = now`
    - 设置 `is_deleted = false`
9.  调用 `TimeBlockRepository::insert_in_tx` 持久化时间块。
10. 调用 `TaskTimeBlockLinkRepository::link_in_tx` 建立任务与时间块的链接。
11. 计算日程日期：
    - 使用 `utc_time_to_local_date_utc_midnight` 将 UTC 时间转换为本地日期的 UTC 零点
    - 例如：`2025-10-02T18:00:00Z (UTC)` → `2025-10-03T00:00:00Z`（如果在 UTC+8 时区）
12. 检查该日期是否已有日程记录（`TaskScheduleRepository::has_schedule_for_day_in_tx`）。
13. 如果没有日程记录，创建新的日程（`TaskScheduleRepository::create_in_tx`）。
14. 提交数据库事务。
15. 组装返回的 `TimeBlockViewDto`：
    - 填充所有基础字段
    - 填充 `linked_tasks`（包含任务摘要）
16. 组装返回的 `TaskCardDto`：
    - 调用 `TaskAssembler::task_to_card_basic` 创建基础卡片
    - 设置 `schedule_status = Scheduled`
    - 填充 `schedules` 字段（包含新创建的日程）
17. 返回 `201 Created` 和包含时间块与任务的响应对象。

## 6. 边界情况 (Edge Cases)

- **任务不存在:** 返回 `404` 错误。
- **`start_time >= end_time`:** 返回 `400` 错误，错误码 `INVALID_TIME_RANGE`。
- **时间范围与现有时间块重叠:** 返回 `409` 错误，错误码 `CONFLICT`。
- **该日期已有日程记录:** 不重复创建，保持幂等性。
- **跨时区的时间处理:** 使用系统时区正确计算日程日期（例如：UTC 晚上 10 点在 UTC+8 时区算第二天）。
- **任务已完成:** 当前实现允许为已完成的任务创建时间块（未来可能需要限制）。
- **并发创建:** 事务隔离保证数据一致性。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询任务是否存在。
    - **`SELECT`:** 1次，查询重叠的时间块（冲突检测）。
    - **`SELECT`:** 1次，检查日程是否已存在。
    - **`INSERT`:** 1条记录到 `time_blocks` 表。
    - **`INSERT`:** 1条记录到 `task_time_block_links` 表。
    - **`INSERT`:** 0-1条记录到 `task_schedules` 表（如果该日期尚无日程）。
    - **`SELECT`:** 1次，查询任务的完整日程列表（用于返回）。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 记录时间块创建和日程创建的详细信息（包含时间转换日志）。
    - 失败时，记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*

### DELETE /api/time-blocks/{id}

<details>
<summary>源文件: <code>src\features\time_blocks\endpoints\delete_time_block.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

DELETE /api/time-blocks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我决定取消或移除一个日历上的时间块时，
> 我希望系统能够：
> 1. 删除这个时间块（软删除，标记为已删除）
> 2. 断开时间块与任务的链接关系
> 3. 保留任务的排期状态（任务仍然在"已排期"列表中，只是没有具体的时间段）

### 2.2. 核心业务逻辑 (Core Business Logic)

软删除时间块（设置 `is_deleted = true`），不物理删除数据。
关键业务规则：
1. 删除时间块**不影响**任务的排期状态（`task_schedules` 记录保留）
2. 删除 `task_time_block_links` 表中的链接关系
3. 任务仍然保持"已排期"状态，只是失去了具体的执行时间段

Cutie 的设计哲学：
- 时间块代表"具体的执行时间"
- 任务排期代表"是否被安排到某一天"
- 删除时间块≠取消任务排期
- 任务仍在 Planned 列表中，只是没有具体时间段

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 时间块ID

### 3.2. 响应 (Responses)

**204 No Content:**

删除成功，无返回内容。

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "TimeBlock not found: {id}"
}
```

## 4. 验证规则 (Validation Rules)

- `id`:
    - **必须**是有效的 UUID 格式。
    - 对应的时间块**必须**存在于数据库中（不论是否已删除）。
    - 违反时返回错误码：`NOT_FOUND`
- **幂等性**:
    - 如果时间块已被删除，仍然返回 `204 No Content`（幂等操作）。
    - 重复删除不会产生错误。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  启动数据库事务（`app_state.db_pool().begin()`）。
2.  调用 `TimeBlockRepository::exists_in_tx` 检查时间块是否存在：
    - 如果时间块不存在，返回 404 错误
3.  调用 `TimeBlockRepository::soft_delete_in_tx` 软删除时间块：
    - 设置 `is_deleted = true`
    - 不删除物理记录（保留审计和历史数据）
4.  调用 `TaskTimeBlockLinkRepository::delete_all_for_block_in_tx` 删除所有任务链接：
    - 断开时间块与任务的关联关系
    - **注意**：不删除 `task_schedules` 记录
5.  提交数据库事务。
6.  返回 `204 No Content`。

## 6. 边界情况 (Edge Cases)

- **时间块不存在:** 返回 `404` 错误。
- **时间块已被删除:** 返回 `204 No Content`（幂等性）。
- **时间块关联多个任务:** 删除所有链接关系，不影响任务的排期状态。
- **时间块没有关联任务:** 正常删除时间块。
- **并发删除:** 事务隔离保证只执行一次实际删除，其他请求返回成功（幂等）。
- **任务状态保持:** 删除时间块后，任务仍然保持"已排期"状态（如果有 `task_schedules` 记录）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，检查时间块是否存在。
    - **`UPDATE`:** 1条记录在 `time_blocks` 表（设置 `is_deleted = true`）。
    - **`DELETE`:** 0-N 条记录在 `task_time_block_links` 表（删除所有链接）。
    - **注意**：**不修改** `task_schedules` 表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **任务状态影响:**
    - 关联的任务失去具体的执行时间段。
    - 任务的排期状态（`schedule_status`）保持不变（如果有其他日程或时间块）。
    - 任务的 `schedules` 列表保持不变。
- **日志记录:**
    - 成功时，可能记录删除操作（如有配置）。
    - 失败时，记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*

### PATCH /api/time-blocks/{id}

<details>
<summary>源文件: <code>src\features\time_blocks\endpoints\update_time_block.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

PATCH /api/time-blocks/{id}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要调整日历上时间块的时间、标题、笔记或所属区域，
> 以便我能够灵活管理我的日程安排，适应计划的变化。
> 特别是在拖动时间块调整时间时，系统应该自动验证是否与其他时间块冲突。

### 2.2. 核心业务逻辑 (Core Business Logic)

更新现有时间块的可变字段（时间范围、标题、笔记、area 等）。
支持部分更新（PATCH 语义），只需提供要更改的字段。
关键业务规则：
1. 如果更新时间范围，必须确保新时间范围不与其他时间块重叠（排除自身）
2. 更新后自动刷新 `updated_at` 时间戳
3. 返回完整的时间块视图（包含关联的任务信息）

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `id` (UUID, required): 时间块ID

**请求体 (Request Body):** `application/json`

所有字段都是可选的（部分更新）：

```json
{
  "start_time": "string (ISO 8601 UTC) | null (optional)",
  "end_time": "string (ISO 8601 UTC) | null (optional)",
  "title": "string | null (optional, 最多255字符, 支持置空)",
  "glance_note": "string | null (optional, 支持置空)",
  "detail_note": "string | null (optional, 支持置空)",
  "area_id": "UUID | null (optional, 支持置空)"
}
```

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `TimeBlockViewDto`

```json
{
  "id": "uuid",
  "start_time": "2025-10-05T14:00:00Z",
  "end_time": "2025-10-05T16:00:00Z",
  "title": "string | null",
  "glance_note": "string | null",
  "detail_note": "string | null",
  "area_id": "uuid | null",
  "linked_tasks": [
    {
      "id": "uuid",
      "title": "string",
      "is_completed": false
    }
  ],
  "is_recurring": false
}
```

**400 Bad Request:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "开始时间必须早于结束时间",
  "details": [
    { "field": "time_range", "code": "INVALID_TIME_RANGE", "message": "开始时间必须早于结束时间" }
  ]
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "TimeBlock not found: {id}"
}
```

**409 Conflict:**

```json
{
  "error_code": "CONFLICT",
  "message": "该时间段与现有时间块重叠，时间块不允许重叠"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "title", "code": "TITLE_TOO_LONG", "message": "标题不能超过255个字符" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `id`:
    - **必须**是有效的 UUID 格式。
    - 对应的时间块**必须**存在且未被删除。
    - 违反时返回错误码：`NOT_FOUND`
- `start_time`:
    - 如果提供，**必须**是有效的 ISO 8601 UTC 时间格式。
    - 如果同时提供 `start_time` 和 `end_time`，**必须**满足 `start_time < end_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- `end_time`:
    - 如果提供，**必须**是有效的 ISO 8601 UTC 时间格式。
    - 如果同时提供 `start_time` 和 `end_time`，**必须**满足 `start_time < end_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- **最终时间范围验证**:
    - 合并现有值和新值后，**必须**满足 `final_start_time < final_end_time`。
    - 违反时返回错误码：`INVALID_TIME_RANGE`
- `title`:
    - 如果提供，长度**必须**小于等于 255 个字符。
    - 违反时返回错误码：`TITLE_TOO_LONG`
- **时间冲突验证**:
    - 如果更新了时间范围，新时间范围**不能**与其他时间块重叠（排除自身）。
    - 违反时返回错误码：`CONFLICT`

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation::validate_update_request` 验证请求体（初步验证）。
2.  启动数据库事务（`app_state.db_pool().begin()`）。
3.  调用 `TimeBlockRepository::find_by_id_in_tx` 查询现有时间块：
    - 如果时间块不存在，返回 404 错误
4.  确定最终的时间范围：
    - `final_start_time = request.start_time.unwrap_or(existing_block.start_time)`
    - `final_end_time = request.end_time.unwrap_or(existing_block.end_time)`
5.  再次验证最终时间范围：
    - 检查 `final_start_time < final_end_time`
    - 如果不满足，返回 400 错误
6.  如果时间范围发生变化（`request.start_time` 或 `request.end_time` 非空）：
    - 调用 `TimeBlockConflictChecker::check_in_tx` 检查时间冲突
    - 传入 `Some(id)` 排除当前时间块
    - 如果存在重叠，返回 409 冲突错误
7.  通过 `Clock` 服务获取当前时间 `now`。
8.  调用 `TimeBlockRepository::update_in_tx` 更新时间块。
9.  提交数据库事务。
10. 重新查询时间块以获取最新数据（`TimeBlockRepository::find_by_id`）。
11. 组装返回的 `TimeBlockViewDto`：
    - 填充所有基础字段
    - 调用 `LinkedTaskAssembler::get_for_time_block` 填充关联任务
12. 返回 `200 OK` 和组装好的 `TimeBlockViewDto`。

## 6. 边界情况 (Edge Cases)

- **时间块不存在:** 返回 `404` 错误。
- **只更新 `start_time`:** 结合现有 `end_time`，验证最终时间范围。
- **只更新 `end_time`:** 结合现有 `start_time`，验证最终时间范围。
- **最终时间范围无效:** 返回 `400` 错误，错误码 `INVALID_TIME_RANGE`。
- **时间范围与其他时间块重叠:** 返回 `409` 错误，错误码 `CONFLICT`。
- **`title` 超过 255 字符:** 返回 `422` 错误，错误码 `TITLE_TOO_LONG`。
- **清空 `title`（设置为 `null`）:** 允许，时间块可以没有标题。
- **更新 `area_id` 为 `null`:** 允许，时间块可以不属于任何区域。
- **空更新（不提供任何字段）:** 仍然更新 `updated_at` 时间戳，返回成功。
- **并发更新:** 事务隔离保证数据一致性，后者可能因冲突检测失败。
- **幂等性:** 相同参数重复调用，结果一致（`updated_at` 会变化）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询现有时间块。
    - **`SELECT`:** 0-1次，查询重叠的时间块（仅当时间范围变化时）。
    - **`UPDATE`:** 1条记录在 `time_blocks` 表。
    - **`SELECT`:** 1次，重新查询更新后的时间块。
    - **`SELECT`:** 1次，查询关联的任务列表。
    - **(事务):** 所有数据库写操作包含在一个数据库事务内。
- **日志记录:**
    - 成功时，记录时间块更新信息（包含 block_id）。
    - 失败时，记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*

### GET /api/time-blocks?start_date={start_date}&end_date={end_date}

<details>
<summary>源文件: <code>src\features\time_blocks\endpoints\list_time_blocks.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/time-blocks?start_date={start_date}&end_date={end_date}

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，当我查看日历视图时，我需要看到特定时间范围内的所有时间块，
> 包括每个时间块关联的任务信息，以便我能够了解我的日程安排和待办事项。

### 2.2. 核心业务逻辑 (Core Business Logic)

查询指定时间范围内的所有未删除的时间块，并为每个时间块组装完整的视图模型。
返回的数据包括：
1. 时间块的基本信息（时间、标题、笔记、区域）
2. 关联的任务摘要列表（任务ID、标题、完成状态）
3. 是否为循环时间块的标记

查询结果按 `start_time` 升序排序，方便前端按时间顺序展示。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**Query Parameters:**
- `start_date` (string, optional): 开始时间（ISO 8601 UTC 格式）
- `end_date` (string, optional): 结束时间（ISO 8601 UTC 格式）

**注意**：两个参数都是可选的：
- 如果都不提供，返回所有时间块
- 如果只提供 `start_date`，返回该时间之后的所有时间块
- 如果只提供 `end_date`，返回该时间之前的所有时间块
- 如果都提供，返回该时间范围内的时间块

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `Array<TimeBlockViewDto>`

```json
[
  {
    "id": "uuid",
    "start_time": "2025-10-05T09:00:00Z",
    "end_time": "2025-10-05T10:00:00Z",
    "title": "string | null",
    "glance_note": "string | null",
    "detail_note": "string | null",
    "area_id": "uuid | null",
    "linked_tasks": [
      {
        "id": "uuid",
        "title": "string",
        "is_completed": false
      }
    ],
    "is_recurring": false
  },
  {
    "id": "uuid",
    "start_time": "2025-10-05T14:00:00Z",
    "end_time": "2025-10-05T15:00:00Z",
    "title": "string | null",
    "glance_note": "string | null",
    "detail_note": "string | null",
    "area_id": "uuid | null",
    "linked_tasks": [],
    "is_recurring": false
  }
]
```

**400 Bad Request:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "时间范围参数格式无效"
}
```

**空结果情况:**

如果指定时间范围内没有时间块，返回空数组 `[]`。

## 4. 验证规则 (Validation Rules)

- `start_date`:
    - 如果提供，**必须**是有效的 ISO 8601 格式（支持 RFC3339）。
    - 如果格式无效，将被忽略（视为未提供）。
- `end_date`:
    - 如果提供，**必须**是有效的 ISO 8601 格式（支持 RFC3339）。
    - 如果格式无效，将被忽略（视为未提供）。
- **时间范围逻辑**:
    - 不要求 `start_date < end_date`（由数据库查询自然处理）。
    - 如果 `start_date >= end_date`，可能返回空数组（取决于数据）。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  解析查询参数：
    - 尝试将 `start_date` 字符串解析为 `DateTime<Utc>`
    - 尝试将 `end_date` 字符串解析为 `DateTime<Utc>`
    - 如果解析失败，将对应参数设为 `None`
2.  调用 `TimeBlockRepository::find_in_range` 查询时间块：
    - 传入 `start_time` 和 `end_time`（可能为 `None`）
    - 查询所有未删除的时间块（`is_deleted = false`）
    - 根据时间范围过滤结果
3.  对每个时间块，调用 `assemble_time_block_view` 组装视图模型：
    - 创建 `TimeBlockViewDto` 基础对象
    - 填充所有基础字段（`id`, `start_time`, `end_time`, `title`, 等）
    - 调用 `LinkedTaskAssembler::get_for_time_block` 查询关联的任务
    - 填充 `linked_tasks` 字段
    - 设置 `is_recurring` 标记（基于 `recurrence_rule` 是否为空）
4.  对结果列表按 `start_time` 升序排序。
5.  返回 `200 OK` 和时间块视图列表。

## 6. 边界情况 (Edge Cases)

- **没有提供时间范围参数:** 返回所有未删除的时间块。
- **时间范围内没有时间块:** 返回空数组 `[]`。
- **`start_date` 格式无效:** 忽略该参数，相当于没有下限。
- **`end_date` 格式无效:** 忽略该参数，相当于没有上限。
- **`start_date >= end_date`:** 可能返回空数组或部分结果（取决于数据）。
- **时间块没有关联任务:** `linked_tasks` 字段为空数组 `[]`。
- **时间块关联多个任务:** `linked_tasks` 包含所有关联任务的摘要。
- **大量时间块:** 当前实现一次性加载所有结果（未来可能需要分页）。
- **跨时区查询:** 所有时间都使用 UTC，前端负责时区转换和展示。

## 7. 预期副作用 (Expected Side Effects)

- **数据库读取:**
    - **`SELECT`:** 1次，查询指定范围内的时间块（`time_blocks` 表）。
    - **`SELECT`:** N次，为每个时间块查询关联的任务（`task_time_block_links` 和 `tasks` 表）。
    - **注意**：当前实现使用 N+1 查询模式，可能需要优化为 JOIN 查询（性能考虑）。
    - **无事务**：只读操作，不使用事务。
- **性能考虑:**
    - 时间块数量较多时，可能需要较长查询时间。
    - 未来可能需要实现分页或虚拟滚动。
- **日志记录:**
    - 失败时，记录详细错误信息。

*（无其他已知副作用，不发送 SSE 事件）*

---

## View Preferences (视图偏好)

### PUT /api/view-preferences

<details>
<summary>源文件: <code>src\features\view_preferences\endpoints\save_view_preference.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

PUT /api/view-preferences

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户,当我在某个视图(如Staging区、今日看板、项目看板等)中拖拽调整任务顺序后,
> 我想要系统能够持久化保存这个排序配置,以便下次打开该视图时能恢复我上次的排序。

### 2.2. 核心业务逻辑 (Core Business Logic)

保存或更新某个视图的任务排序偏好。使用 UPSERT 逻辑(INSERT OR REPLACE),
如果该 `context_key` 已存在则更新,否则创建新记录。
排序配置以任务ID数组的形式存储,数组顺序即为任务的显示顺序。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "context_key": "string (required, 视图上下文标识)",
  "sorted_task_ids": ["string"] (required, 任务ID数组,非空)
}
```

**context_key 格式规范:**
- Staging区: `misc::staging`
- 每日视图: `daily::YYYY-MM-DD` (如 `daily::2025-10-03`)
- Area视图: `area::{uuid}` (如 `area::a1b2c3d4-...`)
- Project视图: `project::{uuid}` (如 `project::a1b2c3d4-...`)

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `ViewPreferenceDto`

```json
{
  "context_key": "daily::2025-10-03",
  "sorted_task_ids": [
    "task-uuid-1",
    "task-uuid-2",
    "task-uuid-3"
  ],
  "updated_at": "2025-10-05T12:00:00Z"
}
```

**422 Unprocessable Entity:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "context_key", "code": "CONTEXT_KEY_EMPTY", "message": "Context key 不能为空" }
  ]
}
```

或

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "sorted_task_ids", "code": "TASK_IDS_EMPTY", "message": "任务ID列表不能为空" }
  ]
}
```

## 4. 验证规则 (Validation Rules)

- `context_key`:
    - **必须**存在且为非空字符串(trim后)。
    - 违反时返回错误码: `CONTEXT_KEY_EMPTY`
- `sorted_task_ids`:
    - **必须**存在且为非空数组。
    - 违反时返回错误码: `TASK_IDS_EMPTY`
    - 注意: 允许包含重复的任务ID(不做唯一性校验)
    - 注意: 不验证任务ID是否真实存在于数据库中(前端负责保证数据有效性)

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  验证请求体:
    - 检查 `context_key` 是否为空(trim后)。
    - 检查 `sorted_task_ids` 数组是否为空。
2.  通过 `Clock` 服务获取当前时间 `now`。
3.  构建 `ViewPreference` 领域实体:
    - 设置 `context_key` 为请求中的值。
    - 设置 `sorted_task_ids` 为请求中的数组。
    - 设置 `updated_at` 为当前时间。
4.  调用数据访问层执行 UPSERT 操作(`database::upsert`):
    - 将 `sorted_task_ids` 数组序列化为 JSON 字符串。
    - 将 `updated_at` 转换为 RFC 3339 字符串。
    - 执行 `INSERT ... ON CONFLICT(context_key) DO UPDATE` SQL。
5.  重新查询保存后的记录(`database::find_by_context_key`):
    - 确保返回的数据与数据库中实际存储的数据一致。
6.  将查询结果转换为 DTO(`ViewPreferenceDto`)。
7.  返回 `200 OK` 和 DTO。

## 6. 边界情况 (Edge Cases)

- **context_key 为空字符串或仅包含空格:** 返回 `422` 错误,错误码 `CONTEXT_KEY_EMPTY`。
- **sorted_task_ids 为空数组:** 返回 `422` 错误,错误码 `TASK_IDS_EMPTY`。
- **sorted_task_ids 包含重复的任务ID:** 允许,不做去重处理(保留原始顺序)。
- **sorted_task_ids 包含不存在的任务ID:** 允许,不做验证(前端负责过滤)。
- **重复调用相同的 context_key:** UPSERT 逻辑,每次更新 `sorted_task_ids` 和 `updated_at`。
- **幂等性:** 相同参数重复调用,结果一致(最后更新时间会改变,但排序数据相同)。
- **并发写入相同 context_key:** SQLite 的 UPSERT 语法保证原子性,后执行的请求会覆盖先执行的结果。

## 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`INSERT` 或 `UPDATE`:** 1条记录到 `view_preferences` 表(取决于记录是否已存在)。
        - 新记录: `INSERT` 1条。
        - 已存在: `UPDATE` 1条(更新 `sorted_task_ids` 和 `updated_at`)。
    - **`SELECT`:** 1次查询 `view_preferences` 表(保存后重新查询以返回最新数据)。
    - **无事务包装:** 单条 UPSERT 语句,无需显式事务(SQLite 隐式事务)。
- **无 SSE 事件:** 此端点不发送 SSE 事件(视图偏好是客户端本地状态,无需广播)。
- **日志记录:**
    - 成功时,以 `INFO` 或 `DEBUG` 级别记录保存操作。
    - 失败时(如验证失败或数据库错误),以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*(无其他已知副作用)*

### GET /api/view-preferences/:context_key

<details>
<summary>源文件: <code>src\features\view_preferences\endpoints\get_view_preference.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/view-preferences/:context_key

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户,我想要获取某个特定视图(如Staging区、今日看板、项目看板等)的任务排序配置,
> 以便应用程序能够按照我上次保存的顺序显示任务。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据视图上下文标识(context_key)查询该视图的任务排序偏好。
返回包含排序后的任务ID数组和最后更新时间的 `ViewPreferenceDto`。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `context_key` (String, required): 视图上下文唯一标识
  - 格式规范:
    - Staging区: `misc::staging`
    - 每日视图: `daily::YYYY-MM-DD` (如 `daily::2025-10-03`)
    - Area视图: `area::{uuid}` (如 `area::a1b2c3d4-...`)
    - Project视图: `project::{uuid}` (如 `project::a1b2c3d4-...`)

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `ViewPreferenceDto`

```json
{
  "context_key": "daily::2025-10-03",
  "sorted_task_ids": [
    "task-uuid-1",
    "task-uuid-2",
    "task-uuid-3"
  ],
  "updated_at": "2025-10-05T12:00:00Z"
}
```

**404 Not Found:**

```json
{
  "error_code": "NOT_FOUND",
  "message": "ViewPreference not found: daily::2025-10-03"
}
```

## 4. 验证规则 (Validation Rules)

- `context_key`:
    - **必须**存在于 URL 路径中。
    - 格式不做强制校验(由调用者保证格式正确性)。
    - 如果数据库中不存在对应记录,返回 `404 NOT_FOUND`。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  从 URL 路径参数中提取 `context_key`。
2.  调用数据访问层查询视图偏好记录(`database::find_by_context_key`)。
3.  如果记录不存在,返回 `404 NOT_FOUND` 错误。
4.  将数据库行(`ViewPreferenceRow`)转换为领域实体(`ViewPreference`):
    - 解析 JSON 字符串 `sorted_task_ids` 为字符串数组。
    - 解析 RFC 3339 字符串 `updated_at` 为 DateTime<Utc>。
5.  将领域实体转换为 DTO(`ViewPreferenceDto`)。
6.  返回 `200 OK` 和 DTO。

## 6. 边界情况 (Edge Cases)

- **context_key 不存在:** 返回 `404 NOT_FOUND` 错误。
- **sorted_task_ids 为空数组:** 正常返回,允许空数组(表示该视图暂无任务或所有任务已删除)。
- **sorted_task_ids 包含已删除的任务ID:** 不做验证,返回原始数据(前端负责过滤无效ID)。
- **context_key 格式错误:** 不做格式校验,直接查询(数据库查询不到则返回404)。
- **幂等性:** 多次查询相同 `context_key`,结果一致(无副作用)。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次查询 `view_preferences` 表。
- **无写操作:** 此端点为只读查询,不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时(如记录不存在),以 `WARN` 级别记录错误信息。

*(无其他已知副作用)*

---

## Views (视图查询)

### GET /api/views/all

<details>
<summary>源文件: <code>src\features\views\endpoints\get_all.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/views/all

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有任务的完整列表（包括已完成和未完成的任务），
> 以便我能全局查看和管理我的所有待办事项。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有未删除的任务（包括已完成和未完成），不限制排期状态。
为每个任务组装完整的 TaskCardDto（包含 schedules、time_blocks 和 area 信息），
并根据实际 schedules 情况动态设置 schedule_status。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- 无

**Query Parameters:**
- 无（当前版本不支持分页、过滤、排序参数）

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskCardDto[]`

```json
[
  {
    "id": "uuid",
    "title": "string",
    "glance_note": "string | null",
    "schedule_status": "staging" | "scheduled",
    "is_completed": boolean,
    "area": { "id": "uuid", "name": "string", "color": "#RRGGBB" } | null,
    "schedules": [...] | null,
    "due_date": { "date": "ISO8601", "type": "deadline" | "scheduled" } | null,
    "has_detail_note": boolean
  },
  ...
]
```

**注意：**
- 空列表返回 `[]`，而不是错误。
- 响应包含已完成（`is_completed: true`）和未完成（`is_completed: false`）的任务。

## 4. 验证规则 (Validation Rules)

- 无输入参数，无需验证。
- 查询条件：
  - `is_deleted = false`（排除已删除任务）
  - 无 `completed_at` 过滤（包含所有任务）

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `database::find_all_tasks` 查询数据库：
    - 查询 `tasks` 表，过滤 `is_deleted = false`
    - 按 `created_at DESC` 排序（最新创建的在前）
2.  遍历每个任务，调用 `assemble_task_card` 进行组装：
    - 调用 `TaskAssembler::task_to_card_basic` 创建基础 TaskCard
    - 调用 `TaskAssembler::assemble_schedules` 查询完整的 schedules（包含 time_blocks）
    - 根据 schedules 是否存在动态设置 `schedule_status`：
      - 如果 `schedules.is_some()` → `ScheduleStatus::Scheduled`
      - 否则 → `ScheduleStatus::Staging`
3.  对任务列表按 `id` 降序排序（保证稳定的显示顺序）。
4.  返回 `200 OK` 和任务列表（`Vec<TaskCardDto>`）。

## 6. 边界情况 (Edge Cases)

- **数据库中没有任务:** 返回空数组 `[]`（200 OK）。
- **所有任务都已删除:** 返回空数组 `[]`（200 OK）。
- **任务数量很大:** 当前无分页机制，可能返回大量数据（性能考虑，建议添加分页）。
- **已完成任务的 schedule_status:** 根据实际 schedules 情况设置（已完成不影响 schedule_status）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次，查询 `tasks` 表（过滤 `is_deleted = false`，按 `created_at DESC` 排序）。
    - **`SELECT`:** N次（N = 任务总数），每个任务查询完整的 schedules。
    - **`SELECT`:** 0-M次（M = schedules 总数），查询 `time_blocks` 表（每个 schedule 可能有时间块）。
    - **`SELECT`:** 0-N次，查询 `areas` 表（如果任务有 area_id）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*

**性能考虑：**
1. 当前实现会一次性返回所有任务，没有分页机制。
2. 如果任务数量超过数百个，建议添加分页参数（limit/offset 或 cursor-based）。
3. 考虑添加客户端缓存或 SSE 订阅机制，减少重复查询。

### GET /api/views/all-incomplete

<details>
<summary>源文件: <code>src\features\views\endpoints\get_all_incomplete.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/views/all-incomplete

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有未完成任务的列表（无论是否已排期），
> 以便我能专注于需要处理的待办事项，而不被已完成的任务干扰。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有未删除且未完成的任务（不限制排期状态）。
为每个任务组装完整的 TaskCardDto（包含 schedules、time_blocks 和 area 信息），
并根据实际 schedules 情况动态设置 schedule_status。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- 无

**Query Parameters:**
- 无（当前版本不支持分页、过滤、排序参数）

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskCardDto[]`

```json
[
  {
    "id": "uuid",
    "title": "string",
    "glance_note": "string | null",
    "schedule_status": "staging" | "scheduled",
    "is_completed": false,
    "area": { "id": "uuid", "name": "string", "color": "#RRGGBB" } | null,
    "schedules": [...] | null,
    "due_date": { "date": "ISO8601", "type": "deadline" | "scheduled" } | null,
    "has_detail_note": boolean
  },
  ...
]
```

**注意：**
- 空列表返回 `[]`，而不是错误。
- 响应中所有任务的 `is_completed` 均为 `false`。

## 4. 验证规则 (Validation Rules)

- 无输入参数，无需验证。
- 查询条件：
  - `is_deleted = false`（排除已删除任务）
  - `completed_at IS NULL`（排除已完成任务）

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `database::find_all_incomplete_tasks` 查询数据库：
    - 查询 `tasks` 表，过滤 `is_deleted = false` 和 `completed_at IS NULL`
    - 按 `created_at DESC` 排序（最新创建的在前）
2.  遍历每个任务，调用 `assemble_task_card` 进行组装：
    - 调用 `TaskAssembler::task_to_card_basic` 创建基础 TaskCard
    - 调用 `TaskAssembler::assemble_schedules` 查询完整的 schedules（包含 time_blocks）
    - 根据 schedules 是否存在动态设置 `schedule_status`：
      - 如果 `schedules.is_some()` → `ScheduleStatus::Scheduled`
      - 否则 → `ScheduleStatus::Staging`
3.  对任务列表按 `id` 降序排序（保证稳定的显示顺序）。
4.  返回 `200 OK` 和任务列表（`Vec<TaskCardDto>`）。

## 6. 边界情况 (Edge Cases)

- **数据库中没有未完成任务:** 返回空数组 `[]`（200 OK）。
- **所有任务都已完成或已删除:** 返回空数组 `[]`（200 OK）。
- **任务数量很大:** 当前无分页机制，可能返回大量数据（性能考虑，建议添加分页）。
- **未完成任务的 schedule_status:** 根据实际 schedules 情况动态设置（staging 或 scheduled）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次，查询 `tasks` 表（过滤 `is_deleted = false` 和 `completed_at IS NULL`，按 `created_at DESC` 排序）。
    - **`SELECT`:** N次（N = 未完成任务数量），每个任务查询完整的 schedules。
    - **`SELECT`:** 0-M次（M = schedules 总数），查询 `time_blocks` 表（每个 schedule 可能有时间块）。
    - **`SELECT`:** 0-N次，查询 `areas` 表（如果任务有 area_id）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*

**性能考虑：**
1. 当前实现会一次性返回所有未完成任务，没有分页机制。
2. 如果未完成任务数量超过数百个，建议添加分页参数（limit/offset 或 cursor-based）。
3. 考虑添加客户端缓存或 SSE 订阅机制，减少重复查询。

### GET /api/views/daily/:date

<details>
<summary>源文件: <code>src\features\views\endpoints\get_daily_tasks.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/views/daily/:date

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看指定日期的所有任务列表（包括已完成和未完成的任务），
> 以便我能了解某一天的工作安排和完成情况。

### 2.2. 核心业务逻辑 (Core Business Logic)

根据 URL 路径中的日期参数（YYYY-MM-DD 格式），查询该日期在 task_schedules 表中的所有任务。
为每个任务组装完整的 TaskCardDto（包含完整的 schedules、time_blocks 和 area 信息），
返回包含任务列表、日期和数量的响应结构。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- `date` (string, required): 日期字符串，格式为 YYYY-MM-DD（例如：2025-10-05）

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `GetDailyTasksResponse`

```json
{
  "tasks": [
    {
      "id": "uuid",
      "title": "string",
      "glance_note": "string | null",
      "schedule_status": "scheduled",
      "is_completed": boolean,
      "area": { "id": "uuid", "name": "string", "color": "#RRGGBB" } | null,
      "schedules": [
        {
          "id": "uuid",
          "scheduled_day": "YYYY-MM-DD",
          "time_blocks": [...]
        }
      ],
      "due_date": { "date": "ISO8601", "type": "deadline" | "scheduled" } | null,
      "has_detail_note": boolean
    },
    ...
  ],
  "date": "YYYY-MM-DD",
  "count": number
}
```

**400 Bad Request:**

```json
{
  "error_code": "VALIDATION_FAILED",
  "message": "输入验证失败",
  "details": [
    { "field": "date", "code": "INVALID_DATE_FORMAT", "message": "日期格式错误，请使用 YYYY-MM-DD 格式" }
  ]
}
```

**注意：** 如果该日期没有任务，返回 `{ "tasks": [], "date": "...", "count": 0 }`。

## 4. 验证规则 (Validation Rules)

- `date` 参数：
    - **必须**存在于 URL 路径中。
    - **必须**符合 YYYY-MM-DD 格式（例如：2025-10-05）。
    - **必须**能够成功解析为有效的日期（NaiveDate）。
    - 违反时返回 `400 VALIDATION_FAILED`，错误码 `INVALID_DATE_FORMAT`。

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  **验证层:** 调用 `validation::parse_date` 解析日期字符串：
    - 使用 `NaiveDate::parse_from_str` 解析 YYYY-MM-DD 格式
    - 转换为 UTC DateTime（时间设为 00:00:00）
    - 解析失败返回 `400 VALIDATION_FAILED` 错误
2.  **数据查询:** 调用 `database::find_tasks_for_date` 查询数据库：
    - 通过 `INNER JOIN task_schedules` 查询该日期的所有任务
    - 使用 `DATE(ts.scheduled_day) = DATE(?)` 进行日期匹配
    - 过滤 `is_deleted = false`（包含已完成和未完成任务）
    - 使用 `DISTINCT` 去重（一个任务在同一天可能有多个时间块）
    - 按 `created_at DESC` 排序（最新创建的在前）
3.  **任务组装:** 调用 `ViewTaskCardAssembler::assemble_batch` 批量组装：
    - 为每个任务查询完整的 schedules（包含 time_blocks）
    - 查询 area 信息（如果有）
    - 组装成 TaskCardDto
4.  **构建响应:** 返回 `GetDailyTasksResponse`：
    - `tasks`: 任务列表
    - `date`: 原始日期字符串
    - `count`: 任务数量
5.  返回 `200 OK` 和响应结构。

## 6. 边界情况 (Edge Cases)

- **日期格式错误:** 返回 `400 VALIDATION_FAILED`，错误码 `INVALID_DATE_FORMAT`。
- **无效日期（如 2025-02-30）:** 返回 `400 VALIDATION_FAILED`，错误码 `INVALID_DATE`。
- **该日期没有任务:** 返回 `{ "tasks": [], "date": "...", "count": 0 }`（200 OK）。
- **任务在该日期有多个时间块:** 任务只出现一次（通过 `DISTINCT` 去重），schedules 字段包含所有时间块。
- **包含已完成和未完成任务:** 不过滤 `completed_at`，两种任务都会返回。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次，通过 `INNER JOIN` 查询 `tasks` 和 `task_schedules` 表（带日期过滤）。
    - **`SELECT`:** N次（N = 该日期的任务数量），每个任务查询完整的 schedules。
    - **`SELECT`:** 0-M次（M = schedules 总数），查询 `time_blocks` 表（每个 schedule 可能有时间块）。
    - **`SELECT`:** 0-N次，查询 `areas` 表（如果任务有 area_id）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 成功时，以 `INFO` 级别记录 "Found N tasks for date YYYY-MM-DD"。
    - 失败时（日期格式错误或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*

### GET /api/views/planned

<details>
<summary>源文件: <code>src\features\views\endpoints\get_planned.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/views/planned

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有已排期（Planned）的未完成任务列表，
> 以便我能看到所有已被安排到具体日期的待办事项，了解未来的工作安排。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有符合"Planned"定义的任务：未删除、未完成、且至少存在一条 task_schedules 记录的任务。
为每个任务组装完整的 TaskCardDto（包含完整的 schedules、time_blocks 和 area 信息），并明确标记 schedule_status 为 Scheduled。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- 无

**Query Parameters:**
- 无（当前版本不支持分页、过滤、排序参数）

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskCardDto[]`

```json
[
  {
    "id": "uuid",
    "title": "string",
    "glance_note": "string | null",
    "schedule_status": "scheduled",
    "is_completed": false,
    "area": { "id": "uuid", "name": "string", "color": "#RRGGBB" } | null,
    "schedules": [
      {
        "id": "uuid",
        "scheduled_day": "YYYY-MM-DD",
        "time_blocks": [
          {
            "id": "uuid",
            "start_time": "HH:MM",
            "end_time": "HH:MM"
          }
        ]
      }
    ],
    "due_date": { "date": "ISO8601", "type": "deadline" | "scheduled" } | null,
    "has_detail_note": boolean
  },
  ...
]
```

**注意：** 空列表返回 `[]`，而不是错误。

## 4. 验证规则 (Validation Rules)

- 无输入参数，无需验证。
- "Planned" 的定义由后端逻辑保证：
  - `is_deleted = false`
  - `completed_at IS NULL`
  - 存在至少一条 `task_schedules` 记录（通过 INNER JOIN 保证）

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `database::find_planned_tasks` 查询数据库：
    - 通过 `INNER JOIN task_schedules` 查询 `tasks` 表
    - 过滤条件：`is_deleted = false` 和 `completed_at IS NULL`
    - 使用 `DISTINCT` 去重（一个任务可能有多个 schedules）
    - 按 `scheduled_day ASC, created_at DESC` 排序（最近的日期优先，同一天内最新创建的在前）
2.  遍历每个任务，调用 `assemble_task_card` 进行组装：
    - 调用 `TaskAssembler::task_to_card_basic` 创建基础 TaskCard
    - 调用 `TaskAssembler::assemble_schedules` 查询完整的 schedules（包含 time_blocks）
    - 明确设置 `schedule_status = Scheduled`
3.  返回 `200 OK` 和任务列表（`Vec<TaskCardDto>`）。

## 6. 边界情况 (Edge Cases)

- **数据库中没有已排期任务:** 返回空数组 `[]`（200 OK）。
- **所有任务都在 staging 或已完成:** 返回空数组 `[]`（200 OK）。
- **任务有多个 schedules:** 任务只出现一次（通过 `DISTINCT` 去重），schedules 字段包含所有日程。
- **任务数量很大:** 当前无分页机制，可能返回大量数据（性能考虑）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次，通过 `INNER JOIN` 查询 `tasks` 和 `task_schedules` 表。
    - **`SELECT`:** N次（N = planned 任务数量），每个任务查询完整的 schedules。
    - **`SELECT`:** 0-M次（M = schedules 总数），查询 `time_blocks` 表（每个 schedule 可能有时间块）。
    - **`SELECT`:** 0-N次，查询 `areas` 表（如果任务有 area_id）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*

### GET /api/views/staging

<details>
<summary>源文件: <code>src\features\views\endpoints\get_staging_view.rs</code></summary>
</details>

## 1. 端点签名 (Endpoint Signature)

GET /api/views/staging

## 2. 预期行为简介 (High-Level Behavior)

### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要查看所有未排期（Staging）的任务列表，
> 以便我能看到所有还没被安排到具体日期的待办事项，并进行后续的排期规划。

### 2.2. 核心业务逻辑 (Core Business Logic)

从数据库中查询所有符合"Staging"定义的任务：未删除、未完成、且不存在任何 task_schedules 记录的任务。
为每个任务组装完整的 TaskCardDto（包含 area 信息、schedules 等上下文），并明确标记 schedule_status 为 Staging。

## 3. 输入输出规范 (Request/Response Specification)

### 3.1. 请求 (Request)

**URL Parameters:**
- 无

**Query Parameters:**
- 无（当前版本不支持分页、过滤、排序参数）

### 3.2. 响应 (Responses)

**200 OK:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskCardDto[]`

```json
[
  {
    "id": "uuid",
    "title": "string",
    "glance_note": "string | null",
    "schedule_status": "staging",
    "is_completed": false,
    "area": { "id": "uuid", "name": "string", "color": "#RRGGBB" } | null,
    "schedules": null,
    "due_date": { "date": "ISO8601", "type": "deadline" | "scheduled" } | null,
    "has_detail_note": boolean
  },
  ...
]
```

**注意：** 空列表返回 `[]`，而不是错误。

## 4. 验证规则 (Validation Rules)

- 无输入参数，无需验证。
- "Staging" 的定义由后端逻辑保证：
  - `is_deleted = false`
  - `completed_at IS NULL`
  - 不存在于 `task_schedules` 表中

## 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `database::find_staging_tasks` 查询数据库：
    - 查询 `tasks` 表，过滤 `is_deleted = false` 和 `completed_at IS NULL`
    - 通过 `NOT EXISTS` 子查询排除所有在 `task_schedules` 中有记录的任务
    - 按 `created_at` 降序排列（最新的在前）
2.  遍历每个任务，调用 `assemble_task_card` 进行组装：
    - 调用 `TaskAssembler::task_to_card_basic` 创建基础 TaskCard
    - 调用 `TaskAssembler::assemble_schedules` 查询完整的 schedules（对于 staging 任务应该为 None）
    - 明确设置 `schedule_status = Staging`
3.  返回 `200 OK` 和任务列表（`Vec<TaskCardDto>`）。

## 6. 边界情况 (Edge Cases)

- **数据库中没有 staging 任务:** 返回空数组 `[]`（200 OK）。
- **所有任务都已排期或已完成:** 返回空数组 `[]`（200 OK）。
- **任务数量很大:** 当前无分页机制，可能返回大量数据（性能考虑）。
- **任务有过去的 schedule 但今天/未来无 schedule:** 该任务**不会**出现在 staging 视图（因为 SQL 查询使用 NOT EXISTS，任何 schedule 都会排除）。

## 7. 预期副作用 (Expected Side Effects)

- **数据库查询:**
    - **`SELECT`:** 1次，查询 `tasks` 表（带 `NOT EXISTS` 子查询过滤 `task_schedules`）。
    - **`SELECT`:** N次（N = staging 任务数量），每个任务查询 `task_schedules` 表（用于组装 schedules，预期为空）。
    - **`SELECT`:** 0-N次，查询 `areas` 表（如果任务有 area_id，由 `TaskAssembler` 内部查询）。
- **无写操作:** 此端点为只读查询，不修改任何数据。
- **无 SSE 事件:** 不发送任何事件。
- **日志记录:**
    - 失败时（数据库错误），以 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*

