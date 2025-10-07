/**
 * useCalendarDrag - 日历拖拽功能
 *
 * 处理从任务列表拖拽任务到日历，创建时间块
 */

import { ref, onMounted, onUnmounted, type Ref } from 'vue'
import type { EventInput } from '@fullcalendar/core'
import type FullCalendar from '@fullcalendar/vue3'
import type { TaskCard } from '@/types/dtos'
import type { ViewMetadata, CalendarViewConfig } from '@/types/drag'
import { useCrossViewDrag, useDragTransfer } from '@/composables/drag'
import { useAreaStore } from '@/stores/area'
import { useTaskStore } from '@/stores/task'

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
    // ✅ 检查是否拖到全日区域
    const target =
      (event.target as HTMLElement) ||
      (document.elementFromPoint(event.clientX, event.clientY) as HTMLElement)
    const dayCell = target?.closest('.fc-daygrid-day') as HTMLElement | null
    const isAllDayArea = !!dayCell

    if (isAllDayArea) {
      // 全天预览：优先从 dayCell 的 data-date 获取具体日期
      let startDate: Date | null = null
      let endDate: Date | null = null

      const dateStr = dayCell?.getAttribute('data-date')
      if (dateStr) {
        // 使用本地时区的日期，转为 UTC ISO（避免时区偏移）
        startDate = new Date(`${dateStr}T00:00:00`)
        endDate = new Date(`${dateStr}T00:00:00`)
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
    } else {
      // 分时预览：使用拖拽位置计算时间
      const dropTime = dependencies.getTimeFromDropPosition(
        event,
        event.currentTarget as HTMLElement
      )

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

        let endTime = new Date(dropTime.getTime() + durationMinutes * 60 * 1000)

        // 截断到当日 24:00，禁止跨天预览
        const dayEnd = new Date(dropTime)
        dayEnd.setHours(0, 0, 0, 0)
        dayEnd.setDate(dayEnd.getDate() + 1)
        let startTimeForPreview = dropTime
        if (endTime.getTime() > dayEnd.getTime()) {
          endTime = dayEnd
          const startCandidate = new Date(endTime.getTime() - durationMinutes * 60 * 1000)
          if (startCandidate.getDate() === dropTime.getDate()) {
            startTimeForPreview = startCandidate
          }
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
      // ✅ 检查是否拖到全天区域
      const target =
        (event.target as HTMLElement) ||
        (document.elementFromPoint(event.clientX, event.clientY) as HTMLElement)
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
          startDate = new Date(`${dateStr}T00:00:00Z`)
          endDate = new Date(`${dateStr}T00:00:00Z`)
          endDate.setUTCDate(endDate.getUTCDate() + 1)
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

        // 创建时间块，并在日界处截断
        let endTime = new Date(dropTime.getTime() + durationMinutes * 60 * 1000)
        const dayEnd = new Date(dropTime)
        dayEnd.setHours(0, 0, 0, 0)
        dayEnd.setDate(dayEnd.getDate() + 1)
        if (endTime.getTime() > dayEnd.getTime()) {
          // 如果超过当日末尾，则将结束时间钉在日末，开始时间为日末 - 时长
          endTime = dayEnd
          const startCandidate = new Date(endTime.getTime() - durationMinutes * 60 * 1000)
          // 防止负越界（理论上不会小于当日0点，这里保底）
          if (startCandidate.getDate() === dropTime.getDate()) {
            // 用更贴合的开始时间代替原 dropTime（视觉更自然，不会触顶回跳）
            calendarView = {
              type: 'calendar',
              id: `calendar-${startCandidate.toISOString()}`,
              config: {
                startTime: startCandidate.toISOString(),
                endTime: endTime.toISOString(),
                isAllDay: false,
              } as CalendarViewConfig,
              label: `${startCandidate.toLocaleTimeString()} - ${endTime.toLocaleTimeString()}`,
            }
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
