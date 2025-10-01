# Cutie 系统架构说明书

> 为新维护者提供的完整架构指南

---

## 📋 目录

1. [系统概览](#系统概览)
2. [后端架构](#后端架构)
3. [前端架构](#前端架构)
4. [数据流](#数据流)
5. [关键设计决策](#关键设计决策)

---

## 系统概览

### 技术栈

**后端（Rust + Tauri）**

- 框架：Axum (HTTP)
- 数据库：SQLite + SQLx
- 架构：单文件组件（SFC）模式

**前端（Vue 3 + TypeScript）**

- 框架：Vue 3 Composition API
- 状态：Pinia
- 路由：Vue Router
- UI：自定义组件 + FullCalendar

### 核心特性

- ✅ 任务与时间块多对多架构
- ✅ Staging 区（替代 Backlog）
- ✅ 智能业务逻辑（孤儿清理、完成规则）
- ✅ 响应式数据流（零 workaround）
- ✅ 层级区域（Area）系统

---

## 后端架构

### 目录结构

```
src-tauri/src/
├── entities/           # 数据实体定义
│   ├── task/
│   │   ├── model.rs           # 数据库实体
│   │   ├── request_dtos.rs    # API 请求 DTOs
│   │   └── response_dtos.rs   # API 响应 DTOs
│   ├── time_block/
│   ├── area/
│   └── ...
│
├── features/          # 功能模块（SFC 模式）
│   ├── tasks/
│   │   ├── endpoints/        # 每个文件是完整的端点
│   │   │   ├── create_task.rs
│   │   │   ├── get_task.rs
│   │   │   └── ...
│   │   ├── shared/
│   │   │   └── assembler.rs  # 装配器（实体→DTO）
│   │   ├── mod.rs            # 路由注册
│   │   └── API_SPEC.md       # CABC 2.0 规范文档
│   ├── time_blocks/
│   ├── areas/
│   └── views/
│
├── shared/            # 共享基础设施
│   ├── core/
│   │   ├── error.rs          # 统一错误处理
│   │   └── utils/            # 工具函数（LexoRank 等）
│   ├── http/                 # HTTP 响应格式
│   └── ports/                # 依赖注入接口
│
└── startup/           # 应用启动
    ├── app_state.rs          # 全局状态
    └── database.rs           # 数据库初始化
```

### 单文件组件（SFC）模式

每个 API 端点是一个独立的 `.rs` 文件，包含：

```rust
// 文档层（CABC 注释）
/*
CABC for `endpoint_name`
- API端点
- 预期行为
- 输入输出规范
*/

// HTTP 处理器
pub async fn handle(...) -> Response {
    match logic::execute(...).await {
        Ok(data) => success_response(data),
        Err(err) => err.into_response(),
    }
}

// 验证层（可选）
mod validation { ... }

// 业务逻辑层
mod logic {
    pub async fn execute(...) -> AppResult<T> {
        // 业务逻辑
    }
}

// 数据访问层
mod database {
    pub async fn query_xxx(...) -> AppResult<T> {
        // SQL 查询
    }
}
```

**优势：**

- 高内聚：一个文件 = 一个功能
- 低耦合：端点之间独立
- 易维护：修改隔离，影响范围小

### 装配器模式

```
entities/           → 纯数据结构（无业务逻辑）
features/shared/    → 装配器（实体 → DTO 转换）
```

**规则：**

- ❌ 不要在 entities 中写转换逻辑
- ✅ 在 features/shared/assembler.rs 中实现

### 依赖注入

通过 `AppState` 注入所有依赖：

```rust
app_state.id_generator().new_uuid()  // ✅ 正确
app_state.clock().now_utc()          // ✅ 正确
app_state.db_pool()                  // ✅ 正确
```

**必须使用正确的 trait 方法名！**

### 数据库

**表名规范：**

- ✅ 全部使用复数：`tasks`, `areas`, `orderings`, `task_schedules`
- ⚠️ 编写前必须查看 `migrations/xxx.sql`

**Schema 位置：**

```
src-tauri/migrations/20241001000000_initial_schema.sql
```

### 关键工具

**LexoRank 排序：**

```rust
use crate::shared::core::utils::{
    generate_initial_sort_order,
    get_rank_after,
    get_mid_lexo_rank,
};
```

**错误处理：**

```rust
AppResult<T>  // 统一返回类型
AppError      // 统一错误类型
?             // 自动转换
```

---

## 前端架构

### 目录结构

```
src/
├── types/
│   └── dtos.ts              # 前端 DTO 定义
│
├── stores/                  # Pinia 状态管理
│   ├── task.ts              # 任务（单一数据源）
│   ├── timeblock.ts         # 时间块
│   ├── view.ts              # 视图索引
│   └── area.ts              # 区域
│
├── components/
│   ├── parts/               # 原子组件
│   │   ├── kanban/          # 看板相关
│   │   ├── AreaSelector.vue
│   │   └── AreaManager.vue
│   └── templates/           # 模板组件
│
├── views/                   # 页面组件
│   ├── HomeView.vue         # 主页（4列看板）
│   ├── AreaTestView.vue     # Area 测试页
│   └── MainLayout.vue       # 主布局
│
└── router/
    └── index.ts             # 路由配置
```

### Pinia Store 架构

**职责分离原则：**

```typescript
// State - 只存储原始数据
const tasks = ref(new Map<string, TaskCard | TaskDetail>())

// Getters - 只读取和计算
const stagingTasks = computed(() =>
  tasks.value.filter(t => t.schedule_status === 'staging')
)

// Actions - 负责 API 调用和修改 State
async function createTask(payload) {
  const response = await fetch(...)
  const newTask = response.data
  addOrUpdateTask(newTask)  // 更新 State
}
```

**关键规则：**

1. **单一数据源** - 每个实体只有一个 Map
2. **组件只读** - 通过 computed 读取，不缓存
3. **操作通过 Action** - 所有修改必须通过 store
4. **创建新对象** - 触发响应式更新

### 数据模型（DTOs）

**三种 DTO：**

```typescript
// 1. TaskCard - 用于列表/看板
interface TaskCard {
  id, title, schedule_status, area, ...
}

// 2. TaskDetail - 用于详情/编辑（继承 TaskCard）
interface TaskDetail extends TaskCard {
  detail_note, schedules, created_at, updated_at
}

// 3. TimeBlockView - 用于日历
interface TimeBlockView {
  id, start_time, end_time, area, linked_tasks
}
```

### 响应式更新链路

```
用户操作
  ↓
Store Action (调用 API)
  ↓
API 返回数据
  ↓
Store 更新（创建新 Map）
  ↓
Getter 重新计算
  ↓
Component Computed 触发
  ↓
Vue 重新渲染
  ↓
UI 更新 ✅
```

**任何一步断裂都会导致 UI 不更新！**

---

## 数据流

### 创建任务流程

```
1. 用户输入 → HomeView.handleAddTask()
   ↓
2. taskStore.createTask({ title })
   ↓
3. POST /api/tasks
   ↓
4. 后端：
   - 创建 tasks 记录
   - 创建 orderings 记录（staging）
   - 返回 TaskCardDto（schedule_status = 'staging'）
   ↓
5. 前端：addOrUpdateTask(newTask)
   ↓
6. stagingTasks getter 重新计算
   ↓
7. Staging 列显示新任务 ✅
```

### 拖拽到日历流程

```
1. 用户拖动任务到日历
   ↓
2. timeBlockStore.createTimeBlockFromTask()
   ↓
3. POST /api/time-blocks/from-task
   ↓
4. 后端（原子操作）：
   - 创建 time_blocks 记录
   - 创建 task_time_block_links 记录
   - 创建 task_schedules 记录
   - 返回 { time_block, updated_task }
   ↓
5. 前端：
   - timeBlockStore.addOrUpdateTimeBlock(time_block)
   - taskStore.addOrUpdateTask(updated_task)
   ↓
6. 两个 store 同时更新 → 两个 getter 重新计算
   ↓
7. UI 同步：
   - 任务从 Staging 消失
   - 任务出现在 Planned
   - 时间块出现在日历
   ✅ 全部即时响应式更新
```

### 完成任务流程

```
1. 用户勾选复选框
   ↓
2. taskStore.completeTask(id)
   ↓
3. POST /api/tasks/:id/completion
   ↓
4. 后端（Cutie 业务逻辑）：
   - 设置 completed_at
   - 当天日程 → outcome = 'COMPLETED_ON_DAY'
   - 未来日程 → 删除
   - 时间块处理：
     * 仅链接此任务 + 自动创建 + 正在发生 → 截断
     * 仅链接此任务 + 自动创建 + 在未来 → 删除
   - 返回 TaskCardDto（is_completed = true）
   ↓
5. 前端：addOrUpdateTask(completedTask)
   ↓
6. completedTasks getter 重新计算
   ↓
7. 任务从所有列表消失 ✅
```

---

## 关键设计决策

### 1. 单一职责端点

**问题：** 一个端点处理多种场景 → 混乱

**解决：** 拆分专用端点

```
POST /time-blocks           → 创建空时间块
POST /time-blocks/from-task → 拖动任务专用
```

**好处：**

- 语义清晰
- 响应针对性强
- 易于维护

### 2. 数据真实性原则

**问题：** Assembler 返回默认值 → 前端接收错误数据

**解决：** 后端必须查询实际状态

```rust
// ❌ 错误
let task_card = TaskAssembler::task_to_card_basic(&task);
return task_card;  // schedule_status = 'staging' (默认)

// ✅ 正确
let mut task_card = TaskAssembler::task_to_card_basic(&task);
let schedules = query_task_schedules(task_id).await?;
task_card.schedule_status = if !schedules.is_empty() {
    Scheduled
} else {
    Staging
};
return task_card;  // 反映真实状态
```

### 3. 智能孤儿清理

**删除任务时：**

- 检查链接的时间块是否成为孤儿
- 检查是否自动创建（title 匹配）
- 如果是 → 同时删除时间块
- 返回被删除的时间块 ID 列表
- 前端同步删除

**好处：**

- 自动清理无意义的空时间块
- 保护用户手动创建的时间块
- 前后端数据完全同步

### 4. Cutie 完成任务逻辑

**精确的业务规则：**

1. 当天日程 → 标记完成
2. 未来日程 → 删除
3. 时间块（过去） → 保留
4. 时间块（正在发生 + 自动创建） → 截断
5. 时间块（未来 + 自动创建） → 删除

**哲学：**

- 尊重历史（过去不动）
- 清理未来（已无意义）
- 保护手动（用户意图）

---

## API 响应格式

### 统一包装

```json
{
  "data": { ... },        // 实际数据
  "timestamp": "...",     // 响应时间
  "request_id": "..."     // 可选
}
```

**前端提取：**

```typescript
const result = await response.json()
const data = result.data // ✅ 必须提取
```

### 修改操作必须返回更新数据

```
创建 → 返回完整的创建对象
更新 → 返回完整的更新对象
删除 → 返回受影响的资源 ID
```

**为什么？** 让前端知道确切的状态变化，触发正确的响应式更新。

---

## 前后端协作

### DTO 一致性

**后端：** `src-tauri/src/entities/task/response_dtos.rs`

```rust
pub struct TaskCardDto { ... }
```

**前端：** `src/types/dtos.ts`

```typescript
export interface TaskCard { ... }
```

**必须完全一致！** 字段名、类型、嵌套结构。

### API 契约

**查看端点规范：**

- `src-tauri/src/features/*/API_SPEC.md`
- 完整的 CABC 2.0 文档
- 请求/响应示例
- 业务逻辑详解

---

## 数据库 Schema

**位置：**

```
src-tauri/migrations/20241001000000_initial_schema.sql
```

**关键表：**

| 表名                  | 用途                   | 关键约束               |
| --------------------- | ---------------------- | ---------------------- |
| tasks                 | 任务实体               | -                      |
| time_blocks           | 时间块实体             | start_time < end_time  |
| task_time_block_links | 任务↔时间块（多对多） | 中间表                 |
| task_schedules        | 任务↔日期             | 判断 staging/scheduled |
| orderings             | 排序                   | LexoRank               |
| areas                 | 区域                   | 层级结构               |

**多对多架构：**

```
Task ←→ task_time_block_links ←→ TimeBlock
  ✅ 一个时间块包含多个任务
  ✅ 一个任务分散在多个时间块
```

---

## 开发工作流

### 添加新端点

1. **查看 Schema**（强制！）
2. **创建 SFC 文件**：`features/xxx/endpoints/new_endpoint.rs`
3. **遵循模板**：文档 + Handler + Logic + Database
4. **注册路由**：`features/xxx/mod.rs`
5. **编写文档**：`features/xxx/API_SPEC.md`

### 修改数据模型

1. **更新 Schema**：`migrations/xxx.sql`
2. **更新实体**：`entities/xxx/model.rs`
3. **更新 DTOs**：`entities/xxx/response_dtos.rs`
4. **更新装配器**：`features/xxx/shared/assembler.rs`
5. **更新前端 DTOs**：`src/types/dtos.ts`
6. **更新 Store**：`src/stores/xxx.ts`

### 调试技巧

**后端日志：**

```rust
tracing::info!("Debug message: {:?}", data);
```

**前端调试：**

- 任务编辑器弹窗底部的调试数据
- 浏览器控制台：`$pinia.state.value.task.tasks`

---

## 最佳实践

### 后端

1. ✅ 查看 Schema 后再写代码
2. ✅ 使用 LexoRank 工具，不要自己实现
3. ✅ 返回真实状态，不要返回默认值
4. ✅ 避免冗余查询
5. ✅ 事务包裹所有写操作

### 前端

1. ✅ 组件直接使用 store.getters
2. ✅ 用 computed 包装读取
3. ✅ 所有操作通过 store.actions
4. ✅ Store 创建新对象触发更新
5. ✅ 不要在组件中缓存数据

---

## 参考文档

- **SFC_SPEC.md** - 单文件组件开发规范
- **PINIA_BEST_PRACTICES.md** - 前端状态管理
- **CUTIE_CONCEPTS.md** - 核心概念速查
- **features/\*/API_SPEC.md** - 每个功能的详细规范

---

**维护者必读：** 遵循这些架构原则，可以确保代码质量和系统稳定性！
