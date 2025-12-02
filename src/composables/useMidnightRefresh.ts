import { onMounted, onBeforeUnmount } from 'vue'
import { useTimeBlockStore } from '@/stores/timeblock'
import { useTaskStore } from '@/stores/task'
import { useRegisterStore } from '@/stores/register'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 全局午夜刷新机制
 *
 * 该 composable 用于监测本地时间跨越午夜 00:00 的时刻，
 * 并自动刷新所有可能包含过期信息的数据。
 *
 * 主要功能：
 * 1. 每分钟检查一次当前日期是否变化
 * 2. 当检测到日期变化（跨越午夜）时，触发全局数据刷新
 * 3. 自动清理定时器，防止内存泄漏
 *
 * 使用方式：
 * - 在 MainLayout.vue 或 App.vue 中调用 `useMidnightRefresh()`
 */
export function useMidnightRefresh() {
  const timeBlockStore = useTimeBlockStore()
  const taskStore = useTaskStore()
  const registerStore = useRegisterStore()

  let timerId: number | null = null
  let lastKnownDate: string = getCurrentDateString()

  /**
   * 获取当前日期字符串 (YYYY-MM-DD)
   */
  function getCurrentDateString(): string {
    const now = new Date()
    const year = now.getFullYear()
    const month = String(now.getMonth() + 1).padStart(2, '0')
    const day = String(now.getDate()).padStart(2, '0')
    return `${year}-${month}-${day}`
  }

  /**
   * 计算距离下一个午夜的毫秒数
   */
  function getMillisecondsUntilMidnight(): number {
    const now = new Date()
    const midnight = new Date(now)
    midnight.setHours(24, 0, 0, 0) // 设置为明天的00:00:00
    return midnight.getTime() - now.getTime()
  }

  /**
   * 刷新所有数据
   */
  async function refreshAllData() {
    logger.info(LogTags.COMPONENT_CALENDAR, 'Midnight detected - refreshing all data', {
      oldDate: lastKnownDate,
      newDate: getCurrentDateString(),
    })

    try {
      // 1. 刷新当前日历日期周围的时间块数据
      const currentCalendarDate = registerStore.readRegister<string>(
        registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME
      )

      if (currentCalendarDate && typeof currentCalendarDate === 'string') {
        // 刷新当前日期及其前后各1个月的时间块数据
        const centerDate = new Date(currentCalendarDate)
        const startDate = new Date(centerDate.getFullYear(), centerDate.getMonth() - 1, 1)
        const endDate = new Date(centerDate.getFullYear(), centerDate.getMonth() + 2, 0)

        // 🔥 使用本地日期格式（YYYY-MM-DD），符合 TIME_CONVENTION.md 规范
        const formatDate = (d: Date) => {
          const y = d.getFullYear()
          const m = String(d.getMonth() + 1).padStart(2, '0')
          const day = String(d.getDate()).padStart(2, '0')
          return `${y}-${m}-${day}`
        }

        const startDateStr = formatDate(startDate)
        const endDateStr = formatDate(endDate)

        logger.debug(LogTags.COMPONENT_CALENDAR, 'Refreshing time blocks after midnight', {
          startDate: startDateStr,
          endDate: endDateStr,
        })

        await timeBlockStore.fetchTimeBlocksForRange(startDateStr, endDateStr)
      }

      // 2. 刷新今天的任务数据（如果有DMA缓存）
      const today = getCurrentDateString()
      await taskStore.fetchDailyTasks_DMA(today)

      // 3. 更新日历日期到新的今天（如果当前正在显示"今天"）
      if (currentCalendarDate && currentCalendarDate === lastKnownDate) {
        // 用户正在查看的是旧的"今天"，自动切换到新的今天
        registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME, today)
        logger.info(LogTags.COMPONENT_CALENDAR, 'Auto-switched to new today', { newDate: today })
      }

      logger.info(LogTags.COMPONENT_CALENDAR, 'Midnight refresh completed successfully')
    } catch (error) {
      logger.error(
        LogTags.COMPONENT_CALENDAR,
        'Failed to refresh data after midnight',
        error instanceof Error ? error : new Error(String(error))
      )
    }
  }

  /**
   * 检查日期是否变化
   */
  function checkDateChange() {
    const currentDate = getCurrentDateString()

    if (currentDate !== lastKnownDate) {
      // 日期已变化，说明跨越了午夜
      refreshAllData()
      lastKnownDate = currentDate
    }
  }

  /**
   * 设置下一次检查的定时器
   * 在午夜前1分钟开始每分钟检查一次，其他时间每小时检查一次
   */
  function scheduleNextCheck() {
    if (timerId !== null) {
      clearTimeout(timerId)
    }

    const msUntilMidnight = getMillisecondsUntilMidnight()
    const oneHour = 60 * 60 * 1000
    const oneMinute = 60 * 1000
    const fiveMinutes = 5 * oneMinute

    let nextCheckDelay: number

    if (msUntilMidnight < fiveMinutes) {
      // 距离午夜不到5分钟，每分钟检查一次
      nextCheckDelay = oneMinute
      logger.debug(LogTags.COMPONENT_CALENDAR, 'Midnight approaching - checking every minute')
    } else {
      // 距离午夜还早，每小时检查一次
      nextCheckDelay = Math.min(oneHour, msUntilMidnight - fiveMinutes)
    }

    timerId = window.setTimeout(() => {
      checkDateChange()
      scheduleNextCheck() // 递归调度下一次检查
    }, nextCheckDelay)

    logger.debug(LogTags.COMPONENT_CALENDAR, 'Next midnight check scheduled', {
      delayMs: nextCheckDelay,
      nextCheckTime: new Date(Date.now() + nextCheckDelay).toISOString(),
    })
  }

  /**
   * 启动午夜监测
   */
  function startMidnightWatch() {
    logger.info(LogTags.COMPONENT_CALENDAR, 'Midnight watch started', {
      currentDate: lastKnownDate,
    })
    scheduleNextCheck()
  }

  /**
   * 停止午夜监测
   */
  function stopMidnightWatch() {
    if (timerId !== null) {
      clearTimeout(timerId)
      timerId = null
      logger.info(LogTags.COMPONENT_CALENDAR, 'Midnight watch stopped')
    }
  }

  // 组件挂载时启动监测
  onMounted(() => {
    startMidnightWatch()
  })

  // 组件卸载时清理
  onBeforeUnmount(() => {
    stopMidnightWatch()
  })

  return {
    startMidnightWatch,
    stopMidnightWatch,
    refreshAllData,
  }
}
