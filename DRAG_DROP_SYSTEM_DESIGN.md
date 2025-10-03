# Cutie 拖放系统技术设计文档

## 文档信息

- **版本**: 1.0
- **日期**: 2025-10-03
- **状态**: 设计阶段

## 1. 背景与问题陈述

### 1.1 业务需求

Cutie 任务管理系统需要支持灵活的拖放操作，允许用户在不同视图（看板）之间移动任务。关键业务场景包括：

1. **看板内排序**: 在同一看板内调整任务顺序
2. **跨看板转移**: 将任务从一个看板拖到另一个看板（如 Staging → Planned）
3. **任务排期**: 将任务拖到日历创建时间块
4. **动态看板**: 支持按日期、项目、优先级等动态生成的看板
5. **多种拖放模式**: 传统拖放 + 吸附式拖放（点击激活 → 移动 → 点击定位）

### 1.2 核心挑战

**业务逻辑的四维决策模型**：

```
业务逻辑 = f(任务信息, 源看板, 目标看板, 拖放模式)
```

- **任务信息**: 任务的当前状态、属性（如 `scheduled_date`, `project_id`, `status`）
- **源看板**: 来源看板的类型和配置（如 `{type: 'status', config: {status: 'staging'}}`）
- **目标看板**: 目标看板的类型和配置（如 `{type: 'date', config: {date: '2025-10-03'}}`）
- **拖放模式**: 拖放的交互方式（普通拖放 vs 吸附式拖放）

### 1.3 技术约束

1. **跨组件通信**: 源看板和目标看板可能在不同的父组件中
2. **动态性**: 看板类型和数量不固定，可按需生成
3. **扩展性**: 未来可能增加新的看板类型和拖放模式
4. **性能**: 大量看板（如 365 个日期看板）时仍需流畅
5. **类型安全**: 需要完整的 TypeScript 类型支持

## 2. 技术方案评估

### 2.1 方案对比

我们评估了两种底层技术方案：

| 维度               | HTML5 DragEvent            | vue-draxis                            |
| ------------------ | -------------------------- | ------------------------------------- |
| **跨组件数据传递** | ✅ `dataTransfer` 原生支持 | ✅ 全局状态 `dragManager.state`       |
| **稳定性**         | ✅ 浏览器原生，20年成熟    | ❌ 存在多个 bug（竞态条件、内存泄漏） |
| **性能**           | ✅ 浏览器优化              | ⚠️ `getComputedStyle` 滥用导致卡顿    |
| **移动端支持**     | ⚠️ 需要 polyfill           | ✅ 基于 PointerEvent                  |
| **学习曲线**       | ✅ 标准 API，文档完善      | ⚠️ 需要学习框架                       |
| **代码重复**       | ⚠️ 需要手动封装            | ✅ 框架提供抽象                       |
| **自定义 Ghost**   | ❌ 受限（仅图片）          | ✅ Vue 组件                           |
| **自动滚动**       | ❌ 需要手动实现            | ✅ 内置支持                           |
| **多模式拖放**     | ⚠️ 需要自己实现            | ⚠️ 需要自己实现                       |

### 2.2 vue-draxis 的致命缺陷

经过深入审查，vue-draxis 存在以下严重问题：

#### 2.2.1 内存泄漏

```typescript
// c-draggable 指令的 beforeUpdate
beforeUpdate(el, binding) {
  // ❌ 每次更新都重新创建闭包
  const { startDrag } = useDraggable(newOptions)

  // ❌ JSON.stringify 在高频更新时性能极差
  JSON.stringify(oldOptions.ghostProps) === JSON.stringify(newOptions?.ghostProps)
}
```

**影响**: 在 v-for 渲染的 50 个任务卡片场景下，每次列表更新会创建 50 个新闭包，导致内存持续增长。

#### 2.2.2 竞态条件

```typescript
startDragByEvent(options, event) {
  if (state.value.isDragging || state.value.isPreparing) return

  manager.endDrag()  // ⚠️ 清理旧状态

  // ❌ 快速双击时，两次调用可能交错执行
  state.value = { isPreparing: true, ... }
  document.addEventListener('pointermove', moveListener)
}
```

**影响**: 快速操作时可能导致拖拽卡死、Ghost 元素残留。

#### 2.2.3 性能问题

```typescript
function findScrollableContainer(element) {
  while (current && current !== document.body) {
    const computedStyle = window.getComputedStyle(current) // ⚠️ 强制重排
    // 在 handlePointerMove 中每 16ms 调用
  }
}
```

**影响**: 深层 DOM 嵌套时拖拽卡顿明显。

### 2.3 推荐方案

**采用 HTML5 DragEvent + 自定义抽象层**

**理由**:

1. ✅ **稳定性优先**: 浏览器原生 API 经过 20 年打磨，无已知 bug
2. ✅ **性能可控**: 我们完全掌控优化点
3. ✅ **渐进增强**: 从轻量工具包开始，按需扩展
4. ✅ **类型安全**: 可以设计完整的 TypeScript 类型系统
5. ✅ **可维护性**: 代码在我们掌控之下，不依赖有 bug 的第三方库

**代价**:

- ⚠️ 需要手动实现自动滚动、Ghost 组件等功能
- ⚠️ 移动端需要额外处理（但 Cutie 是桌面应用，优先级低）

## 3. 架构设计

### 3.1 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                      应用层                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │ KanbanColumn │  │ CuteCalendar │  │ ProjectBoard │      │
│  │  (源/目标)   │  │   (目标)     │  │   (源/目标)  │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                  │               │
│         └─────────────────┼──────────────────┘               │
│                           ↓                                  │
├─────────────────────────────────────────────────────────────┤
│                   业务协调层 (Composable)                    │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  useDragDrop (轻量工具包)                             │  │
│  │  - useDragTransfer: 数据传递                          │  │
│  │  - useAutoScroll: 自动滚动                            │  │
│  │  - useThrottledDragOver: 节流                         │  │
│  │  - useDragState: 状态管理                             │  │
│  └───────────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  useCrossViewDrag (跨看板协调)                        │  │
│  │  - 元数据管理: ViewMetadata                           │  │
│  │  - 模式管理: DragMode                                 │  │
│  │  - 策略路由: dragStrategies                           │  │
│  │  - 状态追踪: currentDragContext                       │  │
│  └───────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                      策略层                                  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  DragStrategy                                          │  │
│  │  - 'status->date': setScheduledDate                    │  │
│  │  - 'date->date': changeScheduledDate                   │  │
│  │  - 'project->project': changeProject                   │  │
│  │  - '*->calendar': createTimeBlock                      │  │
│  │  - 每个策略根据 DragMode 调整行为                     │  │
│  └───────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                   数据访问层 (Pinia Stores)                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │  TaskStore   │  │ TimeBlockStore│  │  ViewStore   │      │
│  │ updateTask() │  │  createBlock()│  │ updateSort() │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
├─────────────────────────────────────────────────────────────┤
│                   底层传输 (HTML5 DragEvent)                 │
│  - dataTransfer: 跨组件数据传递                             │
│  - dragstart, dragover, drop: DOM 事件                      │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 类型系统设计

#### 3.2.1 核心类型定义

```typescript
/**
 * 看板元数据
 * 描述看板的类型、配置和身份
 */
interface ViewMetadata {
  /** 看板类型 */
  type: 'status' | 'date' | 'project' | 'priority' | 'area' | 'custom'

  /** 唯一标识符 */
  id: string

  /** 类型特定的配置 */
  config: ViewConfig

  /** 可选：显示名称 */
  label?: string

  /** 可选：图标 */
  icon?: string
}

/**
 * 看板配置（联合类型）
 */
type ViewConfig =
  | StatusViewConfig
  | DateViewConfig
  | ProjectViewConfig
  | PriorityViewConfig
  | AreaViewConfig
  | CustomViewConfig

interface StatusViewConfig {
  status: 'staging' | 'planned' | 'completed'
}

interface DateViewConfig {
  /** ISO 8601 格式 */
  date: string
}

interface ProjectViewConfig {
  projectId: string
  projectName: string
}

interface PriorityViewConfig {
  priority: 'high' | 'medium' | 'low'
}

interface AreaViewConfig {
  areaId: string
  areaName: string
  color: string
}

interface CustomViewConfig {
  filter: (task: TaskCard) => boolean
  metadata: Record<string, any>
}

/**
 * 拖放模式
 */
type DragMode = 'normal' | 'snap'

interface NormalDragMode {
  mode: 'normal'
}

interface SnapDragMode {
  mode: 'snap'
  /** 激活按钮的上下文 */
  activatedBy: string
  /** 额外的模式参数 */
  params?: Record<string, any>
}

/**
 * 拖拽上下文
 * 携带拖拽过程中的所有信息
 */
interface DragContext {
  /** 被拖拽的任务 */
  task: TaskCard

  /** 源看板元数据 */
  sourceView: ViewMetadata

  /** 拖放模式 */
  dragMode: NormalDragMode | SnapDragMode

  /** 拖拽开始时间（用于性能追踪） */
  startTime: number

  /** 附加数据（可选） */
  metadata?: Record<string, any>
}

/**
 * 策略执行结果
 */
interface StrategyResult {
  /** 是否成功 */
  success: boolean

  /** 错误信息 */
  error?: string

  /** 是否仅重排序（不修改任务数据） */
  reorderOnly?: boolean

  /** 需要更新的视图列表 */
  affectedViews?: string[]

  /** 用户提示消息 */
  message?: string
}

/**
 * 拖放策略函数
 */
type DragStrategy = (context: DragContext, targetView: ViewMetadata) => Promise<StrategyResult>
```

#### 3.2.2 策略映射表类型

```typescript
/**
 * 策略键：source.type -> target.type
 */
type StrategyKey =
  | `${ViewMetadata['type']}->${ViewMetadata['type']}`
  | '*->*'
  | `${ViewMetadata['type']}>*`
  | `*->${ViewMetadata['type']}`

/**
 * 策略注册表
 */
type StrategyRegistry = {
  [key in StrategyKey]?: DragStrategy
}
```

### 3.3 数据流设计

#### 3.3.1 普通拖放流程

```
1. 用户按下鼠标（dragstart）
   ├─ 组件: 收集上下文
   │  └─ context = {
   │       task,
   │       sourceView: this.viewMetadata,
   │       dragMode: { mode: 'normal' }
   │     }
   ├─ HTML5: setData('application/json', context)
   └─ Composable: setDragContext(context)

2. 用户拖动鼠标（dragover）
   ├─ HTML5: 触发目标元素的 dragover 事件
   ├─ 组件: 节流更新视觉反馈
   │  ├─ 高亮目标看板
   │  ├─ 显示预览位置
   │  └─ 显示提示文字（如"放置后将设置排期"）
   └─ Composable: handleAutoScroll()

3. 用户松手（drop）
   ├─ 组件: 读取上下文
   │  └─ context = getDragContext() || parseDataTransfer(event)
   ├─ Composable: 查找策略
   │  └─ strategy = findStrategy(
   │       context.sourceView.type,
   │       targetView.type,
   │       context.dragMode
   │     )
   ├─ 策略层: 执行业务逻辑
   │  └─ result = await strategy(context, targetView)
   ├─ Store 层: 持久化数据
   │  └─ TaskStore.updateTask(...)
   └─ 组件: 显示反馈
      ├─ 成功: Toast 提示
      └─ 失败: 回滚 + 错误提示

4. 清理（dragend）
   ├─ Composable: clearDragContext()
   └─ 组件: 清除视觉状态
```

#### 3.3.2 吸附式拖放流程

```
1. 用户点击激活按钮
   ├─ 组件: 进入吸附模式
   │  └─ snapContext = {
   │       task,
   │       sourceView: this.viewMetadata,
   │       dragMode: { mode: 'snap', activatedBy: 'schedule-button' }
   │     }
   ├─ Composable: startSnapMode(snapContext)
   └─ UI: 显示吸附状态
      ├─ 改变鼠标光标（crosshair）
      ├─ 高亮所有可放置区域
      └─ 显示提示文字（如"点击日期设置排期"）

2. 用户移动鼠标（无拖拽）
   ├─ 组件: 监听 mousemove
   ├─ Composable: 检测 hover 的目标
   │  └─ canDrop(snapContext.sourceView, hoveredView)
   └─ UI: 预览放置效果
      ├─ 高亮当前目标
      └─ 显示"虚影"预览

3. 用户点击定位
   ├─ 组件: 触发 click 事件
   ├─ Composable: 执行策略（同普通拖放）
   └─ UI: 退出吸附模式

4. 用户取消（ESC 或右键）
   ├─ Composable: cancelSnapMode()
   └─ UI: 恢复正常状态
```

### 3.4 策略设计

#### 3.4.1 策略注册表结构

```typescript
const dragStrategies: StrategyRegistry = {
  // ========== 同类型看板之间 ==========

  'status->status': async (context, targetView) => {
    const sourceConfig = context.sourceView.config as StatusViewConfig
    const targetConfig = targetView.config as StatusViewConfig

    // 特殊情况：staging -> planned
    if (sourceConfig.status === 'staging' && targetConfig.status === 'planned') {
      // 根据拖放模式决定行为
      if (context.dragMode.mode === 'snap') {
        // 吸附模式：可能需要额外的日期选择
        const date = await promptDateSelection()
        await taskStore.updateTask(context.task.id, { scheduled_date: date })
      } else {
        // 普通拖放：使用今天
        await taskStore.updateTask(context.task.id, { scheduled_date: getTodayISO() })
      }

      return {
        success: true,
        message: '已设置排期',
        affectedViews: ['staging', 'planned'],
      }
    }

    // 默认：仅重排序
    return { success: true, reorderOnly: true }
  },

  'date->date': async (context, targetView) => {
    const targetDate = (targetView.config as DateViewConfig).date

    await taskStore.updateTask(context.task.id, {
      scheduled_date: targetDate,
    })

    return {
      success: true,
      message: `已改期至 ${formatDate(targetDate)}`,
      affectedViews: [context.sourceView.id, targetView.id],
    }
  },

  'project->project': async (context, targetView) => {
    const targetProjectId = (targetView.config as ProjectViewConfig).projectId

    // 检查权限：已完成的任务不能移动项目
    if (context.task.status === 'completed') {
      return {
        success: false,
        error: '已完成的任务不能移动到其他项目',
      }
    }

    await taskStore.updateTask(context.task.id, {
      project_id: targetProjectId,
    })

    return {
      success: true,
      message: `已移动到项目 ${(targetView.config as ProjectViewConfig).projectName}`,
      affectedViews: [context.sourceView.id, targetView.id],
    }
  },

  // ========== 跨类型拖放 ==========

  'status->date': async (context, targetView) => {
    const targetDate = (targetView.config as DateViewConfig).date

    await taskStore.updateTask(context.task.id, {
      scheduled_date: targetDate,
    })

    return {
      success: true,
      message: `已设置排期：${formatDate(targetDate)}`,
      affectedViews: [context.sourceView.id, targetView.id],
    }
  },

  'date->status': async (context, targetView) => {
    const targetStatus = (targetView.config as StatusViewConfig).status

    // 拖回 staging：取消排期
    if (targetStatus === 'staging') {
      await taskStore.updateTask(context.task.id, {
        scheduled_date: null,
      })

      return {
        success: true,
        message: '已取消排期',
        affectedViews: [context.sourceView.id, targetView.id],
      }
    }

    // 其他状态看板：仅重排序
    return { success: true, reorderOnly: true }
  },

  // ========== 特殊目标：日历 ==========

  '*->calendar': async (context, targetView) => {
    // 日历的 config 包含时间信息
    const calendarConfig = targetView.config as any

    const result = await timeBlockStore.createTimeBlockFromTask({
      task_id: context.task.id,
      start_time: calendarConfig.startTime,
      end_time: calendarConfig.endTime,
    })

    if (!result) {
      return {
        success: false,
        error: '创建时间块失败（可能时间重叠）',
      }
    }

    // 更新任务的 scheduled_date
    const date = extractDateFromISO(calendarConfig.startTime)
    await taskStore.updateTask(context.task.id, {
      scheduled_date: date,
    })

    return {
      success: true,
      message: '已创建时间块',
      affectedViews: [context.sourceView.id, 'calendar'],
    }
  },

  // ========== 默认处理 ==========

  '*->*': async () => {
    return {
      success: false,
      error: '不支持此拖放操作',
    }
  },
}
```

#### 3.4.2 策略查找算法

```typescript
function findStrategy(
  sourceType: ViewMetadata['type'],
  targetType: ViewMetadata['type'],
  dragMode: DragMode['mode']
): DragStrategy {
  // 1. 精确匹配（考虑模式）
  const exactKey = `${sourceType}->${targetType}` as StrategyKey
  if (dragStrategies[exactKey]) {
    return wrapStrategyWithMode(dragStrategies[exactKey]!, dragMode)
  }

  // 2. 源通配符
  const sourceWildcard = `${sourceType}->*` as StrategyKey
  if (dragStrategies[sourceWildcard]) {
    return wrapStrategyWithMode(dragStrategies[sourceWildcard]!, dragMode)
  }

  // 3. 目标通配符
  const targetWildcard = `*->${targetType}` as StrategyKey
  if (dragStrategies[targetWildcard]) {
    return wrapStrategyWithMode(dragStrategies[targetWildcard]!, dragMode)
  }

  // 4. 默认策略
  return dragStrategies['*->*']!
}

/**
 * 包装策略以支持不同的拖放模式
 */
function wrapStrategyWithMode(baseStrategy: DragStrategy, mode: DragMode['mode']): DragStrategy {
  return async (context, targetView) => {
    // 模式特定的前置处理
    if (mode === 'snap') {
      // 吸附模式可能需要额外确认
      const confirmed = await confirmSnapDrop(context, targetView)
      if (!confirmed) {
        return { success: false, error: '用户取消操作' }
      }
    }

    // 执行基础策略
    const result = await baseStrategy(context, targetView)

    // 模式特定的后置处理
    if (mode === 'snap' && result.success) {
      // 吸附模式成功后退出吸附状态
      exitSnapMode()
    }

    return result
  }
}
```

## 4. 实施方案

### 4.1 模块结构

```
src/composables/drag/
├── index.ts                    # 统一导出
├── types.ts                    # 类型定义
├── useDragTransfer.ts          # 数据传递工具
├── useAutoScroll.ts            # 自动滚动
├── useThrottledDragOver.ts     # 节流
├── useDragState.ts             # 状态管理
├── useCrossViewDrag/
│   ├── index.ts                # 跨看板拖放主入口
│   ├── context.ts              # 拖拽上下文管理
│   ├── strategies.ts           # 策略注册表
│   ├── finder.ts               # 策略查找
│   └── modes/
│       ├── normal.ts           # 普通拖放模式
│       └── snap.ts             # 吸附式拖放模式
└── README.md                   # 使用文档
```

### 4.2 核心 API 设计

#### 4.2.1 轻量工具包 API

```typescript
// useDragTransfer.ts
export function useDragTransfer() {
  function setDragData(event: DragEvent, data: any): void
  function getDragData(event: DragEvent): any | null
  function clearDragData(event: DragEvent): void

  return { setDragData, getDragData, clearDragData }
}

// useAutoScroll.ts
export function useAutoScroll(options?: AutoScrollOptions) {
  function startAutoScroll(container: HTMLElement, direction: number): void
  function stopAutoScroll(): void
  function handleAutoScroll(event: DragEvent, container?: HTMLElement): void

  return { startAutoScroll, stopAutoScroll, handleAutoScroll }
}

// useThrottledDragOver.ts
export function useThrottledDragOver<T>(
  callback: (event: DragEvent, ...args: T[]) => void,
  delay?: number
): (event: DragEvent, ...args: T[]) => void

// useDragState.ts
export function useDragState<T = any>() {
  const isDragging: Ref<boolean>
  const draggedItem: Ref<T | null>

  function startDrag(item: T): void
  function endDrag(): void

  return { isDragging, draggedItem, startDrag, endDrag }
}
```

#### 4.2.2 跨看板拖放 API

```typescript
// useCrossViewDrag/index.ts
export function useCrossViewDrag() {
  /**
   * 当前拖拽上下文（只读）
   */
  const currentContext: Readonly<Ref<DragContext | null>>

  /**
   * 是否处于拖拽状态
   */
  const isDragging: Readonly<Ref<boolean>>

  /**
   * 当前拖放模式
   */
  const currentMode: Readonly<Ref<DragMode['mode']>>

  /**
   * 开始普通拖放
   */
  function startNormalDrag(task: TaskCard, sourceView: ViewMetadata): void

  /**
   * 开始吸附式拖放
   */
  function startSnapDrag(
    task: TaskCard,
    sourceView: ViewMetadata,
    activatedBy: string,
    params?: Record<string, any>
  ): void

  /**
   * 处理放置
   */
  async function handleDrop(targetView: ViewMetadata, event?: DragEvent): Promise<StrategyResult>

  /**
   * 检查是否可以放置
   */
  function canDrop(sourceView: ViewMetadata, targetView: ViewMetadata): boolean

  /**
   * 获取放置提示文字
   */
  function getDropHint(sourceView: ViewMetadata, targetView: ViewMetadata): string

  /**
   * 取消拖放
   */
  function cancelDrag(): void

  /**
   * 注册自定义策略
   */
  function registerStrategy(key: StrategyKey, strategy: DragStrategy): void

  return {
    // 状态
    currentContext,
    isDragging,
    currentMode,

    // 操作
    startNormalDrag,
    startSnapDrag,
    handleDrop,
    canDrop,
    getDropHint,
    cancelDrag,

    // 扩展
    registerStrategy,
  }
}
```

### 4.3 组件集成示例

#### 4.3.1 看板组件（源和目标）

```vue
<script setup lang="ts">
import { useDragTransfer, useCrossViewDrag } from '@/composables/drag'
import type { ViewMetadata, TaskCard } from '@/types'

const props = defineProps<{
  viewMetadata: ViewMetadata
  tasks: TaskCard[]
}>()

const emit = defineEmits<{
  taskMoved: [taskId: string]
}>()

const { setDragData, getDragData } = useDragTransfer()
const crossViewDrag = useCrossViewDrag()

// ========== 普通拖放 ==========

function handleDragStart(event: DragEvent, task: TaskCard) {
  if (!event.dataTransfer) return

  // 1. 设置拖拽上下文
  crossViewDrag.startNormalDrag(task, props.viewMetadata)

  // 2. 设置 HTML5 数据（跨组件兼容）
  setDragData(event, {
    type: 'task',
    task,
    sourceView: props.viewMetadata,
  })

  // 3. 视觉反馈
  if (event.target instanceof HTMLElement) {
    event.target.style.opacity = '0.5'
  }
}

function handleDragEnd(event: DragEvent) {
  // 恢复视觉
  if (event.target instanceof HTMLElement) {
    event.target.style.opacity = '1'
  }

  // 清理会由 crossViewDrag 自动处理
}

const isValidDropTarget = ref(false)
const dropHint = ref('')

function handleDragOver(event: DragEvent) {
  event.preventDefault()

  // 检查是否可以放置
  const context = crossViewDrag.currentContext.value
  if (!context) return

  isValidDropTarget.value = crossViewDrag.canDrop(context.sourceView, props.viewMetadata)

  if (isValidDropTarget.value) {
    dropHint.value = crossViewDrag.getDropHint(context.sourceView, props.viewMetadata)
  }
}

function handleDragLeave() {
  isValidDropTarget.value = false
  dropHint.value = ''
}

async function handleDrop(event: DragEvent) {
  event.preventDefault()

  const result = await crossViewDrag.handleDrop(props.viewMetadata, event)

  if (result.success) {
    // 成功提示
    showToast(result.message || '操作成功')
    emit('taskMoved', result.task?.id)
  } else {
    // 错误提示
    showError(result.error || '操作失败')
  }

  // 清理视觉状态
  isValidDropTarget.value = false
  dropHint.value = ''
}

// ========== 吸附式拖放 ==========

function handleActivateSnapDrag(task: TaskCard) {
  crossViewDrag.startSnapDrag(task, props.viewMetadata, 'schedule-button')
}

// 如果处于吸附模式，监听点击
watch(
  () => crossViewDrag.currentMode.value,
  (mode) => {
    if (mode === 'snap') {
      // 添加 ESC 取消监听
      document.addEventListener('keydown', handleEscapeKey)
    } else {
      document.removeEventListener('keydown', handleEscapeKey)
    }
  }
)

function handleEscapeKey(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    crossViewDrag.cancelDrag()
  }
}

// 吸附模式下的点击处理
async function handleSnapClick(event: MouseEvent) {
  if (crossViewDrag.currentMode.value !== 'snap') return

  const result = await crossViewDrag.handleDrop(props.viewMetadata)

  if (result.success) {
    showToast(result.message || '已设置')
  } else {
    showError(result.error || '操作失败')
  }
}
</script>

<template>
  <div
    class="kanban-column"
    :class="{
      'is-snap-mode': crossViewDrag.currentMode.value === 'snap',
      'is-valid-target': isValidDropTarget,
    }"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
    @click="handleSnapClick"
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
      <TaskCard :task="task">
        <!-- 吸附拖放激活按钮 -->
        <button @click.stop="handleActivateSnapDrag(task)" class="snap-drag-trigger">
          📅 设置排期
        </button>
      </TaskCard>
    </div>
  </div>
</template>

<style scoped>
.is-snap-mode {
  cursor: crosshair;
}

.is-valid-target {
  background-color: rgba(74, 144, 226, 0.1);
  border: 2px dashed #4a90e2;
}

.drop-hint {
  position: absolute;
  top: 1rem;
  left: 50%;
  transform: translateX(-50%);
  padding: 0.5rem 1rem;
  background: rgba(74, 144, 226, 0.9);
  color: white;
  border-radius: 4px;
  font-size: 1.2rem;
  pointer-events: none;
  z-index: 100;
}
</style>
```

#### 4.3.2 日历组件（特殊目标）

```vue
<script setup lang="ts">
import { useCrossViewDrag } from '@/composables/drag'
import type { ViewMetadata } from '@/types'

const crossViewDrag = useCrossViewDrag()

// 日历的 ViewMetadata 是动态生成的
function getCalendarViewMetadata(timeSlot: Date): ViewMetadata {
  return {
    type: 'calendar',
    id: `calendar-${timeSlot.toISOString()}`,
    config: {
      startTime: timeSlot.toISOString(),
      endTime: new Date(timeSlot.getTime() + 60 * 60 * 1000).toISOString(),
    },
    label: `${formatTime(timeSlot)} - ${formatTime(new Date(timeSlot.getTime() + 60 * 60 * 1000))}`,
  }
}

// 处理放置到时间槽
async function handleDropOnTimeSlot(event: DragEvent, timeSlot: Date) {
  event.preventDefault()

  const viewMetadata = getCalendarViewMetadata(timeSlot)
  const result = await crossViewDrag.handleDrop(viewMetadata, event)

  if (result.success) {
    showToast('已创建时间块')
  } else {
    showError(result.error || '创建失败')
  }
}

// 吸附模式下的点击
async function handleSnapClickOnTimeSlot(event: MouseEvent, timeSlot: Date) {
  if (crossViewDrag.currentMode.value !== 'snap') return

  const viewMetadata = getCalendarViewMetadata(timeSlot)
  const result = await crossViewDrag.handleDrop(viewMetadata)

  if (result.success) {
    showToast('已创建时间块')
  }
}
</script>

<template>
  <div class="calendar" :class="{ 'is-snap-mode': crossViewDrag.currentMode.value === 'snap' }">
    <div
      v-for="timeSlot in timeSlots"
      :key="timeSlot.toISOString()"
      class="time-slot"
      @drop="handleDropOnTimeSlot($event, timeSlot)"
      @click="handleSnapClickOnTimeSlot($event, timeSlot)"
    >
      {{ formatTime(timeSlot) }}
    </div>
  </div>
</template>
```

### 4.4 实施步骤

#### 阶段 1: 轻量工具包（Week 1）

**目标**: 封装 HTML5 拖放的通用功能，减少代码重复

**任务**:

1. ✅ 实现 `useDragTransfer` - 数据传递工具 (2h)
2. ✅ 实现 `useAutoScroll` - 自动滚动 (3h)
3. ✅ 实现 `useThrottledDragOver` - 节流 (1h)
4. ✅ 实现 `useDragState` - 状态管理 (1h)
5. ✅ 编写单元测试 (3h)
6. ✅ 编写使用文档 (2h)

**验收标准**:

- 所有工具函数通过单元测试
- 在 `SimpleKanbanColumn` 中成功集成，代码量减少 30%

#### 阶段 2: 元数据系统（Week 2）

**目标**: 定义看板元数据类型和生成逻辑

**任务**:

1. ✅ 定义 `ViewMetadata` 类型系统 (2h)
2. ✅ 实现看板元数据生成函数 (3h)
3. ✅ 修改现有看板组件添加元数据 prop (4h)
4. ✅ 编写类型测试 (2h)

**验收标准**:

- 所有看板组件都能生成正确的 `ViewMetadata`
- TypeScript 类型检查通过

#### 阶段 3: 策略系统（Week 3）

**目标**: 实现业务逻辑路由和策略执行

**任务**:

1. ✅ 实现策略注册表 (4h)
2. ✅ 实现策略查找算法 (3h)
3. ✅ 实现内置策略（status, date, project 等） (6h)
4. ✅ 编写策略单元测试 (4h)

**验收标准**:

- 所有策略通过单元测试
- 策略查找性能 < 1ms

#### 阶段 4: 跨看板拖放核心（Week 4）

**目标**: 实现 `useCrossViewDrag` composable

**任务**:

1. ✅ 实现上下文管理 (3h)
2. ✅ 集成策略系统 (3h)
3. ✅ 实现普通拖放模式 (4h)
4. ✅ 实现 `canDrop` 和 `getDropHint` (2h)
5. ✅ 编写集成测试 (4h)

**验收标准**:

- 跨看板拖放功能正常
- 所有业务场景通过测试

#### 阶段 5: 吸附式拖放（Week 5）

**目标**: 实现吸附式拖放模式

**任务**:

1. ✅ 实现吸附模式状态管理 (3h)
2. ✅ 实现吸附模式 UI 反馈 (4h)
3. ✅ 实现吸附模式取消机制（ESC, 右键） (2h)
4. ✅ 集成到看板和日历组件 (4h)
5. ✅ 编写用户体验测试 (3h)

**验收标准**:

- 吸附模式交互流畅
- 用户测试满意度 > 90%

#### 阶段 6: 优化和测试（Week 6）

**目标**: 性能优化和全面测试

**任务**:

1. ✅ 性能分析和优化 (4h)
2. ✅ 边界情况测试 (4h)
3. ✅ 用户验收测试 (4h)
4. ✅ 编写开发者文档 (4h)

**验收标准**:

- 拖放延迟 < 16ms（60fps）
- 所有边界情况覆盖
- 文档完整

## 5. 性能优化

### 5.1 节流和防抖

```typescript
// 拖拽过程中的高频事件节流
const DRAG_OVER_THROTTLE = 16 // ~60fps
const AUTO_SCROLL_THROTTLE = 16

// 策略查找结果缓存
const strategyCache = new Map<string, DragStrategy>()

function findStrategyCached(sourceType: string, targetType: string): DragStrategy {
  const key = `${sourceType}->${targetType}`

  if (strategyCache.has(key)) {
    return strategyCache.get(key)!
  }

  const strategy = findStrategy(sourceType, targetType)
  strategyCache.set(key, strategy)

  return strategy
}
```

### 5.2 虚拟滚动优化

对于大量看板（如 365 个日期看板）的场景：

```typescript
// 只渲染可见的看板
import { useVirtualList } from '@vueuse/core'

const {
  list: visibleViews,
  containerProps,
  wrapperProps,
} = useVirtualList(allViews, {
  itemHeight: 300,
  overscan: 5,
})
```

### 5.3 内存管理

```typescript
// 组件卸载时清理
onUnmounted(() => {
  // 清理拖拽状态
  crossViewDrag.cancelDrag()

  // 清理事件监听
  document.removeEventListener('keydown', handleEscapeKey)

  // 清理策略缓存（如果是动态注册的）
  clearCustomStrategies()
})
```

## 6. 测试策略

### 6.1 单元测试

```typescript
describe('useCrossViewDrag', () => {
  describe('策略查找', () => {
    it('应该找到精确匹配的策略', () => {
      const strategy = findStrategy('status', 'date', 'normal')
      expect(strategy).toBeDefined()
    })

    it('应该回退到通配符策略', () => {
      const strategy = findStrategy('custom', 'custom', 'normal')
      expect(strategy).toBe(dragStrategies['*->*'])
    })
  })

  describe('状态管理', () => {
    it('应该正确设置拖拽上下文', () => {
      const task = createMockTask()
      const view = createMockView('status')

      crossViewDrag.startNormalDrag(task, view)

      expect(crossViewDrag.currentContext.value).toEqual({
        task,
        sourceView: view,
        dragMode: { mode: 'normal' },
      })
    })

    it('应该在 drop 后清理上下文', async () => {
      await crossViewDrag.handleDrop(targetView)

      expect(crossViewDrag.currentContext.value).toBeNull()
    })
  })
})
```

### 6.2 集成测试

```typescript
describe('跨看板拖放集成测试', () => {
  it('应该成功将任务从 staging 拖到 planned', async () => {
    // 准备
    const task = await createTask({ title: 'Test Task' })
    const stagingView = { type: 'status', config: { status: 'staging' } }
    const plannedView = { type: 'status', config: { status: 'planned' } }

    // 执行
    crossViewDrag.startNormalDrag(task, stagingView)
    const result = await crossViewDrag.handleDrop(plannedView)

    // 验证
    expect(result.success).toBe(true)

    const updatedTask = await getTask(task.id)
    expect(updatedTask.scheduled_date).toBeTruthy()
  })

  it('应该阻止不允许的拖放操作', async () => {
    const completedTask = await createTask({ status: 'completed' })
    const result = await crossViewDrag.handleDrop(projectView)

    expect(result.success).toBe(false)
    expect(result.error).toContain('已完成的任务')
  })
})
```

### 6.3 E2E 测试

```typescript
describe('拖放 E2E 测试', () => {
  it('用户可以拖动任务到日历创建时间块', async () => {
    await page.goto('/home')

    // 拖动任务
    const task = await page.locator('[data-task-id="123"]')
    await task.dragTo('[data-calendar-slot="2025-10-03T10:00"]')

    // 验证时间块创建
    await expect(page.locator('[data-time-block]')).toBeVisible()

    // 验证任务更新
    const updatedTask = await page.locator('[data-task-id="123"]')
    await expect(updatedTask).toHaveAttribute('data-scheduled-date', '2025-10-03')
  })

  it('用户可以使用吸附模式设置排期', async () => {
    // 激活吸附模式
    await page.click('[data-snap-trigger="123"]')

    // 验证进入吸附状态
    await expect(page.locator('body')).toHaveClass(/snap-mode/)

    // 点击日期
    await page.click('[data-date-view="2025-10-05"]')

    // 验证任务更新
    const task = await getTaskFromDB('123')
    expect(task.scheduled_date).toBe('2025-10-05')
  })
})
```

## 7. 未来扩展

### 7.1 批量拖放

支持同时拖动多个任务：

```typescript
interface BatchDragContext extends DragContext {
  tasks: TaskCard[] // 多个任务
}

// 策略需要支持批量操作
type BatchDragStrategy = (
  context: BatchDragContext,
  targetView: ViewMetadata
) => Promise<StrategyResult[]>
```

### 7.2 拖放历史和撤销

```typescript
interface DragHistory {
  timestamp: number
  context: DragContext
  targetView: ViewMetadata
  result: StrategyResult
  previousState: any // 用于撤销
}

function useDragHistory() {
  const history: Ref<DragHistory[]>

  function undo(): Promise<void>
  function redo(): Promise<void>

  return { history, undo, redo }
}
```

### 7.3 拖放分析

收集用户行为数据优化 UX：

```typescript
interface DragAnalytics {
  totalDrags: number
  successRate: number
  averageDuration: number
  popularRoutes: Map<StrategyKey, number>
  errorReasons: Map<string, number>
}

function trackDragEvent(context: DragContext, result: StrategyResult): void
```

### 7.4 插件系统

允许第三方扩展拖放功能：

```typescript
interface DragPlugin {
  name: string
  version: string

  // 生命周期钩子
  onDragStart?: (context: DragContext) => void
  onDragEnd?: (result: StrategyResult) => void

  // 注册自定义策略
  registerStrategies?: () => Record<StrategyKey, DragStrategy>

  // 注册自定义看板类型
  registerViewTypes?: () => ViewMetadata['type'][]
}

function installDragPlugin(plugin: DragPlugin): void
```

## 8. 总结

### 8.1 核心决策

1. **技术选型**: HTML5 DragEvent + 自定义抽象层
   - 稳定性优先，避免 vue-draxis 的 bug
   - 完全掌控，满足复杂业务需求

2. **架构设计**: 分层架构 + 策略模式
   - 清晰的职责分离
   - 高度可扩展

3. **类型系统**: 完整的 TypeScript 支持
   - 编译时类型检查
   - 优秀的 IDE 提示

### 8.2 关键优势

- ✅ **四维决策模型**: 完整支持 `f(task, source, target, mode)`
- ✅ **动态看板**: 支持无限数量的动态生成看板
- ✅ **跨组件**: 无缝支持跨父组件拖放
- ✅ **多模式**: 普通拖放 + 吸附式拖放
- ✅ **类型安全**: 完整的 TypeScript 类型系统
- ✅ **可扩展**: 策略模式易于扩展
- ✅ **高性能**: 优化的事件处理和缓存

### 8.3 实施时间线

- **阶段 1-2**: 2 周（基础工具 + 元数据）
- **阶段 3-4**: 2 周（策略 + 核心功能）
- **阶段 5-6**: 2 周（吸附模式 + 优化）
- **总计**: 6 周（约 1.5 个月）

### 8.4 风险评估

| 风险           | 严重度 | 缓解措施                |
| -------------- | ------ | ----------------------- |
| HTML5 API 限制 | 低     | 已充分研究，满足需求    |
| 性能问题       | 中     | 节流、缓存、虚拟滚动    |
| 跨浏览器兼容性 | 低     | Tauri 固定 WebView 版本 |
| 复杂度管理     | 中     | 清晰架构、完整测试      |
| 用户体验       | 中     | 多次用户测试迭代        |

---

**审批签字**:

- 技术负责人: ******\_\_\_****** 日期: **\_\_\_**
- 产品负责人: ******\_\_\_****** 日期: **\_\_\_**
- 项目经理: ******\_\_\_****** 日期: **\_\_\_**
