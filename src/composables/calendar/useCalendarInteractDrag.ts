/**
 * 日历拖放（interact.js 版本）
 *
 * 使用新的 interact.js 拖放系统和策略系统
 */

import { ref, watch, type Ref } from 'vue'
import type { EventInput } from '@fullcalendar/core'
import type FullCalendar from '@fullcalendar/vue3'
import { useAreaStore } from '@/stores/area'
import { useDragStrategy } from '@/composables/drag/useDragStrategy'
import { dragPreviewState } from '@/infra/drag-interact/preview-state'
import { interactManager } from '@/infra/drag-interact/drag-controller'
import type { DragSession } from '@/infra/drag-interact/types'
import { logger, LogTags } from '@/infra/logging/logger'
import { isTaskCard, isTemplate } from '@/types/dtos'
import { apiPost } from '@/stores/shared'
import { parseDateString } from '@/infra/utils/dateUtils'

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

    // 统一从 draggedObject 读取被拖动任务
    const task = (preview.raw as any).draggedObject || (preview as any).raw.ghostTask

    // 🔥 检查是否在日历容器内
    const calendarContainer = calendarRef.value?.$el as HTMLElement
    if (!calendarContainer) {
      previewEvent.value = null
      return
    }

    // 🔥 获取鼠标位置（从 preview.raw）
    const mouseX = (preview.raw as any).mousePosition?.x || 0
    const mouseY = (preview.raw as any).mousePosition?.y || 0

    const target = document.elementFromPoint(mouseX, mouseY) as HTMLElement

    // 🔥 检查鼠标是否在日历容器内
    // 使用 getBoundingClientRect 检查坐标是否在日历边界内
    const rect = calendarContainer.getBoundingClientRect()
    const isOverCalendar =
      mouseX >= rect.left && mouseX <= rect.right && mouseY >= rect.top && mouseY <= rect.bottom

    if (!isOverCalendar) {
      // 鼠标不在日历上，清除预览
      previewEvent.value = null
      // 清除悬浮状态
      if (hoveredEventId.value) {
        const prevHoveredEl = document.querySelector('.fc-event.hover-link-target')
        if (prevHoveredEl) {
          prevHoveredEl.classList.remove('hover-link-target')
        }
        hoveredEventId.value = null
      }
      return
    }

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

      const startDate = parseDateString(dateStr)
      const endDate = new Date(startDate)
      endDate.setDate(endDate.getDate() + 1)

      const areaId = task && (task as any).area_id ? (task as any).area_id : undefined
      const area = areaId ? areaStore.getAreaById(areaId) : null
      const previewColor = area?.color || '#9ca3af'

      previewEvent.value = {
        id: 'preview-event',
        title: ((task as any)?.title ?? (task as any)?.name ?? '任务') as string,
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
    // 伪造一个 DragEvent 来调用现有的时间计算逻辑
    const fakeEvent = new DragEvent('dragover', {
      clientX: mouseX,
      clientY: mouseY,
    })

    const dropTime = dependencies.getTimeFromDropPosition(fakeEvent, calendarContainer)
    if (!dropTime) {
      previewEvent.value = null
      return
    }

    // 根据任务的 estimated_duration 计算预览时间块长度
    const rawDuration = (task && (task as any).estimated_duration) as number | undefined
    const durationMinutes = typeof rawDuration === 'number' && rawDuration > 0 ? rawDuration : 15
    const durationMs = durationMinutes * 60 * 1000
    let endTime = new Date(dropTime.getTime() + durationMs)

    // 截断到"当前日历视图"的当日 24:00
    let dayStart = new Date(dropTime)
    if (calendarRef.value) {
      const api = calendarRef.value.getApi()
      const baseDate = api.getDate()
      dayStart = new Date(baseDate)
    }
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
    const previewColor = area?.color || '#9ca3af'

    previewEvent.value = {
      id: 'preview-event',
      title: ((task as any)?.title ?? (task as any)?.name ?? '任务') as string,
      start: startTimeForPreview.toISOString(),
      end: endTime.toISOString(),
      allDay: false,
      color: previewColor,
      classNames: ['preview-event'],
      display: 'block',
    }
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
        const currentPreview = dragPreviewState.value
        const mousePos = currentPreview?.raw.mousePosition
        if (!mousePos) {
          logger.warn(LogTags.COMPONENT_CALENDAR, 'No mouse position in preview state')
          return
        }

        const target = document.elementFromPoint(mousePos.x, mousePos.y) as HTMLElement

        const dayCell = target?.closest('.fc-daygrid-day') as HTMLElement | null
        const isAllDay = !!dayCell

        let viewKey: string
        let calendarConfig: any

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

          logger.debug(LogTags.COMPONENT_CALENDAR, 'All-day drop', { viewKey, calendarConfig })
        } else {
          // 计算分时
          const fakeEvent = new DragEvent('drop', {
            clientX: mousePos.x,
            clientY: mousePos.y,
          })

          const dropTime = dependencies.getTimeFromDropPosition(fakeEvent, calendarContainer)

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
