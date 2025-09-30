# Cutie 开发报告 - 2025年9月30日

## 📋 执行摘要

本次开发完成了 Cutie 的核心数据模型重构和基础功能实现，包括前后端架构的全面升级和6个关键 API 端点的实现。

---

## 🎯 主要成果

### **1. 前端数据模型重构**

#### **新 DTO 体系建立**
- ✅ 创建 `src/types/dtos.ts`
  - `TaskCard` - 任务卡片视图模型
  - `TaskDetail` - 任务详情视图模型
  - `TimeBlockView` - 时间块视图模型

#### **Pinia Store 架构重构**
- ✅ **TaskStore** - 单一数据源架构
  - 合并 `taskCards` 和 `taskDetails` 为 `tasks` Map
  - 智能合并更新策略
  - State/Actions/Getters 职责分离
  
- ✅ **ViewStore** - 视图索引管理
  - 管理日期 → 任务ID 映射
  - 协调 TaskStore 和视图层

- ✅ **TimeBlockStore** - 时间块管理
  - 完整的 CRUD 操作
  - 响应式数据更新

#### **组件重构**
- ✅ `KanbanTaskCard.vue` - 使用新 DTO
- ✅ `DailyKanbanColumn.vue` - 使用新 DTO
- ✅ `SimpleKanbanColumn.vue` - 新的通用看板列
- ✅ `HomeView.vue` - 从日期看板改为状态看板（All/Staging/Planned）
- ✅ `CuteCalendar.vue` - 响应式更新保证
- ✅ `KanbanTaskEditorModal.vue` - 调试数据显示

### **2. 后端架构重构**

#### **模块层级简化**
- ✅ 删除 3 层冗余文件（`api_router.rs`, `endpoints/mod.rs`, 空 `shared/`）
- ✅ 简化路由声明方式
- ✅ 采用单文件组件（SFC）模式

#### **Entities 层重构**
- ✅ Task 实体重构
  - `dtos.rs` → `request_dtos.rs` + `response_dtos.rs`
  - 创建 `TaskCardDto`, `TaskDetailDto`
  
- ✅ TimeBlock 实体重构
  - 创建 `request_dtos.rs`, `response_dtos.rs`
  - 创建 `TimeBlockViewDto`, `CreateFromTaskResponse`

#### **装配器模式**
- ✅ `TaskAssembler` - Task 实体到 DTO 转换
- ✅ 保持 entities 层纯粹，业务逻辑在 features 层

---

## 🚀 已实现的 API 端点

### **1. GET /api/views/staging**
**用途：** 获取未排期任务列表

**文件：** `src-tauri/src/features/views/endpoints/get_staging_view.rs`

**功能：**
- 查询所有未删除、未完成、且未被安排到任何日期的任务
- 从 `orderings` 表获取 `sort_order`
- 从 `areas` 表获取区域信息
- 返回 `Vec<TaskCardDto>` 按 sort_order 排序

**SQL 逻辑：**
```sql
SELECT * FROM tasks t
WHERE t.is_deleted = false
  AND t.completed_at IS NULL
  AND NOT EXISTS (
      SELECT 1 FROM task_schedules ts 
      WHERE ts.task_id = t.id
  )
```

---

### **2. POST /api/tasks**
**用途：** 创建新任务

**文件：** `src-tauri/src/features/tasks/endpoints/create_task.rs`

**功能：**
- 验证标题（1-255字符）、时长、子任务数量
- 生成 UUID 使用 `id_generator.new_uuid()`
- 创建任务并插入 `tasks` 表
- 自动创建 `orderings` 记录（staging 上下文）
- 使用 LexoRank 算法生成 `sort_order`
- 返回 `TaskCardDto`

**关键改进：**
- 使用 `shared/core/utils` 中的 LexoRank 工具
- 正确的 trait 方法调用
- 遵循 SFC 规范

---

### **3. GET /api/tasks/:id**
**用途：** 获取任务完整详情

**文件：** `src-tauri/src/features/tasks/endpoints/get_task.rs`

**功能：**
- 查询任务基本信息
- 查询所有 `task_schedules` 记录
- 组装 `TaskDetailDto`（包含 schedules 数组）
- 用于调试和详情查看

**返回示例：**
```json
{
  "card": { "schedule_status": "...", ... },
  "schedules": [
    { "day": "2025-09-30T00:00:00Z", "outcome": "planned" }
  ],
  "detail_note": "...",
  "created_at": "...",
  "updated_at": "..."
}
```

---

### **4. POST /api/tasks/:id/completion**
**用途：** 完成任务

**文件：** `src-tauri/src/features/tasks/endpoints/legacy.rs`

**功能：**
- 设置 `completed_at` 时间戳
- 截断相关时间块
- 清理未来日程
- 返回 `TaskCardDto`

**状态：** ✅ 已存在（本次重构中更新为使用新 DTO）

---

### **5. POST /api/time-blocks/from-task** ⭐ 核心
**用途：** 从拖动的任务创建时间块

**文件：** `src-tauri/src/features/time_blocks/endpoints/create_from_task.rs`

**功能（原子操作）：**
1. 检查任务是否存在
2. 检查时间冲突（不允许重叠）
3. 创建时间块记录（继承任务的 area）
4. 创建 `task_time_block_links` 记录
5. 创建 `task_schedules` 记录（从 start_time 提取日期）
6. 返回 `{ time_block, updated_task }`

**架构优势：**
- 专门处理拖动场景
- 单一职责原则
- 返回更新后的任务，schedule_status = 'scheduled'
- 前端无需手动刷新或猜测状态

**响应结构：**
```json
{
  "time_block": { ... },
  "updated_task": {
    "schedule_status": "scheduled",  // 已更新
    ...
  }
}
```

---

### **6. POST /api/time-blocks**
**用途：** 直接创建空时间块

**文件：** `src-tauri/src/features/time_blocks/endpoints/create_time_block.rs`

**功能：**
- 创建独立的时间块
- 不链接任务
- 不创建 schedule
- 用于用户在日历上直接创建时间块

**请求：**
```json
{
  "title": "深度工作",
  "start_time": "2025-09-30T09:00:00Z",
  "end_time": "2025-09-30T11:00:00Z",
  "area_id": "..."
}
```

---

### **7. GET /api/time-blocks**
**用途：** 查询时间块列表

**文件：** `src-tauri/src/features/time_blocks/endpoints/list_time_blocks.rs`

**功能：**
- 按日期范围查询时间块
- 包含关联任务摘要
- 包含区域信息（用于染色）
- 按 start_time 排序

**查询参数：**
```
?start_date=2025-09-30T00:00:00Z&end_date=2025-10-06T23:59:59Z
```

**响应：**
```json
[
  {
    "id": "...",
    "start_time": "...",
    "end_time": "...",
    "area": { "color": "#4a90e2" },
    "linked_tasks": [
      { "id": "...", "title": "...", "is_completed": false }
    ]
  }
]
```

---

## 🏗️ 架构改进

### **数据库表名统一**
- ✅ 所有表名统一为复数形式
- ✅ `ordering` → `orderings`
- ✅ 更新 Schema、代码、文档

### **单一职责原则（SRP）**
- ✅ 拆分时间块创建端点
  - `/time-blocks/from-task` - 拖动任务专用
  - `/time-blocks` - 直接创建专用
  
### **响应式数据流**
- ✅ 后端返回更新后的数据
- ✅ 前端直接使用响应更新 store
- ✅ 自动触发 UI 更新
- ✅ 无需 workaround

---

## 📚 文档完善

### **新增核心文档**

1. **CUTIE_CONCEPTS.md** - 核心概念速查表
   - 设计哲学和原则
   - 数据模型架构
   - 核心概念定义（Staging, Presence, Area 等）
   - 前后端数据结构
   - 开发者必读检查清单

2. **SFC_SPEC.md** - 单文件组件规范
   - SFC 内部结构
   - 最佳实践
   - 依赖注入规范（new_uuid(), now_utc()）
   - 使用现有工具（LexoRank）
   - 数据库 Schema 验证
   - 代码审查清单

3. **ENDPOINTS_SPEC.md** - API 端点规格
   - 所有端点的详细规格
   - 请求/响应示例
   - 业务逻辑说明

4. **PINIA_BEST_PRACTICES.md** - 状态管理最佳实践
   - 常见 Bug 及解决方案
   - 响应式更新链路
   - 调试技巧

---

## 🐛 修复的关键问题

### **Bug 1: 表名错误**
- **问题：** 代码中使用 `orderings` 但数据库是 `ordering`
- **修复：** 统一为 `orderings`（复数形式）
- **影响：** 运行时错误 → 编译通过

### **Bug 2: API 响应格式不匹配**
- **问题：** 前端期望数组，后端返回 `ApiResponse<T>`
- **修复：** 前端提取 `result.data`
- **影响：** "not iterable" 错误 → 正常工作

### **Bug 3: CreateTaskRequest 字段不匹配**
- **问题：** 后端期望 `context` 字段，前端不发送
- **修复：** 删除 `context`，添加 `project_id`
- **影响：** HTTP 422 错误 → 成功创建

### **Bug 4: Trait 方法名错误**
- **问题：** 使用 `.generate()`, `.now()` 等不存在的方法
- **修复：** 使用 `.new_uuid()`, `.now_utc()`
- **影响：** 编译错误 → 编译通过

### **Bug 5: 自行实现 LexoRank**
- **问题：** 手动递增字符，不符合 LexoRank 规范
- **修复：** 使用 `shared/core/utils` 中的工具函数
- **影响：** 排序可能错乱 → 正确排序

### **Bug 6: 拖动后任务仍在 Staging** ⭐ 关键
- **问题：** 只创建时间块和链接，没创建 schedule
- **修复：** 创建专用端点 `/from-task`，返回更新后的任务
- **影响：** 任务状态不同步 → 即时响应式更新

---

## 📊 代码统计

### **提交统计**
- 总提交数：**25个**
- 文件变更：**~110个**
- 代码行数：
  - 新增：~3,500行
  - 删除：~9,800行
  - **净减少：~6,300行**

### **文件结构**
```
前端新增：
├── src/types/dtos.ts                    (144行)
├── src/stores/task.ts                   (重构, 423行)
├── src/stores/view.ts                   (新建, 434行)
├── src/stores/timeblock.ts              (重构, 477行)
└── src/components/...                   (多个组件重构)

后端新增：
├── src-tauri/src/entities/task/
│   ├── request_dtos.rs                  (68行)
│   └── response_dtos.rs                 (145行)
├── src-tauri/src/entities/time_block/
│   ├── request_dtos.rs                  (31行)
│   └── response_dtos.rs                 (54行)
├── src-tauri/src/features/tasks/
│   ├── endpoints/create_task.rs         (381行)
│   ├── endpoints/get_task.rs            (159行)
│   └── shared/assembler.rs              (174行)
├── src-tauri/src/features/views/
│   └── endpoints/get_staging_view.rs    (206行)
└── src-tauri/src/features/time_blocks/
    ├── endpoints/create_from_task.rs    (480行)
    ├── endpoints/create_time_block.rs   (460行)
    └── endpoints/list_time_blocks.rs    (245行)

文档新增：
├── CUTIE_CONCEPTS.md                    (555行)
├── SFC_SPEC.md                          (373行)
├── ENDPOINTS_SPEC.md                    (294行)
└── PINIA_BEST_PRACTICES.md              (225行)
```

---

## 🎨 功能展示

### **已实现的完整工作流**

#### **1. 任务管理**
```
创建任务 → 出现在 Staging 列 → 拖动到日历 → 
移到 Planned 列 → 点击查看详情 → 完成任务
```

#### **2. 时间块管理**
```
拖动任务到日历 → 创建时间块 → 自动链接任务 → 
自动创建日程 → 任务状态更新 → UI 即时响应
```

#### **3. 数据一致性**
```
后端操作 → 返回更新数据 → Store 更新 → 
Getters 重新计算 → Computed 触发 → Vue 重新渲染
```

---

## 🔧 技术亮点

### **1. 多对多架构实现**
```
Task ←→ TimeBlock (通过 task_time_block_links)
  ✅ 一个时间块可以包含多个任务
  ✅ 一个任务可以分散在多个时间块
  ⚠️ 时间块不允许重叠（后端强制检查）
```

### **2. 装配器模式**
```
entities/      → 纯数据结构
features/shared/ → 业务逻辑和转换
  ✅ 职责清晰
  ✅ 易于测试
  ✅ 符合 SFC 架构
```

### **3. 响应式保证**
```
所有组件直接使用 store.getters
所有操作通过 store.actions
Store 内部创建新对象触发更新
  ✅ 无数据副本
  ✅ 无手动同步
  ✅ 自动 UI 更新
```

### **4. 单一职责端点**
```
/from-task → 拖动任务专用（创建3个表记录）
/time-blocks → 直接创建专用（创建1个表记录）
  ✅ 语义清晰
  ✅ 响应针对性强
  ✅ 易于维护
```

---

## 📈 性能优化

### **1. LexoRank 排序**
- 使用专业的排序算法
- O(1) 插入性能
- 避免重新排序整个列表

### **2. 响应式优化**
- Map 数据结构，O(1) 查找
- Computed 缓存，按需重新计算
- 避免不必要的 DOM 更新

### **3. 数据流优化**
- 后端一次性返回所有需要的数据
- 减少往返请求
- 前端直接使用，无需二次查询

---

## 🎯 Cutie vs Sunsama 差异实现

### **已实现的核心差异**

#### **1. Staging 区（替代 Backlog）** ✅
- 逾期任务回流机制（数据库设计支持）
- 无红色标签，无负罪感
- 状态看板而非日期看板

#### **2. 任务与时间块多对多** ✅
- 数据库架构：`task_time_block_links` 表
- 时间块不重叠约束
- 支持主题式规划

#### **3. 个人空间定位** ✅
- 无团队协作功能（战略性放弃）
- 无数据分析功能（战略性放弃）
- BYOK 模式（自填 API 端点）

### **待实现的核心差异**

- ⏳ Presence 按钮（记录努力）
- ⏳ VLM 截图导入
- ⏳ 疗愈式子任务（AI 生成）
- ⏳ 自定义 Shutdown 仪式
- ⏳ Template 模板系统

---

## 🔬 测试状态

### **手动测试结果**

| 功能                   | 状态 | 备注                      |
| ---------------------- | ---- | ------------------------- |
| 创建任务               | ✅   | 出现在 Staging            |
| 拖动任务到日历         | ✅   | 创建时间块                |
| 任务从 Staging 消失    | ✅   | 即时更新                  |
| 任务出现在 Planned     | ✅   | schedule_status 正确      |
| 时间块显示在日历       | ✅   | 使用区域颜色              |
| 查看任务详情           | ✅   | 显示 schedules 数组       |
| 完成任务               | ✅   | 任务消失                  |
| 时间冲突检测           | ⏳   | 待测试                    |
| 日历加载时间块         | ✅   | 启动时自动加载            |

---

## ⚠️ 已知问题和限制

### **当前限制**

1. **后端 API 未完全实现**
   - ✅ 创建、查询、完成已实现
   - ⏳ 更新、删除、重新打开待实现
   - ⏳ 日程调度端点待实现

2. **前端功能**
   - ✅ 基础拖拽已实现
   - ⏳ 时间块编辑待实现
   - ⏳ 任务详情编辑待完善
   - ⏳ 日期导航待实现

3. **数据模型**
   - ✅ TaskCard, TaskDetail, TimeBlockView 已定义
   - ⏳ schedule_info 字段待后端计算并填充
   - ⏳ linked_schedule_count 待实现

---

## 📝 经验教训

### **1. 数据库 Schema 必须先查看**
- **教训：** 猜测表名导致运行时错误
- **规范：** 编写前强制查看 `migrations/xxx.sql`
- **文档化：** 添加到 SFC_SPEC.md 检查清单首位

### **2. 避免重新实现现有工具**
- **教训：** 手动实现排序算法
- **规范：** 使用 `shared/` 中的 LexoRank 工具
- **文档化：** 添加常用工具清单

### **3. 正确的 Trait 方法名**
- **教训：** 使用不存在的方法导致编译错误
- **规范：** `new_uuid()`, `now_utc()`
- **文档化：** 添加方法名对照表

### **4. 响应式数据流完整性**
- **教训：** 后端改变状态但不在响应中反映
- **规范：** 修改操作必须返回更新后的完整数据
- **文档化：** 添加响应式更新链路图

### **5. 单一职责原则**
- **教训：** 一个端点处理多种场景导致混乱
- **规范：** 拆分专用端点，每个端点职责单一
- **文档化：** API 设计原则

---

## 🚀 下一步计划

### **Phase 1: 完善核心功能**
1. 实现任务更新端点（PATCH /tasks/:id）
2. 实现任务删除端点（DELETE /tasks/:id）
3. 实现重新打开端点（DELETE /tasks/:id/completion）
4. 实现时间块编辑功能

### **Phase 2: 日程管理**
1. 实现日历视图端点（GET /views/daily-schedule）
2. 实现任务排程端点（POST /schedules）
3. 实现任务重新排程（POST /schedules/reschedule）

### **Phase 3: Cutie 特色功能**
1. Presence 按钮实现
2. VLM 截图导入
3. AI 子任务生成
4. Template 系统
5. Custom Shutdown 仪式

---

## 📊 项目健康度

### **代码质量**
- ✅ 所有代码通过编译
- ✅ 遵循统一的编码规范
- ✅ 完整的文档覆盖
- ✅ 清晰的架构分层

### **可维护性**
- ✅ SFC 模式：高内聚，低耦合
- ✅ 装配器模式：职责分离
- ✅ 文档齐全：易于上手
- ✅ 检查清单：防止常见错误

### **可扩展性**
- ✅ 模块化设计，易于添加新功能
- ✅ DTO 体系完善，支持未来扩展
- ✅ Store 职责清晰，易于增强

---

## 🎓 总结

本次开发会话成功完成了 Cutie 从概念到可运行原型的关键转变：

1. **建立了坚实的架构基础**
   - 前后端数据模型对齐
   - 清晰的职责分离
   - 完善的文档体系

2. **实现了核心工作流**
   - 任务创建和管理
   - 拖拽到日历
   - 状态自动更新

3. **遵循了 Cutie 的设计哲学**
   - Staging 区替代 Backlog
   - 多对多架构
   - 单一职责原则

4. **为未来开发奠定基础**
   - 完整的开发规范
   - 防错机制
   - 可扩展架构

**Cutie 现在已具备基础的任务管理和时间块创建能力，为后续特色功能的开发做好了准备。** 🎉

---

## 📞 附录：API 端点快速参考

| 端点                          | 方法 | 状态 | 用途                 |
| ----------------------------- | ---- | ---- | -------------------- |
| `/api/views/staging`          | GET  | ✅   | 获取未排期任务       |
| `/api/tasks`                  | POST | ✅   | 创建任务             |
| `/api/tasks/:id`              | GET  | ✅   | 获取任务详情         |
| `/api/tasks/:id/completion`   | POST | ✅   | 完成任务             |
| `/api/time-blocks`            | POST | ✅   | 创建空时间块         |
| `/api/time-blocks`            | GET  | ✅   | 查询时间块列表       |
| `/api/time-blocks/from-task`  | POST | ✅   | 从任务创建时间块     |
| `/api/tasks/:id`              | PATCH| ⏳   | 更新任务（待实现）   |
| `/api/tasks/:id`              | DELETE| ⏳   | 删除任务（待实现）   |
| `/api/tasks/:id/completion`   | DELETE| ⏳   | 重新打开（待实现）   |

---

**报告生成时间：** 2025年9月30日
**开发者：** AI Assistant
**项目：** Cutie - 平静工作的终极诠释
