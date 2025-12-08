/**
 * 用户设置相关类型定义
 */

/**
 * 设置值类型
 */
export type ValueType = 'string' | 'number' | 'boolean' | 'object' | 'array'

/**
 * 用户设置 DTO
 * Key 格式: {category}.{group?}.{name}
 * 示例: appearance.theme, ai.conversation.api_key, debug.test_string
 */
export interface UserSettingDto {
  setting_key: string
  setting_value: string // JSON 字符串
  value_type: ValueType
  updated_at: string // ISO 8601 UTC
  created_at: string // ISO 8601 UTC
}

/**
 * 更新单个设置请求
 */
export interface UpdateSettingRequest {
  value: any // JSON value
  value_type: ValueType
}

/**
 * 批量更新设置请求
 */
export interface UpdateBatchSettingsRequest {
  settings: Array<{
    key: string
    value: any
    value_type: ValueType
  }>
}

/**
 * 批量更新响应
 */
export interface BatchUpdateResponse {
  updated_count: number
  settings: UserSettingDto[]
}

/**
 * 重置响应
 */
export interface ResetResponse {
  reset_count: number
  settings: UserSettingDto[]
}
