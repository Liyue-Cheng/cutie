/**
 * useThrottledDragOver - 节流拖拽事件处理
 *
 * 减少高频 dragover 事件的处理次数，提升性能
 */

import { ref } from 'vue'

const DEFAULT_DELAY = 16 // ~60fps

/**
 * 节流拖拽事件处理
 * @param callback - 回调函数
 * @param delay - 节流延迟（毫秒）
 * @returns 节流后的处理函数
 */
export function useThrottledDragOver<T extends any[]>(
  callback: (event: DragEvent, ...args: T) => void,
  delay: number = DEFAULT_DELAY
): (event: DragEvent, ...args: T) => void {
  const lastExecutionTime = ref(0)
  const timeoutId = ref<number | null>(null)

  return function throttled(event: DragEvent, ...args: T) {
    const now = Date.now()

    // 如果距离上次执行时间超过 delay，立即执行
    if (now - lastExecutionTime.value >= delay) {
      callback(event, ...args)
      lastExecutionTime.value = now
    } else {
      // 否则，清除之前的定时器，设置新的定时器
      if (timeoutId.value !== null) {
        clearTimeout(timeoutId.value)
      }

      timeoutId.value = window.setTimeout(
        () => {
          callback(event, ...args)
          lastExecutionTime.value = Date.now()
          timeoutId.value = null
        },
        delay - (now - lastExecutionTime.value)
      )
    }
  }
}

/**
 * 创建节流处理器（composable 版本）
 */
export function useThrottledCallback<T extends any[]>(
  callback: (...args: T) => void,
  delay: number = DEFAULT_DELAY
) {
  const lastExecutionTime = ref(0)
  const timeoutId = ref<number | null>(null)

  function throttled(...args: T) {
    const now = Date.now()

    if (now - lastExecutionTime.value >= delay) {
      callback(...args)
      lastExecutionTime.value = now
    } else {
      if (timeoutId.value !== null) {
        clearTimeout(timeoutId.value)
      }

      timeoutId.value = window.setTimeout(
        () => {
          callback(...args)
          lastExecutionTime.value = Date.now()
          timeoutId.value = null
        },
        delay - (now - lastExecutionTime.value)
      )
    }
  }

  function cancel() {
    if (timeoutId.value !== null) {
      clearTimeout(timeoutId.value)
      timeoutId.value = null
    }
  }

  return {
    throttled,
    cancel,
  }
}
