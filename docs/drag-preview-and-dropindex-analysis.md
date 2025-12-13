# 模板拖拽预览 & DropIndex 计算问题分析

## 一、模板拖拽预览功能实现

### 1.1 问题背景

将模板拖动到日程/任务列表时，没有实现预览元素。原因是模板和任务是**不同类型**，现有的 `useInteractDrag` 预览逻辑假设拖入的对象和列表项是同类型。

### 1.2 解决方案：预览转换器

在 `useInteractDrag` 中增加 `previewTransformer` 参数，将拖动的 Template 转换为临时的 TaskCard 预览。

### 1.3 修改的文件

#### `src/composables/drag/useInteractDrag.ts`

新增内容：
- `PreviewTransformer<T>` 类型定义
- `previewTransformer` 可选参数
- 跨类型预览处理逻辑

```typescript
/**
 * 预览转换器函数类型
 */
export type PreviewTransformer<T> = (draggedObject: unknown, objectType: DragObjectType) => T | null

export interface UseInteractDragOptions<T = DragObject> {
  // ... 其他参数

  /**
   * 预览转换器（可选）
   * 当拖动的对象类型与本列表的 objectType 不同时，
   * 使用此函数将拖动对象转换为可在本列表中显示的预览对象。
   */
  previewTransformer?: PreviewTransformer<T>
}
```

跨类型预览处理逻辑（在 `displayItems` computed 中）：

```typescript
// 当拖动对象类型与本列表类型不同时，尝试使用预览转换器
if (previewObjectType !== objectType) {
  if (!previewTransformer || targetZoneId !== currentViewId) {
    return currentItems
  }

  const transformedPreview = previewTransformer(draggedObject, previewObjectType)
  if (!transformedPreview) {
    return currentItems
  }

  // 在目标位置插入转换后的预览对象
  if (dropIndex !== undefined) {
    const previewList = [...currentItems]
    const safeIndex = Math.max(0, Math.min(dropIndex, previewList.length))
    previewList.splice(safeIndex, 0, {
      ...transformedPreview,
      _isPreview: true,
      _isTransformedPreview: true,
      _dragCompact: isCompact,
    })
    return previewList
  }
  return currentItems
}
```

#### `src/components/assembles/tasks/list/TaskList.vue`

新增内容：
- 导入 `Template`, `DragObjectType` 类型
- `templateToTaskPreview` 转换函数
- `transitionEnabled` 状态控制动画
- 在 `onDrop` 中处理跨类型拖放

```typescript
const templateToTaskPreview = (draggedObject: unknown, objectType: DragObjectType): TaskCard | null => {
  if (objectType === 'template') {
    const template = draggedObject as Template
    return {
      id: `preview-${template.id}`,
      title: template.title,
      glance_note: template.glance_note_template,
      is_completed: false,
      is_archived: false,
      is_deleted: false,
      deleted_at: null,
      subtasks: template.subtasks_template,
      estimated_duration: template.estimated_duration_template,
      area_id: template.area_id,
      project_id: null,
      section_id: null,
      schedule_info: null,
      due_date: null,
      schedules: null,
      has_detail_note: !!template.detail_note_template,
      recurrence_id: null,
      recurrence_original_date: null,
      recurrence_expiry_behavior: null,
    }
  }
  return null
}
```

#### `src/infra/drag/strategies/strategy-utils.ts`

修改 `extractObjectIds` 函数，过滤掉预览元素 ID：

```typescript
export function extractObjectIds(context: Record<string, any>): string[] {
  let ids: string[] = []
  // ... 提取逻辑

  // 过滤掉预览元素（ID 以 "preview-" 开头的）
  return ids.filter((id) => !id.startsWith('preview-'))
}
```

### 1.4 遇到的问题及解决

#### 问题 1：排序位置包含预览元素 ID

**现象**：`sort_position.next_task_id` 使用了 `preview-xxx`，导致后端 422 错误。

**原因**：`targetContext.displayTasks` 包含了预览元素。

**解决**：在 `extractObjectIds` 中过滤掉 `preview-` 开头的 ID。

#### 问题 2：预览任务消失时有动画闪烁

**现象**：模板 drop 后，旧预览任务被移除、新任务加入时播放动画。

**原因**：
- 任务拖动：预览 ID 和真实 ID **相同**，Vue 认为是同一元素
- 模板拖动：预览 ID (`preview-xxx`) 和真实 ID (新 UUID) **不同**，Vue 触发 leave + enter 动画

**解决**：在跨类型 drop 时禁用 TransitionGroup 动画：

```vue
<TransitionGroup :name="transitionEnabled ? 'task-list' : ''" ...>
```

```typescript
onDrop: async (session) => {
  const isTransformedDrop = session.object?.type !== 'task'

  if (isTransformedDrop) {
    transitionEnabled.value = false
  }

  // ... 执行策略

  if (isTransformedDrop) {
    setTimeout(() => {
      transitionEnabled.value = true
    }, 100)
  }
}
```

#### 问题 3：预览元素和真实任务同时存在（闪烁）

**现象**：SSE 推送新任务时，预览状态还未清除，导致瞬间显示两个任务。

**原因**：清除预览状态的时机晚于 SSE 推送。

**解决**：在 drop 开始时立即清除预览状态：

```typescript
if (isTransformedDrop) {
  transitionEnabled.value = false
  dragPreviewActions.clear()  // 立即清除
}
```

#### 问题 4：清除预览后排序信息丢失

**现象**：任务出现在末尾，排序信息丢失。

**原因**：清除预览后，`dragPreviewState.value?.computed.dropIndex` 变成 `undefined`。

**解决**：在清除预览前保存所有需要的信息：

```typescript
// 在清除预览前保存
const savedDropIndex = dragPreviewState.value?.computed.dropIndex
const savedTaskIds = displayItems.value.map((t) => t.id)
const savedDisplayTasks = [...displayItems.value]

if (isTransformedDrop) {
  dragPreviewActions.clear()
}

// 使用保存的值
const result = await dragStrategy.executeDrop(session, props.viewKey, {
  targetContext: {
    taskIds: savedTaskIds,
    displayTasks: savedDisplayTasks,
    dropIndex: savedDropIndex,
    viewKey: props.viewKey,
  },
})
```

---

## 二、DropIndex 计算问题分析

### 2.1 现有两套算法

| 函数 | 使用场景 | 设计目的 |
|------|----------|----------|
| `calculateDropIndexForZone` | 进入新区域时 | 一次性精确定位 |
| `calculateDropIndexWithDirectionalGate` | 在当前区域内移动时 | 方向感知步进，追求稳定性 |

### 2.2 `calculateDropIndex` 算法（utils.ts）

**有历史位置时**：使用邻居触发区逐步移动

```
  ┌─────────────────────┐
  │      Task 0         │ ← 底部 10% 是"上移触发区"
  ├─────────────────────┤
  │      Task 1         │ ← 当前 dropIndex = 1
  ├─────────────────────┤
  │      Task 2         │ ← 顶部 10% 是"下移触发区"
  └─────────────────────┘
```

- 上移：鼠标进入上一项的底部 10%（至少 8px）→ `dropIndex - 1`
- 下移：鼠标进入下一项的顶部 10%（至少 8px）→ `dropIndex + 1`
- 支持循环步进（可跨多格）

**无历史位置时**：按 bottom 定位

```typescript
for (let i = 0; i < wrappers.length; i++) {
  if (mouseY <= wrappers[i].getBoundingClientRect().bottom) {
    return i
  }
}
return wrappers.length
```

### 2.3 `calculateDropIndexWithDirectionalGate` 算法（drag-controller.ts）

**设计目的**：防抖/稳定

```
场景：鼠标在分界线附近小幅抖动

精确算法：
  - 鼠标上移 1px → dropIndex = 2
  - 鼠标下移 1px → dropIndex = 3
  → 预览元素疯狂跳动！

方向门控算法：
  - 只有明确"进入触发区"才步进
  - 小幅抖动不触发
  → 稳定
```

**实现方式**：检测"从触发区外进入触发区"的瞬间

```typescript
const wasOutside = this.lastMouseY > enterThreshold  // 上一帧在边界下方
const nowInside = pointerY <= enterThreshold         // 当前帧在边界上方

if (wasOutside && nowInside) {
  return lastIndex - 1  // 只步进一格
}
```

### 2.4 方向门控的必要性

**解决高度差异导致的抖动问题**：

```
交换前:                    交换后:
┌─────────────────┐       ┌─────────────────────────┐
│    Task A (50px)│       │                         │
├─────────────────┤       │      Task B (200px)     │
│                 │       │                         │
│   Task B (200px)│ ←鼠标  ├─────────────────────────┤
│                 │       │    Task A (50px)        │ ←鼠标还在这！
└─────────────────┘       └─────────────────────────┘

鼠标位置没变，但元素高度变了
→ 用位置计算会得到"需要交换回去"
→ 无限抖动！
```

方向门控的意义：只有鼠标明确向某方向移动并进入触发区，才步进。交换后即使鼠标落在"错误"位置，只要不继续移动，就不会触发反向交换。

### 2.5 当前算法的缺陷

**问题**：方向门控算法只步进一格，快速移动时会失效。

**场景**：dropIndex = 3，鼠标从 y=350 快速跳到 y=150

```
触发区是 Task 2 的底部 10%：[290, 300]

检测：
- wasOutside = 350 > 290 = true
- nowInside = 150 <= 290 = true
- 触发！但只返回 dropIndex = 2

正确答案应该是 dropIndex = 1（鼠标在 Task 1 内）
```

**更严重的情况**：如果鼠标恰好停在触发区内（如 y=295），下一帧又跳到别处：

```
触发区 [290, 300]

帧1: lastMouseY=350, pointerY=295
- wasOutside = 350 > 290 = true
- nowInside = 295 <= 290 = false  ← 在触发区内，没有穿过边界！
- 不触发

帧2: lastMouseY=295, pointerY=150
- wasOutside = 295 > 290 = true
- nowInside = 150 <= 290 = true
- 触发！dropIndex = 2

但鼠标实际在 Task 1，正确应该是 dropIndex = 1
```

### 2.6 两个问题的冲突

| 问题 | 解决方案 | 副作用 |
|------|----------|--------|
| 高度差异抖动 | 方向门控（只步进一次） | 快速移动只能步进一格 |
| 快速移动跨多格 | 位置计算（循环步进） | 高度差异时会抖动 |

### 2.7 解决方案：方向限制 + 循环步进 ✅ 已实现

**核心思想**：结合两种算法的优点

```typescript
// 向下移动：循环检测，dropIndex 只能增大
if (deltaY > 0) {
  let newIndex = lastIndex
  while (newIndex < wrappers.length) {
    const nextIdx = newIndex + 1
    // 检查下一个位置的触发区（顶部 10%）
    const enterThreshold = nextEl.rect.top + zonePx
    if (pointerY >= enterThreshold) {
      newIndex = nextIdx  // 继续循环
    } else {
      break  // 停止
    }
  }
  return newIndex
}

// 向上移动：循环检测，dropIndex 只能减小
if (deltaY < 0) {
  let newIndex = lastIndex
  while (newIndex > 0) {
    const prevIdx = newIndex - 1
    // 检查上一个位置的触发区（底部 10%）
    const enterThreshold = prevEl.rect.bottom - zonePx
    if (pointerY <= enterThreshold) {
      newIndex = prevIdx  // 继续循环
    } else {
      break  // 停止
    }
  }
  return newIndex
}
```

**效果**：
1. ✅ 快速移动可以跨多格（循环步进）
2. ✅ 高度变化不会导致反向抖动（方向限制：向下只能增大，向上只能减小）

---

## 三、提交记录

```
commit 642037b
Add template-to-task drag preview support

- Add previewTransformer parameter to useInteractDrag for cross-type drag preview
- Implement templateToTaskPreview in TaskList and SimpleKanbanColumn
- Filter out preview IDs (preview-*) in extractObjectIds to fix sort position
- Disable TransitionGroup animation during cross-type drop to prevent flicker
- Clear preview state immediately on drop and save context before clearing
```

修改的文件：
- `src/composables/drag/useInteractDrag.ts`
- `src/components/assembles/tasks/list/TaskList.vue`
- `src/components/assembles/tasks/kanban/SimpleKanbanColumn.vue`
- `src/infra/drag/strategies/strategy-utils.ts`
