# Cutie 前端开发手册

## 一、前端架构概览

### 1.1 核心概念

**CPU 指令集架构 (ISA)**

- 将所有 API 调用抽象为"指令"
- 统一的错误处理、重试、超时机制
- 支持乐观更新、请求去重
- 指令优先级调度

**RTL Store 架构**

- Register (寄存器)：纯响应式状态
- Transmission (传输线)：Getters 和计算属性
- Logic (逻辑门)：Mutations，纯数据操作
- DMA：直接数据加载，绕过指令系统

**拖放策略系统**

- 基于 Interact.js
- 策略匹配引擎
- 响应式预览渲染
- 越界回弹支持

---

## 二、CPU 指令系统开发

### 2.1 创建指令集文件

**位置**：`src/cpu/isa/{domain}-isa.ts`

**基本结构**：

```typescript
import type { ISADefinition } from './types'
import { useMyStore } from '@/stores/my'
import type { MyDTO } from '@/types/dtos'

export const MyISA: ISADefinition = {
  // 指令名：domain.action (小写，点分隔)
  'my.create': {
    meta: {
      description: '创建资源',
      category: 'my',
      resourceIdentifier: (payload) => [`my:${payload.id}`],
      priority: 5, // 优先级 (1-10)
      timeout: 10000, // 超时时间 (毫秒)
    },

    request: {
      method: 'POST',
      url: '/my-resources',
      // 可选：动态 URL
      // url: (payload) => `/resources/${payload.id}`,

      // 可选：请求体转换
      // body: (payload) => ({ name: payload.title }),
    },

    commit: async (result: MyDTO) => {
      const store = useMyStore()
      store.addOrUpdate_mut(result)
    },
  },
}
```

### 2.2 指令字段详解

**meta.resourceIdentifier**

- 用途：请求去重，同一资源的并发请求会被合并
- 格式：返回字符串数组，通常为 `['resource:id']`

**meta.priority**

- 数字越大优先级越高
- 建议范围：1-10
- 默认值：5

**request.url**

- 静态 URL：直接字符串
- 动态 URL：函数 `(payload) => string`

**request.body**

- 默认：整个 payload 作为请求体
- 自定义：函数 `(payload) => object`

**commit**

- 成功后的状态更新
- 必须调用 Store 的 `_mut` 方法
- 可以异步执行

### 2.3 支持乐观更新

```typescript
'my.update': {
  meta: { /* ... */ },

  request: { /* ... */ },

  // 乐观更新配置
  optimistic: {
    snapshot: (payload) => {
      const store = useMyStore()
      return {
        original: store.getItemById(payload.id),
      }
    },

    apply: (payload) => {
      const store = useMyStore()
      store.addOrUpdate_mut({
        ...payload,
        updated_at: new Date().toISOString(),
      })
    },

    rollback: (snapshot) => {
      const store = useMyStore()
      if (snapshot.original) {
        store.addOrUpdate_mut(snapshot.original)
      }
    },
  },

  commit: async (result) => {
    const store = useMyStore()
    store.addOrUpdate_mut(result)
  },
}
```

### 2.4 注册指令集

**文件**：`src/cpu/isa/index.ts`

```typescript
import { MyISA } from './my-isa'

export const ISA: ISADefinition = {
  ...TaskISA,
  ...ScheduleISA,
  ...MyISA, // 添加新的指令集
}
```

### 2.5 调用指令

**在组件中**：

```typescript
import { pipeline } from '@/cpu'

// 基本调用
const result = await pipeline.dispatch('my.create', {
  title: 'My Resource',
})

// 带错误处理
try {
  const result = await pipeline.dispatch('my.update', {
    id: 'uuid',
    title: 'Updated',
  })
} catch (error) {
  console.error('Update failed:', error)
}
```

---

## 三、Pinia Store 开发

### 3.1 Store 目录结构

```
src/stores/{domain}/
  index.ts              # Store 主入口
  core.ts               # 状态和 Mutations
  view-operations.ts    # DMA 数据加载
  event-handlers.ts     # SSE 事件处理
  types.ts              # 类型定义
```

### 3.2 Store 主入口

**文件**：`src/stores/{domain}/index.ts`

```typescript
import { defineStore } from 'pinia'
import * as core from './core'
import * as view from './view-operations'
import * as events from './event-handlers'

export const useMyStore = defineStore('my', () => {
  return {
    // ========== STATE (寄存器) - 只读 ==========
    items: core.items,

    // ========== GETTERS (多路复用器) ==========
    allItems: core.allItems,
    getItemById: core.getItemById,
    filteredItems: core.filteredItems,

    // ========== MUTATIONS (寄存器写入) ==========
    addOrUpdate_mut: core.addOrUpdate_mut,
    remove_mut: core.remove_mut,
    clear_mut: core.clear_mut,

    // ========== DMA (数据加载) ==========
    fetchAllItems: view.fetchAllItems,

    // ========== EVENT HANDLING ==========
    initEventSubscriptions: events.initEventSubscriptions,
  }
})

// 导出类型
export type { CreateItemPayload, UpdateItemPayload } from './types'
```

### 3.3 核心状态文件

**文件**：`src/stores/{domain}/core.ts`

```typescript
import { ref, computed } from 'vue'
import type { MyItem } from '@/types/dtos'

// ========== State ==========
export const items = ref(new Map<string, MyItem>())

// ========== Getters ==========
export const allItems = computed(() => Array.from(items.value.values()))

export const getItemById = computed(() => (id: string) => items.value.get(id))

export const filteredItems = computed(() => allItems.value.filter((item) => !item.is_deleted))

// ========== Mutations ==========
export function addOrUpdate_mut(item: MyItem) {
  const newMap = new Map(items.value)
  newMap.set(item.id, item)
  items.value = newMap
}

export function remove_mut(id: string) {
  const newMap = new Map(items.value)
  newMap.delete(id)
  items.value = newMap
}

export function clear_mut() {
  items.value = new Map()
}
```

**Mutation 命名规则**：

- ✅ 必须以 `_mut` 结尾
- ✅ 使用驼峰命名法
- ✅ 动词开头（add, update, remove, set, clear）

### 3.4 DMA 数据加载

**文件**：`src/stores/{domain}/view-operations.ts`

```typescript
import { apiGet } from '@/stores/shared'
import type { MyItem } from '@/types/dtos'
import { addOrUpdate_mut, clear_mut } from './core'

export async function fetchAllItems(): Promise<void> {
  const items: MyItem[] = await apiGet('/my-resources')
  clear_mut()
  items.forEach(addOrUpdate_mut)
}

export async function fetchItemById(id: string): Promise<void> {
  const item: MyItem = await apiGet(`/my-resources/${id}`)
  addOrUpdate_mut(item)
}
```

### 3.5 SSE 事件处理

**文件**：`src/stores/{domain}/event-handlers.ts`

```typescript
import type { MyItem } from '@/types/dtos'
import * as core from './core'
import { logger, LogTags } from '@/infra/logging/logger'

export function initEventSubscriptions() {
  const { eventBus } = useEventBus()

  // 创建事件
  eventBus.on('my.created', (data: MyItem) => {
    logger.info(LogTags.STORE_MY, 'Item created', { id: data.id })
    core.addOrUpdate_mut(data)
  })

  // 更新事件
  eventBus.on('my.updated', (data: MyItem) => {
    logger.info(LogTags.STORE_MY, 'Item updated', { id: data.id })
    core.addOrUpdate_mut(data)
  })

  // 删除事件
  eventBus.on('my.deleted', (data: { id: string }) => {
    logger.info(LogTags.STORE_MY, 'Item deleted', { id: data.id })
    core.remove_mut(data.id)
  })

  logger.info(LogTags.STORE_MY, 'Event subscriptions initialized')
}

function useEventBus() {
  const eventBus = (window as any).__eventBus__
  if (!eventBus) {
    throw new Error('EventBus not initialized')
  }
  return { eventBus }
}
```

**重要**：事件处理器必须调用 `_mut` 版本的函数！

---

## 四、拖放系统开发

### 4.1 创建拖放策略

**位置**：`src/infra/drag/strategies/{feature}-scheduling.ts`

**策略结构**：

```typescript
import type { Strategy } from '../types'
import { pipeline } from '@/cpu'
import {
  extractTaskIds,
  insertTaskAt,
  removeTaskFrom,
  extractDate,
  createOperationRecord,
} from './strategy-utils'

export const myStrategy: Strategy = {
  id: 'unique-strategy-id',
  name: 'Human Readable Name',

  conditions: {
    source: {
      viewKey: 'misc::source', // 精确匹配
      // 或 viewKey: /^daily::\d{4}-\d{2}-\d{2}$/,  // 正则匹配
      objectType: 'task', // 可选
    },
    target: {
      viewKey: 'misc::target',
    },
    priority: 90, // 优先级 (数字越大越优先)
  },

  action: {
    name: 'action_identifier',
    description: '操作的详细描述',

    async execute(ctx) {
      const operations = []

      try {
        // 步骤 1: 执行业务逻辑
        const result = await pipeline.dispatch('domain.action', {
          /* payload */
        })
        operations.push(createOperationRecord('action_type', ctx.targetViewId, payload))

        // 步骤 2: 更新视图排序
        const targetSorting = extractTaskIds(ctx.targetContext)
        const newSorting = insertTaskAt(targetSorting, result.id, ctx.dropIndex)

        await pipeline.dispatch('viewpreference.update_sorting', {
          view_key: ctx.targetViewId,
          sorted_task_ids: newSorting,
          original_sorted_task_ids: targetSorting,
        })

        return {
          success: true,
          message: '✅ 操作成功',
          operations,
          affectedViews: [ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ 操作失败: ${error.message}`,
          operations,
          affectedViews: [],
        }
      }
    },
  },

  tags: ['domain', 'scheduling', 'multi-step'],
}
```

### 4.2 策略上下文 (ctx) 字段

```typescript
ctx = {
  // 拖放会话
  session: DragSession,

  // 被拖动的对象
  task: TaskCard,
  object: any,  // 原始对象

  // 源和目标视图
  sourceViewId: string,
  targetViewId: string,
  sourceZone: string,
  targetZone: string,

  // 插入位置
  dropIndex: number,

  // 上下文数据（由组件提供）
  sourceContext: {
    taskIds: string[],
    displayTasks: TaskCard[],
    viewKey: string,
  },
  targetContext: {
    taskIds: string[],
    displayTasks: TaskCard[],
    dropIndex: number,
    viewKey: string,
  },
}
```

### 4.3 工具函数

**extractTaskIds**：从上下文提取任务 ID 数组

```typescript
const taskIds = extractTaskIds(ctx.targetContext)
// => ['id1', 'id2', 'id3']
```

**insertTaskAt**：在指定位置插入任务

```typescript
const newSorting = insertTaskAt(taskIds, 'new-id', 2)
// => ['id1', 'id2', 'new-id', 'id3']
```

**removeTaskFrom**：从列表中移除任务

```typescript
const newSorting = removeTaskFrom(taskIds, 'id2')
// => ['id1', 'id3']
```

**extractDate**：从 viewKey 提取日期

```typescript
const date = extractDate('daily::2025-10-16')
// => '2025-10-16'
```

### 4.4 注册策略

**文件**：`src/infra/drag/strategies/index.ts`

```typescript
// 导出策略
export { myStrategy, myOtherStrategy } from './my-scheduling'
```

策略会在 `src/main.ts` 中自动注册：

```typescript
import { initializeDragStrategies } from '@/infra/drag'
initializeDragStrategies()
```

### 4.5 ViewKey 规范

**格式**：`type::identifier`

**类型**：

- `misc`: 杂项视图 (all, staging, planned, completed, template)
- `daily`: 日期视图 (YYYY-MM-DD)
- `area`: 区域视图 (UUID)
- `project`: 项目视图 (UUID)

**示例**：

```
misc::staging
misc::template
daily::2025-10-16
area::a1b2c3d4-1234-5678-90ab-cdef12345678
```

---

## 五、组件开发

### 5.1 看板列组件集成拖放

**关键步骤**：

**1. 定义 ViewKey 和 Metadata**

```typescript
const VIEW_KEY = 'misc::myview'
const viewMetadata = computed<ViewMetadata>(() => ({
  id: VIEW_KEY,
  type: 'status',
  label: '我的视图',
}))
```

**2. 准备容器和任务数据**

```typescript
const kanbanContainerRef = ref<HTMLElement | null>(null)
const tasks = computed(() => store.filteredTasks)
```

**3. 集成 useInteractDrag**

```typescript
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'

const dragStrategy = useDragStrategy()

const { displayTasks } = useInteractDrag({
  viewMetadata,
  tasks,
  containerRef: kanbanContainerRef,
  draggableSelector: `.task-card-wrapper-${VIEW_KEY.replace(/::/g, '--')}`,

  onDrop: async (session) => {
    const result = await dragStrategy.executeDrop(session, VIEW_KEY, {
      sourceContext: session.metadata?.sourceContext || {},
      targetContext: {
        taskIds: tasks.value.map((t) => t.id),
        displayTasks: tasks.value,
        dropIndex: dragPreviewState.value?.computed.dropIndex,
        viewKey: VIEW_KEY,
      },
    })

    if (!result.success) {
      logger.error(LogTags.DRAG, 'Drop failed', new Error(result.message))
    }
  },
})
```

**4. 模板结构**

```vue
<template>
  <CutePane class="my-kanban">
    <!-- ⚠️ ref 必须绑定到 HTMLElement，不能是组件 -->
    <div ref="kanbanContainerRef" class="kanban-dropzone-wrapper">
      <div
        v-for="task in displayTasks"
        :key="task.id"
        :class="`task-card-wrapper task-card-wrapper-${VIEW_KEY.replace(/::/g, '--')}`"
        :data-task-id="task.id"
      >
        <TaskCard :task="task" />
      </div>
    </div>
  </CutePane>
</template>
```

**关键点**：

- ✅ `kanbanContainerRef` 绑定到 `<div>`，不是 `<CutePane>`
- ✅ 每个卡片有 `data-task-id` 属性
- ✅ 类名包含 ViewKey（替换 `::` 为 `--`）
- ✅ 使用 `displayTasks`（包含预览逻辑）

### 5.2 调用指令创建资源

```typescript
import { pipeline } from '@/cpu'

async function handleCreate() {
  try {
    const result = await pipeline.dispatch('my.create', {
      title: newItemTitle.value,
    })

    logger.info(LogTags.COMPONENT, 'Created successfully', { id: result.id })

    // 重置表单
    newItemTitle.value = ''
  } catch (error) {
    logger.error(
      LogTags.COMPONENT,
      'Create failed',
      error instanceof Error ? error : new Error(String(error))
    )
    alert('创建失败')
  }
}
```

### 5.3 调用指令更新资源

```typescript
async function handleUpdate(id: string, updates: Partial<MyItem>) {
  try {
    await pipeline.dispatch('my.update', {
      id,
      ...updates,
    })
  } catch (error) {
    logger.error(LogTags.COMPONENT, 'Update failed', error)
  }
}
```

### 5.4 调用指令删除资源

```typescript
async function handleDelete(id: string) {
  if (!confirm('确定删除吗？')) return

  try {
    await pipeline.dispatch('my.delete', { id })
  } catch (error) {
    logger.error(LogTags.COMPONENT, 'Delete failed', error)
  }
}
```

---

## 六、类型定义

### 6.1 Payload 类型

**文件**：`src/stores/{domain}/types.ts`

```typescript
export interface CreateItemPayload {
  title: string
  description?: string
  area_id?: string
}

export interface UpdateItemPayload {
  id: string
  title?: string
  description?: string
  area_id?: string
}

export interface DeleteItemPayload {
  id: string
}
```

### 6.2 DTO 类型

**文件**：`src/types/dtos.ts`

```typescript
export interface MyItem {
  id: string
  title: string
  description: string | null
  area_id: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
}
```

---

## 七、调试与日志

### 7.1 使用 Logger

```typescript
import { logger, LogTags } from '@/infra/logging/logger'

// Info 日志
logger.info(LogTags.COMPONENT, 'Operation completed', {
  id: item.id,
  duration: 123,
})

// 错误日志
logger.error(
  LogTags.STORE,
  'Failed to fetch data',
  error instanceof Error ? error : new Error(String(error)),
  { endpoint: '/api/items' }
)

// Debug 日志
logger.debug(LogTags.DRAG_STRATEGY, 'Strategy matched', {
  strategyId: 'my-strategy',
  priority: 90,
})
```

### 7.2 常用 LogTags

```typescript
LogTags.COMPONENT // 组件
LogTags.STORE // Store
LogTags.SYSTEM_PIPELINE // CPU Pipeline
LogTags.DRAG_STRATEGY // 拖放策略
LogTags.DRAG_CROSS_VIEW // 跨视图拖放
LogTags.COMPONENT_KANBAN_COLUMN // 看板列
```

---

## 八、最佳实践

### 8.1 代码组织

**按功能模块组织**

```
src/
  cpu/isa/              # 指令集定义
  stores/{domain}/      # Store 按领域划分
  components/
    parts/{feature}/    # 组件按功能划分
  infra/drag/           # 拖放系统
```

### 8.2 命名规范

**文件名**：kebab-case

```
my-feature.ts
task-card.vue
```

**指令名**：小写，点分隔

```
task.create
schedule.update
viewpreference.update_sorting
```

**Mutation**：驼峰，`_mut` 后缀

```
addOrUpdate_mut
remove_mut
clear_mut
```

**ViewKey**：小写，双冒号

```
misc::staging
daily::2025-10-16
```

### 8.3 避免的反模式

**❌ 直接调用 fetch/axios**

```typescript
// 错误
const response = await fetch('/api/items')
```

**✅ 使用指令**

```typescript
// 正确
const items = await pipeline.dispatch('item.list')
```

**❌ Mutation 中包含业务逻辑**

```typescript
// 错误
export function addItem_mut(item: Item) {
  if (item.area_id) {
    // 验证逻辑...
  }
  items.value.set(item.id, item)
}
```

**✅ Mutation 只做数据操作**

```typescript
// 正确
export function addItem_mut(item: Item) {
  const newMap = new Map(items.value)
  newMap.set(item.id, item)
  items.value = newMap
}
```

**❌ 在事件处理器中使用旧函数名**

```typescript
// 错误
eventBus.on('item.created', (data) => {
  core.addItem(data) // ❌ 缺少 _mut
})
```

**✅ 使用 \_mut 版本**

```typescript
// 正确
eventBus.on('item.created', (data) => {
  core.addItem_mut(data) // ✅
})
```

---

## 九、常见问题

### Q1: displayTasks 和原始 tasks 有什么区别？

**A**: `displayTasks` 包含拖放预览逻辑：

- 在目标列显示预览元素
- 在源列移除被拖动元素
- 越界时回到原始状态

### Q2: 什么时候用 dispatch，什么时候用 execute？

**A**:

- `dispatch`: 异步执行，推荐使用
- `execute`: 旧 API，已废弃

### Q3: 如何处理多步骤操作？

**A**: 在拖放策略中串联多个指令：

```typescript
const result1 = await pipeline.dispatch('step1', ...)
const result2 = await pipeline.dispatch('step2', { id: result1.id })
const result3 = await pipeline.dispatch('step3', ...)
```

### Q4: 如何调试拖放不生效？

**A**: 检查清单：

1. ✅ `kanbanContainerRef` 绑定到 HTMLElement
2. ✅ 每个卡片有 `data-task-id` 属性
3. ✅ 类名匹配 `draggableSelector`
4. ✅ ViewKey 在 `viewAdapter.ts` 中已支持
5. ✅ 拖放策略已导出并注册
6. ✅ 策略的 viewKey 条件匹配

---

## 十、开发检查清单

**创建新功能前**：

- [ ] 确定 ViewKey
- [ ] 确定需要哪些指令
- [ ] 确定需要哪些拖放策略

**开发过程**：

- [ ] 定义 ISA 指令
- [ ] 注册 ISA
- [ ] 创建/更新 Store
- [ ] 添加 Mutations (\_mut 后缀)
- [ ] 添加事件处理器
- [ ] 创建拖放策略
- [ ] 导出策略
- [ ] 更新组件

**测试验证**：

- [ ] 无 linter 错误
- [ ] 指令调用成功
- [ ] Store 状态正确更新
- [ ] 拖放功能正常
- [ ] SSE 事件正确触发
- [ ] 日志输出正常
