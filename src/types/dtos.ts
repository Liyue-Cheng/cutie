/**
 * Cutie 前端数据模型 (DTOs) V2.2 - 最终验收版
 *
 * 核心原则：
 * - 为UI定制: 所有数据结构都为前端渲染而优化，后端负责所有计算、聚合和状态派生。
 * - 状态明确: 避免前端进行逻辑判断，将隐式状态转换为明确的布尔值或枚举。
 * - 时间标准化: 所有时间信息都以 ISO 8601 UTC 字符串格式进行交换。
 */

// --- Base Types ---

export type ID = string

/**
 * 日程状态：任务是否被安排到日历
 */
export type ScheduleStatus = 'scheduled' | 'staging'

/**
 * 截止日期类型
 */
export type DueDateType = 'soft' | 'hard'

/**
 * 当日结局类型
 */
export type DailyOutcome = 'planned' | 'presence_logged' | 'completed' | 'carried_over'

// --- DTO Interfaces ---

/**
 * TaskCard (任务卡片视图模型)
 *
 * 用途: 在各种看板（每日看板、Staging区、项目列表等）上显示的、信息丰富的任务卡片。
 * 它既要轻量，又要提供足够的上下文以支持高效的"一瞥"和流畅的交互。
 */
export interface TaskCard {
  // --- 核心身份 ---
  id: string
  title: string
  glance_note: string | null

  // --- 核心状态 (已解耦) ---
  is_completed: boolean
  is_archived: boolean
  schedule_status: ScheduleStatus

  // --- 详细信息 ---
  subtasks: Array<{
    id: string
    title: string
    is_completed: boolean
    sort_order: string
  }> | null

  // --- 上下文与聚合信息 ---
  area_id: string | null // ✅ 前端通过 area_id 从 area store 获取完整信息

  project_id: string | null

  schedule_info: {
    outcome_for_today: DailyOutcome | null
    is_recurring: boolean
    linked_schedule_count: number
  } | null

  due_date: {
    date: string
    type: DueDateType
    is_overdue: boolean
  } | null

  // --- 日程与时间片信息 ---
  /**
   * 完整的日程列表（包含每天的时间片）
   * null = staging 任务（未安排）
   * [] = planned 任务但无具体时间片
   */
  schedules: Array<{
    scheduled_day: string // YYYY-MM-DD
    outcome: DailyOutcome
    time_blocks: Array<{
      id: string
      start_time: string
      end_time: string
      title: string | null
      glance_note: string | null
    }>
  }> | null

  // --- UI提示标志 ---
  has_detail_note: boolean
}

/**
 * TaskDetail (任务详情视图模型)
 *
 * 用途: 当用户需要查看或编辑一个任务的全部信息时，由后端返回的、最完整的数据模型。
 * 它继承了 TaskCard 的所有属性，并增加了额外的深度信息。
 */
export interface TaskDetail extends TaskCard {
  // --- 额外增加的深度信息 ---

  // 1. 完整的详细笔记
  detail_note: string | null

  // 2. schedules 已从 TaskCard 继承（包含 time_blocks）

  // 3. 完整的项目信息 (如果有关联)
  project: {
    id: string
    name: string
    // 未来可扩展...
  } | null

  // 4. 审计与调试信息
  created_at: string
  updated_at: string
}

/**
 * TimeBlockView (时间块视图模型)
 *
 * 用途: 在日历时间轴上显示的、代表一个时间段的视觉元素。
 */
export interface TimeBlockView {
  // --- 核心身份与时间 ---
  id: string
  start_time: string
  end_time: string
  is_all_day: boolean

  // --- 显示内容 ---
  title: string | null
  glance_note: string | null
  detail_note: string | null

  // --- 染色信息 ---
  area_id: string | null // ✅ 前端通过 area_id 从 area store 获取完整信息

  // --- 关联的任务摘要 ---
  linked_tasks: Array<{
    id: string
    title: string
    is_completed: boolean
  }>

  // --- 其他元信息 ---
  is_recurring: boolean
}
