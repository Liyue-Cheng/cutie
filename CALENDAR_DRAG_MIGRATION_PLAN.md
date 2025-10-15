# 日历拖放迁移方案

**日期**: 2025-10-15  
**目标**: 将日历拖放从 HTML5 DnD + 旧策略系统迁移到 interact.js + 新策略系统  
**状态**: 📋 规划中

---

## 📊 现有实现分析

### 1. 当前架构

```
CuteCalendar.vue
  ↓
useCalendarDrag (composable)
  ├── 监听全局 dragstart/dragend
  ├── handleDragEnter/Over/Leave/Drop
  ├── 计算预览时间块 (previewEvent)
  ├── 处理全日/分时事件
  └── 调用 useCrossViewDrag.handleDrop
      ↓
useCrossViewDrag/strategies.ts
  └── anyToCalendar 策略
      └── 调用 timeBlockStore.createTimeBlock
```

### 2. 关键功能特性

#### A. **预览功能**

- ✅ 全日预览（allDay: true）
- ✅ 分时预览（allDay: false）
- ✅ 根据 `estimated_duration` 计算预览长度
  - tiny（0 或 null）→ 15 分钟
  - 其他 → 使用 `estimated_duration`
- ✅ 截断到当日 24:00（不跨天）
- ✅ 悬浮在已有事件上显示链接图标

#### B. **放置逻辑**

1. **拖放到已有事件** → 链接任务到时间块 (`apiPost('/time-blocks/:id/link-task')`)
2. **拖放到全日区域** → 创建全天时间块
3. **拖放到分时区域** → 创建分时时间块

#### C. **时间计算**

- 使用 `useTimePosition` 计算鼠标位置对应的时间
- 使用 `useAutoScroll` 自动滚动
- 支持日期切换时清除缓存

---

## 🎯 迁移目标

### 不变的部分

1. ✅ 预览效果完全一致
2. ✅ 全日/分时两种模式
3. ✅ 截断到当日 24:00
4. ✅ 链接到已有事件
5. ✅ 自动滚动
6. ✅ 时间计算逻辑

### 要改变的部分

1. ❌ 移除 `useCalendarDrag` composable
2. ❌ 移除 `useCrossViewDrag` 调用
3. ❌ 移除旧的策略系统 (`anyToCalendar`)
4. ✅ 使用 `interact.js` 控制器
5. ✅ 使用新的策略系统
6. ✅ 统一拖放 API

---

## 🔧 迁移步骤

### Step 1: 创建日历拖放策略

创建 `src/infra/drag/strategies/calendar-scheduling.ts`

```typescript
/**
 * 日历调度策略
 *
 * 所有拖放到日历的策略：
 * - staging -> calendar (全日/分时)
 * - daily -> calendar (全日/分时)
 * - 任何视图 -> calendar (全日/分时)
 */

import type { Strategy } from '../types'
import { commandBus } from '@/commandBus'
import { logger, LogTags } from '@/infra/logging/logger'
import { extractTaskIds } from './strategy-utils'

/**
 * 策略：任何视图 -> Calendar（全日）
 */
export const anyToCalendarAllDayStrategy: Strategy = {
  id: 'any-to-calendar-allday',
  name: 'Any to Calendar (All Day)',

  conditions: {
    source: {
      // 匹配任何源
    },
    target: {
      viewKey: /^calendar-allday-/, // 匹配 calendar-allday-{ISO}
    },
    priority: 100,
  },

  action: {
    name: 'create_allday_timeblock',
    description: '拖放到日历全日区域，创建全天时间块',

    async execute(ctx) {
      try {
        // 从 targetContext 解析时间信息
        const targetConfig = ctx.targetContext.calendarConfig
        if (!targetConfig) {
          return {
            success: false,
            message: '❌ 缺少日历配置信息',
          }
        }

        const { startTime, endTime, isAllDay } = targetConfig

        // 🎯 步骤 1: 如果是 tiny 任务，先更新 estimated_duration
        if (ctx.task.estimated_duration === null || ctx.task.estimated_duration === 0) {
          await commandBus.emit('task.update', {
            id: ctx.task.id,
            updates: { estimated_duration: 15 },
          })
        }

        // 🎯 步骤 2: 创建时间块
        const createPayload = {
          task_id: ctx.task.id,
          start_time: startTime,
          end_time: endTime,
          start_time_local: '00:00:00',
          end_time_local: '23:59:59',
          time_type: 'FLOATING' as const,
          creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
          is_all_day: true,
        }

        await commandBus.emit('timeblock.create', createPayload)

        return {
          success: true,
          message: '✅ 已创建全天时间块',
          affectedViews: [ctx.sourceViewId, 'calendar'],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ 创建时间块失败: ${error instanceof Error ? error.message : String(error)}`,
        }
      }
    },
  },

  tags: ['calendar', 'allday', 'timeblock'],
}

/**
 * 策略：任何视图 -> Calendar（分时）
 */
export const anyToCalendarTimedStrategy: Strategy = {
  id: 'any-to-calendar-timed',
  name: 'Any to Calendar (Timed)',

  conditions: {
    source: {
      // 匹配任何源
    },
    target: {
      viewKey: /^calendar-[^a]/, // 匹配 calendar-{ISO}（排除 calendar-allday-）
    },
    priority: 100,
  },

  action: {
    name: 'create_timed_timeblock',
    description: '拖放到日历分时区域，创建分时时间块',

    async execute(ctx) {
      try {
        // 从 targetContext 解析时间信息
        const targetConfig = ctx.targetContext.calendarConfig
        if (!targetConfig) {
          return {
            success: false,
            message: '❌ 缺少日历配置信息',
          }
        }

        let { startTime, endTime } = targetConfig

        // 🔥 截断到当日 24:00
        const start = new Date(startTime)
        let end = new Date(endTime)
        const dayEnd = new Date(start)
        dayEnd.setHours(0, 0, 0, 0)
        dayEnd.setDate(dayEnd.getDate() + 1)

        if (end.getTime() > dayEnd.getTime()) {
          end = dayEnd
        }

        // 计算本地时间字符串
        const startTimeLocal = start.toTimeString().split(' ')[0] // HH:mm:ss
        const endTimeLocal = end.toTimeString().split(' ')[0]

        // 🎯 步骤 1: 如果是 tiny 任务，先更新 estimated_duration
        if (ctx.task.estimated_duration === null || ctx.task.estimated_duration === 0) {
          await commandBus.emit('task.update', {
            id: ctx.task.id,
            updates: { estimated_duration: 15 },
          })
        }

        // 🎯 步骤 2: 创建时间块
        const createPayload = {
          task_id: ctx.task.id,
          start_time: start.toISOString(),
          end_time: end.toISOString(),
          start_time_local: startTimeLocal,
          end_time_local: endTimeLocal,
          time_type: 'FLOATING' as const,
          creation_timezone: Intl.DateTimeFormat().resolvedOptions().timeZone,
          is_all_day: false,
        }

        await commandBus.emit('timeblock.create', createPayload)

        return {
          success: true,
          message: '✅ 已创建时间块',
          affectedViews: [ctx.sourceViewId, 'calendar'],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ 创建时间块失败: ${error instanceof Error ? error.message : String(error)}`,
        }
      }
    },
  },

  tags: ['calendar', 'timed', 'timeblock'],
}
```

---

### Step 2: 创建日历拖放 Composable

创建 `src/composables/calendar/useCalendarInteractDrag.ts`

```typescript
/**
 * 日历拖放（interact.js 版本）
 */

import { ref, computed, watch, nextTick, type Ref } from 'vue'
import type { EventInput } from '@fullcalendar/core'
import type FullCalendar from '@fullcalendar/vue3'
import { useAreaStore } from '@/stores/area'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact/preview-state'
import { dragController } from '@/infra/drag-interact/drag-controller'
import { logger, LogTags } from '@/infra/logging/logger'

export function useCalendarInteractDrag(
  calendarRef: Ref<InstanceType<typeof FullCalendar> | null>,
  dependencies: {
    getTimeFromDropPosition: (event: DragEvent, currentTarget: HTMLElement) => Date | null
    handleAutoScroll: (event: DragEvent, calendarContainer: HTMLElement) => void
    stopAutoScroll: () => void
  }
) {
  const previewEvent = ref<EventInput | null>(null)
  const hoveredEventId = ref<string | null>(null)
  const areaStore = useAreaStore()
  const dragStrategy = useDragStrategy()

  /**
   * 更新预览事件（根据 dragPreviewState）
   */
  function updatePreviewFromDragState() {
    const preview = dragPreviewState.value
    if (!preview) {
      previewEvent.value = null
      return
    }

    const { ghostTask } = preview.raw
    const task = ghostTask

    // 🔥 检查是否在日历容器内
    const calendarContainer = calendarRef.value?.$el as HTMLElement
    if (!calendarContainer) return

    // 🔥 获取鼠标位置（从 preview 或 interact.js）
    const mouseX = preview.computed.mousePosition?.x || 0
    const mouseY = preview.computed.mousePosition?.y || 0

    const target = document.elementFromPoint(mouseX, mouseY) as HTMLElement

    // 🔥 检查是否悬浮在已有事件上
    const fcEvent = target?.closest('.fc-event') as HTMLElement | null
    if (fcEvent) {
      const eventEl = fcEvent as any
      const eventId = eventEl?.fcSeg?.eventRange?.def?.publicId
      if (eventId && eventId !== 'preview-event') {
        hoveredEventId.value = eventId
        previewEvent.value = null // 清除预览，显示链接图标
        fcEvent.classList.add('hover-link-target')
        return
      }
    } else {
      // 清除悬浮状态
      if (hoveredEventId.value) {
        const prevHoveredEl = document.querySelector('.fc-event.hover-link-target')
        if (prevHoveredEl) {
          prevHoveredEl.classList.remove('hover-link-target')
        }
        hoveredEventId.value = null
      }
    }

    // 🔥 检查是否在全日区域
    const dayCell = target?.closest('.fc-daygrid-day') as HTMLElement | null
    if (dayCell) {
      // 全日预览
      const dateStr = dayCell.getAttribute('data-date')
      if (!dateStr) return

      const startDate = new Date(dateStr + 'T00:00:00')
      const endDate = new Date(startDate)
      endDate.setDate(endDate.getDate() + 1)

      const area = task.area_id ? areaStore.getAreaById(task.area_id) : null
      const previewColor = area?.color || '#9ca3af'

      previewEvent.value = {
        id: 'preview-event',
        title: task.title,
        start: startDate.toISOString(),
        end: endDate.toISOString(),
        allDay: true,
        color: previewColor,
        classNames: ['preview-event'],
        display: 'block',
      }
      return
    }

    // 🔥 分时预览
    // 使用 dependencies.getTimeFromDropPosition
    // ... (类似旧代码的逻辑)
  }

  /**
   * 监听 dragPreviewState 变化
   */
  watch(
    dragPreviewState,
    () => {
      updatePreviewFromDragState()
    },
    { deep: true }
  )

  /**
   * 注册日历为 dropzone
   */
  function registerCalendarDropzone() {
    const calendarContainer = calendarRef.value?.$el as HTMLElement
    if (!calendarContainer) return

    dragController.registerDropzone('calendar', calendarContainer, {
      onEnter: (session) => {
        logger.debug(LogTags.COMPONENT_CALENDAR, 'Drag entered calendar')
      },
      onLeave: (session) => {
        previewEvent.value = null
        hoveredEventId.value = null
        dependencies.stopAutoScroll()
      },
      onDrop: async (session) => {
        // 🎯 处理拖放

        // 1. 检查是否拖到已有事件上（链接）
        if (hoveredEventId.value) {
          // 调用链接 API
          await fetch(`/api/time-blocks/${hoveredEventId.value}/link-task`, {
            method: 'POST',
            body: JSON.stringify({ task_id: session.object.data.id }),
          })
          return
        }

        // 2. 检查是否在全日/分时区域
        const target = document.elementFromPoint(
          session.metadata.mousePosition.x,
          session.metadata.mousePosition.y
        ) as HTMLElement

        const dayCell = target?.closest('.fc-daygrid-day') as HTMLElement | null
        const isAllDay = !!dayCell

        let viewKey: string
        let calendarConfig: any

        if (isAllDay) {
          const dateStr = dayCell.getAttribute('data-date')
          const startDate = new Date(dateStr + 'T00:00:00')
          const endDate = new Date(startDate)
          endDate.setDate(endDate.getDate() + 1)

          viewKey = `calendar-allday-${startDate.toISOString()}`
          calendarConfig = {
            startTime: startDate.toISOString(),
            endTime: endDate.toISOString(),
            isAllDay: true,
          }
        } else {
          // 计算分时
          const dropTime = dependencies.getTimeFromDropPosition(
            // 伪造一个 DragEvent
            new DragEvent('drop', {
              clientX: session.metadata.mousePosition.x,
              clientY: session.metadata.mousePosition.y,
            }),
            calendarContainer
          )

          if (!dropTime) return

          // 根据 estimated_duration 计算结束时间
          const duration = session.object.data.estimated_duration || 15
          const durationMs = duration * 60 * 1000
          let endTime = new Date(dropTime.getTime() + durationMs)

          // 截断到当日 24:00
          const dayEnd = new Date(dropTime)
          dayEnd.setHours(24, 0, 0, 0)
          if (endTime.getTime() > dayEnd.getTime()) {
            endTime = dayEnd
          }

          viewKey = `calendar-${dropTime.toISOString()}`
          calendarConfig = {
            startTime: dropTime.toISOString(),
            endTime: endTime.toISOString(),
            isAllDay: false,
          }
        }

        // 🎯 执行策略
        const result = await dragStrategy.executeDrop(session, viewKey, {
          sourceContext: session.metadata?.sourceContext || {},
          targetContext: {
            calendarConfig,
          },
        })

        if (result.success) {
          logger.info(LogTags.COMPONENT_CALENDAR, result.message)
        } else {
          logger.error(LogTags.COMPONENT_CALENDAR, result.message)
          alert(result.message)
        }
      },
    })
  }

  return {
    previewEvent,
    registerCalendarDropzone,
  }
}
```

---

### Step 3: 修改 `CuteCalendar.vue`

```diff
<template>
  <div
    class="calendar-container"
    :class="`zoom-${currentZoom}x`"
-    @dragenter="drag.handleDragEnter"
-    @dragover="drag.handleDragOver"
-    @dragleave="drag.handleDragLeave"
-    @drop="drag.handleDrop"
  >
    <!-- 日期显示栏 -->
    <div class="calendar-header">
      <div class="date-display">
        <span class="date-text">{{ formattedDate }}</span>
      </div>
    </div>

    <FullCalendar ref="calendarRef" :options="calendarOptions" />

    <!-- ... -->
  </div>
</template>

<script setup lang="ts">
- import { useCalendarDrag } from '@/composables/calendar/useCalendarDrag'
+ import { useCalendarInteractDrag } from '@/composables/calendar/useCalendarInteractDrag'

// ...

// 拖拽功能
- const drag = useCalendarDrag(calendarRef, {
+ const drag = useCalendarInteractDrag(calendarRef, {
  getTimeFromDropPosition,
- clearCache,
- resetCache,
  handleAutoScroll,
  stopAutoScroll,
})
- drag.initialize()
+ drag.registerCalendarDropzone()

// ...
</script>
```

---

### Step 4: 注册日历策略

修改 `src/infra/drag/strategies/index.ts`

```typescript
import { strategyRegistry } from '../strategy-registry'
import {
  stagingToDailyStrategy,
  dailyToStagingStrategy,
  dailyToDailyStrategy,
  dailyReorderStrategy,
  stagingReorderStrategy,
} from './task-scheduling'
import { anyToCalendarAllDayStrategy, anyToCalendarTimedStrategy } from './calendar-scheduling'

export function initializeDragStrategies() {
  // 注册看板策略
  strategyRegistry.registerBatch([
    stagingToDailyStrategy,
    dailyToStagingStrategy,
    dailyToDailyStrategy,
    dailyReorderStrategy,
    stagingReorderStrategy,
  ])

  // 🔥 注册日历策略
  strategyRegistry.registerBatch([anyToCalendarAllDayStrategy, anyToCalendarTimedStrategy])
}
```

---

## ⚠️ 关键注意事项

### 1. **时间计算依赖**

- ✅ 保留 `useTimePosition` composable（不变）
- ✅ 保留 `useAutoScroll` composable（不变）
- ✅ 保留 `useDecorativeLine` composable（不变）

### 2. **预览事件同步**

- 需要将 `dragPreviewState` 映射到 FullCalendar 的 `EventInput`
- 监听 `dragPreviewState` 变化实时更新 `previewEvent`

### 3. **已有事件检测**

- 使用 `document.elementFromPoint()` 检测鼠标下的元素
- 通过 `fcEvent?.fcSeg?.eventRange?.def?.publicId` 获取事件 ID
- 不依赖 interact.js 的事件检测

### 4. **链接任务到时间块**

- 不走策略系统（特殊逻辑）
- 直接调用 API: `POST /time-blocks/:id/link-task`

### 5. **Command 定义**

需要在 `commandBus/types.ts` 添加：

```typescript
| {
    type: 'timeblock.create'
    payload: {
      task_id: string
      start_time: string  // ISO
      end_time: string    // ISO
      start_time_local: string  // HH:mm:ss
      end_time_local: string    // HH:mm:ss
      time_type: 'FLOATING' | 'FIXED'
      creation_timezone: string
      is_all_day: boolean
    }
  }
```

---

## 🎯 实施顺序

1. ✅ **第一步**: 创建 `calendar-scheduling.ts` 策略
2. ✅ **第二步**: 创建 `useCalendarInteractDrag.ts` composable
3. ✅ **第三步**: 注册日历策略
4. ✅ **第四步**: 修改 `CuteCalendar.vue`
5. ✅ **第五步**: 添加 `timeblock.create` command
6. ✅ **第六步**: 测试所有场景

---

## 🧪 测试场景

| 场景         | 预期行为           | 状态 |
| ------------ | ------------------ | ---- |
| 拖到全日区域 | 创建全天时间块     | ⏳   |
| 拖到分时区域 | 创建分时时间块     | ⏳   |
| 拖到已有事件 | 链接任务到时间块   | ⏳   |
| Tiny 任务    | 自动更新为 15 分钟 | ⏳   |
| 跨天预览     | 截断到 24:00       | ⏳   |
| 预览颜色     | 使用 Area 颜色     | ⏳   |
| 自动滚动     | 边缘自动滚动       | ⏳   |
| 日期切换     | 清除预览           | ⏳   |

---

## 📚 相关文档

1. [拖放系统完整报告](DRAG_DROP_SYSTEM_COMPLETE_REPORT.md)
2. [策略链设计](src/infra/drag/STRATEGY_CHAIN_DESIGN.md)
3. [灵活上下文设计](FLEXIBLE_CONTEXT_DESIGN.md)

---

**版本**: 规划 v1.0  
**状态**: 📋 待实施  
**预计工作量**: 4-6 小时

