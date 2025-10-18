/**
 * ViewPreference ISA - 视图偏好指令集
 *
 * 包含指令：
 * - viewpreference.update_sorting: 更新视图任务排序
 *
 * 特点：
 * - 支持乐观更新（排序需要即时反馈）
 * - 自动回滚（失败时恢复原始排序）
 */

import { useViewStore } from '@/stores/view'
import type { ISADefinition } from './types'

export const ViewPreferenceISA: ISADefinition = {
  'viewpreference.update_sorting': {
    meta: {
      description: '更新视图任务排序',
      category: 'system',
      resourceIdentifier: (payload) => [`viewpreference:${payload.view_key}`],
      priority: 5,
      timeout: 2000, // 🔥 优化：从 5000ms 降低到 2000ms，因为后端已优化
    },
    optimistic: {
      enabled: true,
      apply: (payload) => {
        const viewStore = useViewStore()

        // 保存原始排序（用于回滚）
        const snapshot = {
          view_key: payload.view_key,
          original_sorted_task_ids: payload.original_sorted_task_ids || null,
        }

        // 🔥 立即更新排序
        viewStore.updateSortingOptimistic_mut(payload.view_key, payload.sorted_task_ids)

        return snapshot
      },
      rollback: (snapshot) => {
        const viewStore = useViewStore()

        // 🔥 回滚到原始排序
        if (snapshot.original_sorted_task_ids) {
          viewStore.updateSortingOptimistic_mut(
            snapshot.view_key,
            snapshot.original_sorted_task_ids
          )
        } else {
          // 没有提供原始顺序，清除排序
          viewStore.clearSorting(snapshot.view_key)
        }
      },
    },
    request: {
      method: 'PUT',
      url: (payload) => `/view-preferences/${encodeURIComponent(payload.view_key)}`,
      body: (payload) => ({
        sorted_task_ids: payload.sorted_task_ids,
      }),
    },
    // 不需要 commit（排序已经在乐观更新中完成）
  },
}
