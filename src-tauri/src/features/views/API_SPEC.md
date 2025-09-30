# Views API 规范文档

本文档按照 CABC 规范 2.0 定义 Views 功能模块的所有 API 端点。

Views 模块提供聚合视图，用于不同的任务列表展示。

---

## 端点清单

1. [GET /api/views/all](#get-apiviewsall) - 所有任务
2. [GET /api/views/all-incomplete](#get-apiviewsall-incomplete) - 所有未完成任务
3. [GET /api/views/planned](#get-apiviewsplanned) - 已排期任务
4. [GET /api/views/staging](#get-apiviewsstaging) - 未排期任务（Staging 区）

---

## GET /api/views/all

### 1. 端点签名

`GET /api/views/all`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要看到所有任务（包括已完成的），以便回顾我的工作历史。

#### 2.2. 核心业务逻辑

查询所有未删除的任务，无论完成状态，自动计算 schedule_status。

#### 2.3. 预期 UI 响应

"All" 列显示所有任务，包括已完成任务（可能显示为划线）。

### 3. 输入输出规范

#### 3.1. 请求

无参数

#### 3.2. 响应

**200 OK:** `Vec<TaskCardDto>`

### 4. 验证规则

无

### 5. 业务逻辑详解

1. 查询所有 is_deleted = false 的任务
2. 为每个任务：
   - 组装基础 TaskCard
   - 查询 task_schedules 判断 schedule_status
   - 查询 orderings 获取 sort_order
   - 查询 areas 获取区域信息
3. 按 created_at DESC 排序
4. 返回数组

### 6. 边界情况

- 无任务 → 返回空数组

### 7. 数据访问详情

- SELECT: 1次（所有任务）
- SELECT: N次（每个任务的 schedule、ordering、area）
- 只读操作

### 8. 预期副作用

- 无

---

## GET /api/views/all-incomplete

### 1. 端点签名

`GET /api/views/all-incomplete`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要看到所有未完成的任务，以便专注于当前需要处理的工作。

#### 2.2. 核心业务逻辑

查询所有未删除且未完成的任务，自动计算 schedule_status。

#### 2.3. 预期 UI 响应

"Incomplete" 列显示所有未完成任务。

### 3. 输入输出规范

#### 3.1. 请求

无参数

#### 3.2. 响应

**200 OK:** `Vec<TaskCardDto>`

### 4. 验证规则

无

### 5. 业务逻辑详解

1. 查询 WHERE is_deleted = false AND completed_at IS NULL
2. 为每个任务组装 TaskCard（同 /all）
3. 按 created_at DESC 排序
4. 返回数组

### 6. 边界情况

- 无未完成任务 → 返回空数组

### 7. 数据访问详情

- SELECT: 1次（未完成任务）
- SELECT: N次（关联信息）
- 只读操作

### 8. 预期副作用

- 无

---

## GET /api/views/planned

### 1. 端点签名

`GET /api/views/planned`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要看到所有已排期的任务，以便了解哪些任务已经被安排了时间。

#### 2.2. 核心业务逻辑

查询所有有 task_schedules 记录的未完成任务。

#### 2.3. 预期 UI 响应

"Planned" 列显示已排期任务，按最近的 scheduled_day 排序。

### 3. 输入输出规范

#### 3.1. 请求

无参数

#### 3.2. 响应

**200 OK:** `Vec<TaskCardDto>`，所有任务的 schedule_status = 'scheduled'

### 4. 验证规则

无

### 5. 业务逻辑详解

1. INNER JOIN tasks 和 task_schedules
2. WHERE is_deleted = false AND completed_at IS NULL
3. 为每个任务组装 TaskCard（schedule_status 明确设为 'scheduled'）
4. 按 scheduled_day ASC 排序
5. 返回数组

### 6. 边界情况

- 无已排期任务 → 返回空数组

### 7. 数据访问详情

- SELECT: 1次（JOIN 查询）
- SELECT: N次（关联信息）
- 只读操作

### 8. 预期副作用

- 无

---

## GET /api/views/staging

### 1. 端点签名

`GET /api/views/staging`

### 2. 预期行为简介

#### 2.1. 用户故事

> 作为用户，我想要看到所有未排期的任务（Staging 区），以便决定接下来要安排哪些任务。

#### 2.2. 核心业务逻辑

**Cutie 核心概念：Staging 替代传统的 Backlog**

- 查询没有 task_schedules 记录的未完成任务
- 逾期任务也会回流到这里（无红色标签）

#### 2.3. 预期 UI 响应

"Staging" 列显示未排期任务，支持添加新任务。

### 3. 输入输出规范

#### 3.1. 请求

无参数

#### 3.2. 响应

**200 OK:** `Vec<TaskCardDto>`，所有任务的 schedule_status = 'staging'

### 4. 验证规则

无

### 5. 业务逻辑详解

1. 查询 WHERE NOT EXISTS (task_schedules)
2. AND is_deleted = false AND completed_at IS NULL
3. 为每个任务组装 TaskCard（schedule_status = 'staging'）
4. 获取 sort_order、area
5. 按 sort_order 排序
6. 返回数组

### 6. 边界情况

- 无未排期任务 → 返回空数组

### 7. 数据访问详情

- SELECT: 1次（NOT EXISTS 子查询）
- SELECT: N次（orderings, areas）
- 只读操作

### 8. 预期副作用

- 无

---

**文件路径：** `src-tauri/src/features/views/`
**端点实现：** `endpoints/` 目录下的对应文件
**依赖：** 使用 tasks 模块的 TaskAssembler
