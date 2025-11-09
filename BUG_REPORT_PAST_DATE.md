# Bug Report: 过去看板任务显示不合理

## 问题描述

过去看板（Past Date Kanban）显示的任务状态不正确。

## 预期行为 vs 实际行为

### 测试场景

假设有以下任务历史：

**任务A：**

- 10月1日：排期（outcome = PLANNED）
- 10月2日：排期 + 工作（outcome = PRESENCE_LOGGED）
- 10月3日：排期 + 完成（outcome = COMPLETED_ON_DAY，completed_at = 2024-10-03）

### 预期行为

查看过去看板时，应该根据**该天的outcome**显示任务状态：

| 日期    | 显示任务A | 状态显示 | 原因                                        |
| ------- | --------- | -------- | ------------------------------------------- |
| 10月1日 | ✅ 显示   | 未完成   | outcome = PLANNED（只是计划了）             |
| 10月2日 | ✅ 显示   | 未完成   | outcome = PRESENCE_LOGGED（工作了但未完成） |
| 10月3日 | ✅ 显示   | 已完成   | outcome = COMPLETED_ON_DAY（在该天完成）    |

### 实际行为（Bug）

查看过去看板时，根据任务的**全局completed_at**状态显示：

| 日期    | 显示任务A | 状态显示      | 错误原因                                                    |
| ------- | --------- | ------------- | ----------------------------------------------------------- |
| 10月1日 | ✅ 显示   | **已完成** ❌ | 使用了task.completed_at（10月3日完成），而不是该天的outcome |
| 10月2日 | ✅ 显示   | **已完成** ❌ | 同上                                                        |
| 10月3日 | ✅ 显示   | 已完成 ✅     | 正确（outcome = COMPLETED_ON_DAY）                          |

## 根本原因

### 后端问题

**文件：** `src-tauri/src/features/endpoints/views/get_daily_tasks.rs`

**问题SQL（第238-250行）：**

```rust
SELECT DISTINCT t.id, t.title, ..., t.completed_at, ...
FROM tasks t
INNER JOIN task_schedules ts ON ts.task_id = t.id
WHERE ts.scheduled_date = ?
  AND t.deleted_at IS NULL
  AND t.archived_at IS NULL
ORDER BY t.created_at DESC
```

**问题：**

- 只查询了 `task.completed_at`（任务的全局完成状态）
- 没有查询 `task_schedules.outcome`（该天的具体结果）
- 无法区分"该天完成"和"其他天完成"

### 前端显示问题

**文件：** `src/types/dtos.ts` - `TaskCard` 接口

```typescript
export interface TaskCard {
  is_completed: boolean // ❌ 这是全局状态，不是该天的状态
  // ...
  schedules: Array<{
    scheduled_day: string
    outcome: DailyOutcome // ✅ 这才是该天的真实状态
    // ...
  }> | null
}
```

**问题：**

- 前端使用 `task.is_completed` 来显示任务状态
- 应该使用 `task.schedules[当前日期].outcome === 'COMPLETED_ON_DAY'`

## 解决方案

### 方案1：修改后端API（推荐）

修改 `get_daily_tasks` 端点，返回"该天视角"的任务状态：

```rust
// 新增字段到响应
pub struct TaskCardForDate {
    // ... 现有字段

    // 新增：该天的outcome
    outcome_for_date: DailyOutcome,  // PLANNED | PRESENCE_LOGGED | COMPLETED_ON_DAY | CARRIED_OVER

    // 或者重新定义 is_completed 为该天的完成状态
    is_completed: bool,  // = (outcome_for_date == COMPLETED_ON_DAY)
}
```

**SQL修改：**

```sql
SELECT
    DISTINCT t.id, t.title, ...,
    ts.outcome,  -- ⭐ 新增：查询该天的outcome
    t.completed_at  -- 保留全局状态用于参考
FROM tasks t
INNER JOIN task_schedules ts ON ts.task_id = t.id
WHERE ts.scheduled_date = ?
  AND t.deleted_at IS NULL
  AND t.archived_at IS NULL
ORDER BY t.created_at DESC
```

### 方案2：修改前端逻辑

前端在显示过去看板时，使用schedule的outcome判断：

```typescript
// src/components/parts/kanban/KanbanTaskCard.vue

const isCompletedOnThisDate = computed(() => {
  if (!isDateKanban.value || !props.viewMetadata?.config) {
    return props.task.is_completed // 非日期看板，使用全局状态
  }

  const config = props.viewMetadata.config as DateViewConfig
  const kanbanDate = config.date

  // 查找该天的schedule
  const scheduleForDate = props.task.schedules?.find((s) => s.scheduled_day === kanbanDate)

  // 该天完成 = outcome是COMPLETED_ON_DAY
  return scheduleForDate?.outcome === 'COMPLETED_ON_DAY'
})
```

## 影响范围

### 受影响的功能

1. ✅ **过去看板** - 严重影响，无法正确回顾历史
2. ❌ **今日看板** - 不受影响（今日完成的任务outcome正确）
3. ❌ **未来看板** - 不受影响（未来任务不会完成）
4. ❌ **暂存区** - 不受影响（未排期任务）

### 用户体验影响

**致命问题：**

- 用户无法正确回顾过去的工作记录
- 过去的"计划但未完成"的任务显示为"已完成"
- 无法准确统计每天的完成率

## 测试用例

### 测试1：多天排期，最后一天完成

```
任务：写文档
- 10月1日：排期（PLANNED）
- 10月2日：排期（PLANNED）
- 10月3日：完成（COMPLETED_ON_DAY）

查看10月1日看板：
- 预期：显示"未完成"
- 实际：显示"已完成" ❌
```

### 测试2：跨周任务

```
任务：大型重构
- 10月1日：PRESENCE_LOGGED（工作了）
- 10月2日：PRESENCE_LOGGED（工作了）
- 10月3日：PRESENCE_LOGGED（工作了）
- 10月4日：COMPLETED_ON_DAY（完成）

查看10月1-3日看板：
- 预期：都显示"未完成"（只是在场工作）
- 实际：都显示"已完成" ❌
```

## 优先级

**P0 - 致命Bug**

- 影响核心功能（历史回顾）
- 用户体验严重受损
- 数据展示完全错误

## 备注

这个bug说明了一个重要的设计问题：

- 任务有"全局状态"（completed_at）
- 任务在某天有"局部状态"（outcome）
- 在日期看板中，应该优先显示局部状态

这也解释了为什么`task_schedules`表要设计`outcome`字段！






