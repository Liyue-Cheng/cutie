# 日历显示修复 - 显示所有日期的任务

## 问题描述

日历在月视图下无法显示过去日期的任务，只能显示未来的任务。

## 根本原因

`useCalendarEvents.ts` 中使用了 `taskStore.plannedTasks` 来获取要显示的任务：

```typescript
// ❌ 错误：只包含未来的任务
taskStore.plannedTasks.forEach((task) => {
  // ...
})
```

而 `plannedTasks` 的定义是：

```typescript
const plannedTasks = computed(() => {
  const today = new Date().toISOString().split('T')[0]!
  return allTasksArray.value.filter((task) => {
    // ...
    // 🔥 只包含有未来或今天日程的任务
    const hasFutureOrTodaySchedule =
      task.schedules?.some((schedule) => schedule.scheduled_day >= today) ?? false
    return hasFutureOrTodaySchedule
  })
})
```

这导致：

- ✅ 今天和未来的任务可以显示
- ❌ 过去的任务无法显示

## 解决方案

使用 `taskStore.allTasks` 并过滤出有日程的任务：

```typescript
// ✅ 正确：包含所有有日程的任务（包括过去的日期）
taskStore.allTasks.forEach((task) => {
  // 跳过已完成、已删除、没有日程的任务
  if (task.is_completed || task.is_deleted || !task.schedules || task.schedules.length === 0) return

  // 遍历任务的所有日程
  task.schedules?.forEach((schedule) => {
    // 为每个日期创建日历事件
    events.push({
      // ...
      start: schedule.scheduled_day,
      // ...
    })
  })
})
```

## 修改文件

- `src/composables/calendar/useCalendarEvents.ts`
  - 第 148-151 行：从 `taskStore.plannedTasks` 改为 `taskStore.allTasks`
  - 添加过滤条件：`!task.schedules || task.schedules.length === 0`

## 测试要点

### 1. 过去日期的任务

- [ ] 创建一个任务并排期到过去的日期（如昨天）
- [ ] 切换到月视图，应该能看到该任务
- [ ] 点击该日期格子，应该能看到任务详情

### 2. 今天的任务

- [ ] 创建一个任务并排期到今天
- [ ] 应该能在日历上看到

### 3. 未来的任务

- [ ] 创建一个任务并排期到未来（如明天）
- [ ] 应该能在日历上看到

### 4. 跨月任务

- [ ] 创建任务并排期到上个月、本月、下个月
- [ ] 切换月份，每个月都应该显示对应的任务

### 5. 筛选功能

- [ ] 取消勾选"已排期任务"，过去和未来的任务都应该隐藏
- [ ] 重新勾选，应该重新显示

## 相关概念

### `plannedTasks` vs `allTasks`

| 属性           | 范围                           | 用途                     |
| -------------- | ------------------------------ | ------------------------ |
| `plannedTasks` | 有未来或今天日程的未完成任务   | Staging/Planned 看板视图 |
| `allTasks`     | 所有任务（包括已完成、已删除） | 需要过滤使用             |
| `stagingTasks` | 没有未来或今天日程的未完成任务 | Staging 区域             |

### 日历显示逻辑

日历需要显示**特定日期**的任务，而不是"未来"的任务：

```
日历视图：按日期显示
├── 2024-01-01: 任务A, 任务B
├── 2024-01-02: 任务C
├── ...
└── 2024-01-31: 任务Z

看板视图：按状态分组
├── Staging: 没有未来日程的任务
└── Planned: 有未来日程的任务
```

## 提交信息

```
fix: 日历月视图显示所有日期的任务，包括过去的日期

- 从 taskStore.plannedTasks 改为 taskStore.allTasks
- plannedTasks 只包含未来的任务，不适合日历显示
- 日历需要显示特定日期的所有任务，包括过去的日期

Fixes: 日历无法显示过去日期的任务
```
