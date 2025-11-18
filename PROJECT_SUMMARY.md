# Project 功能实现总结

> 基于 PROJECT_IMPLEMENTATION_PLAN.md 的关键决策汇总

## ✅ 已确定的核心决策

### 1. 数据库设计

**Projects 表**：
- 状态仅保留：`ACTIVE` 和 `COMPLETED`（移除 PAUSED 和 ARCHIVED）
- 新增统计字段：`total_tasks` 和 `completed_tasks`（后端维护）
- 不存储 color（从 Area 继承）
- 不存储 sort_order（使用 view_preferences）

**ProjectSections 表**：
- 使用独立表
- 排序使用 `sort_order` 字段（Lexorank 算法）
- **不使用 view_preferences**

**删除策略**：
- 软删除项目 → 同时软删除所有关联的 sections 和 tasks
- 软删除 section → 任务的 section_id 设为 NULL（保留 project_id）

### 2. 后端端点

**新增端点**：
- `POST /projects/:id/complete-all` - 批量完成项目所有任务
- 查询项目下所有未完成任务并批量完成
- 更新项目状态为 COMPLETED

**项目完成逻辑**：
- CircularProgress 点击 → 确认对话框 → 调用 complete-all 端点
- 重新打开项目时，**不处理任务的完成状态**（已完成的保持已完成）

**统计信息维护**：
- 每次任务的 `project_id` 或 `is_completed` 变化时
- 调用 `update_statistics()` 更新项目的 total_tasks 和 completed_tasks

### 3. 前端架构

**布局结构**：
```
ProjectsPanel
  ├── 左侧 50% - TwoRowLayout (垂直)
  │     ├── 上栏 (空)
  │     └── 下栏 (水平 30%/70%)
  │           ├── ProjectListPanel (项目列表)
  │           └── ProjectDetailPanel (项目详情 + TaskList)
  └── 右侧 50% - TwoRowLayout (垂直)
        ├── 上栏 - Dummy 内容
        └── 下栏 - DoubleRowTimeline (时间线)
```

**颜色继承**：
- 前端实时查询 areaStore 获取颜色（不在后端组装）

**Section 排序**：
- 使用 `sort_order` 字段（Lexorank）
- 不使用 view_preferences

**项目列表排序**：
- 使用 view_preferences
- ViewKey: `misc::projects`
- 存储项目 ID 数组（复用 sorted_task_ids 字段）

### 4. View Context 规范

**新增 ViewKey**：
- `misc::projects` - 项目列表排序
- `project::{project_id}` - 项目所有任务（统计用）
- `project::{project_id}::section::all` - 无 section 任务
- `project::{project_id}::section::{section_id}` - 特定 section 任务

**ViewContext 类型**：
```typescript
| { type: 'project'; projectId: string }
| { type: 'project_section'; projectId: string; sectionId: string }
```

### 5. 拖放策略

**支持的 8 种场景**：
1. Project → Daily（创建日程，保留项目归属）
2. Section → Daily（创建日程，保留项目+章节归属）
3. Daily → Project（设置项目归属，保留日程）
4. Daily → Section（设置项目+章节归属，保留日程）
5. Project → Project（同项目内重排）
6. Project → Section（移动到章节）
7. Section → Section（跨章节移动）
8. Section → Project（移回项目，清除 section_id）

**不支持**：
- Staging ↔ Project/Section（设置项目归属应通过任务编辑器）

### 6. 对话框组件

**需要实现的 5 个对话框**：
1. CreateProjectDialog
2. EditProjectDialog（含删除按钮）
3. CreateSectionDialog
4. EditSectionDialog（含删除按钮）
5. ConfirmCompleteProjectDialog

### 7. 验证规则

**项目**：
- name: 必填，1-200 字符
- description: 可选，0-2000 字符
- due_date: 可选，YYYY-MM-DD，>= 今天
- area_id: 可选，有效 UUID

**Section**：
- title: 必填，1-200 字符
- description: 可选，0-2000 字符

### 8. 时间线集成

**DoubleRowTimeline**：
- 使用默认行为（显示所有任务）
- 不需要特殊筛选逻辑
- 支持完整的拖放操作

## 📋 实施清单

**阶段 1: 数据库和后端**（17 个文件）
- [ ] 修改 initial_schema.sql
- [ ] 创建 entities/project.rs
- [ ] 创建 entities/project_section.rs
- [ ] 创建 shared/project_repository.rs
- [ ] 创建 shared/project_section_repository.rs
- [ ] 创建 10 个 endpoint 文件（5 项目 + 4 章节 + 1 完成）
- [ ] 注册路由

**阶段 2: 前端基础**（10+ 个文件）
- [ ] 更新 types/dtos.ts
- [ ] 创建 cpu/isa/project-isa.ts（8 个指令）
- [ ] 创建 stores/project/（4 个文件）
- [ ] 扩展 services/viewAdapter.ts
- [ ] 扩展 composables/useViewTasks.ts

**阶段 3: UI 组件**（5 个组件 + 5 个对话框）
- [ ] CircularProgress.vue
- [ ] ProjectListPanel.vue
- [ ] ProjectDetailPanel.vue
- [ ] ProjectsPanel.vue
- [ ] 5 个对话框组件
- [ ] 路由集成

**阶段 4: 拖放策略**（8 个场景）
- [ ] infra/drag/strategies/project-scheduling.ts

## 🎯 关键技术决策依据

1. **Section 用 sort_order**：Section 不会跨项目移动，固定排序足够
2. **后端维护统计**：避免前端频繁计算，减少性能开销
3. **前端查询颜色**：避免后端每次组装时查询 Area，简化逻辑
4. **软删除策略**：保留数据用于审计，支持未来的恢复功能
5. **状态简化**：ACTIVE 和 COMPLETED 足够表达项目生命周期
6. **重新打开不处理任务**：保留历史完成状态，避免意外数据变化

## 📝 待补充细节

**循环任务集成**（优先级 P2）：
- 模板是否支持 project_id？
- 实例化时是否继承？
- 实例修改是否影响模板？

**建议**：模板支持 project_id，实例继承，修改不同步（除非使用批量更新端点）

---

**文档版本**: V1.0
**最后更新**: 2025-11-17
**状态**: 设计完成，等待实施
