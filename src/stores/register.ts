import { ref } from 'vue'
import { defineStore } from 'pinia'
import { logger, LogTags } from '@/infra/logging/logger'

/**
 * UI 寄存器 Store - 管理全局 UI 状态的键值对存储
 *
 * 🎯 设计目标：
 * - 彻底消除 props drilling（透传）和组件钻探
 * - 提供类型安全的全局状态存储
 * - 简单的读写 API
 * - 支持任意类型的值存储
 *
 * 📝 使用示例：
 * ```ts
 * const registerStore = useRegisterStore()
 *
 * // 写入
 * registerStore.writeRegister('currentCalendarDate', '2025-10-19')
 *
 * // 读取
 * const date = registerStore.readRegister<string>('currentCalendarDate')
 * ```
 */

export const useRegisterStore = defineStore('register', () => {
  // ==================== 状态 ====================

  /**
   * 核心寄存器：键值对存储
   * 使用 Map 而不是 Record 以获得更好的类型推断和性能
   */
  const registers = ref<Map<string, unknown>>(new Map())

  // ==================== 操作方法 ====================

  /**
   * 写入寄存器
   * @param key 寄存器键名
   * @param value 要存储的值
   */
  function writeRegister<T>(key: string, value: T): void {
    const oldValue = registers.value.get(key)
    registers.value.set(key, value)

    logger.debug(LogTags.STORE_UI, 'Register write', {
      key,
      oldValue,
      newValue: value,
    })
  }

  /**
   * 读取寄存器
   * @param key 寄存器键名
   * @param defaultValue 如果键不存在时返回的默认值
   * @returns 存储的值，如果不存在则返回 defaultValue
   */
  function readRegister<T>(key: string, defaultValue?: T): T | undefined {
    const value = registers.value.get(key) as T | undefined

    if (value === undefined && defaultValue !== undefined) {
      return defaultValue
    }

    return value
  }

  /**
   * 删除寄存器
   * @param key 寄存器键名
   * @returns 是否成功删除
   */
  function deleteRegister(key: string): boolean {
    const existed = registers.value.has(key)
    registers.value.delete(key)

    if (existed) {
      logger.debug(LogTags.STORE_UI, 'Register deleted', { key })
    }

    return existed
  }

  /**
   * 检查寄存器是否存在
   * @param key 寄存器键名
   * @returns 是否存在
   */
  function hasRegister(key: string): boolean {
    return registers.value.has(key)
  }

  /**
   * 清空所有寄存器
   */
  function clearAllRegisters(): void {
    const count = registers.value.size
    registers.value.clear()

    logger.info(LogTags.STORE_UI, 'All registers cleared', { count })
  }

  /**
   * 获取所有寄存器键名
   * @returns 所有键名的数组
   */
  function getAllRegisterKeys(): string[] {
    return Array.from(registers.value.keys())
  }

  // ==================== 预定义寄存器键名（可选，用于类型提示） ====================

  /**
   * 常用寄存器键名
   * 使用常量避免拼写错误
   */
  const RegisterKeys = {
    /** HomeView 中日历显示的日期 (string, YYYY-MM-DD) */
    CURRENT_CALENDAR_DATE_HOME: 'currentCalendarDate_Home',

    /** 当前无限看板的滚动位置 (number) */
    KANBAN_SCROLL_POSITION: 'kanbanScrollPosition',

    /** 当前选中的视图 (string) */
    CURRENT_VIEW: 'currentView',

    /** HomeView 的显示模式 ('default' | 'board' | 'calendar') */
    HOME_VIEW_MODE: 'homeViewMode',

    // 可以继续添加更多预定义键名...
  } as const

  return {
    // 核心方法
    writeRegister,
    readRegister,
    deleteRegister,
    hasRegister,
    clearAllRegisters,
    getAllRegisterKeys,

    // 预定义键名（可选使用）
    RegisterKeys,

    // 直接暴露 registers 用于响应式监听（高级用法）
    registers,
  }
})
