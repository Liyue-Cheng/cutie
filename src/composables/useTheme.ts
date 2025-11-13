import { watch } from 'vue'
import { useUserSettingsStore } from '@/stores/user-settings'

/**
 * 全局主题管理 composable
 * 负责监听主题设置变化并应用到 document.body
 */

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
 * 应该在应用启动时调用一次
 */
export function initializeTheme() {
  const store = useUserSettingsStore()

  // 监听主题变化并立即应用当前主题
  watch(() => store.theme, (newTheme) => {
    applyTheme(newTheme)
  }, { immediate: true })
}

export function useTheme() {
  return {
    applyTheme,
    initializeTheme
  }
}