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

**文件：** `src-tauri/src/features/xxx/shared/assembler.rs` 或 `assemblers/*.rs`

**需要修改：**

- 从实体转 DTO 的转换逻辑
- **⚠️ 跨功能装配器**：检查是否有其他功能模块也在组装该 DTO

**示例：**

```rust
pub fn task_to_card_basic(task: &Task) -> TaskCardDto {
    TaskCardDto {
        // ... 现有字段
        new_field: task.new_field.clone(),  // ← 新增
    }
}
```

**⚠️ 特殊情况：跨功能依赖**

某些实体/DTO 可能被多个功能模块使用，例如：

**TimeBlock 实体的跨功能依赖：**

- **装配器**：`features/tasks/shared/assemblers/time_block_assembler.rs` 组装 `TimeBlockViewDto`
- **Repository**：`features/tasks/shared/repositories/task_time_block_link_repository.rs` 查询 `TimeBlock` 实体

修改 `TimeBlock` 实体或 `TimeBlockViewDto` 时，必须同时更新：

1. `features/time_blocks/` 下的所有代码
2. `features/tasks/shared/assemblers/time_block_assembler.rs` 的装配逻辑
3. `features/tasks/shared/repositories/task_time_block_link_repository.rs` 的 SQL 查询

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

### 场景3：给 TimeBlock 添加新字段 `is_all_day`

#### **步骤清单：**

- [ ] 1. 更新 `migrations/20241001000000_initial_schema.sql`：添加 `is_all_day BOOLEAN NOT NULL DEFAULT FALSE`
- [ ] 2. 更新 `entities/time_block/model.rs`：
  - TimeBlock struct 添加 `pub is_all_day: bool`
  - TimeBlockRow struct 添加 `pub is_all_day: bool`
  - TryFrom 实现添加字段映射
- [ ] 3. 更新 `entities/time_block/response_dtos.rs`：TimeBlockViewDto 添加字段
- [ ] 4. 更新 `entities/time_block/request_dtos.rs`：
  - CreateTimeBlockRequest 添加 `pub is_all_day: Option<bool>`
  - UpdateTimeBlockRequest 添加 `pub is_all_day: Option<bool>`
- [ ] 5. 更新 `features/time_blocks/shared/repositories/time_block_repository.rs`：
  - 所有 SELECT 语句添加 `is_all_day`
  - INSERT 语句添加字段和绑定
  - UPDATE 语句添加字段更新逻辑
- [ ] 6. 更新 `features/time_blocks/shared/conflict_checker.rs`：添加业务逻辑（如全天事件不冲突）
- [ ] 7. 更新所有 time_blocks 端点：
  - `create_time_block.rs` - 处理新字段
  - `update_time_block.rs` - 处理新字段
  - `create_from_task.rs` - 设置默认值
  - `list_time_blocks.rs` - 返回新字段
- [ ] 8. **⚠️ 跨功能装配器**：更新 `features/tasks/shared/assemblers/time_block_assembler.rs`：
  - `assemble_for_event_in_tx` - SQL 查询添加字段
  - `assemble_for_event_in_tx` - DTO 初始化添加字段
  - `assemble_view` - DTO 初始化添加字段
- [ ] 8.1. **⚠️ 跨功能 Repository**：更新 `features/tasks/shared/repositories/task_time_block_link_repository.rs`：
  - `find_linked_time_blocks_in_tx` - SQL 查询添加字段（查询 TimeBlock 实体）
- [ ] 9. 更新 `src/types/dtos.ts`：TimeBlockView 添加 `is_all_day: boolean`
- [ ] 10. 更新 `src/stores/timeblock.ts`：
  - CreateTimeBlockPayload 添加 `is_all_day?: boolean`
  - UpdateTimeBlockPayload 添加 `is_all_day?: boolean`
- [ ] 11. 更新 `src/components/parts/CuteCalendar.vue`：
  - 渲染时使用 `is_all_day`
  - 创建/更新时传递 `is_all_day`
  - 处理全天/分时转换逻辑

#### **必须同步的文件：**

- 后端：13-16个文件（包括跨功能装配器和跨功能 Repository）
- 前端：3个文件

#### **关键注意事项：**

- ⚠️ **跨功能依赖**：TimeBlock 被 Task 功能模块依赖，必须同步更新：
  - `features/tasks/shared/assemblers/time_block_assembler.rs` - 组装 DTO
  - `features/tasks/shared/repositories/task_time_block_link_repository.rs` - 查询实体
- 使用以下命令查找所有依赖点：
  ```bash
  grep -rn "TimeBlockViewDto {" src-tauri/src/features
  grep -rn "SELECT.*FROM time_blocks" src-tauri/src/features/tasks
  ```

---

### 场景4：添加新实体（如 Project）

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

**现象：** 编译错误 `missing field 'xxx' in initializer of 'XxxDto'`

**原因：** Assembler 返回的 DTO 缺少新字段

**解决：**

1. 更新主装配器：`features/xxx/shared/assembler.rs`
2. **⚠️ 检查跨功能装配器**：使用 `grep -rn "XxxDto {" src-tauri/src/features` 查找所有组装该 DTO 的位置
3. **⚠️ 检查跨功能 Repository**：使用 `grep -rn "SELECT.*FROM xxx_table" src-tauri/src/features` 查找所有查询该实体的位置
4. 逐一更新所有装配器和 Repository 的 SQL 查询

**真实案例：**

- 修改 `TimeBlock` 实体添加 `is_all_day` 字段时
- 除了 `features/time_blocks/` 下的代码
- 还需要修改：
  - `features/tasks/shared/assemblers/time_block_assembler.rs` - 装配器的 SQL 和 DTO 初始化
  - `features/tasks/shared/repositories/task_time_block_link_repository.rs` - Repository 的 SQL 查询

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

# ⚠️ 关键：搜索所有组装该 DTO 的位置
grep -rn "TimeBlockViewDto {" src-tauri/src/features
grep -rn "TaskCardDto {" src-tauri/src/features
```

**确保：**

- 找到所有依赖
- **特别注意跨功能模块的装配器**
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
  → entities/time_block/model.rs: TimeBlock, TimeBlockRow
  → entities/time_block/response_dtos.rs: TimeBlockViewDto
  → entities/time_block/request_dtos.rs: CreateTimeBlockRequest, UpdateTimeBlockRequest
  → features/time_blocks/shared/repositories/time_block_repository.rs: CRUD SQL
  → features/time_blocks/shared/conflict_checker.rs: 冲突检查逻辑
  → features/time_blocks/endpoints/*.rs: 所有时间块端点
  → features/tasks/shared/assemblers/time_block_assembler.rs: ⚠️ 跨功能装配器
  → features/tasks/shared/repositories/task_time_block_link_repository.rs: ⚠️ 跨功能查询
  → src/types/dtos.ts: TimeBlockView
  → src/stores/timeblock.ts: CreateTimeBlockPayload, UpdateTimeBlockPayload
  → 组件: CuteCalendar
```

**关键关联：**

- 包含 Area（AreaSummary）
- 包含 LinkedTasks（任务摘要）
- **被 Task 功能依赖**：
  - `features/tasks/shared/assemblers/time_block_assembler.rs` 会组装 TimeBlockViewDto
  - `features/tasks/shared/repositories/task_time_block_link_repository.rs` 会查询 TimeBlock 实体

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

# ⚠️ 修改 DTO 后必须执行：查找所有组装该 DTO 的位置
grep -rn "TimeBlockViewDto {" src-tauri/src/features
grep -rn "TaskCardDto {" src-tauri/src/features

# 查找特定实体的所有 SQL 查询
grep -rn "SELECT.*FROM time_blocks" src-tauri/src
grep -rn "INSERT INTO time_blocks" src-tauri/src
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

## 💡 经验教训

### 教训1：跨功能装配器容易被遗漏

**案例：** 2025-10-05 修改 `TimeBlock` 实体添加 `is_all_day` 字段

**问题：**

- 更新了 `features/time_blocks/` 下的所有代码
- 编译通过，以为完成了
- 运行时发现 `features/tasks/shared/assemblers/time_block_assembler.rs` 报错：`missing field 'is_all_day'`

**原因：**

- TimeBlock 被 Task 功能模块依赖
- Task 模块有自己的装配器来组装 `TimeBlockViewDto`
- 这种跨功能依赖不在常规的依赖链中

**解决方案：**

1. 修改任何 DTO 后，必须执行：
   ```bash
   # 查找所有组装该 DTO 的位置
   grep -rn "XxxDto {" src-tauri/src/features
   ```
2. 检查所有组装该 DTO 的位置，不仅限于该实体的功能模块
3. 更新文档，明确标注跨功能依赖

**预防措施：**

- 在依赖链图中明确标注跨功能装配器
- 修改检查清单中增加"跨功能装配器检查"步骤
- 使用全局搜索确认所有组装点

### 教训2：跨功能 Repository 的 SQL 查询容易遗漏

**案例：** 2025-10-05 修改 `TimeBlock` 实体添加 `is_all_day` 字段

**问题：**

- 更新了 `features/time_blocks/` 下的所有 SQL 查询
- 更新了装配器 `time_block_assembler.rs`
- 编译通过，以为完成了
- 修改任务的 area 时报错：`no column found for name: is_all_day`

**原因：**

- `features/tasks/shared/repositories/task_time_block_link_repository.rs` 中的 `find_linked_time_blocks_in_tx` 函数
- 直接查询 `time_blocks` 表，返回 `TimeBlock` 实体
- SQL 查询中缺少 `is_all_day` 字段

**解决方案：**

1. 修改实体后，使用以下命令查找所有 SQL 查询：

   ```bash
   # 查找主功能模块的查询
   grep -rn "SELECT.*FROM time_blocks" src-tauri/src/features/time_blocks

   # 查找跨功能模块的查询
   grep -rn "SELECT.*FROM time_blocks" src-tauri/src/features/tasks
   grep -rn "SELECT.*FROM time_blocks" src-tauri/src/features
   ```

2. 逐一更新所有 SELECT 语句的字段列表
3. 特别注意 Repository 中的查询，不仅仅是装配器

---

**记住：数据结构是系统的骨架，修改需谨慎且全面！特别注意跨功能模块的隐藏依赖！**
