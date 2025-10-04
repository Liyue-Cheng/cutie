
# CABC V2.1 规范手册

## 第一部分：CABC (Cutie API Behavior Contract) 规范手册

### 第一条：适用范围
本规范适用于Cutie的所有API端点和公共抽象方功能、每一个公共方法都必须在代码的文档注释中，提供一份完整的CABC文档。

### 第二条：文档结构
每一份CABC文档必须严格包含以下六个部分：

1.  **函数/端点签名 (Function Signature):**
    *   **内容:** 方法在代码中的完整、精确的函数签名。
    *   **目的:** 提供机器可验证的、最基础的类型契约。

2.  **预期行为简介 (High-Level Behavior):**
    *   **内容:** 一到两句、使用**业务语言**而非技术语言的概括性描述。
    *   **目的:** 让任何阅读者能立刻理解该方法的核心业务意图。

3.  **输入输出规范 (Input/Output Specification - The Contract):**
    *   **内容:** 包含“前置条件”、“后置条件”和“不变量”三个子部分。
    *   **目的:** 采用“契约式设计”思想，精确定义方法的责任边界。
    *   **3.1. 前置条件 (Pre-conditions):**
        *   **定义:** 调用此方法前，输入参数和相关系统状态**必须**满足的条件。
        *   **要求:** 必须穷举所有输入参数的有效域（类型、格式、范围、非空约束等）和依赖的系统状态（如“关联的任务必须存在”）。不满足前置条件是调用者的错误。
    *   **3.2. 后置条件 (Post-conditions):**
        *   **定义:** 如果方法成功执行（不返回错误），系统状态**必须**达到的新状态，以及返回值的确切结构和含义。
        *   **要求:** 必须清晰、可验证。它们将直接转化为测试用例的断言（assertions）。
    *   **3.3. 不变量 (Invariants):**
        *   **定义:** 在方法执行期间及执行完毕后，**永远不能被破坏**的系统级或实体级的规则。
        *   **要求:** 用于声明那些比单个方法更宏观的、必须始终保持一致的系统属性（如“任务的ID和创建时间永远不变”）。

4.  **边界情况 (Edge Cases):**
    *   **内容:** 对所有可预见的“非阳光路径”或特殊输入组合的行为进行明确定义。
    *   **目的:** 消除模糊地带，确保软件在异常情况下的行为是确定和可预测的。
    *   **要求:** **必须**包含对幂等性（重复调用）、无效输入、状态冲突等场景的详细描述。例如：“若任务已完成，再次调用此方法，则必须直接返回成功，不产生任何副作用。”

5.  **预期副作用 (Expected Side Effects):**
    *   **内容:** 明确列出该方法除了返回值之外，对系统产生的**所有**可观测到的影响。
    *   **目的:** 让调用者对一个方法调用的“全部成本”有清晰的认知。
    *   **要求:** 必须包含但不限于：数据库的写操作（增、删、改）、事务的边界、对外部系统（如AI服务）的调用、缓存的失效、消息的推送、日志的记录等。文档末尾隐含一句“无其他已知副作用”。

### 第三条：完备性要求
一份CABC文档被认为是“不完备的”，如果它未能对一种可预见的输入或状态组合的行为做出明确定义。发现“不完备”或“不确定行为”将被视为一个需要进行版本变更的“文档缺陷”。

---

## 第二部分：端点 CABC 示例

### 1. 端点签名 (Endpoint Signature)

`POST /api/tasks`

### 2. 预期行为简介 (High-Level Behavior)

#### 2.1. 用户故事 / 场景 (User Story / Scenario)

> 作为一个用户，我想要在任何看板（如Staging区或今日看板）上快速创建一个新任务，以便我能立即捕捉我的想法，而不需要复杂的步骤。

#### 2.2. 核心业务逻辑 (Core Business Logic)

为了实现快速创建，后端将在数据库中创建一个新的`Task`实体。为了让它能被立即看到和排序，系统会同时为这个新任务在它所属的上下文（由请求中的`context`字段决定，若无则默认为Staging区）中，创建一个`Ordering`记录。

### 3. 输入输出规范 (Request/Response Specification)

#### 3.1. 请求 (Request)

**请求体 (Request Body):** `application/json`

```json
{
  "title": "string (required, 1-255 chars)",
    ...
    ...
    "id": "string"
  }
}
```

#### 3.2. 响应 (Responses)

**201 Created:**

*   **Content-Type:** `application/json`
*   **Schema:** `TaskCardDto`

```json
{
  "id": "a1b2c3d4-...",
    ...
....
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

### 4. 验证规则 (Validation Rules)

- `title`:
    - **必须**存在。
    - **必须**为非空字符串 (trim后)。
    - 长度**必须**小于等于 255 个字符。
- `estimated_duration` (如果未来添加):
    - 如果提供，**必须**是大于等于 0 的整数。
    - 如果提供，**必须**小于等于 10080 (7天)。
- `subtasks` (如果未来添加):
    - 如果提供，其数组长度**必须**小于等于 50。

### 5. 业务逻辑详解 (Business Logic Walkthrough)

1.  调用 `validation` 模块对请求体进行验证。
2.  启动数据库事务。
3.  通过 `Clock` 服务获取当前时间 `now`。
4.  通过 `IdGenerator` 服务生成新的 `task_id`。
5.  根据请求数据，构造一个 `Task` 领域实体对象。
6.  调用 `database` 模块的 `insert_task` 函数持久化 `Task`。
7.  根据请求中的 `context`，确定新任务的排序上下文（默认为 `MISC::staging_all`）。
8.  调用 `database` 模块的 `calculate_new_sort_order` 函数获取该上下文中的新排序值。
9.  调用 `database` 模块的 `insert_ordering` 函数持久化排序信息。
10. 提交数据库事务。
11. 从数据库中重新或附加查询 `Area` 等关联信息。
12. 调用 `Assembler` 将 `Task` 实体和所有上下文信息组装成 `TaskCardDto`。
13. 返回 `201 Created` 和组装好的 `TaskCardDto`。

### 6. 边界情况 (Edge Cases)

- **`area_id` 不存在:** 如果请求中提供了 `area_id`，但在 `areas` 表中找不到对应的记录，**必须**返回 `404 Not Found` 错误，并回滚事务。
- **并发创建:** （暂不考虑V1.0）在高并发下，`calculate_new_sort_order` 的逻辑需要能处理可能的竞态条件，以保证 `sort_order` 的唯一性和正确性。

### 7. 预期副作用 (Expected Side Effects)

- **数据库写入:**
    - **`SELECT`:** 1次，查询 `areas` 表以验证 `area_id` 是否存在（如果提供）。
    - **`SELECT`:** 1次，查询 `orderings` 表以获取当前上下文的最大 `sort_order`。
    - **`INSERT`:** 1条记录到 `tasks` 表。
    - **`INSERT`:** 1条记录到 `orderings` 表。
    - **(事务):** 以上所有数据库写操作**必须**包含在一个数据库事务内。
- **日志记录:**
    - 成功时，以 `INFO` 级别记录 "Task created successfully" 及任务ID。
    - 失败时（如验证失败或数据库错误），以 `WARN` 或 `ERROR` 级别记录详细错误信息。

*（无其他已知副作用）*