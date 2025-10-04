# Cutie 拖放系统 Composables

基于 `DRAG_DROP_SYSTEM_DESIGN.md` 设计文档实现的完整拖放框架。

## 📦 模块结构

```
src/composables/drag/
├── index.ts                    # 统一导出
├── useDragTransfer.ts          # 数据传递工具
├── useAutoScroll.ts            # 自动滚动
├── useThrottledDragOver.ts     # 节流
├── useDragState.ts             # 状态管理
├── useCrossViewDrag/
│   ├── index.ts                # 跨看板拖放主入口
│   ├── context.ts              # 拖拽上下文管理
│   ├── strategies.ts           # 策略注册表
│   └── finder.ts               # 策略查找
└── README.md                   # 本文档
```

## 🚀 快速开始

### 1. 轻量工具包

#### useDragTransfer - 数据传递

```typescript
import { useDragTransfer } from '@/composables/drag'

const { setDragData, getDragData } = useDragTransfer()

// 在 dragstart 事件中设置数据
function handleDragStart(event: DragEvent, task: TaskCard) {
  setDragData(event, {
    type: 'task',
    task,
    sourceView: viewMetadata,
    dragMode: { mode: 'normal' },
  })
}

// 在 drop 事件中获取数据
function handleDrop(event: DragEvent) {
  const data = getDragData(event)
  if (data) {
    console.log('拖拽的任务:', data.task)
  }
}
```

#### useAutoScroll - 自动滚动

```typescript
import { useAutoScroll } from '@/composables/drag'

const { handleAutoScroll, stopAutoScroll } = useAutoScroll({
  edgeSize: 50, // 边缘触发距离（像素）
  speed: 5, // 滚动速度
  maxSpeed: 20, // 最大速度
})

function handleDragOver(event: DragEvent) {
  handleAutoScroll(event) // 自动检测并滚动
}

function handleDragEnd() {
  stopAutoScroll()
}
```

#### useThrottledDragOver - 节流

```typescript
import { useThrottledDragOver } from '@/composables/drag'

const throttledHandler = useThrottledDragOver((event: DragEvent) => {
  // 这个函数最多每 16ms 执行一次（~60fps）
  updatePreview(event)
}, 16)

function handleDragOver(event: DragEvent) {
  throttledHandler(event)
}
```

#### useDragState - 状态管理

```typescript
import { useDragState } from '@/composables/drag'
import type { TaskCard } from '@/types/dtos'

const { isDragging, draggedItem, startDrag, endDrag } = useDragState<TaskCard>()

function handleDragStart(task: TaskCard) {
  startDrag(task)
}

function handleDragEnd() {
  endDrag()
}

// 在其他组件中访问全局拖拽状态
watch(isDragging, (dragging) => {
  if (dragging) {
    console.log('正在拖拽:', draggedItem.value)
  }
})
```

---

### 2. 跨看板拖放核心

#### 基础用法

```vue
<script setup lang="ts">
import { useCrossViewDrag } from '@/composables/drag'
import type { ViewMetadata, TaskCard } from '@/types/drag'

// 定义看板元数据
const viewMetadata: ViewMetadata = {
  type: 'date',
  id: 'daily-2025-10-03',
  config: {
    date: '2025-10-03',
  },
  label: '2025年10月3日',
}

const props = defineProps<{
  tasks: TaskCard[]
}>()

const crossViewDrag = useCrossViewDrag()

// ========== 普通拖放 ==========

function handleDragStart(event: DragEvent, task: TaskCard) {
  crossViewDrag.startNormalDrag(task, viewMetadata)

  // 可选：同时设置 HTML5 数据（向后兼容）
  const { setDragData } = useDragTransfer()
  setDragData(event, {
    type: 'task',
    task,
    sourceView: viewMetadata,
    dragMode: { mode: 'normal' },
  })
}

const isValidDropTarget = ref(false)
const dropHint = ref('')

function handleDragOver(event: DragEvent) {
  event.preventDefault()

  const context = crossViewDrag.currentContext.value
  if (!context) return

  // 检查是否可以放置
  isValidDropTarget.value = crossViewDrag.canDrop(context.sourceView, viewMetadata)

  if (isValidDropTarget.value) {
    // 获取提示文字
    dropHint.value = crossViewDrag.getDropHint(context.sourceView, viewMetadata)
  }
}

function handleDragLeave() {
  isValidDropTarget.value = false
  dropHint.value = ''
}

async function handleDrop(event: DragEvent) {
  event.preventDefault()

  const result = await crossViewDrag.handleDrop(viewMetadata, event)

  if (result.success) {
    // 🎉 成功
    console.log('✅', result.message)

    if (result.reorderOnly) {
      console.log('仅重排序，无需刷新数据')
    } else {
      console.log('需要刷新的视图:', result.affectedViews)
    }
  } else {
    // ❌ 失败
    console.error('❌', result.error)
  }

  // 清理视觉状态
  isValidDropTarget.value = false
  dropHint.value = ''
}

function handleDragEnd() {
  // 拖拽结束，清理状态
  // useCrossViewDrag 会自动清理上下文
}
</script>

<template>
  <div
    class="kanban-column"
    :class="{ 'is-valid-target': isValidDropTarget }"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <div v-if="dropHint" class="drop-hint">
      {{ dropHint }}
    </div>

    <div
      v-for="task in tasks"
      :key="task.id"
      draggable="true"
      @dragstart="handleDragStart($event, task)"
      @dragend="handleDragEnd"
    >
      <TaskCard :task="task" />
    </div>
  </div>
</template>
```

#### 吸附式拖放（未来功能）

```vue
<script setup lang="ts">
// 激活吸附模式
function handleActivateSnapDrag(task: TaskCard) {
  crossViewDrag.startSnapDrag(
    task,
    viewMetadata,
    'schedule-button', // 激活来源
    { originalDate: task.scheduled_date } // 额外参数
  )
}

// 监听 ESC 取消
onMounted(() => {
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && crossViewDrag.isSnapMode.value) {
      crossViewDrag.cancelDrag()
    }
  })
})

// 吸附模式下的点击处理
async function handleSnapClick(event: MouseEvent) {
  if (!crossViewDrag.isSnapMode.value) return

  const result = await crossViewDrag.handleDrop(viewMetadata)
  // ... 处理结果
}
</script>

<template>
  <div :class="{ 'is-snap-mode': crossViewDrag.isSnapMode.value }" @click="handleSnapClick">
    <TaskCard :task="task">
      <button @click.stop="handleActivateSnapDrag(task)">📅 设置排期</button>
    </TaskCard>
  </div>
</template>

<style scoped>
.is-snap-mode {
  cursor: crosshair;
}
</style>
```

---

## 🎯 ViewMetadata 元数据系统

每个看板需要提供元数据来标识自己：

### 状态看板

```typescript
const stagingView: ViewMetadata = {
  type: 'status',
  id: 'status-staging',
  config: {
    status: 'staging',
  },
  label: 'Staging',
}
```

### 日期看板

```typescript
const dateView: ViewMetadata = {
  type: 'date',
  id: 'daily-2025-10-03',
  config: {
    date: '2025-10-03', // YYYY-MM-DD
  },
  label: '2025年10月3日',
}
```

### 项目看板

```typescript
const projectView: ViewMetadata = {
  type: 'project',
  id: 'project-abc123',
  config: {
    projectId: 'abc123',
    projectName: '我的项目',
  },
  label: '我的项目',
}
```

### 日历（特殊）

```typescript
const calendarView: ViewMetadata = {
  type: 'calendar',
  id: `calendar-${startTime}`,
  config: {
    startTime: '2025-10-03T10:00:00Z',
    endTime: '2025-10-03T11:00:00Z',
  },
  label: '10:00 - 11:00',
}
```

### 自定义看板

```typescript
const customView: ViewMetadata = {
  type: 'custom',
  id: 'custom-high-priority-urgent',
  config: {
    filter: (task: TaskCard) => task.priority === 'high' && task.is_urgent,
    metadata: {
      filterName: '高优先级且紧急',
      color: 'red',
    },
  },
  label: '紧急任务',
}
```

---

## 🔧 策略系统

### 内置策略

当前实现的策略（仅打印日志）：

| 策略键             | 场景                   | 行为                     |
| ------------------ | ---------------------- | ------------------------ |
| `status->status`   | 状态看板间拖动         | staging→planned 设置排期 |
| `date->date`       | 日期看板间拖动         | 改期                     |
| `project->project` | 项目看板间拖动         | 更改项目                 |
| `status->date`     | 从状态看板拖到日期看板 | 设置排期                 |
| `date->status`     | 从日期看板拖回状态看板 | 取消排期（拖回 staging） |
| `*->calendar`      | 拖到日历               | 创建时间块               |
| `*->*`             | 默认（不支持）         | 返回错误                 |

### 注册自定义策略

```typescript
import { useCrossViewDrag } from '@/composables/drag'
import type { DragStrategy } from '@/types/drag'

const crossViewDrag = useCrossViewDrag()

// 定义自定义策略
const myCustomStrategy: DragStrategy = async (context, targetView) => {
  console.log('执行自定义策略:', context.task.title)

  // 在这里执行实际的业务逻辑
  // await taskStore.updateTask(...)

  return {
    success: true,
    message: '自定义操作完成',
    affectedViews: [context.sourceView.id, targetView.id],
  }
}

// 注册策略
crossViewDrag.registerStrategy('custom->date', myCustomStrategy)

// 列出所有策略
const allStrategies = crossViewDrag.listStrategies()
console.log('已注册的策略:', allStrategies)

// 注销策略
crossViewDrag.removeStrategy('custom->date')
```

---

## 🐛 调试

### 控制台日志

所有 composables 都会输出详细的控制台日志：

```
[useDragTransfer] Data set: { type: 'task', taskId: '123', ... }
[DragContext] 🚀 Started normal drag: { taskId: '123', ... }
[StrategyFinder] 🔍 Finding strategy: { sourceType: 'status', targetType: 'date' }
  ✅ Found exact match: status->date
[Strategy] 📊➡️📅 status -> date { task: 'My Task', from: 'staging', to: '2025-10-03' }
  ➡️ Action: Set scheduled_date to 2025-10-03
[useCrossViewDrag] ✅ Drop handled: { success: true, message: '已设置排期' }
[DragContext] ✅ Cleared context: { duration: '342ms', mode: 'normal' }
```

### 状态检查

```typescript
const crossViewDrag = useCrossViewDrag()

// 检查当前状态
console.log('是否正在拖拽:', crossViewDrag.isDragging.value)
console.log('当前模式:', crossViewDrag.currentMode.value)
console.log('拖拽的任务:', crossViewDrag.currentTask.value)
console.log('源看板:', crossViewDrag.sourceView.value)
console.log('拖拽持续时间:', crossViewDrag.getDragDuration())

// 检查策略信息
const info = crossViewDrag.getStrategyInfo(sourceView, targetView)
console.log('策略信息:', {
  exists: info.exists, // 是否有策略
  priority: info.priority, // 优先级: 'exact', 'source-wildcard', 'target-wildcard', 'default'
  key: info.key, // 策略键
})
```

---

## 📚 类型定义

所有类型都在 `src/types/drag.ts` 中定义：

```typescript
import type {
  ViewMetadata,
  ViewConfig,
  DragContext,
  DragStrategy,
  StrategyResult,
  DragMode,
  AutoScrollOptions,
} from '@/types/drag'
```

---

## ⚠️ 当前状态

### ✅ 已完成

- ✅ 完整的类型系统
- ✅ 轻量工具包（数据传递、自动滚动、节流、状态管理）
- ✅ 跨看板拖放核心（上下文、策略、查找）
- ✅ 策略框架（仅打印日志）
- ✅ 完整的 TypeScript 类型支持
- ✅ 详细的控制台日志

### 🚧 未完成（待集成实际业务逻辑）

- ⏳ 策略函数内的实际 API 调用（当前只打印日志）
- ⏳ 与现有组件的集成
- ⏳ 吸附式拖放的 UI 实现
- ⏳ 批量拖放
- ⏳ 撤销/重做

---

## 🎉 下一步

### 1. 集成到现有组件

修改 `SimpleKanbanColumn.vue`、`CuteCalendar.vue` 等组件，使用新的 `useCrossViewDrag`。

### 2. 实现实际业务逻辑

在 `strategies.ts` 中替换 `console.log` 为实际的 Store 调用：

```typescript
// 示例：实现 status->date 策略
const statusToDate: DragStrategy = async (context, targetView) => {
  const targetDate = (targetView.config as DateViewConfig).date

  // ✅ 实际调用 API
  await taskStore.addSchedule(context.task.id, targetDate)

  return {
    success: true,
    message: `已设置排期：${targetDate}`,
    affectedViews: [context.sourceView.id, targetView.id],
  }
}
```

### 3. 测试

编写单元测试和集成测试，确保所有策略正常工作。

---

## 📞 支持

如有问题，请参考：

- 设计文档：`DRAG_DROP_SYSTEM_DESIGN.md`
- 类型定义：`src/types/drag.ts`
- 示例代码：本 README 中的示例

---

**🎨 Enjoy dragging!**
