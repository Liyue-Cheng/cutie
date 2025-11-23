/**
 * useCalendarInteractDrag - 日历拖放系统（基于 interact.js）
 *
 * 🎯 核心功能：
 * - 处理从 Kanban/TaskList 拖拽任务到日历
 * - 实时显示拖拽预览（跟随鼠标的半透明任务卡片）
 * - 支持拖拽到全天区域或分时区域
 * - 支持拖拽到已有时间块上（链接任务）
 *
 * 🔑 技术栈：
 * - interact.js：底层拖拽引擎
 * - dragPreviewState：全局拖拽状态（由 interact manager 管理）
 * - useDragStrategy：统一的拖放策略系统
 * - getTimeFromDropPosition：坐标 → 时间转换
 *
 * 🎨 预览样式：
 * - 全天区域：显示为全天任务卡片
 * - 分时区域：显示为时间格卡片（带时间范围）
 * - 悬停在已有时间块上：显示链接图标（🔗）
 *
 * 📌 与框选系统的区别：
 * - 框选：由 CuteCalendar 的 mouse 事件驱动
 * - 拖拽：由 interact.js 驱动，监听 dragPreviewState
 * - 两套系统互不干扰（框选只在空白区域启动）
 */

import { ref, watch, type Ref } from 'vue'
import type { EventInput } from '@fullcalendar/core'
import type FullCalendar from '@fullcalendar/vue3'
import { useAreaStore } from '@/stores/area'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState, previewMousePosition } from '@/infra/drag-interact/preview-state'
import { interactManager } from '@/infra/drag-interact/drag-controller'
import type { DragSession, Position } from '@/infra/drag-interact/types'
import { logger, LogTags } from '@/infra/logging/logger'
import { isTaskCard, isTemplate } from '@/types/dtos'
import { apiPost } from '@/stores/shared'
import { parseDateString } from '@/infra/utils/dateUtils'
import { getDefaultAreaColor } from '@/infra/utils/themeUtils'

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

  const POSITION_EPSILON = 0.5
  let lastPreviewPosition: Position | null = null
  let lastPreviewKey: string | null = null

  function clearHoveredEvent() {
    if (!hoveredEventId.value) {
      return
    }
    const prevHoveredEl = document.querySelector('.fc-event.hover-link-target')
    if (prevHoveredEl) {
      prevHoveredEl.classList.remove('hover-link-target')
    }
    hoveredEventId.value = null
  }

  /**
   * 更新预览事件（根据 dragPreviewState 与鼠标位置）
   *
   * 🎯 功能：
   * - 监听全局拖拽状态（dragPreviewState）
   * - 根据鼠标位置计算预览事件的时间和位置
   * - 写入 previewEvent.value，触发 FullCalendar 重新渲染
   *
   * ⚡ 性能优化：
   * - 位置阈值（POSITION_EPSILON）：鼠标移动 < 0.5px 不更新
   * - previewKey 缓存：时间范围未变化时不重新创建事件对象
   *
   * 🔍 检测逻辑：
   * 1. 是否在日历区域内（getBoundingClientRect）
   * 2. 是否悬停在已有事件上（.fc-event）
   * 3. 是否在全天区域（.fc-daygrid-day）
   * 4. 是否在分时区域（默认）
   *
   * @param positionOverride 手动指定鼠标位置（用于强制更新）
   * @param force 是否强制更新（忽略位置阈值）
   */
  function updatePreviewFromDragState(positionOverride?: Position | null, force = false) {
    const preview = dragPreviewState.value

    if (!preview) {
      lastPreviewPosition = null
      lastPreviewKey = null
      previewEvent.value = null
      clearHoveredEvent()
      return
    }

    const calendarContainer = calendarRef.value?.$el as HTMLElement
    if (!calendarContainer) {
      lastPreviewPosition = null
      previewEvent.value = null
      clearHoveredEvent()
      return
    }

    const position =
      positionOverride ??
      previewMousePosition.value ??
      ((preview.raw as any).mousePosition as Position | undefined) ??
      null

    if (!position) {
      lastPreviewPosition = null
      lastPreviewKey = null
      previewEvent.value = null
      clearHoveredEvent()
      return
    }

    if (
      !force &&
      lastPreviewPosition &&
      Math.abs(lastPreviewPosition.x - position.x) < POSITION_EPSILON &&
      Math.abs(lastPreviewPosition.y - position.y) < POSITION_EPSILON
    ) {
      return
    }

    lastPreviewPosition = { ...position }

    const { x: mouseX, y: mouseY } = position
    const rect = calendarContainer.getBoundingClientRect()
    const isOverCalendar =
      mouseX >= rect.left && mouseX <= rect.right && mouseY >= rect.top && mouseY <= rect.bottom

    if (!isOverCalendar) {
      previewEvent.value = null
      lastPreviewKey = null
      clearHoveredEvent()
      return
    }

    const target = document.elementFromPoint(mouseX, mouseY) as HTMLElement | null

    // 🔥 检查是否悬浮在已有事件上
    const fcEvent = target?.closest('.fc-event') as HTMLElement | null
    if (fcEvent) {
      const eventEl = fcEvent as any
      const eventRange = eventEl?.fcSeg?.eventRange
      const eventId = eventRange?.def?.publicId
      const eventType =
        eventRange?.def?.extendedProps?.type ||
        eventEl?.dataset?.eventType ||
        eventEl?.dataset?.type

      const isLinkableType =
        eventType === 'timeblock' ||
        eventType === 'time-block' ||
        eventType === 'timeblock_event' ||
        eventType === 'time_block'

      if (!isLinkableType) {
        clearHoveredEvent()
        hoveredEventId.value = null
        // 保留预览事件
      } else if (eventId && eventId !== 'preview-event') {
        clearHoveredEvent()
        hoveredEventId.value = eventId
        previewEvent.value = null // 清除预览，显示链接图标
        fcEvent.classList.add('hover-link-target')
        return
      }
    } else {
      clearHoveredEvent()
    }

    // 统一从 draggedObject 读取被拖动任务
    const task = (preview.raw as any).draggedObject || (preview as any).raw.ghostTask

    // 🔥 检查是否在全日区域
    const dayCell = target?.closest('.fc-daygrid-day') as HTMLElement | null
    if (dayCell) {
      const dateStr = dayCell.getAttribute('data-date')
      if (!dateStr) {
        previewEvent.value = null
        lastPreviewKey = null
        return
      }

      const startDate = parseDateString(dateStr)
      const endDate = new Date(startDate)
      endDate.setDate(endDate.getDate() + 1)

      const previewKey = `allday-${dateStr}`
      if (!force && lastPreviewKey === previewKey && previewEvent.value) {
        return
      }

      const areaId = task && (task as any).area_id ? (task as any).area_id : undefined
      const area = areaId ? areaStore.getAreaById(areaId) : null
      const previewColor = area?.color || getDefaultAreaColor()

      const isRecurringTask = Boolean(task && (task as any).recurrence_id)
      const taskTitle = ((task as any)?.title ?? (task as any)?.name ?? '任务') as string
      const classNames = isRecurringTask
        ? ['task-event', 'recurring-task', 'preview-task-event']
        : ['task-event', 'preview-task-event']

      previewEvent.value = {
        id: 'preview-event',
        title: `${taskTitle}`,
        start: startDate.toISOString(),
        end: endDate.toISOString(),
        allDay: true,
        color: previewColor,
        classNames,
        display: 'block',
        extendedProps: {
          type: 'task',
          taskId: (task as any)?.id,
          scheduleDay: dateStr,
          isRecurring: isRecurringTask,
          isPreview: true,
          scheduleOutcome: null,
          isCompleted: Boolean(task && (task as any).is_completed),
          previewColor,
        },
      }
      lastPreviewKey = previewKey
      return
    }

    const eventLike = { clientX: mouseX, clientY: mouseY } as DragEvent

    const dropTime = dependencies.getTimeFromDropPosition(eventLike, calendarContainer)
    if (!dropTime) {
      previewEvent.value = null
      return
    }

    const rawDuration = (task && (task as any).estimated_duration) as number | undefined
    const durationMinutes = typeof rawDuration === 'number' && rawDuration > 0 ? rawDuration : 15
    const durationMs = durationMinutes * 60 * 1000
    let endTime = new Date(dropTime.getTime() + durationMs)

    const dayStart = new Date(dropTime)
    dayStart.setHours(0, 0, 0, 0)
    const dayEnd = new Date(dayStart)
    dayEnd.setHours(23, 59, 59, 999)

    let startTimeForPreview = dropTime
    if (endTime.getTime() > dayEnd.getTime()) {
      endTime = dayEnd
      const adjustedStartMs = Math.max(dayStart.getTime(), endTime.getTime() - durationMs)
      startTimeForPreview = new Date(adjustedStartMs)
    }

    const areaId2 = task && (task as any).area_id ? (task as any).area_id : undefined
    const area = areaId2 ? areaStore.getAreaById(areaId2) : null
    const previewColor = area?.color || getDefaultAreaColor()

    const previewKey = `timed-${startTimeForPreview.toISOString()}-${endTime.toISOString()}`
    if (!force && lastPreviewKey === previewKey && previewEvent.value) {
      return
    }

    const taskId = (task as any)?.id
    previewEvent.value = {
      id: 'preview-event',
      title: ((task as any)?.title ?? (task as any)?.name ?? '任务') as string,
      start: startTimeForPreview.toISOString(),
      end: endTime.toISOString(),
      allDay: false,
      color: 'transparent',
      backgroundColor: 'transparent',
      borderColor: 'transparent',
      classNames: ['preview-event'],
      display: 'block',
      extendedProps: {
        type: 'task',
        taskId,
        scheduleDay: undefined,
        isRecurring: Boolean(task && (task as any).recurrence_id),
        isPreview: true,
        scheduleOutcome: null,
        isCompleted: Boolean(task && (task as any).is_completed),
        previewColor,
        areaColor: previewColor,
      },
    }
    lastPreviewKey = previewKey
  }

  /**
   * 监听 dragPreviewState 变化
   */
  watch(
    dragPreviewState,
    () => {
      updatePreviewFromDragState(undefined, true)
    },
    { deep: false }
  )

  watch(
    previewMousePosition,
    (position) => {
      updatePreviewFromDragState(position ?? null)
    },
    { flush: 'sync' }
  )

  /**
   * 注册日历为 dropzone
   */
  function registerCalendarDropzone() {
    const calendarContainer = calendarRef.value?.$el as HTMLElement
    if (!calendarContainer) {
      logger.warn(
        LogTags.COMPONENT_CALENDAR,
        'Calendar container not found, cannot register dropzone'
      )
      return
    }

    interactManager.registerDropzone(calendarContainer, {
      zoneId: 'calendar',
      type: 'calendar',
      onDrop: async (session: DragSession) => {
        logger.debug(LogTags.COMPONENT_CALENDAR, 'Drop in calendar', { session })

        // 🎯 处理拖放

        // 1. 检查是否拖到已有事件上（链接）
        if (hoveredEventId.value) {
          logger.info(LogTags.COMPONENT_CALENDAR, 'Linking task to existing time block', {
            eventId: hoveredEventId.value,
          })

          try {
            // 调用链接 API
            await apiPost(`/time-blocks/${hoveredEventId.value}/link-task`, {
              task_id: session.object.data.id,
            })

            logger.info(LogTags.COMPONENT_CALENDAR, 'Successfully linked task')
            // 清理状态
            previewEvent.value = null
            hoveredEventId.value = null
            const prevHoveredEl = document.querySelector('.fc-event.hover-link-target')
            if (prevHoveredEl) {
              prevHoveredEl.classList.remove('hover-link-target')
            }
          } catch (error) {
            const errorMessage =
              error instanceof Error ? error.message : (error as any).message || '未知错误'
            logger.error(
              LogTags.COMPONENT_CALENDAR,
              'Failed to link task',
              error instanceof Error ? error : new Error(String(error))
            )
            alert('链接任务失败：' + errorMessage)
          }
          return
        }

        // 2. 检查是否在全日/分时区域
        // 从 dragPreviewState 获取当前鼠标位置
        const mousePos = previewMousePosition.value
        if (!mousePos) {
          logger.warn(LogTags.COMPONENT_CALENDAR, 'No mouse position in preview state')
          return
        }

        const target = document.elementFromPoint(mousePos.x, mousePos.y) as HTMLElement

        const dayCell = target?.closest('.fc-daygrid-day') as HTMLElement | null
        const isAllDay = !!dayCell

        let viewKey: string
        let calendarConfig: any

        const calendarApi = calendarRef.value?.getApi()
        const currentViewTypeName = calendarApi?.view?.type || ''

        const targetContextExtras: Record<string, any> = {
          calendarViewType: currentViewTypeName,
        }

        if (isAllDay) {
          const dateStr = dayCell.getAttribute('data-date')
          if (!dateStr) {
            logger.warn(LogTags.COMPONENT_CALENDAR, 'No date attribute on day cell')
            return
          }

          const startDate = parseDateString(dateStr)
          const endDate = new Date(startDate)
          endDate.setDate(endDate.getDate() + 1)

          viewKey = `calendar-allday-${startDate.toISOString()}`
          calendarConfig = {
            startTime: startDate.toISOString(),
            endTime: endDate.toISOString(),
            isAllDay: true,
          }

          targetContextExtras.calendarDate = dateStr

          logger.debug(LogTags.COMPONENT_CALENDAR, 'All-day drop', { viewKey, calendarConfig })
        } else {
          // 计算分时
          const dropEventLike = { clientX: mousePos.x, clientY: mousePos.y } as DragEvent

          const dropTime = dependencies.getTimeFromDropPosition(dropEventLike, calendarContainer)

          if (!dropTime) {
            logger.warn(LogTags.COMPONENT_CALENDAR, 'Failed to calculate drop time')
            return
          }

          // 根据对象类型计算持续时间（任务优先，其次模板，默认15分钟）
          const rawObj: any = session.object.data as any
          let duration = 15
          if (isTaskCard(rawObj)) {
            const est = rawObj.estimated_duration
            duration = typeof est === 'number' && est > 0 ? est : 15
          } else if (isTemplate(rawObj)) {
            const est = rawObj.estimated_duration_template
            duration = typeof est === 'number' && est > 0 ? est : 15
          }
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

          logger.debug(LogTags.COMPONENT_CALENDAR, 'Timed drop', { viewKey, calendarConfig })
        }

        // 🎯 执行策略
        const result = await dragStrategy.executeDrop(session, viewKey, {
          sourceContext: session.metadata?.sourceContext || {},
          targetContext: {
            calendarConfig,
            ...targetContextExtras,
          },
        })

        if (result.success) {
          logger.info(LogTags.COMPONENT_CALENDAR, result.message || 'Drop successful')
          previewEvent.value = null
        } else {
          logger.error(LogTags.COMPONENT_CALENDAR, result.message || 'Drop failed')
          alert(result.message || '创建时间块失败')
        }
      },
    })

    logger.info(LogTags.COMPONENT_CALENDAR, 'Calendar dropzone registered')
  }

  return {
    previewEvent,
    registerCalendarDropzone,
  }
}
