/**
 * 视图上下文 Composable
 *
 * 提供当前视图的上下文信息，用于完成任务等操作
 */

import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { getTodayDateString } from '@/infra/utils/dateUtils'

/**
 * 获取当前视图上下文
 *
 * 返回格式：{type}::{identifier}
 * 例如：
 * - "daily::2025-10-01" - 日期视图
 * - "misc::staging" - Staging 视图
 * - "misc::incomplete" - 所有未完成任务视图
 * - "area::{uuid}" - 区域视图
 */
export function useViewContext() {
  const route = useRoute()

  const viewContext = computed(() => {
    const routeName = route.name as string | undefined
    const params = route.params

    // 根据路由名称生成视图上下文
    switch (routeName) {
      case 'daily':
      case 'timeline-daily':
        // 日期视图：daily::2025-10-01
        if (params.date) {
          return `daily::${params.date}`
        }
        // 如果没有日期参数，使用今天
        // ⚠️ 使用 getTodayDateString() 获取本地日期，符合 TIME_CONVENTION.md
        return `daily::${getTodayDateString()}`

      case 'staging':
        return 'misc::staging'

      case 'incomplete':
      case 'all-incomplete':
        return 'misc::incomplete'

      case 'planned':
        return 'misc::planned'

      case 'completed':
        return 'misc::completed'

      case 'all':
        return 'misc::all'

      case 'area':
        // 区域视图：area::{uuid}
        if (params.areaId) {
          return `area::${params.areaId}`
        }
        // 降级为 staging
        return 'misc::staging'

      case 'project':
        // 项目视图：project::{uuid}
        if (params.projectId) {
          return `project::${params.projectId}`
        }
        // 降级为 staging
        return 'misc::staging'

      default:
        // 默认降级为 staging
        return 'misc::staging'
    }
  })

  return {
    viewContext,
  }
}
