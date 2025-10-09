import type { ViewMetadata, DateViewConfig } from '@/types/drag'
import { logger, LogTags } from '@/services/logger'

/**
 * 解析 viewKey 获取类型和标识符
 * @param viewKey 视图上下文键，格式：{type}::{identifier}
 * @returns 解析结果 { type, id }
 */
export function parseViewKey(viewKey: string): { type: string; id: string } {
  const parts = viewKey.split('::')
  if (parts.length < 2) {
    throw new Error(`Invalid viewKey format: ${viewKey}. Expected format: {type}::{identifier}`)
  }

  const [type, ...idParts] = parts
  const id = idParts.join('::') // 支持复合标识符（未来扩展）

  if (!type || !id) {
    throw new Error(`Invalid viewKey format: ${viewKey}. Type and id cannot be empty`)
  }

  return { type, id }
}

/**
 * 根据 viewKey 自动生成最小可用的 ViewMetadata
 * 用于父组件未提供 viewMetadata 时的兜底
 *
 * @param viewKey 视图上下文键
 * @returns ViewMetadata 或 undefined（如果无法解析）
 */
export function deriveViewMetadata(viewKey?: string): ViewMetadata | undefined {
  if (!viewKey) {
    return undefined
  }

  try {
    const { type, id } = parseViewKey(viewKey)

    switch (type) {
      case 'daily': {
        // 日期看板：提供 DateViewConfig
        const config: DateViewConfig = {
          date: id, // YYYY-MM-DD
        }
        return {
          id: viewKey,
          type: 'date',
          config,
          label: `日期: ${id}`,
        }
      }

      case 'area': {
        // 区域看板：基础元数据
        return {
          id: viewKey,
          type: 'area',
          label: `区域: ${id}`,
        } as ViewMetadata
      }

      case 'project': {
        // 项目看板：基础元数据
        return {
          id: viewKey,
          type: 'project',
          label: `项目: ${id}`,
        } as ViewMetadata
      }

      case 'misc': {
        // 杂项看板：根据 id 提供友好标签
        const labels: Record<string, string> = {
          all: '所有任务',
          staging: 'Staging 区',
          planned: '已安排',
          incomplete: '未完成',
          completed: '已完成',
          archive: '归档', // 🆕 添加归档支持
        }

        return {
          id: viewKey,
          type: 'status', // 使用 ViewType 中的有效值
          label: labels[id] || `杂项: ${id}`,
          config: {}, // 提供空配置对象
        } as ViewMetadata
      }

      default: {
        // 未知类型：提供基础元数据
        logger.warn(LogTags.API_VIEW_ADAPTER, 'Unknown viewKey type', { type, viewKey })
        return {
          id: viewKey,
          type: type as any,
          label: `${type}: ${id}`,
        } as ViewMetadata
      }
    }
  } catch (error) {
    logger.error(
      LogTags.API_VIEW_ADAPTER,
      'Failed to derive ViewMetadata',
      error instanceof Error ? error : new Error(String(error)),
      { viewKey }
    )
    return undefined
  }
}

/**
 * 验证 viewKey 格式是否符合规范
 * @param viewKey 视图上下文键
 * @returns 是否有效
 */
export function validateViewKey(viewKey: string): boolean {
  try {
    const { type, id } = parseViewKey(viewKey)

    // 检查类型是否支持
    const supportedTypes = ['misc', 'daily', 'area', 'project']
    if (!supportedTypes.includes(type)) {
      return false
    }

    // 检查标识符是否非空
    if (!id || id.trim().length === 0) {
      return false
    }

    // 特殊验证：misc 类型的 id 必须是预定义值
    if (type === 'misc') {
      const validMiscIds = ['all', 'staging', 'planned', 'incomplete', 'completed', 'archive']
      if (!validMiscIds.includes(id)) {
        return false
      }
    }

    // 特殊验证：daily 类型的 id 应该是日期格式 YYYY-MM-DD
    if (type === 'daily') {
      const dateRegex = /^\d{4}-\d{2}-\d{2}$/
      if (!dateRegex.test(id)) {
        return false
      }
    }

    return true
  } catch {
    return false
  }
}

/**
 * 生成标准的 viewKey
 * @param type 视图类型
 * @param id 标识符
 * @returns 标准格式的 viewKey
 */
export function createViewKey(type: string, id: string): string {
  if (!type || !id) {
    throw new Error('Type and id are required')
  }

  const viewKey = `${type}::${id}`

  if (!validateViewKey(viewKey)) {
    throw new Error(`Generated invalid viewKey: ${viewKey}`)
  }

  return viewKey
}
