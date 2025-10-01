# Cutie 数据结构及其耦合说明书

> 修改 Schema 或 DTO 时的完整检查清单

---

## ⚠️ 核心原则

**数据结构的修改会产生连锁反应！**

修改前必须理解：

1. 哪些文件定义了这个数据
2. 哪些代码依赖这个数据
3. 需要同步修改哪些地方

---

## 📊 数据结构层次

```
数据库 Schema (SQLite)
    ↓
后端实体 (Rust entities)
    ↓
后端 DTO (Rust response_dtos)
    ↓
前端 DTO (TypeScript dtos.ts)
    ↓
Pinia Store (TypeScript stores)
    ↓
Vue 组件 (TypeScript components)
```

---

## 🗄️ 修改数据库 Schema

### 修改位置

```
src-tauri/migrations/20241001000000_initial_schema.sql
```

### 影响范围

#### **1. 后端实体层**

**文件：** `src-tauri/src/entities/xxx/model.rs`

**需要修改：**

- Rust struct 定义
- `xxxRow` struct（数据库行映射）
- `TryFrom<xxxRow>` 实现

**示例：添加字段**

```rust
// model.rs
pub struct Task {
    // ... 现有字段
    pub new_field: Option<String>,  // ← 新增
}

// Row struct
pub struct TaskRow {
    // ... 现有字段
    pub new_field: Option<String>,  // ← 新增
}

// TryFrom
impl TryFrom<TaskRow> for Task {
    fn try_from(row: TaskRow) -> Result<Self, Self::Error> {
        Ok(Task {
            // ... 现有字段
            new_field: row.new_field,  // ← 新增
        })
    }
}
```

#### **2. 后端 DTO 层**

**文件：** `src-tauri/src/entities/xxx/response_dtos.rs`

**需要修改：**

- `xxxDto` struct 定义

**示例：**

```rust
pub struct TaskCardDto {
    // ... 现有字段
    pub new_field: Option<String>,  // ← 新增
}
```

#### **3. 装配器层**

**文件：** `src-tauri/src/features/xxx/shared/assembler.rs`

**需要修改：**

- 从实体转 DTO 的转换逻辑

**示例：**

```rust
pub fn task_to_card_basic(task: &Task) -> TaskCardDto {
    TaskCardDto {
        // ... 现有字段
        new_field: task.new_field.clone(),  // ← 新增
    }
}
```

#### **4. 数据访问层**

**文件：** 各个端点的 `mod database`

**需要修改：**

- SQL SELECT 语句（添加字段）
- SQL INSERT 语句（添加字段和绑定）
- SQL UPDATE 语句（如果可更新）

**示例：**

```rust
// SELECT
let query = r#"
    SELECT id, title, ..., new_field  -- ← 添加
    FROM tasks
"#;

// INSERT
let query = r#"
    INSERT INTO tasks (id, title, ..., new_field)  -- ← 添加
    VALUES (?, ?, ..., ?)  -- ← 添加占位符
"#;

sqlx::query(query)
    .bind(task.id)
    // ...
    .bind(&task.new_field)  // ← 添加绑定
```

#### **5. 前端 DTO 层**

**文件：** `src/types/dtos.ts`

**需要修改：**

- TypeScript interface 定义

**示例：**

```typescript
export interface TaskCard {
  // ... 现有字段
  new_field: string | null // ← 新增
}
```

#### **6. 前端 Store 层**

**文件：** `src/stores/xxx.ts`

**需要修改：**

- Payload 类型（如果字段可编辑）

**示例：**

```typescript
export interface UpdateTaskPayload {
  // ... 现有字段
  new_field?: string | null // ← 新增
}
```

#### **7. 前端组件层**

**文件：** 使用该数据的所有组件

**需要修改：**

- 显示新字段的 UI
- 编辑新字段的表单

**示例：**

```vue
<template>
  <div>{{ task.new_field }}</div>
  <!-- 显示 -->
</template>
```

---

## 🔗 常见修改场景

### 场景1：给 Task 添加新字段 `priority`

#### **步骤清单：**

- [ ] 1. 更新 `migrations/xxx.sql`：添加 `priority` 列
- [ ] 2. 更新 `entities/task/model.rs`：Task struct + TaskRow
- [ ] 3. 更新 `entities/task/response_dtos.rs`：TaskCardDto
- [ ] 4. 更新 `entities/task/request_dtos.rs`：CreateTaskRequest, UpdateTaskRequest
- [ ] 5. 更新 `features/tasks/shared/assembler.rs`：转换逻辑
- [ ] 6. 更新所有端点的 SQL：
  - `create_task.rs` - INSERT
  - `update_task.rs` - UPDATE（可选）
  - `get_task.rs` - SELECT
- [ ] 7. 更新 `src/types/dtos.ts`：TaskCard interface
- [ ] 8. 更新 `src/stores/task.ts`：Payload 类型
- [ ] 9. 更新 UI 组件：显示和编辑

#### **必须同步的文件：**

- 后端：8-10个文件
- 前端：3-5个文件

---

### 场景2：修改 DTO 结构（非 Schema）

**示例：** 把 `TaskCard.area` 从 `{ id, name, color }` 改为只保留 `color`

#### **影响范围：**

**后端：**

- [ ] `entities/task/response_dtos.rs` - 修改 AreaSummary
- [ ] `features/tasks/shared/assembler.rs` - 修改转换逻辑
- [ ] 所有组装 Area 的端点（get_task, get_staging_view 等）

**前端：**

- [ ] `src/types/dtos.ts` - 修改 interface
- [ ] 所有使用 `task.area.xxx` 的组件
  - `KanbanTaskCard.vue` - 显示标签
  - `AreaTestView.vue` - 按 area 筛选
  - 等等...

---

### 场景3：添加新实体（如 Project）

#### **完整步骤：**

**1. 数据库**

- [ ] `migrations/xxx.sql`：CREATE TABLE projects

**2. 后端实体**

- [ ] `entities/project/model.rs`
- [ ] `entities/project/request_dtos.rs`
- [ ] `entities/project/response_dtos.rs`
- [ ] `entities/project/mod.rs`
- [ ] `entities/mod.rs`：pub use project::\*

**3. 后端功能**

- [ ] `features/projects/mod.rs`
- [ ] `features/projects/endpoints/create_project.rs`
- [ ] `features/projects/endpoints/...`
- [ ] `features/projects/shared/assembler.rs`
- [ ] `features/projects/API_SPEC.md`
- [ ] `features/mod.rs`：注册路由

**4. 前端**

- [ ] `src/types/dtos.ts`：添加 Project interface
- [ ] `src/stores/project.ts`：创建新 store
- [ ] 相关 UI 组件

---

## 🔍 依赖关系图

### Task 数据结构的依赖

```
migrations/xxx.sql (tasks 表)
  ↓
entities/task/model.rs (Task, TaskRow)
  ↓
entities/task/response_dtos.rs (TaskCardDto, TaskDetailDto)
  ↓                                ↓
features/tasks/shared/assembler.rs  src/types/dtos.ts (TaskCard, TaskDetail)
  ↓                                  ↓
features/tasks/endpoints/*.rs        src/stores/task.ts
  ↓                                  ↓
API 响应                             src/components/**/*.vue
```

### 跨实体依赖

**TaskCard 包含 Area：**

```
TaskCardDto {
  area: Option<AreaSummary>  // ← 依赖 Area
}
```

**修改 Area 结构时：**

- [ ] `entities/area/*`
- [ ] `entities/task/response_dtos.rs`（AreaSummary）
- [ ] 所有组装 TaskCard 的地方

**TimeBlockView 包含 Task：**

```
TimeBlockViewDto {
  linked_tasks: Vec<LinkedTaskSummary>  // ← 依赖 Task
}
```

**修改 Task 基本字段时：**

- [ ] 检查 `LinkedTaskSummary` 是否需要更新
- [ ] 所有查询 linked_tasks 的端点

---

## 📝 修改检查清单模板

### 添加/修改字段检查清单

```markdown
## 修改：给 Task 添加 xxx 字段

### 数据库层

- [ ] migrations/xxx.sql - 添加列
- [ ] 删除旧数据库文件

### 后端实体层

- [ ] entities/task/model.rs - Task struct
- [ ] entities/task/model.rs - TaskRow struct
- [ ] entities/task/model.rs - TryFrom 实现

### 后端 DTO 层

- [ ] entities/task/response_dtos.rs - TaskCardDto
- [ ] entities/task/response_dtos.rs - TaskDetailDto（如果需要）
- [ ] entities/task/request_dtos.rs - CreateTaskRequest（如果可创建）
- [ ] entities/task/request_dtos.rs - UpdateTaskRequest（如果可编辑）

### 后端装配器层

- [ ] features/tasks/shared/assembler.rs - task_to_card_basic

### 后端端点层

- [ ] features/tasks/endpoints/create_task.rs - INSERT SQL
- [ ] features/tasks/endpoints/get_task.rs - SELECT SQL
- [ ] features/tasks/endpoints/update_task.rs - UPDATE SQL（如果可编辑）
- [ ] features/views/endpoints/get_staging_view.rs - SELECT SQL
- [ ] features/views/endpoints/get_all.rs - SELECT SQL
- [ ] ... 所有查询 Task 的端点

### 前端 DTO 层

- [ ] src/types/dtos.ts - TaskCard interface
- [ ] src/types/dtos.ts - TaskDetail interface（如果需要）

### 前端 Store 层

- [ ] src/stores/task.ts - Payload 类型

### 前端组件层

- [ ] 所有显示任务的组件
- [ ] 任务编辑器（如果可编辑）

### 测试

- [ ] 创建任务 - 新字段是否正确保存
- [ ] 查询任务 - 新字段是否正确返回
- [ ] 更新任务 - 新字段是否可编辑
- [ ] UI 显示 - 新字段是否正确展示
```

---

## 🚨 常见错误

### 错误1：只改了 Schema，忘记改实体

**现象：** 编译错误或运行时 SQL 错误

**原因：** Row struct 字段数不匹配

**解决：** 同步更新 `xxxRow` struct

### 错误2：只改了后端 DTO，忘记改前端

**现象：** 前端类型错误或 undefined

**原因：** 前后端 DTO 不一致

**解决：** 同步更新 `src/types/dtos.ts`

### 错误3：改了 DTO，忘记改装配器

**现象：** 编译错误

**原因：** Assembler 返回的 DTO 缺少新字段

**解决：** 更新 `assembler.rs` 的转换逻辑

### 错误4：改了结构，忘记改 SQL

**现象：** SQL 查询返回错误列数

**原因：** SELECT 字段列表过时

**解决：** 更新所有 SELECT 语句

---

## 💡 安全修改策略

### 策略1：向后兼容

**添加字段时：**

- 使用 `Option<T>`（nullable）
- 提供默认值
- 旧数据仍能正常工作

**删除字段时：**

- 先废弃（deprecated），后续版本再删除
- 或创建新 DTO 版本（v2）

### 策略2：全局搜索

**修改前：**

```bash
# 搜索所有使用该字段的地方
grep -r "field_name" src-tauri/src
grep -r "field_name" src
```

**确保：**

- 找到所有依赖
- 逐一更新

### 策略3：测试驱动

**修改后：**

1. 编译检查：`cargo check`
2. 类型检查：前端 linter
3. 运行测试：手动测试所有相关功能
4. 检查调试数据：任务编辑器底部

---

## 📋 快速参考：主要数据结构

### Task

**依赖链：**

```
Schema: tasks 表
  → entities/task/model.rs: Task
  → entities/task/response_dtos.rs: TaskCardDto, TaskDetailDto
  → features/tasks/shared/assembler.rs: TaskAssembler
  → src/types/dtos.ts: TaskCard, TaskDetail
  → src/stores/task.ts
  → 组件: KanbanTaskCard, KanbanTaskEditorModal, HomeView
```

**关键关联：**

- 包含 Area（AreaSummary）
- 包含 Subtasks
- 包含 ScheduleInfo

### TimeBlock

**依赖链：**

```
Schema: time_blocks 表
  → entities/time_block/model.rs: TimeBlock
  → entities/time_block/response_dtos.rs: TimeBlockViewDto
  → src/types/dtos.ts: TimeBlockView
  → src/stores/timeblock.ts
  → 组件: CuteCalendar
```

**关键关联：**

- 包含 Area（AreaSummary）
- 包含 LinkedTasks（任务摘要）

### Area

**依赖链：**

```
Schema: areas 表
  → entities/area/model.rs: Area
  → entities/area/response_dtos.rs: AreaDto
  → src/stores/area.ts
  → 组件: AreaManager, AreaSelector
```

**被依赖：**

- TaskCardDto.area
- TimeBlockViewDto.area

**修改 Area 影响：**

- Task 相关代码
- TimeBlock 相关代码

---

## 🛠️ 实用工具

### 依赖检查脚本

```bash
# 检查某个字段的所有引用
grep -rn "schedule_status" src-tauri/src
grep -rn "schedule_status" src

# 检查 DTO 定义
grep -rn "interface TaskCard" src
grep -rn "struct TaskCardDto" src-tauri/src
```

### 重新生成数据库

```bash
# 删除旧数据库
rm src-tauri/*.db*

# 重新运行应用，migrations 会自动执行
cargo tauri dev
```

---

## 📖 相关文档

- **ARCHITECTURE.md** - 系统整体架构
- **SFC_SPEC.md** - 后端开发规范（4.7 数据真实性原则）
- **PINIA_BEST_PRACTICES.md** - 前端状态管理
- **migrations/xxx.sql** - 数据库 Schema（真理来源）

---

**记住：数据结构是系统的骨架，修改需谨慎且全面！**
