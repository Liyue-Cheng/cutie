const DEFAULT_FALLBACK_COLOR = '#9ca3af'

function readCssVariable(variableName: string): string | null {
  if (typeof window === 'undefined') {
    return null
  }
  const target = document.body || document.documentElement
  if (!target) {
    return null
  }
  const computed = window.getComputedStyle(target)
  const value = computed.getPropertyValue(variableName)
  if (!value) {
    return null
  }
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

/**
 * 获取当前主题下的默认区域颜色
 * 优先读取 CSS 变量 --color-area-default，否则退回到灰色
 */
export function getDefaultAreaColor(): string {
  const cssValue = readCssVariable('--color-area-default')
  return cssValue || DEFAULT_FALLBACK_COLOR
}

export function getDefaultAreaColorFallback() {
  return DEFAULT_FALLBACK_COLOR
}
