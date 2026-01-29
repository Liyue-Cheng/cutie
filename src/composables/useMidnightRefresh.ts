import { onMounted, onBeforeUnmount } from 'vue'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 全局午夜刷新机制
 *
 * 该 composable 用于监测本地时间跨越午夜 00:00 的时刻，
 * 并自动刷新页面以确保所有 UI 状态都是最新的。
 *
 * 主要功能：
 * 1. 智能调度检查（距离午夜远时每小时检查，接近时每分钟检查）
 * 2. 当检测到日期变化（跨越午夜）时，直接刷新页面
 * 3. 自动清理定时器，防止内存泄漏
 *
 * 使用方式：
 * - 在 MainLayout.vue 或 App.vue 中调用 `useMidnightRefresh()`
 */
export function useMidnightRefresh() {
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
   * 午夜刷新 - 直接刷新页面
   *
   * 采用页面刷新而非增量更新的原因：
   * 1. 确保所有 UI 状态（包括各种缓存、计算属性）都能正确更新
   * 2. 避免遗漏某些需要刷新的状态
   * 3. 午夜时用户通常不在使用，页面刷新不影响体验
   */
  function refreshAllData() {
    logger.info(LogTags.COMPONENT_CALENDAR, 'Midnight detected - reloading page', {
      oldDate: lastKnownDate,
      newDate: getCurrentDateString(),
    })

    // 直接刷新页面，等同于 F5
    window.location.reload()
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
