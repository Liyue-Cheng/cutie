/**
 * 全局对话框管理 composable
 * 替代原生 alert/confirm/prompt
 */
import { ref, readonly } from 'vue'

export type DialogType = 'alert' | 'confirm' | 'prompt'

export interface DialogOptions {
  title?: string
  message: string
  type?: DialogType
  confirmText?: string
  cancelText?: string
  inputPlaceholder?: string
  inputValue?: string
  danger?: boolean
}

interface DialogState {
  visible: boolean
  options: DialogOptions
  resolve: ((value: boolean | string | null) => void) | null
}

const state = ref<DialogState>({
  visible: false,
  options: {
    message: '',
    type: 'alert',
  },
  resolve: null,
})

function showDialog(options: DialogOptions): Promise<boolean | string | null> {
  return new Promise((resolve) => {
    state.value = {
      visible: true,
      options: {
        type: 'alert',
        confirmText: '确定',
        cancelText: '取消',
        ...options,
      },
      resolve,
    }
  })
}

function closeDialog(result: boolean | string | null) {
  if (state.value.resolve) {
    state.value.resolve(result)
  }
  state.value = {
    visible: false,
    options: { message: '', type: 'alert' },
    resolve: null,
  }
}

/**
 * 显示提示框（替代 alert）
 */
export function showAlert(message: string, title?: string): Promise<boolean> {
  return showDialog({
    type: 'alert',
    message,
    title,
  }) as Promise<boolean>
}

/**
 * 显示确认框（替代 confirm）
 */
export function showConfirm(
  message: string,
  options?: { title?: string; confirmText?: string; cancelText?: string; danger?: boolean }
): Promise<boolean> {
  return showDialog({
    type: 'confirm',
    message,
    ...options,
  }) as Promise<boolean>
}

/**
 * 显示输入框（替代 prompt）
 */
export function showPrompt(
  message: string,
  options?: { title?: string; placeholder?: string; defaultValue?: string }
): Promise<string | null> {
  return showDialog({
    type: 'prompt',
    message,
    inputPlaceholder: options?.placeholder,
    inputValue: options?.defaultValue,
    title: options?.title,
  }) as Promise<string | null>
}

/**
 * 用于 Dialog 组件内部的 composable
 */
export function useDialog() {
  return {
    state: readonly(state),
    closeDialog,
  }
}

// 导出便捷函数供全局使用
export const dialog = {
  alert: showAlert,
  confirm: showConfirm,
  prompt: showPrompt,
}
