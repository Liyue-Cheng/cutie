/**
 * 主题初始化模块
 *
 * 负责初始化主题系统，监听主题设置变化
 */

import { watch } from 'vue'
import { useUserSettingsStore } from '@/stores/user-settings'

/**
 * 应用主题到 body
 */
function applyTheme(themeName: string) {
  // 移除所有主题类
  document.body.classList.remove(
    'theme-rose-pine',
    'theme-rose-pine-dawn',
    'theme-rose-pine-moon',
    'theme-cutie',
    'theme-business'
  )

  // 应用新主题类（rose-pine 是默认主题，不需要类名）
  if (themeName !== 'rose-pine') {
    document.body.classList.add(`theme-${themeName}`)
  }
}

/**
 * 初始化主题系统
 */
export function initTheme(): void {
  const store = useUserSettingsStore()

  // 监听主题变化并立即应用当前主题
  watch(
    () => store.theme,
    (newTheme) => {
      applyTheme(newTheme)
    },
    { immediate: true }
  )
}
