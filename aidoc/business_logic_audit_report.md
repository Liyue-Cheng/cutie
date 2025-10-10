# 业务逻辑审计报告：前端多次调用分析

## 1. 核心摘要

本报告旨在识别在前后端交互中，因后端缺少原子化的业务事件端点，而导致前端需要发起多次API调用来完成单一用户操作的场景。这些场景不仅增加了前端的复杂性，也可能引发数据不一致的风险。

## 2. 分析方法

通过审查前端的用户操作（尤其是在 `composables` 和 `stores` 中编排的业务逻辑），并与后端 `features` 目录中提供的端点进行比对，找出“一个用户动作，多次后端请求”的模式。

## 3. 已识别的问题场景

本节将详细列出所有发现的前端多次调用问题。

### 3.1 在每日看板中创建任务

- **用户操作**: 在 `InfiniteDailyKanban` 的某一日期列中，输入任务标题并按回车。
- **预期业务逻辑**: 创建一个新任务，并将其安排到指定的日期。这是一个单一的业务事件：“创建一个已排期的任务”。
- **前端当前实现**: (`HomeView.vue` 的 `handleAddTask` 函数)
  1. 调用 `taskStore.createTask({ title })` 创建一个任务。 (API 调用 1: `POST /api/tasks`)
  2. 任务创建成功后，获取返回的新任务 `id`。
  3. 调用 `taskStore.addSchedule(newTask.id, date)` 为该任务添加日程。 (API 调用 2: `POST /api/tasks/{id}/schedules`)

- **问题分析**:
  - **多次网络往返**: 完成一个用户操作需要两次独立的 HTTP 请求，增加了延迟和失败的可能性。
  - **非原子性**: 如果第一次 API 调用成功（任务已创建），但第二次调用失败（日程未添加），则会导致数据不一致。任务被创建了，但没有如预期地出现在看板上，而是留在了 Staging 区。
  - **前端逻辑复杂**: 前端需要编排这两个连续的调用，并处理其中可能出现的失败情况。

- **后端缺失的端点**:
  后端缺少一个能够原子化地“创建并排期任务”的端点。

- **建议**:
  - **后端**: 创建一个新的端点，例如 `POST /api/tasks/with-schedule`，或者在现有的 `POST /api/tasks` 端点上增加一个可选的 `scheduled_day` 字段。
  - **前端**: 将 `handleAddTask` 的逻辑修改为只调用这一个新端点。

**相关代码片段:**

```typescript
// src/views/HomeView.vue:71
async function handleAddTask(title: string, date: string) {
  // ...
  try {
    // 1. 创建任务
    const newTask = await taskStore.createTask({ title })
    if (!newTask) {
      /* ... */ return
    }

    // 2. 立即为任务添加日程
    const updatedTask = await taskStore.addSchedule(newTask.id, date)
    if (!updatedTask) {
      /* ... */ return
    }
  } catch (error) {
    // ...
  }
}
```

### 3.2 从模板拖拽到日期看板创建任务

- **用户操作**: 从右侧的模板列表，将一个模板拖拽到主界面的每日看板上。
- **预期业务逻辑**: 这是一个单一的业务事件：“根据模板创建一个新任务，并将其安排到目标日期”。
- **前端当前实现**: (`composables/drag/useTemplateDrop.ts` 的 `handleTemplateDrop` 函数)
  1. 调用 `templateStore.createTaskFromTemplate(templateId)` 创建一个基于模板的任务。 (API 调用 1: `POST /api/templates/{id}/create-task`)
  2. 任务创建成功后，获取返回的新任务 `id`。
  3. 调用 `taskStore.addSchedule(newTask.id, targetDate)` 为该任务添加日程。 (API 调用 2: `POST /api/tasks/{id}/schedules`)

- **问题分析**:
  - 与上一个问题类似，这个操作也被拆分成了两个独立的 API 调用，存在**非原子性**和**多次网络往返**的问题。
  - 如果 `addSchedule` 失败，用户会看到一个已创建但未排期的任务出现在 Staging 区，这与用户的拖放操作意图不符。

- **后端缺失的端点**:
  后端缺少一个能够原子化地“从模板创建并排期任务”的端点。

- **建议**:
  - **后端**: 增强 `POST /api/templates/{id}/create-task` 端点，使其可以接受一个可选的 `scheduled_day` 参数。在后端服务中，如果提供了此参数，就在同一个事务中完成任务创建和日程添加。
  - **前端**: 修改 `handleTemplateDrop` 逻辑，只调用一次增强后的 `createTaskFromTemplate` 端点，并传递目标日期。

**相关代码片段:**

```typescript
// src/composables/drag/useTemplateDrop.ts:89
// 1. 从模板创建任务
const newTask = await templateStore.createTaskFromTemplate(dragData.templateId)

// ...

// 2. 添加日程到目标日期
await taskStore.addSchedule(newTask.id, targetDate)
```

### 3.3 创建循环规则

- **用户操作**: 在任务编辑器中，配置好循环规则并点击“确定”。
- **预期业务逻辑**: 这是一个单一的业务事件：“为某个任务创建一个循环规则”。这个过程应该包括：
  1. 创建一个与任务内容相同的**循环模板**。
  2. 创建一个**循环规则**，并将其与新创建的模板关联。
  3. （可选）如果任务已排期，将该任务作为循环的第一个实例链接到新规则。
- **前端当前实现**: (`components/parts/recurrence/RecurrenceConfigDialog.vue` 的 `handleSave` 函数)
  1. 调用 `templateStore.createTemplate(...)` 创建一个循环模板。 (API 调用 1: `POST /api/templates`)
  2. 模板创建成功后，获取返回的新模板 `id`。
  3. 调用 `recurrenceStore.createRecurrence(...)` 创建循环规则，并传入模板 `id` 和源任务 `id`。 (API 调用 2: `POST /api/recurrences`)

- **问题分析**:
  - **非原子性**: 这个操作同样被拆分成了两个独立的 API 调用。如果 `createTemplate` 成功但 `createRecurrence` 失败，系统中会产生一个无用的“孤儿”模板。
  - **前端逻辑复杂**: 前端需要手动编排这两个连续的调用，并处理中间状态。
  - **后端职责不清**: 后端 `POST /api/recurrences` 端点接收一个 `template_id`，这意味着它假设模板已存在。它还接收一个 `source_task_id`，这暗示了它需要处理将现有任务转换为第一个实例的逻辑，但这个逻辑与模板创建是分离的。

- **后端缺失的端点**:
  后端缺少一个能够原子化地“从一个现有任务创建循环规则”的端点。

- **建议**:
  - **后端**: 创建一个新的业务事件端点，例如 `POST /api/tasks/{id}/make-recurring`。这个端点将接收循环规则的配置（RRULE 字符串等），并在一个事务中完成以下所有操作：
    1. 从指定的 `task_id` 创建循环模板。
    2. 创建循环规则并关联到新模板。
    3. 将原任务链接为第一个实例。
  - **前端**: 将 `handleSave` 的逻辑修改为只调用这一个新端点。

**相关代码片段:**

```typescript
// src/components/parts/recurrence/RecurrenceConfigDialog.vue:86
async function handleSave() {
  try {
    // 1. 创建循环模板
    const template = await templateStore.createTemplate({
      /* ... */
    })

    // 2. 创建循环规则
    await recurrenceStore.createRecurrence({
      template_id: template.id,
      rule: ruleString.value,
      source_task_id: props.task.id, // 传入原任务ID
      // ...
    })

    // ...
  } catch (error) {
    // ...
  }
}
```
