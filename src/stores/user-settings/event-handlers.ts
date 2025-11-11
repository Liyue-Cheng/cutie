/**
 * UserSettings Store 事件处理器
 *
 * 职责：
 * - 处理 SSE 推送的用户设置事件
 * - 更新 store 中的设置数据
 */

import { logger, LogTags } from '@/infra/logging/logger'
import type { createUserSettingsCore } from './core'
import type { UserSettingDto } from '@/types/user-settings'

/**
 * 创建事件处理功能
 */
export function createEventHandlers(core: ReturnType<typeof createUserSettingsCore>) {
  /**
   * 初始化事件订阅（由 main.ts 调用）
   *
   * v4.0: 所有事件通过 INT（中断管理器）注册
   */
  function initEventSubscriptions() {
    import('@/cpu/interrupt/InterruptHandler').then(({ interruptHandler }) => {
      // 🔥 注册到 INT（中断管理器）
      interruptHandler.on('user_settings.updated', handleSettingUpdated)
      interruptHandler.on('user_settings.batch_updated', handleBatchUpdated)
      interruptHandler.on('user_settings.reset', handleSettingsReset)

      logger.info(LogTags.STORE, 'UserSettings event subscriptions initialized (v4.0 - via INT)')
    })
  }

  /**
   * 处理单个设置更新事件
   */
  function handleSettingUpdated(event: any) {
    try {
      const setting: UserSettingDto = event.payload
      core.addOrUpdateSetting_mut(setting)

      logger.info(LogTags.STORE, 'Setting updated from SSE', {
        key: setting.setting_key,
      })
    } catch (error) {
      logger.error(
        LogTags.STORE,
        'Failed to process setting updated event',
        error instanceof Error ? error : new Error(String(error)),
        {
          event,
        }
      )
    }
  }

  /**
   * 处理批量设置更新事件
   */
  function handleBatchUpdated(event: any) {
    try {
      const payload = event.payload
      const settings: UserSettingDto[] = payload.settings || []

      core.addOrUpdateBatch_mut(settings)

      logger.info(LogTags.STORE, 'Batch settings updated from SSE', {
        count: settings.length,
      })
    } catch (error) {
      logger.error(
        LogTags.STORE,
        'Failed to process batch updated event',
        error instanceof Error ? error : new Error(String(error)),
        {
          event,
        }
      )
    }
  }

  /**
   * 处理设置重置事件
   */
  function handleSettingsReset(event: any) {
    try {
      const payload = event.payload
      const settings: UserSettingDto[] = payload.settings || []

      core.replaceAll_mut(settings)

      logger.info(LogTags.STORE, 'Settings reset from SSE', {
        count: settings.length,
      })
    } catch (error) {
      logger.error(
        LogTags.STORE,
        'Failed to process settings reset event',
        error instanceof Error ? error : new Error(String(error)),
        {
          event,
        }
      )
    }
  }

  return {
    initEventSubscriptions,
    handleSettingUpdated,
    handleBatchUpdated,
    handleSettingsReset,
  }
}

