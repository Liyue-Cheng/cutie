# Project 功能软件设计文档

> **版本**: V1.0
> **最后更新**: 2025-11-17
> **文档类型**: 软件定义书
> **范围**: Project + ProjectSection 全栈实现

---

## 目录

1. [概述与核心设计](#1-概述与核心设计)
2. [数据模型设计](#2-数据模型设计)
3. [后端架构设计](#3-后端架构设计)
4. [前端架构设计](#4-前端架构设计)
5. [交互设计与用户体验](#5-交互设计与用户体验)
6. [View Context 规范](#6-view-context-规范)
7. [开发实施指南](#7-开发实施指南)

---

## 1. 概述与核心设计

### 1.1 功能目标

实现一个**项目管理系统**，允许用户：
- 创建和管理项目（Projects）
- 在项目下创建章节（Sections）组织任务
- 任务可以直接属于项目，或属于项目下的某个章节
- 项目颜色从所属 Area 继承，保持视觉一致性
- 项目排序通过 view_preferences 统一管理

### 1.2 核心设计决策

#### 数据结构决策

**Projects 表简化**：
- ❌ 移除 `type` 字段 - 只保留 PROJECT 类型
- ❌ 移除 `resources` 字段 - 用途不明确
- ❌ 移除 `color` 字段 - 从 area 继承，避免冗余
- ❌ 移除 `sort_order` 字段 - 由 view_preferences 统一管理
- ❌ 移除所有 `external_source_*` 字段 - 项目是内部数据
- ❌ 移除 `PAUSED` 和 `ARCHIVED` 状态 - 只保留 ACTIVE 和 COMPLETED
- ✅ 新增 `description` 字段 - 项目描述
- ✅ 新增 `due_date` 字段 - 项目截止日期

**ProjectSections 独立表**：
- 选择独立表而非 JSON 字段的原因：
  1. 外键约束保证数据完整性
  2. 易于查询和索引
  3. 符合项目架构风格（所有关联都是独立表）
  4. 易于扩展（未来可添加更多字段）

**Tasks 表扩展**：
- 新增 `section_id` 字段
- 业务约束：`section_id` 不为空时，`project_id` 必须不为空
- 验证：section 必须属于对应的 project

#### 架构决策

**数据流模型**：
```
Areas (领域) - 提供颜色
  └── Projects (项目)
        ├── ProjectSections (章节) - 可选
        │     └── Tasks (任务)
        └── Tasks (任务) - 直接属于项目
```

**ViewKey 设计**：
```
project::{project_id}                            # 项目所有任务
project::{project_id}::section::all              # 项目无section任务
project::{project_id}::section::{section_id}     # 特定section任务
```

---

## 2. 数据模型设计

### 2.1 数据库 Schema

> **重要提示**：由于会删除旧数据库，直接修改 `20241001000000_initial_schema.sql`

#### 2.1.1 Projects 表（修改）

```sql
DROP TABLE IF EXISTS projects;

CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT,                     -- 项目描述

    -- 状态管理（仅 ACTIVE 和 COMPLETED）
    status TEXT NOT NULL DEFAULT 'ACTIVE'
        CHECK (status IN ('ACTIVE', 'COMPLETED')),

    -- 时间信息
    due_date TEXT,                        -- 截止日期 (YYYY-MM-DD)
    completed_at TEXT,                    -- 完成时间 (UTC RFC 3339)

    -- 关联（颜色从 area 继承）
    area_id TEXT,

    -- 统计信息（后端维护）
    total_tasks INTEGER NOT NULL DEFAULT 0,
    completed_tasks INTEGER NOT NULL DEFAULT 0,

    -- 元数据
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,

    FOREIGN KEY (area_id) REFERENCES areas(id)
);

-- 索引
CREATE INDEX idx_projects_updated_at ON projects(updated_at);
CREATE INDEX idx_projects_is_deleted ON projects(is_deleted);
CREATE INDEX idx_projects_area_id ON projects(area_id);
CREATE INDEX idx_projects_status ON projects(status);
CREATE INDEX idx_projects_due_date ON projects(due_date);
```

#### 2.1.2 ProjectSections 表（新增）

```sql
CREATE TABLE project_sections (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,                     -- 章节描述
    sort_order TEXT,                      -- 排序字段
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_project_sections_project_id ON project_sections(project_id);
CREATE INDEX idx_project_sections_is_deleted ON project_sections(is_deleted);
CREATE INDEX idx_project_sections_updated_at ON project_sections(updated_at);
```

#### 2.1.3 Tasks 表（修改）

```sql
DROP TABLE IF EXISTS tasks;

CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    -- ... 其他字段保持不变
    project_id TEXT,
    section_id TEXT,                      -- 🆕 新增字段
    area_id TEXT,
    -- ... 其他字段保持不变

    FOREIGN KEY (project_id) REFERENCES projects(id),
    FOREIGN KEY (section_id) REFERENCES project_sections(id),
    FOREIGN KEY (area_id) REFERENCES areas(id),

    -- 业务约束：如果有 section_id，必须有 project_id
    CHECK (section_id IS NULL OR project_id IS NOT NULL)
);

-- 新增索引
CREATE INDEX idx_tasks_section_id ON tasks(section_id);
```

#### 2.1.4 实际操作步骤

1. **备份** `initial_schema.sql`（可选）
2. **修改 projects 表**：删除字段 + 新增字段
3. **添加 project_sections 表**：在 projects 表之后
4. **修改 tasks 表**：添加 section_id 字段和约束
5. **删除旧数据库**：`rm src-tauri/*.db*`
6. **重启应用**：触发迁移

### 2.2 数据模型关系

#### 核心实体

| 实体 | 职责 | 关键字段 |
|------|------|---------|
| **Project** | 项目容器 | name, description, status (ACTIVE/COMPLETED), due_date, area_id, total_tasks, completed_tasks |
| **ProjectSection** | 项目章节 | project_id, title, description, sort_order |
| **Task** | 任务实体 | project_id, section_id, ... |

#### 数据完整性规则

1. **Project → Area**: 可选关联，用于颜色继承
2. **Section → Project**: 强制关联，CASCADE 删除
3. **Task → Project**: 可选关联
4. **Task → Section**: 可选关联，但必须有对应的 project_id
5. **颜色继承**: Project 不存储颜色，运行时从 Area 获取（前端查询）
6. **Section 排序**: 使用 `sort_order` 字段（Lexorank），不使用 view_preferences
7. **统计信息**: `total_tasks` 和 `completed_tasks` 由后端维护，每次任务变化时更新
8. **删除策略**: 软删除项目时，同时软删除所有关联的 sections 和 tasks

---

## 3. 后端架构设计

### 3.1 架构概述

遵循 **Feature-Sliced Architecture** + **SFC (Single File Component)** 模式。

#### 目录结构

```
src-tauri/src/
├── entities/
│   ├── project.rs           # ProjectRow, Project, ProjectDto
│   └── project_section.rs   # SectionRow, Section, SectionDto
├── features/
│   ├── shared/
│   │   ├── project_repository.rs        # 项目数据访问
│   │   └── project_section_repository.rs
│   ├── endpoints/
│   │   └── projects/
│   │       ├── create_project.rs        # POST /projects
│   │       ├── update_project.rs        # PATCH /projects/:id
│   │       ├── delete_project.rs        # DELETE /projects/:id
│   │       ├── list_projects.rs         # GET /projects
│   │       ├── get_project.rs           # GET /projects/:id
│   │       ├── create_section.rs        # POST /projects/:id/sections
│   │       ├── update_section.rs        # PATCH /projects/:id/sections/:sid
│   │       ├── delete_section.rs        # DELETE /projects/:id/sections/:sid
│   │       └── list_sections.rs         # GET /projects/:id/sections
│   └── projects.rs          # 路由注册
```

### 3.2 核心组件职责

#### Entities (实体层)

**职责**：
- 定义数据库行结构 (`ProjectRow`)
- 定义业务实体 (`Project`)
- 定义 DTO (`ProjectDto`)
- 实现类型转换 (`TryFrom<Row>`, `From<Entity>`)

**关键类型**：
```rust
// 数据库 → 内部实体
impl TryFrom<ProjectRow> for Project

// 内部实体 → DTO（API 响应）
impl From<Project> for ProjectDto

// 枚举类型
enum ProjectStatus { Active, Paused, Completed, Archived }
```

#### Repositories (数据访问层)

**ProjectRepository 职责**：
- `list_all(pool)` - 查询所有项目
- `find_by_id(pool, id)` - 根据 ID 查询
- `find_by_area(pool, area_id)` - 根据 area 查询
- `insert(tx, project)` - 插入项目
- `update(tx, project)` - 更新项目
- `soft_delete(tx, id, now)` - 软删除项目
- `update_statistics(tx, project_id)` - 更新项目统计信息

**统计信息维护**：
```rust
// 每次任务的 project_id 或完成状态变化时调用
pub async fn update_statistics(tx: &mut Transaction, project_id: &str) -> AppResult<()> {
    let total = count_tasks_by_project(tx, project_id).await?;
    let completed = count_completed_tasks_by_project(tx, project_id).await?;

    sqlx::query!(
        "UPDATE projects SET total_tasks = ?, completed_tasks = ?, updated_at = ? WHERE id = ?",
        total, completed, now, project_id
    ).execute(tx).await?;

    Ok(())
}
```

**ProjectSectionRepository 职责**：
- `list_by_project(pool, project_id)` - 查询项目的所有章节（按 sort_order 排序）
- `find_by_id(pool, id)` - 根据 ID 查询
- `insert(tx, section)` - 插入章节
- `update(tx, section)` - 更新章节
- `soft_delete(tx, id, now)` - 软删除章节
- `reorder(tx, section_id, new_sort_order)` - 更新排序（Lexorank）

**设计原则**：
- 所有写操作使用事务 (`Transaction`)
- 所有读操作使用连接池 (`SqlitePool`)
- 统一错误处理 (`AppResult<T>`)
- Section 排序使用 Lexorank 算法保证插入性能

#### Endpoints (端点层)

**SFC 端点结构**：
```rust
/// CABC 文档（8个章节）
/// 1. 端点签名
/// 2. 预期行为简介
/// 3. 输入输出规范
/// 4. 验证规则
/// 5. 业务逻辑详解
/// 6. 边界情况
/// 7. 预期副作用
/// 8. 契约

// HTTP 处理器
pub async fn handle(...) -> Response {
    match logic::execute(...).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// 业务逻辑层
mod logic {
    pub async fn execute(...) -> AppResult<Dto> {
        // 1. 验证输入
        // 2. 获取依赖 (id_generator, clock)
        // 3. 获取写入许可
        // 4. 开启事务
        // 5. 数据库操作 (使用 Repository)
        // 6. 写入 Event Outbox (事务内)
        // 7. 提交事务
        // 8. 返回 DTO
    }
}

// 数据库层 - 如果需要特殊查询
mod database { ... }

// 事件层
mod events { ... }
```

**关键原则**：
- ✅ 使用 `success_response(dto)` 包装响应
- ✅ 使用 `acquire_write_permit()` 串行化写操作
- ✅ 在事务内写入 Event Outbox
- ✅ SSE 事件与 HTTP 响应数据一致

#### 路由注册

**features/projects.rs**：
```rust
pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Projects
        .route("/", get(list_projects))
        .route("/", post(create_project))
        .route("/:id", get(get_project))
        .route("/:id", patch(update_project))
        .route("/:id", delete(delete_project))
        .route("/:id/complete-all", post(complete_all_tasks))  // 🆕 批量完成
        // Sections
        .route("/:project_id/sections", get(list_sections))
        .route("/:project_id/sections", post(create_section))
        .route("/:project_id/sections/:id", patch(update_section))
        .route("/:project_id/sections/:id", delete(delete_section))
}
```

#### 关键端点说明

**POST /projects/:id/complete-all** - 批量完成项目任务：

**行为**：
1. 查询项目下所有未完成任务（包括所有 sections）
2. 遍历批量完成任务（调用任务完成逻辑）
3. 更新项目状态：`status = 'COMPLETED'`, `completed_at = now()`
4. 更新统计信息：`completed_tasks = total_tasks`
5. 发送 SSE 事件：`project.completed`

**DELETE /projects/:id** - 软删除项目：

**行为**：
1. 软删除项目：`SET is_deleted = TRUE`
2. 软删除所有关联 sections：`SET is_deleted = TRUE WHERE project_id = ?`
3. 软删除所有关联 tasks：`SET is_deleted = TRUE WHERE project_id = ?`
4. 保留数据用于恢复或审计
5. 发送 SSE 事件：`project.deleted`

**PATCH /projects/:id** - 更新项目（支持重新打开）：

**行为**：
- 允许更新：name, description, status, due_date, area_id
- 如果 `status` 从 COMPLETED 改为 ACTIVE：
  - 清除 `completed_at`
  - **不处理任务的完成状态**（已完成的任务保持已完成）
  - 更新统计信息保持不变

**features/mod.rs**：
```rust
pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .nest("/tasks", tasks::create_routes())
        .nest("/projects", projects::create_routes())  // 🆕
        // ... 其他路由
}
```

### 3.3 SSE 事件设计

**事件类型**：
- `project.created` - 项目创建
- `project.updated` - 项目更新
- `project.deleted` - 项目删除
- `project_section.created` - 章节创建
- `project_section.updated` - 章节更新
- `project_section.deleted` - 章节删除

**事件载荷**：完整的 DTO 对象（与 HTTP 响应一致）

---

## 4. 前端架构设计

### 4.1 架构概述

遵循 **CPU 指令集架构** + **RTL Store 设计** + **拖放策略系统**。

#### 目录结构

```
src/
├── types/
│   └── dtos.ts                    # ProjectCard, ProjectSection
├── cpu/
│   └── isa/
│       ├── project-isa.ts         # Project 指令集
│       └── index.ts               # 注册 ISA
├── stores/
│   └── project/
│       ├── index.ts               # Store 主入口
│       ├── core.ts                # State + Getters + Mutations
│       ├── view-operations.ts     # DMA 数据加载
│       └── event-handlers.ts      # SSE 事件处理
├── services/
│   └── viewAdapter.ts             # ViewKey 解析和元数据
├── composables/
│   └── useViewTasks.ts            # 扩展支持 project viewKey
├── infra/
│   └── drag/
│       └── strategies/
│           ├── project-scheduling.ts  # Project 拖放策略
│           └── index.ts               # 导出策略
└── components/
    ├── parts/
    │   └── CircularProgress.vue       # 圆饼进度指示器
    └── organisms/
        ├── ProjectListPanel.vue       # 项目列表面板
        ├── ProjectDetailPanel.vue     # 项目详情面板
        └── ProjectsPanel.vue          # 主容器
```

### 4.2 核心组件职责

#### 类型定义 (types/dtos.ts)

```typescript
export interface ProjectCard {
  id: string
  name: string
  description: string | null
  status: 'ACTIVE' | 'PAUSED' | 'COMPLETED' | 'ARCHIVED'
  due_date: string | null
  completed_at: string | null
  area_id: string | null
  created_at: string
  updated_at: string
}

export interface ProjectSection {
  id: string
  project_id: string
  title: string
  description: string | null
  sort_order: string | null
  created_at: string
  updated_at: string
}
```

#### CPU 指令集 (cpu/isa/project-isa.ts)

**指令清单**：
- `project.create` - 创建项目
- `project.update` - 更新项目
- `project.delete` - 删除项目
- `project.list` - 列出所有项目
- `project_section.create` - 创建章节
- `project_section.update` - 更新章节
- `project_section.delete` - 删除章节

**指令结构**：
```typescript
'project.create': {
  meta: {
    description: '创建项目',
    category: 'project',
    resourceIdentifier: (payload) => ['project:new'],
    priority: 5,
  },
  request: {
    method: 'POST',
    url: '/projects',
    body: (payload) => payload,
  },
  commit: async (result: ProjectCard) => {
    const store = useProjectStore()
    store.addOrUpdateProject_mut(result)
  },
}
```

**设计原则**：
- 所有 API 调用通过指令集声明
- commit 函数调用 Store 的 `_mut` 方法
- 支持乐观更新（可选）

#### Pinia Store (stores/project/)

**Store 结构（RTL 硬件设计）**：

**State (寄存器)**：
- `projects: Map<string, ProjectCard>`
- `sections: Map<string, ProjectSection>`

**Getters (多路复用器)**：
- `allProjects` - 所有项目
- `activeProjects` - 活跃项目
- `getProjectById(id)` - 根据 ID 获取
- `getSectionsByProject(projectId)` - 获取项目的章节

**Mutations (寄存器写入)**：
- `addOrUpdateProject_mut(project)` - 添加/更新项目
- `removeProject_mut(id)` - 移除项目
- `addOrUpdateSection_mut(section)` - 添加/更新章节
- `removeSection_mut(id)` - 移除章节
- `clearAll_mut()` - 清空所有数据

**DMA (数据加载)**：
- `fetchAllProjects()` - 加载所有项目

**Event Handling (SSE 中断)**：
- `initEventSubscriptions()` - 注册 SSE 事件处理器

**关键原则**：
- ✅ Mutation 必须以 `_mut` 结尾
- ✅ State 使用 Map 结构（不可变更新）
- ✅ 事件处理器调用 `_mut` 函数
- ✅ 不在 Store 中直接调用 API

#### 验证规则与对话框

**验证规则**：

*项目验证*：
- `name`: 必填，1-200 字符
- `description`: 可选，0-2000 字符
- `due_date`: 可选，格式 YYYY-MM-DD，必须 >= 今天
- `area_id`: 可选，必须是有效的 area UUID

*Section 验证*：
- `title`: 必填，1-200 字符
- `description`: 可选，0-2000 字符

**对话框组件**：

1. **CreateProjectDialog** - 创建项目
2. **EditProjectDialog** - 编辑项目（含删除）
3. **CreateSectionDialog** - 创建章节
4. **EditSectionDialog** - 编辑章节（含删除）
5. **ConfirmCompleteProjectDialog** - 确认完成项目

#### View Adapter (services/viewAdapter.ts)

**扩展 ViewContext**：
```typescript
export type ViewContext =
  | { type: 'misc'; id: string }
  | { type: 'daily'; date: string }
  | { type: 'project'; projectId: string }                    // 🆕
  | { type: 'project_section'; projectId: string; sectionId: string }  // 🆕
  | { type: 'upcoming'; ... }
```

**关键函数**：
- `parseViewKey(viewKey)` - 解析 ViewKey 为 ViewContext
- `deriveViewMetadata(viewKey)` - 生成视图元数据
- `getContextKey(context)` - ViewContext → ViewKey

#### useViewTasks Composable

**扩展支持 Project ViewKey**：
```typescript
// project::{project_id} - 项目所有任务
if (parts[0] === 'project' && parts.length === 2) {
  return taskStore.allTasks.filter(
    (task) => task.project_id === projectId && !task.archived_at && !task.deleted_at
  )
}

// project::{project_id}::section::all - 无section任务
if (parts[0] === 'project' && parts[2] === 'section' && parts[3] === 'all') {
  return taskStore.allTasks.filter(
    (task) => task.project_id === projectId && !task.section_id && ...
  )
}

// project::{project_id}::section::{section_id} - 特定section任务
if (parts[0] === 'project' && parts[2] === 'section' && parts[3]) {
  return taskStore.allTasks.filter(
    (task) => task.section_id === sectionId && ...
  )
}
```

---

## 5. 交互设计与用户体验

### 5.1 组件架构

#### 总体结构

```
ProjectsPanel (主容器)
  └── 水平分割（左右各 50%）
      ├── 左侧 - TwoRowLayout (垂直分割)
      │     ├── 上栏 (暂时空着，未来扩展)
      │     └── 下栏 - 项目区域
      │           ├── ProjectListPanel (左 30%)
      │           │     ├── 控制栏（标题 + 创建按钮）
      │           │     └── 项目卡片列表
      │           │           ├── CircularProgress (进度指示器)
      │           │           ├── 项目信息（名称、任务数、截止日期）
      │           │           └── 项目状态
      │           └── ProjectDetailPanel (右 70%)
      │                 ├── 项目头部（名称、描述、操作按钮）
      │                 └── 任务列表区域
      │                       ├── TaskList (无section任务) - 可选
      │                       └── TaskList × N (各个section)
      └── 右侧 - TwoRowLayout (垂直分割)
            ├── 上栏 - Dummy 内容区
            │     └── (占位内容，未来功能预留)
            └── 下栏 - DoubleRowTimeline (时间线)
                  ├── 按月显示日期单元格（2列网格）
                  ├── 显示任务排期和截止日期
                  ├── 自动滚动到今天
                  └── 支持拖放操作
```

**布局说明**：
- **左右分割**：ProjectsPanel 使用水平分割，左右各 50%
- **左侧 TwoRowLayout**：上栏暂时空着，下栏包含项目列表和详情（再次水平分割为 30%/70%）
- **右侧 TwoRowLayout**：上栏 Dummy 内容，下栏显示月度时间线
- **时间线功能**：集成 DoubleRowTimeline 组件，显示当月任务排期，支持拖放排期操作

#### 5.1.1 CircularProgress - 圆饼进度指示器

**设计规格**：
- **形态**：SVG 圆环进度条
- **尺寸**：
  - `small`: 2.1rem (21px) - 与 large checkbox 尺寸一致
  - `normal`: 4.8rem (48px)
  - `large`: 6.4rem (64px)
- **颜色**：
  - 未开始 (0%): 灰色 `#d1d5db`
  - 进行中 (1-49%): 橙色 `#f59e0b`
  - 进行中 (50-99%): 蓝色 `#4a90e2`
  - 已完成 (100%): 绿色 `#10b981`
- **动画**：进度变化使用 cubic-bezier 缓动

**交互行为**：
- 单击时弹出确认对话框，询问用户是否完成项目下所有未完成任务
- 确认后批量完成所有任务，项目状态变为 COMPLETED

**技术实现**：
- 使用 SVG `<circle>` + `stroke-dasharray` + `stroke-dashoffset`
- 进度计算：`(completed / total) * 100`
- 圆环偏移量：`circumference * (1 - progress)`
- 点击事件触发确认对话框

#### 5.1.2 ProjectListPanel - 项目列表

**功能需求**：
- 显示所有活跃项目（`status === 'ACTIVE'`）
- 支持点击选择项目（高亮显示）
- 显示项目进度（CircularProgress）
- 显示项目基本信息（名称、任务数、截止日期）
- 显示项目状态标签
- 项目颜色条（从 area 继承）

**交互行为**：
- 点击项目卡片 → 触发 `@select-project` 事件
- 点击"创建项目"按钮 → 触发 `@create-project` 事件
- 选中的项目卡片添加 `.active` 样式类

**数据来源**：
- 项目列表：`projectStore.activeProjects`
- 项目颜色：`areaStore.getAreaById(project.area_id).color`
- 任务数量：`taskStore.allTasks.filter(...).length`

#### 5.1.3 ProjectDetailPanel - 项目详情

**功能需求**：
- 显示选中项目的详细信息
- 显示项目头部（名称、描述、area标签）
- 提供操作按钮（编辑项目、添加章节）
- 使用 TaskList 组件显示任务列表
- 支持多个 section，每个 section 一个 TaskList
- 空状态提示（未选择项目时）

**交互行为**：
- 点击"编辑项目"按钮 → 触发 `@edit-project` 事件
- 点击"添加章节"按钮 → 触发 `@create-section` 事件
- TaskList 组件处理任务的拖放和交互

**数据来源**：
- 当前项目：`projectStore.getProjectById(projectId)`
- 项目 area：`areaStore.getAreaById(project.area_id)`
- 项目章节：`projectStore.getSectionsByProject(projectId)`
- 任务列表：通过 TaskList 的 `view-key` 自动获取

**ViewKey 使用**：
```vue
<!-- 无 section 的任务 -->
<TaskList :view-key="`project::${project.id}::section::all`" />

<!-- 各个 section 的任务 -->
<TaskList
  v-for="section in sections"
  :view-key="`project::${project.id}::section::${section.id}`"
/>
```

#### 5.1.4 ProjectsPanel - 主容器

**功能需求**：
- 使用 TwoRowLayout 组织左右面板
- 管理项目选择状态（`selectedProjectId`）
- 协调子组件间的事件通信
- 初始化时加载项目数据
- 默认选择第一个项目

**状态管理**：
- `selectedProjectId: ref<string | undefined>()` - 当前选中的项目

**事件处理**：
- `handleSelectProject(id)` - 选择项目
- `handleCreateProject()` - 创建项目（TODO: 打开对话框）
- `handleEditProject(id)` - 编辑项目（TODO: 打开对话框）
- `handleCreateSection(projectId)` - 创建章节（TODO: 打开对话框）

**TwoRowLayout 配置**：
- `split-direction="horizontal"` - 水平分割（左右布局）
- `split-ratio="0.3"` - 左侧占 30%，右侧占 70%

### 5.2 拖放策略设计

#### 支持的拖放场景

| 场景 | 源 | 目标 | 操作 | 优先级 |
|------|----|----|------|-------|
| 1 | Project | Daily | 创建日程（保留 project_id） | 90 |
| 2 | Section | Daily | 创建日程（保留 project_id + section_id） | 90 |
| 3 | Daily | Project | 设置 project_id（保留日程） | 85 |
| 4 | Daily | Section | 设置 project_id + section_id（保留日程） | 90 |
| 5 | Project | Project | 同项目内重排 | 80 |
| 6 | Project | Section | 移动到 section | 85 |
| 7 | Section | Section | 跨 section 移动 | 85 |
| 8 | Section | Project | 移回项目（清除 section_id） | 85 |

**设计说明**：
- **Project/Section → Daily（排期）**：
  - 从项目视图拖任务到日历 = 安排日程
  - 保留任务的项目归属（project_id + section_id）
  - 行为类似 Staging → Daily，但保留项目关联
- **Daily → Project/Section（分配项目）**：
  - 从日历拖任务到项目视图 = 设置项目归属
  - 保留任务的日程安排
  - 可以给已排期的任务分配项目
- **不支持 Staging ↔ Project/Section 拖放**：
  - Staging 筛选条件是"无有效排期"，与 project_id 无关
  - 任务可以同时有 project_id 和无排期（会出现在 Staging）
  - 设置/清除 project_id 应通过任务编辑器，而非拖放
- **项目分类 ≠ 排期系统**：
  - Project/Section 是任务的组织分类（类似 Area）
  - 拖放到 Project 视图 = 改变项目归属
  - 拖放到 Daily 视图 = 安排日程
  - 两者是独立的维度，可以同时设置

#### 策略实现模式

**基本结构**：
```typescript
export const strategyName: Strategy = {
  id: 'strategy-id',
  name: 'Strategy Name',

  conditions: {
    source: {
      viewKey: 'misc::staging' | /^project::...$/,
      objectType: 'task',
    },
    target: {
      viewKey: /^project::...$/,
    },
    priority: 90,
  },

  action: {
    name: 'action_name',
    description: '操作描述',
    async execute(ctx) {
      const operations = []

      try {
        // 步骤 1: 更新任务 (task.update)
        await pipeline.dispatch('task.update', { id, updates })

        // 步骤 2: 更新源视图排序 (viewpreference.update_sorting)
        await pipeline.dispatch('viewpreference.update_sorting', { ... })

        // 步骤 3: 更新目标视图排序
        await pipeline.dispatch('viewpreference.update_sorting', { ... })

        return {
          success: true,
          message: '✅ 操作成功',
          operations,
          affectedViews: [ctx.sourceViewId, ctx.targetViewId],
        }
      } catch (error) {
        return { success: false, message: `❌ ${error.message}`, operations }
      }
    },
  },

  tags: ['project', 'scheduling'],
}
```

**关键工具函数**：
- `extractTaskIds(context)` - 提取任务 ID 列表
- `insertTaskAt(list, taskId, index)` - 在指定位置插入
- `removeTaskFrom(list, taskId)` - 从列表移除
- `createOperationRecord(type, viewId, payload)` - 创建操作记录

### 5.3 路由集成

**HomeView 集成**：
```vue
<template>
  <ProjectsPanel v-if="viewType === 'projects'" />
  <RecentTaskPanel v-else-if="viewType === 'recent'" />
  <StagingTaskPanel v-else-if="viewType === 'staging'" />
</template>

<script setup lang="ts">
const viewType = computed(() => route.query.view || 'recent')
</script>
```

**MainLayout 导航**：
```vue
<li @click="$router.push({ path: '/', query: { view: 'projects' } })">
  <CuteIcon name="Folder" :size="16" /><span>Projects</span>
</li>
```

---

## 6. View Context 规范

### 6.1 ViewKey 格式定义

#### 项目容器视图

| 视图类型 | Context Key 格式 | 说明 | 排序内容 |
|---------|-----------------|------|---------|
| 项目列表 | `misc::projects` | ProjectListPanel 中的项目列表 | 项目卡片的排序 |

**说明**：
- 项目列表使用 `misc::projects` 作为 ViewKey
- view_preferences 存储项目 ID 的排序数组
- 格式：`sorted_task_ids: '["project-uuid-1", "project-uuid-2", ...]'`
- 虽然字段名是 `sorted_task_ids`，但存储的是 project IDs（复用现有字段）

#### 项目内任务视图

| 视图类型 | Context Key 格式 | 说明 | 排序内容 |
|---------|-----------------|------|---------|
| 项目所有任务 | `project::{project_id}` | 显示项目的所有任务（用于统计，不用于显示） | 任务排序 |
| 项目无 Section 任务 | `project::{project_id}::section::all` | 显示直接属于项目但不属于任何 section 的任务 | 任务排序 |
| 项目章节任务 | `project::{project_id}::section::{section_id}` | 显示特定章节的任务 | 任务排序 |

**说明**：
- 每个 TaskList 组件对应一个独立的 ViewKey
- ProjectDetailPanel 中可能有多个 TaskList，每个都有自己的排序
- view_preferences 为每个 ViewKey 存储对应的任务 ID 排序

### 6.2 ViewKey 示例

```javascript
// 项目列表排序
'misc::projects'
// sorted_task_ids: '["proj-uuid-1", "proj-uuid-2", "proj-uuid-3"]'

// 项目所有任务（统计用）
'project::a1b2c3d4-1234-5678-90ab-cdef12345678'
// sorted_task_ids: '["task-1", "task-2", ...]'

// 项目的无 section 任务列表
'project::a1b2c3d4-1234-5678-90ab-cdef12345678::section::all'
// sorted_task_ids: '["task-1", "task-3", ...]'

// 项目章节任务列表
'project::a1b2c3d4-1234-5678-90ab-cdef12345678::section::s1s2s3s4-5678-90ab-cdef-123456789abc'
// sorted_task_ids: '["task-2", "task-4", ...]'
```

### 6.3 ViewContext 类型扩展

**TypeScript 类型定义**：
```typescript
// src/services/viewAdapter.ts
export type ViewContext =
  | { type: 'misc'; id: string }                                          // 包括 'projects'
  | { type: 'daily'; date: string }                                      // YYYY-MM-DD
  | { type: 'project'; projectId: string }                               // 🆕 项目视图
  | { type: 'project_section'; projectId: string; sectionId: string }    // 🆕 章节视图
  | { type: 'upcoming'; timeRange: string; taskType: string }
```

**ViewKey 解析函数扩展**：
```typescript
function parseViewKey(viewKey: string): ViewContext | null {
  const parts = viewKey.split('::')

  // misc::projects
  if (parts[0] === 'misc') {
    return { type: 'misc', id: parts[1] }
  }

  // project::{project_id}
  if (parts[0] === 'project' && parts.length === 2) {
    return { type: 'project', projectId: parts[1] }
  }

  // project::{project_id}::section::all
  if (parts[0] === 'project' && parts[2] === 'section' && parts[3] === 'all') {
    return { type: 'project_section', projectId: parts[1], sectionId: 'all' }
  }

  // project::{project_id}::section::{section_id}
  if (parts[0] === 'project' && parts[2] === 'section' && parts.length === 4) {
    return { type: 'project_section', projectId: parts[1], sectionId: parts[3] }
  }

  // ... 其他类型
}
```

### 6.4 验证规则

**项目 ViewKey 验证**：
- 格式：`misc::projects` 或 `project::{uuid}` 或 `project::{uuid}::section::{uuid|all}`
- UUID 必须符合标准格式（带连字符）
- section 后可以是 UUID 或特殊值 `all`

**实现要点**：
```typescript
function validateContextKey(key: string): boolean {
  const parts = key.split('::')

  // misc::projects
  if (parts[0] === 'misc' && parts[1] === 'projects') {
    return true
  }

  // project 相关
  if (parts[0] === 'project') {
    // project::{uuid}
    if (parts.length === 2) {
      return isValidUUID(parts[1])
    }
    // project::{uuid}::section::{uuid|all}
    if (parts.length === 4 && parts[2] === 'section') {
      return isValidUUID(parts[1]) && (parts[3] === 'all' || isValidUUID(parts[3]))
    }
  }

  return false
}
```

### 6.5 view_preferences 数据示例

```sql
-- 项目列表排序
INSERT INTO view_preferences (context_key, sorted_task_ids, updated_at) VALUES
('misc::projects', '["proj-uuid-1", "proj-uuid-2", "proj-uuid-3"]', '2025-11-17T10:00:00Z');

-- 项目 A 的无 section 任务排序
INSERT INTO view_preferences (context_key, sorted_task_ids, updated_at) VALUES
('project::proj-uuid-1::section::all', '["task-1", "task-3", "task-5"]', '2025-11-17T10:01:00Z');

-- 项目 A 的章节 S1 任务排序
INSERT INTO view_preferences (context_key, sorted_task_ids, updated_at) VALUES
('project::proj-uuid-1::section::section-uuid-1', '["task-2", "task-4"]', '2025-11-17T10:02:00Z');

-- 项目 A 的章节 S2 任务排序
INSERT INTO view_preferences (context_key, sorted_task_ids, updated_at) VALUES
('project::proj-uuid-1::section::section-uuid-2', '["task-6", "task-7", "task-8"]', '2025-11-17T10:03:00Z');
```

### 6.6 组件使用示例

**ProjectListPanel 组件**：
```typescript
// 项目列表使用 misc::projects 作为 viewKey
const projectViewKey = 'misc::projects'
const { items: sortedProjects } = useViewPreference(projectViewKey, allProjects)

// 拖放重排后更新 view_preferences
await pipeline.dispatch('viewpreference.update_sorting', {
  context_key: 'misc::projects',
  sorted_task_ids: sortedProjectIds,  // 实际是 project IDs
})
```

**ProjectDetailPanel 组件**：
```vue
<template>
  <!-- 无 section 的任务列表 -->
  <TaskList
    v-if="hasTasksWithoutSection"
    :title="'未分类任务'"
    :view-key="`project::${selectedProject.id}::section::all`"
  />

  <!-- 各个 section 的任务列表 -->
  <TaskList
    v-for="section in sections"
    :key="section.id"
    :title="section.title"
    :view-key="`project::${selectedProject.id}::section::${section.id}`"
  />
</template>
```

### 6.7 拖放排序更新流程

**项目列表拖放重排**：
1. 用户在 ProjectListPanel 中拖放项目卡片
2. 前端计算新的排序数组（project IDs）
3. 调用 `viewpreference.update_sorting` 指令
4. 后端更新 `view_preferences` 表的 `misc::projects` 记录
5. SSE 事件通知前端更新
6. ProjectListPanel 重新渲染

**项目内任务拖放重排**：
1. 用户在 TaskList 中拖放任务
2. TaskList 组件自动处理排序更新（已有逻辑）
3. 使用对应的 viewKey（如 `project::xxx::section::all`）
4. 更新 view_preferences 表对应记录
5. SSE 事件通知更新

**跨 Section 拖放**：
1. 从 Section A 拖任务到 Section B
2. 更新任务的 `section_id` 字段
3. 更新两个 ViewKey 的排序：
   - 源 ViewKey：移除该任务 ID
   - 目标 ViewKey：在指定位置插入该任务 ID
4. 两次调用 `viewpreference.update_sorting`

---

## 7. 开发实施指南

### 7.1 开发顺序建议

#### 阶段 1：数据库和后端基础

1. **修改数据库 Schema**
   - 修改 `initial_schema.sql`
   - 删除旧数据库并重启
   - 验证表结构正确

2. **创建 Entities**
   - `entities/project.rs` - ProjectRow, Project, ProjectDto
   - `entities/project_section.rs` - SectionRow, Section, SectionDto
   - 实现类型转换 traits

3. **创建 Repositories**
   - `features/shared/project_repository.rs`
   - `features/shared/project_section_repository.rs`
   - 实现 CRUD 操作

4. **创建端点**
   - Projects 端点（5个）
   - Sections 端点（4个）
   - 完整的 CABC 文档

5. **注册路由**
   - `features/projects.rs`
   - `features/mod.rs`

6. **测试后端**
   - `cargo check` 通过
   - `cargo clippy` 无警告
   - 使用 curl/Postman 测试 API

#### 阶段 2：前端基础

1. **类型定义**
   - `types/dtos.ts` - ProjectCard, ProjectSection
   - 更新 TaskCard 添加 section_id

2. **CPU 指令集**
   - `cpu/isa/project-isa.ts`
   - 注册到 `cpu/isa/index.ts`

3. **Pinia Store**
   - `stores/project/core.ts` - State + Getters + Mutations
   - `stores/project/event-handlers.ts` - SSE 事件处理
   - `stores/project/view-operations.ts` - DMA 数据加载
   - `stores/project/index.ts` - 组合导出

4. **View Adapter 扩展**
   - 扩展 ViewContext 类型
   - 实现 parseViewKey 对 project 的支持
   - 实现 deriveViewMetadata

5. **useViewTasks 扩展**
   - 支持 project viewKey
   - 支持 section viewKey

6. **测试前端基础**
   - 无 TypeScript 错误
   - 指令可以调用
   - Store 状态正确更新
   - SSE 事件正确接收

#### 阶段 3：UI 组件

1. **CircularProgress 组件**
   - `components/parts/CircularProgress.vue`
   - 支持不同尺寸
   - 动态颜色

2. **ProjectListPanel 组件**
   - `components/organisms/ProjectListPanel.vue`
   - 集成 CircularProgress
   - 项目列表渲染

3. **ProjectDetailPanel 组件**
   - `components/organisms/ProjectDetailPanel.vue`
   - 集成 TaskList 组件
   - 支持多个 section

4. **ProjectsPanel 主容器**
   - `components/organisms/ProjectsPanel.vue`
   - 集成 TwoRowLayout
   - 状态管理

5. **路由集成**
   - 在 HomeView 中集成
   - 在 MainLayout 中添加导航

6. **测试 UI**
   - 组件渲染正确
   - 交互功能正常
   - 样式符合设计

#### 阶段 4：拖放功能

1. **创建拖放策略**
   - `infra/drag/strategies/project-scheduling.ts`
   - 实现 9 种拖放场景
   - 导出到 `strategies/index.ts`

2. **测试拖放**
   - 从 Staging 拖到 Project
   - 从 Staging 拖到 Section
   - 从 Daily 拖到 Project
   - 项目内重排
   - Section 间移动

### 7.2 开发检查清单

#### 数据库检查

- [ ] initial_schema.sql 已修改
- [ ] projects 表字段正确
- [ ] project_sections 表已创建
- [ ] tasks 表添加 section_id
- [ ] 索引全部创建
- [ ] 外键约束正确

#### 后端检查

- [ ] Entities 定义完整
- [ ] Repositories 实现完整
- [ ] 9 个端点全部实现
- [ ] 所有端点有 CABC 文档
- [ ] 使用 success_response 包装
- [ ] 使用 acquire_write_permit
- [ ] 事务内写入 Event Outbox
- [ ] 路由已注册
- [ ] cargo check 通过
- [ ] cargo clippy 无警告

#### 前端检查

- [ ] 类型定义完整
- [ ] ISA 指令定义完整
- [ ] ISA 已注册
- [ ] Store 结构完整
- [ ] Mutations 使用 _mut 后缀
- [ ] 事件处理器正确
- [ ] ViewAdapter 扩展完成
- [ ] useViewTasks 扩展完成
- [ ] CircularProgress 组件完成
- [ ] ProjectListPanel 组件完成
- [ ] ProjectDetailPanel 组件完成
- [ ] ProjectsPanel 组件完成
- [ ] 路由集成完成
- [ ] 无 TypeScript 错误
- [ ] 无 ESLint 警告

#### 拖放检查

- [ ] 9 种策略全部实现
- [ ] 策略已导出
- [ ] 拖放功能正常
- [ ] 排序更新正确

#### 集成测试

- [ ] 创建项目功能正常
- [ ] 编辑项目功能正常
- [ ] 删除项目功能正常
- [ ] 创建章节功能正常
- [ ] 任务拖放到项目正常
- [ ] 任务拖放到章节正常
- [ ] 项目列表显示正确
- [ ] 项目详情显示正确
- [ ] 进度指示器显示正确
- [ ] SSE 实时更新正常
- [ ] 颜色继承正确

### 7.3 关键原则和最佳实践

#### 后端原则

1. **单一职责**：每个端点一个文件，一个职责
2. **事务边界**：所有写操作在事务内完成
3. **错误处理**：统一使用 AppResult 和 AppError
4. **事件一致性**：SSE 事件载荷与 HTTP 响应一致
5. **文档完整**：每个端点必须有完整的 CABC 文档

#### 前端原则

1. **声明式编程**：所有 API 调用通过指令集
2. **不可变性**：Store 状态使用 Map，创建新 Map 而非修改
3. **命名规范**：Mutations 必须以 `_mut` 结尾
4. **类型安全**：所有 Payload 和 DTO 有明确类型
5. **组件职责**：TaskList 处理任务交互，Panel 处理布局和状态

#### 拖放原则

1. **策略独立**：每个策略处理一种场景
2. **优先级排序**：更具体的策略优先级更高
3. **操作链**：使用 pipeline.dispatch 串联多个指令
4. **错误处理**：捕获错误并返回友好提示
5. **记录操作**：createOperationRecord 记录所有操作

---

## 附录：关键文件清单

### 后端文件

```
src-tauri/
├── migrations/
│   └── 20241001000000_initial_schema.sql    (修改)
├── src/
│   ├── entities/
│   │   ├── project.rs                       (新增)
│   │   └── project_section.rs               (新增)
│   ├── features/
│   │   ├── shared/
│   │   │   ├── project_repository.rs        (新增)
│   │   │   └── project_section_repository.rs (新增)
│   │   ├── endpoints/
│   │   │   └── projects/                    (新增目录)
│   │   │       ├── create_project.rs
│   │   │       ├── update_project.rs
│   │   │       ├── delete_project.rs
│   │   │       ├── list_projects.rs
│   │   │       ├── get_project.rs
│   │   │       ├── create_section.rs
│   │   │       ├── update_section.rs
│   │   │       ├── delete_section.rs
│   │   │       └── list_sections.rs
│   │   ├── projects.rs                      (新增)
│   │   └── mod.rs                           (修改)
```

### 前端文件

```
src/
├── types/
│   └── dtos.ts                              (修改)
├── cpu/
│   └── isa/
│       ├── project-isa.ts                   (新增)
│       └── index.ts                         (修改)
├── stores/
│   └── project/                             (新增目录)
│       ├── index.ts
│       ├── core.ts
│       ├── view-operations.ts
│       └── event-handlers.ts
├── services/
│   └── viewAdapter.ts                       (修改)
├── composables/
│   └── useViewTasks.ts                      (修改)
├── infra/
│   └── drag/
│       └── strategies/
│           ├── project-scheduling.ts        (新增)
│           └── index.ts                     (修改)
└── components/
    ├── parts/
    │   └── CircularProgress.vue             (新增)
    └── organisms/
        ├── ProjectListPanel.vue             (新增)
        ├── ProjectDetailPanel.vue           (新增)
        └── ProjectsPanel.vue                (新增)
```

### 修改文件列表

**后端**：
- 1 个迁移文件修改
- 2 个 Entity 文件新增
- 2 个 Repository 文件新增
- 9 个端点文件新增
- 2 个路由文件新增/修改

**前端**：
- 1 个类型文件修改
- 1 个 ISA 文件新增 + 1 个索引文件修改
- 4 个 Store 文件新增
- 2 个 Service 文件修改
- 1 个拖放策略文件新增
- 4 个组件文件新增

**总计**：约 26 个文件需要新增或修改

---

**文档结束**
