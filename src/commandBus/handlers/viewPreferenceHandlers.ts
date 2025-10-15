/**
 * View Preference Command Handlers
 *
 * 视图偏好命令处理器 - 遵循 Frontend-as-a-CPU 架构
 *
 * 📋 架构原则：
 * - ✅ 只负责业务逻辑编排
 * - ✅ 不直接操作 Store 状态
 * - ✅ 通过 Transaction Processor 统一处理结果
 * - ✅ 自动生成 Correlation ID
 * - ✅ 支持乐观更新（立即更新 + 失败回滚）
 */

import { generateCorrelationId } from '@/infra/correlation'
import { apiPut } from '@/stores/shared'
import { logger, LogTags } from '@/infra/logging/logger'
import { useViewStore } from '@/stores/view'
import type { CommandHandlerMap } from '../types'

/**
 * 处理视图排序更新命令
 *
 * 🔥 乐观更新流程：
 * 1. 立即更新本地状态（预测成功）
 * 2. 发送 API 请求
 * 3. 如果失败，回滚到原始状态
 */
export const handleUpdateSorting: CommandHandlerMap['view.update_sorting'] = async (payload) => {
  const { view_key, sorted_task_ids, original_sorted_task_ids } = payload
  const correlationId = generateCorrelationId()

  // ✅ 移除旧的日志噪音
  const viewStore = useViewStore()

  try {
    // ========== 阶段 1: 乐观更新（立即应用） ==========
    viewStore.updateSortingOptimistic_mut(view_key, sorted_task_ids)

    // ========== 阶段 2: 发送 API 请求 ==========
    const requestBody = {
      sorted_task_ids,
    }

    await apiPut(`/view-preferences/${encodeURIComponent(view_key)}`, requestBody, correlationId)

    // ========== 阶段 3: 成功确认 ==========
    // 成功，无需额外日志
  } catch (error) {
    // ========== 阶段 4: 失败回滚 ==========
    // ✅ 只在错误时记录
    logger.error(
      LogTags.SYSTEM_COMMAND,
      'Failed to update view sorting, rolling back',
      error instanceof Error ? error : new Error(String(error)),
      {
        view_key,
        correlationId,
      }
    )

    // 回滚到原始状态
    if (original_sorted_task_ids) {
      viewStore.updateSortingOptimistic_mut(view_key, original_sorted_task_ids)
    } else {
      // 没有提供原始顺序，清除排序
      viewStore.clearSorting(view_key)
    }

    // 重新抛出错误
    throw error
  }
}

/**
 * View Preference 命令处理器导出
 */
export const viewPreferenceHandlers = {
  'view.update_sorting': handleUpdateSorting,
}
