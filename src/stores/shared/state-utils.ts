import { ref, type Ref } from 'vue'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * 响应式状态管理工具
 *
 * 提供标准化的状态操作方法，确保：
 * - 响应式更新的一致性
 * - Map 操作的不可变性
 * - 统一的状态管理模式
 */

/**
 * 更新 Map 中的单个项目
 * @param mapRef 响应式 Map 引用
 * @param key 键
 * @param value 值
 */
export function updateMapItem<K, V>(mapRef: Ref<Map<K, V>>, key: K, value: V): void {
  const newMap = new Map(mapRef.value)
  newMap.set(key, value)
  mapRef.value = newMap
}

/**
 * 批量更新 Map 中的多个项目
 * @param mapRef 响应式 Map 引用
 * @param items 要更新的项目数组
 * @param getKey 获取键的函数
 */
export function updateMapItems<K, V>(
  mapRef: Ref<Map<K, V>>,
  items: V[],
  getKey: (item: V) => K
): void {
  const newMap = new Map(mapRef.value)
  for (const item of items) {
    const key = getKey(item)
    newMap.set(key, item)
  }
  mapRef.value = newMap
}

/**
 * 从 Map 中删除项目
 * @param mapRef 响应式 Map 引用
 * @param key 要删除的键
 */
export function removeMapItem<K, V>(mapRef: Ref<Map<K, V>>, key: K): void {
  const newMap = new Map(mapRef.value)
  newMap.delete(key)
  mapRef.value = newMap
}

/**
 * 批量从 Map 中删除项目
 * @param mapRef 响应式 Map 引用
 * @param keys 要删除的键数组
 */
export function removeMapItems<K, V>(mapRef: Ref<Map<K, V>>, keys: K[]): void {
  const newMap = new Map(mapRef.value)
  for (const key of keys) {
    newMap.delete(key)
  }
  mapRef.value = newMap
}

/**
 * 清空 Map
 * @param mapRef 响应式 Map 引用
 */
export function clearMap<K, V>(mapRef: Ref<Map<K, V>>): void {
  mapRef.value = new Map()
}

/**
 * 创建标准的加载状态管理
 */
export function createLoadingState() {
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  /**
   * 包装异步操作，自动管理加载状态
   */
  const withLoading = async <T>(
    operation: () => Promise<T>,
    errorPrefix = 'Operation failed'
  ): Promise<T | null> => {
    isLoading.value = true
    error.value = null

    try {
      const result = await operation()
      return result
    } catch (e) {
      error.value = `${errorPrefix}: ${e}`
      logger.error(LogTags.STORE_TASKS, errorPrefix, e instanceof Error ? e : new Error(String(e)))
      return null
    } finally {
      isLoading.value = false
    }
  }

  return {
    isLoading,
    error,
    withLoading,
  }
}
