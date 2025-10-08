/**
 * useCalendarDrag - 日历拖拽功能
 *
 * 处理从任务列表拖拽任务到日历，创建时间块
 */

import { ref, onMounted, onUnmounted, type Ref } from 'vue'
import type { EventInput } from '@fullcalendar/core'
import type FullCalendar from '@fullcalendar/vue3'
import type { TaskCard } from '@/types/dtos'
import { parseDateString } from '@/utils/dateUtils'
import type { ViewMetadata, CalendarViewConfig } from '@/types/drag'
import { useCrossViewDrag, useDragTransfer } from '@/composables/drag'
import { useAreaStore } from '@/stores/area'
import { useTaskStore } from '@/stores/task'
import { apiBaseUrl } from '@/composables/useApiConfig'

export function useCalendarDrag(
  calendarRef: Ref<InstanceType<typeof FullCalendar> | null>,
  dependencies: {
    getTimeFromDropPosition: (event: DragEvent, currentTarget: HTMLElement) => Date | null
    clearCache: () => void
    resetCache: () => void
    handleAutoScroll: (event: DragEvent, calendarContainer: HTMLElement) => void
    stopAutoScroll: () => void
  }
) {
  const previewEvent = ref<EventInput | null>(null)
  const isDragging = ref(false)
  const currentDraggedTask = ref<TaskCard | null>(null)
  const isProcessingDrop = ref(false) // 标志：正在处理 drop 操作
  const hoveredEventId = ref<string | null>(null) // 悬浮在已有事件上时的事件ID

  // 节流控制
  const lastUpdateTime = ref(0)
  const UPDATE_THROTTLE = 16 // 约60fps

  const crossViewDrag = useCrossViewDrag()
  const dragTransfer = useDragTransfer()
  const areaStore = useAreaStore()
  const taskStore = useTaskStore()

  /**
   * 全局拖拽开始处理
   */
  function handleGlobalDragStart(event: DragEvent) {
    try {
      // 使用统一的 dragTransfer 获取数据
      const dragData = dragTransfer.getDragData(event)
      if (dragData && dragData.type === 'task') {
        currentDraggedTask.value = dragData.task
      }
    } catch (error) {
      // 忽略解析错误
    }
  }

  /**
   * 全局拖拽结束处理
   */
  function handleGlobalDragEnd() {
    currentDraggedTask.value = null
    // 如果正在处理 drop，不要清除预览（让 handleDrop 控制清理）
    if (!isProcessingDrop.value) {
      clearPreviewEvent()
    }
    dependencies.stopAutoScroll()
  }

  /**
   * 拖拽进入日历区域
   */
  function handleDragEnter(event: DragEvent) {
    event.preventDefault()

    // 🔍 检查点4：重置几何缓存，确保日期切换后位置准确
    dependencies.resetCache()

    // 检查是否包含任务数据（使用统一的 dragTransfer）
    if (dragTransfer.hasDragData(event)) {
      isDragging.value = true
      console.log('[CHK-1] dragenter: hasDragData=true, isDragging set')
    }
  }

  /**
   * 拖拽在日历上移动
   */
  function handleDragOver(event: DragEvent) {
    event.preventDefault()

    // 🔍 检查点1：effectAllowed/dropEffect 匹配
    if (event.dataTransfer) {
      console.log(
        '[CHK-1] dragover: dropEffect(before)=',
        event.dataTransfer.dropEffect,
        'effectAllowed=',
        event.dataTransfer.effectAllowed,
        'types=',
        Array.from(event.dataTransfer.types)
      )
      event.dataTransfer.dropEffect = 'copy'
    }

    // 节流更新预览，避免过于频繁的计算
    const now = Date.now()
    if (isDragging.value && now - lastUpdateTime.value > UPDATE_THROTTLE) {
      updatePreviewEvent(event)
      dependencies.handleAutoScroll(event, event.currentTarget as HTMLElement)
      lastUpdateTime.value = now
    }
  }

  /**
   * 拖拽离开日历区域
   */
  function handleDragLeave(event: DragEvent) {
    // 检查是否真的离开了日历区域
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect()
    const x = event.clientX
    const y = event.clientY

    if (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom) {
      clearPreviewEvent()
      dependencies.stopAutoScroll()
    }
  }

  /**
   * 更新预览事件
   */
  function updatePreviewEvent(event: DragEvent) {
    console.log('[DEBUG-PREVIEW] 🔍 updatePreviewEvent called')

    // ✅ 检查是否拖到全日区域
    const target =
      (event.target as HTMLElement) ||
      (document.elementFromPoint(event.clientX, event.clientY) as HTMLElement)

    // ✅ 检查是否悬浮在已有事件上
    const fcEvent = target?.closest('.fc-event') as HTMLElement | null
    console.log('[DEBUG-PREVIEW] fcEvent found:', !!fcEvent, fcEvent?.className)

    if (fcEvent) {
      // 获取事件ID
      const eventEl = fcEvent as any
      if (eventEl?.fcSeg?.eventRange?.def?.publicId) {
        const eventId = eventEl.fcSeg.eventRange.def.publicId
        console.log('[DEBUG-PREVIEW] Event ID detected:', eventId)

        // 不是预览事件才设置
        if (eventId !== 'preview-event') {
          console.log('[DEBUG-PREVIEW] ✅ Hovering on real event, CLEARING preview')
          hoveredEventId.value = eventId
          // 清除预览，不显示预览块
          const wasPreview = previewEvent.value !== null
          previewEvent.value = null
          console.log('[DEBUG-PREVIEW] Preview cleared (was showing:', wasPreview, ')')
          // ✅ 添加简化的视觉反馈（仅链子图标）
          fcEvent.classList.add('hover-link-target')
          return
        } else {
          console.log('[DEBUG-PREVIEW] ⚠️ Hovering on preview-event itself, ignoring')
        }
      }
    } else {
      console.log('[DEBUG-PREVIEW] No fcEvent found, checking if need to clear hover state')
      // 清除悬浮状态
      if (hoveredEventId.value) {
        console.log('[DEBUG-PREVIEW] Clearing hover state for:', hoveredEventId.value)
        const prevHoveredEl = document.querySelector('.fc-event.hover-link-target')
        if (prevHoveredEl) {
          prevHoveredEl.classList.remove('hover-link-target')
        }
        hoveredEventId.value = null
      }
    }

    const dayCell = target?.closest('.fc-daygrid-day') as HTMLElement | null
    const isAllDayArea = !!dayCell

    if (isAllDayArea) {
      // 全天预览：优先从 dayCell 的 data-date 获取具体日期
      let startDate: Date | null = null
      let endDate: Date | null = null

      const dateStr = dayCell?.getAttribute('data-date')
      if (dateStr) {
        // 解析 YYYY-MM-DD 为本地日期对象
        startDate = parseDateString(dateStr)
        endDate = parseDateString(dateStr)
        endDate.setDate(endDate.getDate() + 1)
      } else if (calendarRef.value) {
        // 回退：使用当前视图日期
        const calendarApi = calendarRef.value.getApi()
        const currentDate = calendarApi.getDate()
        currentDate.setHours(0, 0, 0, 0)
        startDate = new Date(currentDate)
        endDate = new Date(currentDate)
        endDate.setDate(endDate.getDate() + 1)
      } else {
        return
      }

      const previewTitle = currentDraggedTask.value?.title || '任务'
      const area = currentDraggedTask.value?.area_id
        ? areaStore.getAreaById(currentDraggedTask.value.area_id)
        : null
      const previewColor = area?.color || '#9ca3af'

      console.log('[DEBUG-PREVIEW] 📅 Creating ALL-DAY preview')
      previewEvent.value = {
        id: 'preview-event',
        title: previewTitle,
        start: startDate.toISOString(),
        end: endDate.toISOString(),
        allDay: true, // ✅ 全天预览
        color: previewColor,
        classNames: ['preview-event'],
        display: 'block',
      }
      console.log('[DEBUG-PREVIEW] ✅ All-day preview created:', previewEvent.value)
    } else {
      // 分时预览：使用拖拽位置计算时间
      const dropTime = dependencies.getTimeFromDropPosition(
        event,
        event.currentTarget as HTMLElement
      )
      console.log('[DEBUG-PREVIEW] dropTime calculated:', dropTime)

      if (dropTime) {
        // 根据任务的 estimated_duration 计算预览时间块长度
        // 如果是 tiny（0 或 null），使用 15 分钟
        const task = currentDraggedTask.value
        let durationMinutes = 60 // 默认1小时
        if (task) {
          const estimatedDuration = task.estimated_duration
          if (estimatedDuration === null || estimatedDuration === 0) {
            durationMinutes = 15 // tiny 任务使用 15 分钟
          } else {
            durationMinutes = estimatedDuration
          }
        }

        const durationMs = durationMinutes * 60 * 1000
        let endTime = new Date(dropTime.getTime() + durationMs)

        // 截断到“当前日历视图”的当日 24:00，禁止跨天预览（保留“当前视图日期”的部分）
        let dayStart = new Date(dropTime)
        if (calendarRef.value) {
          const api = calendarRef.value.getApi()
          const baseDate = api.getDate()
          dayStart = new Date(baseDate)
        }
        dayStart.setHours(0, 0, 0, 0)
        const dayEnd = new Date(dayStart)
        dayEnd.setHours(23, 59, 59, 999) // 当天最后一刻
        let startTimeForPreview = dropTime
        if (endTime.getTime() > dayEnd.getTime()) {
          endTime = dayEnd
          const adjustedStartMs = Math.max(dayStart.getTime(), endTime.getTime() - durationMs)
          startTimeForPreview = new Date(adjustedStartMs)
        }

        const previewTitle = currentDraggedTask.value?.title || '任务'
        const area = currentDraggedTask.value?.area_id
          ? areaStore.getAreaById(currentDraggedTask.value.area_id)
          : null
        const previewColor = area?.color || '#9ca3af'

        previewEvent.value = {
          id: 'preview-event',
          title: previewTitle,
          start: startTimeForPreview.toISOString(),
          end: endTime.toISOString(),
          allDay: false, // ✅ 分时预览
          color: previewColor,
          classNames: ['preview-event'],
          display: 'block',
        }
      }
    }

    console.log('[CuteCalendar] Preview event updated:', previewEvent.value)
  }

  /**
   * 清除预览事件
   */
  function clearPreviewEvent() {
    previewEvent.value = null
    isDragging.value = false
    // 清除悬浮状态
    if (hoveredEventId.value) {
      const prevHoveredEl = document.querySelector('.fc-event.hover-link-target')
      if (prevHoveredEl) {
        prevHoveredEl.classList.remove('hover-link-target')
      }
      hoveredEventId.value = null
    }
    // 清理缓存
    dependencies.clearCache()
    // 停止自动滚动
    dependencies.stopAutoScroll()
  }

  /**
   * 处理拖拽放下
   */
  async function handleDrop(event: DragEvent) {
    event.preventDefault()

    // 🔍 检查点1 & 2：drop 是否被触发
    console.log(
      '[CHK-1] ✅ DROP FIRED! target=',
      (event.target as HTMLElement)?.className,
      'effectAllowed=',
      event.dataTransfer?.effectAllowed,
      'dropEffect=',
      event.dataTransfer?.dropEffect
    )

    // 标记开始处理 drop，防止 dragend 事件清除预览
    isProcessingDrop.value = true

    try {
      // ✅ 优先：在 drop 时直接命中检测，找到鼠标下的事件（避免只在顶部小区域触发）
      const target =
        (event.target as HTMLElement) ||
        (document.elementFromPoint(event.clientX, event.clientY) as HTMLElement)
      const fcEvent = target?.closest('.fc-event') as HTMLElement | null

      // 从命中的 DOM 解析事件ID
      let eventIdToLink: string | null = null
      if (fcEvent) {
        const eventEl = fcEvent as any
        const publicId = eventEl?.fcSeg?.eventRange?.def?.publicId
        if (publicId && publicId !== 'preview-event' && publicId !== 'temp-creating') {
          eventIdToLink = publicId
        }
      }

      // 回退：使用 hover 记录到的事件ID
      if (!eventIdToLink && hoveredEventId.value) {
        eventIdToLink = hoveredEventId.value
      }

      // ✅ 检查是否拖到已有事件上（链接任务到时间块）
      if (eventIdToLink && currentDraggedTask.value) {
        console.log('[CuteCalendar] Linking task to existing time block:', eventIdToLink)

        try {
          // 调用链接API（使用动态端口）
          const response = await fetch(
            `${apiBaseUrl.value}/time-blocks/${eventIdToLink}/link-task`,
            {
              method: 'POST',
              headers: {
                'Content-Type': 'application/json',
              },
              body: JSON.stringify({
                task_id: currentDraggedTask.value.id,
              }),
            }
          )

          if (!response.ok) {
            const errorData = await response.json()
            console.error('[CuteCalendar] Failed to link task:', errorData)
            alert('链接任务失败：' + (errorData.message || '未知错误'))
          } else {
            const result = await response.json()
            console.log('[CuteCalendar] Successfully linked task:', result)
            // 刷新任务数据会通过SSE事件自动触发
          }
        } catch (error) {
          console.error('[CuteCalendar] Error linking task:', error)
          alert('链接任务时发生错误')
        } finally {
          // 清理状态
          clearPreviewEvent()
          const prevHoveredEl = document.querySelector('.fc-event.hover-link-target')
          if (prevHoveredEl) {
            prevHoveredEl.classList.remove('hover-link-target')
          }
          hoveredEventId.value = null
          isProcessingDrop.value = false
        }
        return
      }
      // ✅ 检查是否拖到全天区域（复用上面的 target 变量）
      const dayCell = target?.closest('.fc-daygrid-day') as HTMLElement | null
      const isAllDayArea = !!dayCell

      let calendarView: ViewMetadata | null = null

      if (isAllDayArea) {
        console.log('[CuteCalendar] isAllDayArea=true')
        // 全天事件：优先从 dayCell 的 data-date 获取具体日期
        let startDate: Date | null = null
        let endDate: Date | null = null

        const dateStr = dayCell?.getAttribute('data-date')
        if (dateStr) {
          startDate = parseDateString(dateStr)
          endDate = parseDateString(dateStr)
          endDate.setDate(endDate.getDate() + 1)
        } else if (calendarRef.value) {
          const calendarApi = calendarRef.value.getApi()
          const currentDate = calendarApi.getDate()
          currentDate.setHours(0, 0, 0, 0)
          startDate = new Date(currentDate)
          endDate = new Date(currentDate)
          endDate.setDate(endDate.getDate() + 1)
        } else {
          clearPreviewEvent()
          isProcessingDrop.value = false
          return
        }

        calendarView = {
          type: 'calendar',
          id: `calendar-allday-${startDate.toISOString()}`,
          config: {
            startTime: startDate.toISOString(),
            endTime: endDate.toISOString(),
            isAllDay: true, // ✅ 标记为全天事件
          } as CalendarViewConfig,
          label: `全天 ${startDate.toLocaleDateString()}`,
        }
      } else {
        // 分时事件：获取拖拽位置对应的时间
        const dropTime = dependencies.getTimeFromDropPosition(
          event,
          event.currentTarget as HTMLElement
        )

        if (!dropTime) {
          clearPreviewEvent()
          isProcessingDrop.value = false
          return
        }

        // 根据任务的 estimated_duration 计算时间块长度
        // 如果是 tiny（0 或 null），使用 15 分钟
        const task = currentDraggedTask.value
        let durationMinutes = 60 // 默认1小时
        if (task) {
          const estimatedDuration = task.estimated_duration
          if (estimatedDuration === null || estimatedDuration === 0) {
            durationMinutes = 15 // tiny 任务使用 15 分钟
          } else {
            durationMinutes = estimatedDuration
          }
        }

        // 创建时间块，并在“当前日历视图”的日界处截断（保留“当前视图日期”的部分）
        const durationMsDrop = durationMinutes * 60 * 1000
        let endTime = new Date(dropTime.getTime() + durationMsDrop)
        let dayStart = new Date(dropTime)
        if (calendarRef.value) {
          const api = calendarRef.value.getApi()
          const baseDate = api.getDate()
          dayStart = new Date(baseDate)
        }
        dayStart.setHours(0, 0, 0, 0)
        const dayEnd = new Date(dayStart)
        dayEnd.setHours(23, 59, 59, 999) // 当天最后一刻
        if (endTime.getTime() > dayEnd.getTime()) {
          // 如果超过当日末尾，则将结束时间钉在日末，开始时间为 max(日始, 日末 - 时长)
          endTime = dayEnd
          const adjustedStartMs = Math.max(dayStart.getTime(), endTime.getTime() - durationMsDrop)
          const adjustedStart = new Date(adjustedStartMs)
          calendarView = {
            type: 'calendar',
            id: `calendar-${adjustedStart.toISOString()}`,
            config: {
              startTime: adjustedStart.toISOString(),
              endTime: endTime.toISOString(),
              isAllDay: false,
            } as CalendarViewConfig,
            label: `${adjustedStart.toLocaleTimeString()} - ${endTime.toLocaleTimeString()}`,
          }
        }

        // 如果上面未因越界而重置 calendarView，则按原始 dropTime 生成
        if (!calendarView) {
          calendarView = {
            type: 'calendar',
            id: `calendar-${dropTime.toISOString()}`,
            config: {
              startTime: dropTime.toISOString(),
              endTime: endTime.toISOString(),
              isAllDay: false, // ✅ 标记为分时事件
            } as CalendarViewConfig,
            label: `${dropTime.toLocaleTimeString()} - ${endTime.toLocaleTimeString()}`,
          }
        }
      }

      // 若意外未生成视图，安全返回
      if (!calendarView) {
        console.error('[Calendar] ❌ Missing calendarView before drop handling')
        clearPreviewEvent()
        isProcessingDrop.value = false
        return
      }

      // 🔍 检查点5：确认策略调用
      console.log('[CHK-5] About to call crossViewDrag.handleDrop with calendarView=', calendarView)

      // 🆕 统一走策略系统
      const result = await crossViewDrag.handleDrop(calendarView, event)

      // 🔍 检查点5：策略结果
      console.log('[CHK-5] Strategy result:', result)

      if (result.success) {
        console.log('[Calendar] ✅ Drop handled via strategy:', result.message)

        // 如果策略返回了更新后的任务，更新到 store
        if (result.updatedTask) {
          taskStore.addOrUpdateTask(result.updatedTask)
        }

        clearPreviewEvent()
      } else {
        console.error('[Calendar] ❌ Drop failed:', result.error)
        alert(`创建时间块失败: ${result.error}`)
        clearPreviewEvent()
      }
    } catch (error) {
      console.error('处理拖拽失败:', error)

      // 清除预览
      clearPreviewEvent()

      // 显示错误信息给用户
      let errorMessage = '创建时间块失败'
      if (error instanceof Error) {
        errorMessage = error.message
      } else if (typeof error === 'string') {
        errorMessage = error
      }

      console.error(`创建时间块失败: ${errorMessage}`)
      alert(`创建时间块失败: ${errorMessage}`)
    } finally {
      // 无论成功还是失败，都要重置标志
      isProcessingDrop.value = false
    }
  }

  /**
   * 初始化 - 注册全局监听器
   */
  function initialize() {
    onMounted(() => {
      // 🔍 检查点2：全局 drop 捕获监听（检测是否被内部拦截）
      document.addEventListener(
        'drop',
        (e) => {
          const target = e.target as HTMLElement
          console.log(
            '[CHK-2] 🌍 Global drop capture! target=',
            target?.className,
            'tagName=',
            target?.tagName
          )
        },
        true
      ) // 捕获阶段

      // 监听全局拖拽开始事件
      document.addEventListener('dragstart', handleGlobalDragStart)
      document.addEventListener('dragend', handleGlobalDragEnd)
    })

    onUnmounted(() => {
      // 清理事件监听器
      document.removeEventListener('dragstart', handleGlobalDragStart)
      document.removeEventListener('dragend', handleGlobalDragEnd)
    })
  }

  return {
    previewEvent,
    isDragging,
    handleDragEnter,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    clearPreviewEvent,
    initialize,
  }
}
