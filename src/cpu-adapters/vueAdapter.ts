/**
 * Vue响应式状态适配器
 *
 * 提供Vue响应式状态工厂
 */

import { ref, watch } from 'vue'
import type { IReactiveState } from '@cutie/cpu-pipeline'

/**
 * 创建Vue响应式状态
 */
export function createVueReactiveState<T>(initialValue: T): IReactiveState<T> {
  const state = ref(initialValue)
  const subscribers: Array<(value: T) => void> = []

  // 监听Vue的ref变化
  watch(
    state,
    (newValue) => {
      subscribers.forEach((cb) => cb(newValue as T))
    },
    { deep: true }
  )

  return {
    get value() {
      return state.value as T
    },
    setValue(newValue: T) {
      state.value = newValue as any
    },
    subscribe(callback) {
      subscribers.push(callback)
      return () => {
        const index = subscribers.indexOf(callback)
        if (index > -1) subscribers.splice(index, 1)
      }
    },
  }
}
