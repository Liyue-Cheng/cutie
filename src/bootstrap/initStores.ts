/**
 * Store 初始化模块
 *
 * 集中管理所有 Store 的：
 * 1. 事件订阅初始化 (SSE)
 * 2. 初始数据加载 (fetch all)
 *
 * 注意：执行顺序很重要，某些 store 依赖其他 store 的数据
 */

import { logger } from '@/infra/logging/logger'
import { pipeline } from '@/cpu'

/**
 * Store 初始化配置
 */
interface StoreInitConfig {
  name: string
  importFn: () => Promise<{ [key: string]: () => any }>
  storeName: string
  initEvents: boolean
  fetchData?: {
    method?: string // store 方法名，默认 'fetchAll'
    usePipeline?: string // 使用 pipeline 指令
  }
}

/**
 * Store 初始化配置列表
 *
 * 顺序很重要：
 * 1. area - 被 task 引用
 * 2. recurrence - 被 task 引用
 * 3. userSettings - 被主题系统使用
 * 4. task - 核心数据
 * 5. timeBlock - 依赖 task
 * 6. project - 独立
 * 7. template - 独立
 * 8. trash - 独立
 */
const STORE_CONFIGS: StoreInitConfig[] = [
  {
    name: 'Area',
    importFn: () => import('@/stores/area'),
    storeName: 'useAreaStore',
    initEvents: true,
    fetchData: { method: 'fetchAll' },
  },
  {
    name: 'Recurrence',
    importFn: () => import('@/stores/recurrence'),
    storeName: 'useRecurrenceStore',
    initEvents: true,
    fetchData: { method: 'fetchAllRecurrences' },
  },
  {
    name: 'UserSettings',
    importFn: () => import('@/stores/user-settings'),
    storeName: 'useUserSettingsStore',
    initEvents: true,
    fetchData: { usePipeline: 'user_settings.fetch_all' },
  },
  {
    name: 'Task',
    importFn: () => import('@/stores/task'),
    storeName: 'useTaskStore',
    initEvents: true,
    // 不预加载，按需加载
  },
  {
    name: 'TimeBlock',
    importFn: () => import('@/stores/timeblock'),
    storeName: 'useTimeBlockStore',
    initEvents: true,
    // 不预加载，按需加载
  },
  {
    name: 'Project',
    importFn: () => import('@/stores/project'),
    storeName: 'useProjectStore',
    initEvents: true,
    // 不预加载，按需加载
  },
  {
    name: 'Template',
    importFn: () => import('@/stores/template'),
    storeName: 'useTemplateStore',
    initEvents: true,
    // 不预加载，按需加载
  },
  {
    name: 'Trash',
    importFn: () => import('@/stores/trash'),
    storeName: 'useTrashStore',
    initEvents: true,
    // 不预加载，按需加载
  },
]

/**
 * 初始化所有 Store
 */
export async function initStores(): Promise<void> {
  logger.info('System:Init', 'Initializing stores...', {
    count: STORE_CONFIGS.length,
  })

  for (const config of STORE_CONFIGS) {
    try {
      // 动态导入 store
      const module = await config.importFn()
      const useStore = module[config.storeName]

      if (!useStore) {
        logger.error(
          'System:Init',
          `Store not found: ${config.storeName}`,
          new Error(`Store ${config.storeName} not exported from module`)
        )
        continue
      }

      const store = useStore()

      // 初始化事件订阅
      if (config.initEvents && typeof store.initEventSubscriptions === 'function') {
        store.initEventSubscriptions()
      }

      // 加载初始数据
      if (config.fetchData) {
        if (config.fetchData.usePipeline) {
          // 使用 pipeline 指令
          await pipeline.dispatch(config.fetchData.usePipeline, {})
        } else if (config.fetchData.method) {
          // 使用 store 方法
          const method = store[config.fetchData.method]
          if (typeof method === 'function') {
            await method.call(store)
          }
        }
      }

      logger.debug('System:Init', `Store initialized: ${config.name}`, {
        events: config.initEvents,
        fetchData: !!config.fetchData,
      })
    } catch (error) {
      logger.error(
        'System:Init',
        `Failed to initialize store: ${config.name}`,
        error instanceof Error ? error : new Error(String(error))
      )
      // 继续初始化其他 store，不要因为一个失败而中断
    }
  }

  logger.info('System:Init', 'All stores initialized')
}
