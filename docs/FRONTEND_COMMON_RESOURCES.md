# Cutie 前端公共资源

## 一、工具函数

### 1.1 API 客户端

**位置**: `src/infra/http/api-client.ts`

**使用方法**:

```typescript
import { apiGet, apiPost, apiPatch, apiDelete } from '@/stores/shared'

// GET 请求
const items = await apiGet<MyItem[]>('/items')

// POST 请求
const created = await apiPost<MyItem>('/items', {
  title: 'New Item',
})

// PATCH 请求
const updated = await apiPatch<MyItem>('/items/uuid', {
  title: 'Updated',
})

// DELETE 请求
await apiDelete('/items/uuid')
```

**特性**:

- 自动等待 API 就绪
- 自动解包响应 (`response.data`)
- 统一错误处理
- Correlation ID 支持

### 1.2 时间工具

**位置**: `src/utils/time.ts`

**常用函数**:

```typescript
import { getTodayString, formatDate, parseDate, addDays, isSameDay } from '@/utils/time'

// 获取今天的日期字符串
const today = getTodayString() // => "2025-10-16"

// 格式化日期
const formatted = formatDate(new Date()) // => "2025-10-16"

// 日期加减
const tomorrow = addDays(today, 1) // => "2025-10-17"
const yesterday = addDays(today, -1) // => "2025-10-15"

// 日期比较
const same = isSameDay('2025-10-16', '2025-10-16') // => true
```

### 1.3 UUID 生成

**位置**: `src/utils/uuid.ts`

```typescript
import { generateUUID } from '@/utils/uuid'

const id = generateUUID() // => "a1b2c3d4-..."
```

---

## 二、日志系统

### 2.1 Logger

**位置**: `src/infra/logging/logger.ts`

**使用方法**:

```typescript
import { logger, LogTags } from '@/infra/logging/logger'

// Info 级别
logger.info(LogTags.COMPONENT, 'User action completed', {
  userId: 'uuid',
  action: 'create',
})

// Debug 级别
logger.debug(LogTags.DRAG_STRATEGY, 'Strategy matched', {
  strategyId: 'my-strategy',
})

// Warning 级别
logger.warn(LogTags.STORE, 'Deprecated API called')

// Error 级别
logger.error(LogTags.SYSTEM_PIPELINE, 'Request failed', error, { endpoint: '/api/items' })
```

### 2.2 日志标签 (LogTags)

**常用标签**:

```typescript
LogTags.COMPONENT // 通用组件
LogTags.COMPONENT_KANBAN_COLUMN // 看板列组件
LogTags.STORE // Store
LogTags.STORE_TASK // Task Store
LogTags.SYSTEM_PIPELINE // CPU Pipeline
LogTags.SYSTEM_API // API 系统
LogTags.DRAG_STRATEGY // 拖放策略
LogTags.DRAG_CROSS_VIEW // 跨视图拖放
LogTags.VIEW_HOME // Home 视图
```

---

## 三、拖放系统

### 3.1 useInteractDrag

**位置**: `src/composables/drag/useInteractDrag.ts`

**用途**: 为看板列组件添加拖放功能

**使用方法**:

```typescript
import { useInteractDrag } from '@/composables/drag/useInteractDrag'

const { displayTasks, isDragging, isReceiving } = useInteractDrag({
  viewMetadata: computed(() => ({ id: VIEW_KEY, type: 'status', label: '标题' })),
  tasks: computed(() => myTasks.value),
  containerRef: kanbanContainerRef,
  draggableSelector: '.task-card-wrapper',
  onDrop: async (session) => {
    // 处理放置
  },
})
```

**返回值**:

- `displayTasks`: 包含预览逻辑的任务列表
- `isDragging`: 是否正在拖动
- `isReceiving`: 是否正在接收拖放

### 3.2 useDragStrategy

**位置**: `src/composables/drag/useDragStrategy.ts`

**用途**: 执行拖放策略

**使用方法**:

```typescript
import { useDragStrategy } from '@/composables/drag/useDragStrategy'

const dragStrategy = useDragStrategy()

const result = await dragStrategy.executeDrop(session, targetViewKey, {
  sourceContext: {
    /* ... */
  },
  targetContext: {
    /* ... */
  },
})

if (!result.success) {
  console.error(result.message)
}
```

### 3.3 拖放预览状态

**位置**: `src/infra/drag-interact/preview-state.ts`

**使用方法**:

```typescript
import { dragPreviewState } from '@/infra/drag-interact/preview-state'

// 访问当前预览状态
const dropIndex = dragPreviewState.value?.computed.dropIndex
const ghostTask = dragPreviewState.value?.raw.ghostTask
```

---

## 四、视图适配器

### 4.1 deriveViewMetadata

**位置**: `src/services/viewAdapter.ts`

**用途**: 从 viewKey 推导 ViewMetadata

**使用方法**:

```typescript
import { deriveViewMetadata } from '@/services/viewAdapter'

const metadata = deriveViewMetadata('misc::staging')
// => { id: 'misc::staging', type: 'status', label: '暂存区' }

const metadata = deriveViewMetadata('daily::2025-10-16')
// => { id: 'daily::2025-10-16', type: 'date', label: '2025-10-16', config: { date: '2025-10-16' } }
```

### 4.2 validateViewKey

**用途**: 验证 viewKey 格式

**使用方法**:

```typescript
import { validateViewKey } from '@/services/viewAdapter'

const isValid = validateViewKey('misc::staging') // => true
const isValid = validateViewKey('invalid') // => false
```

---

## 五、拖放决策服务

### 5.1 makeDragDecision

**位置**: `src/services/dragDecisionService.ts`

**用途**: 决定跨日期拖动时的行为

**使用方法**:

```typescript
import { makeDragDecision } from '@/services/dragDecisionService'

const decision = makeDragDecision(
  task,
  sourceDate,  // "2025-10-16"
  targetDate,  // "2025-10-17"
  today        // "2025-10-16"
)

// 返回值
{
  keepSourceElement: boolean,      // 是否保留源元素
  deleteSourceSchedule: boolean,   // 是否删除源日程
  createTargetSchedule: boolean,   // 是否创建目标日程
  updateScheduleOutcome: 'CARRIED_OVER' | null,
}
```

**决策规则**:

- 今天拖到未来：保留源日程，创建目标日程，标记为 CARRIED_OVER
- 过去拖到今天/未来：删除源日程，创建目标日程
- 未来拖到未来：删除源日程，创建目标日程

---

## 六、Composables

### 6.1 useViewTasks

**位置**: `src/composables/useViewTasks.ts`

**用途**: 根据 viewKey 获取任务列表

**使用方法**:

```typescript
import { useViewTasks } from '@/composables/useViewTasks'

const { tasks } = useViewTasks('misc::staging')
// => tasks 是响应式的，自动过滤和排序
```

### 6.2 useContextMenu

**位置**: `src/composables/useContextMenu.ts`

**用途**: 创建右键菜单

**使用方法**:

```typescript
import { useContextMenu } from '@/composables/useContextMenu'

const contextMenu = useContextMenu()

function handleRightClick(event: MouseEvent) {
  contextMenu.show(event, [
    {
      label: '编辑',
      action: () => {
        /* ... */
      },
    },
    {
      label: '删除',
      action: () => {
        /* ... */
      },
    },
  ])
}
```

### 6.3 useTimeBlockDrag

**位置**: `src/composables/useTimeBlockDrag.ts`

**用途**: 日历时间块拖放

**使用方法**:

```typescript
import { useTimeBlockDrag } from '@/composables/useTimeBlockDrag'

const { isDragging, draggedBlock } = useTimeBlockDrag()
```

---

## 七、类型定义

### 7.1 DTO 类型

**位置**: `src/types/dtos.ts`

**常用类型**:

```typescript
import type { TaskCard, TaskDetail, Template, TimeBlock, Area, Schedule } from '@/types/dtos'
```

**TaskCard** (任务卡片):

```typescript
interface TaskCard {
  id: string
  title: string
  glance_note: string | null
  schedule_status: 'staging' | 'planned'
  is_completed: boolean
  estimated_duration: number | null
  area_id: string | null
  schedule_info: {
    outcome_for_today: DailyOutcome | null
    is_recurring: boolean
    linked_schedule_count: number
  }
  has_detail_note: boolean
  /* ...更多字段 */
}
```

### 7.2 拖放类型

**位置**: `src/types/drag.ts`

**ViewMetadata**:

```typescript
interface ViewMetadata {
  id: string // viewKey
  type: ViewType // 'status' | 'date' | 'area' | 'project'
  label: string // 显示名称
  config?: any // 配置数据
}
```

**DragSession**:

```typescript
interface DragSession {
  id: string
  source: {
    viewId: string
    viewKey: string
    elementId: string
  }
  object: {
    type: 'task' | 'template'
    data: any
    originalIndex: number
  }
  dragMode: 'normal' | 'copy'
  target?: {
    /* ... */
  }
  metadata?: any
}
```

---

## 八、状态管理 Stores

### 8.1 常用 Stores

```typescript
import { useTaskStore } from '@/stores/task'
import { useTemplateStore } from '@/stores/template'
import { useScheduleStore } from '@/stores/schedule'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useAreaStore } from '@/stores/area'
import { useViewStore } from '@/stores/view'
import { useTrashStore } from '@/stores/trash'
```

### 8.2 Store 使用模式

```typescript
const taskStore = useTaskStore()

// 访问状态（只读）
const allTasks = taskStore.allTasks

// 访问 Getter
const task = taskStore.getTaskById('uuid')

// 不要直接调用 Mutation
// ❌ taskStore.addTask_mut(task)

// 使用指令
// ✅ await pipeline.dispatch('task.create', payload)
```

---

## 九、CPU Pipeline

### 9.1 使用 Pipeline

**位置**: `src/cpu/index.ts`

**使用方法**:

```typescript
import { pipeline } from '@/cpu'

// 调用指令
const result = await pipeline.dispatch('domain.action', {
  /* payload */
})

// 获取管道状态
const stats = pipeline.getStats()
console.log(stats.totalExecuted, stats.totalFailed)
```

### 9.2 常用指令

**任务**:

```typescript
await pipeline.dispatch('task.create', { title: '...' })
await pipeline.dispatch('task.update', { id: '...', title: '...' })
await pipeline.dispatch('task.delete', { id: '...' })
await pipeline.dispatch('task.complete', { id: '...' })
```

**日程**:

```typescript
await pipeline.dispatch('schedule.create', { task_id: '...', scheduled_day: '...' })
await pipeline.dispatch('schedule.update', { task_id: '...', scheduled_day: '...', outcome: '...' })
await pipeline.dispatch('schedule.delete', { task_id: '...', scheduled_day: '...' })
```

**模板**:

```typescript
await pipeline.dispatch('template.create', { title: '...' })
await pipeline.dispatch('template.update', { id: '...', title: '...' })
await pipeline.dispatch('template.delete', { id: '...' })
await pipeline.dispatch('template.create_task', { template_id: '...', variables: {} })
await pipeline.dispatch('template.from_task', { task_id: '...', title: '...' })
```

**视图偏好**:

```typescript
await pipeline.dispatch('viewpreference.update_sorting', {
  view_key: '...',
  sorted_task_ids: ['id1', 'id2'],
  original_sorted_task_ids: ['id2', 'id1'],
})
```

---

## 十、组件库

### 10.1 基础组件

**CutePane**: 面板容器

```vue
<CutePane>
  <template #header>标题</template>
  <div>内容</div>
</CutePane>
```

**CuteCard**: 卡片容器

```vue
<CuteCard>
  <div>卡片内容</div>
</CuteCard>
```

**CuteButton**: 按钮

```vue
<CuteButton @click="handleClick">点击</CuteButton>
```

**CuteIcon**: 图标

```vue
<CuteIcon name="check" />
```

**CuteCheckbox**: 复选框

```vue
<CuteCheckbox v-model="isChecked" />
```

### 10.2 业务组件

**TaskCard**: 任务卡片

```vue
<TaskCard :task="task" @task-completed="handleComplete" />
```

**TemplateCard**: 模板卡片

```vue
<TemplateCard :template="template" @open-editor="handleEdit" />
```

**AreaTag**: 区域标签

```vue
<AreaTag :area-id="areaId" />
```

**TimeDurationPicker**: 时长选择器

```vue
<TimeDurationPicker v-model="duration" />
```

---

## 十一、样式变量

### 11.1 CSS 变量

**位置**: `src/style.css`

**颜色**:

```css
--color-primary: #4a90e2;
--color-text-primary: #1a1a1a;
--color-text-secondary: #666;
--color-text-tertiary: #999;
--color-border-default: #e5e5e5;
--color-background-content: #ffffff;
--color-card-available: #f8f9fa;
```

**使用方法**:

```vue
<style scoped>
.my-component {
  color: var(--color-text-primary);
  border: 1px solid var(--color-border-default);
}
</style>
```

---

## 十二、开发工具

### 12.1 浏览器扩展

- Vue DevTools: 查看组件树和状态
- Redux DevTools: 查看 Pinia Store

### 12.2 调试技巧

**打印拖放状态**:

```typescript
import { dragPreviewState } from '@/infra/drag-interact/preview-state'

watch(dragPreviewState, (state) => {
  console.log('Drag preview:', state)
})
```

**打印 Pipeline 状态**:

```typescript
import { pipeline } from '@/cpu'

console.log('Pipeline stats:', pipeline.getStats())
```

**打印 Store 状态**:

```typescript
const taskStore = useTaskStore()
console.log('All tasks:', taskStore.allTasks)
```

---

## 十三、常见工具函数

### 13.1 数组操作

```typescript
// 插入元素
function insertAt<T>(arr: T[], item: T, index: number): T[] {
  return [...arr.slice(0, index), item, ...arr.slice(index)]
}

// 移除元素
function removeItem<T>(arr: T[], item: T): T[] {
  return arr.filter((i) => i !== item)
}

// 移动元素
function moveItem<T>(arr: T[], from: number, to: number): T[] {
  const result = [...arr]
  const [item] = result.splice(from, 1)
  result.splice(to, 0, item)
  return result
}
```

### 13.2 对象操作

```typescript
// 深拷贝
function deepClone<T>(obj: T): T {
  return JSON.parse(JSON.stringify(obj))
}

// 选择字段
function pick<T, K extends keyof T>(obj: T, keys: K[]): Pick<T, K> {
  const result = {} as Pick<T, K>
  keys.forEach((key) => {
    result[key] = obj[key]
  })
  return result
}
```

### 13.3 字符串操作

```typescript
// 截断文本
function truncate(text: string, maxLength: number): string {
  return text.length > maxLength ? text.slice(0, maxLength) + '...' : text
}

// 驼峰转短横线
function kebabCase(str: string): string {
  return str.replace(/([a-z])([A-Z])/g, '$1-$2').toLowerCase()
}
```
