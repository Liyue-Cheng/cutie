/**
 * 策略导出入口
 *
 * 集中导出所有策略，便于统一注册
 */

// 任务调度策略
export {
  stagingToDailyStrategy,
  dailyToDailyStrategy,
  dailyToStagingStrategy,
  dailyReorderStrategy,
  stagingReorderStrategy,
} from './task-scheduling'

// 日历调度策略
export { anyToCalendarAllDayStrategy, anyToCalendarTimedStrategy } from './calendar-scheduling'

// 模板调度策略
export {
  templateToDailyStrategy,
  dailyToTemplateStrategy,
  templateReorderStrategy,
} from './template-scheduling'

// 未来可以添加更多策略模块：
// export * from './task-status'
// export * from './task-ordering'
