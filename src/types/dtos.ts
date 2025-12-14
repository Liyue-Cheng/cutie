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
 * 截止日期类型
 */
export type DueDateType = 'SOFT' | 'HARD'

/**
 * 当日结局类型
 */
export type DailyOutcome = 'planned' | 'presence_logged' | 'completed' | 'carried_over'

/**
 * 时间类型：定义时间块的时间解释方式
 */
export type TimeType = 'FLOATING' | 'FIXED'

/**
 * 模板类别
 */
export type TemplateCategory = 'GENERAL' | 'RECURRENCE'

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
  is_deleted: boolean
  deleted_at: string | null // ISO 8601 UTC 字符串，null 表示未删除
  // schedule_status 已删除 - 根据 schedules 实时计算

  // --- 详细信息 ---
  subtasks: Array<{
    id: string
    title: string
    is_completed: boolean
    sort_order: string
  }> | null

  // --- 时间估算 ---
  estimated_duration: number | null // 预期时长（分钟），null 表示未设置

  // --- 上下文与聚合信息 ---
  area_id: string | null // ✅ 前端通过 area_id 从 area store 获取完整信息

  project_id: string | null
  section_id: string | null // 项目章节ID

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

  // --- 循环任务相关字段 ---
  recurrence_id: string | null // 循环规则ID，null 表示非循环任务
  recurrence_original_date: string | null // 循环任务的原始日期 (YYYY-MM-DD)
  recurrence_expiry_behavior: string | null // 循环任务的过期行为 ("CARRYOVER_TO_STAGING" | "EXPIRE")

  // --- 排序信息 ---
  /**
   * 视图上下文 → LexoRank 字符串
   * 后端若无排序信息会省略该字段
   */
  sort_positions?: Record<string, string>
}

/**
 * Daily 视图批量查询响应
 */
export interface DailyViewTasksPayload {
  view_key: string
  date: string
  count: number
  tasks: TaskCard[]
}

export interface DailyRangeMeta {
  start_view_key: string
  end_view_key: string
  start_date: string
  end_date: string
  total_days: number
}

export interface BatchDailyTasksResponse {
  range: DailyRangeMeta
  views: DailyViewTasksPayload[]
  total_tasks: number
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
  start_time: string // UTC ISO 8601 时间戳
  end_time: string // UTC ISO 8601 时间戳
  /** 本地开始时间 (HH:MM:SS)，仅在time_type=FLOATING时有值 */
  start_time_local: string | null
  /** 本地结束时间 (HH:MM:SS)，仅在time_type=FLOATING时有值 */
  end_time_local: string | null
  /** 时间类型：FLOATING(浮动时间) 或 FIXED(固定时间) */
  time_type: TimeType
  /** 创建时的时区（占位字段） */
  creation_timezone: string | null
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

  // --- 循环相关字段 ---
  is_recurring: boolean
  recurrence_id: string | null // 循环规则ID（通过 time_block_recurrence_links 表关联）
  recurrence_original_date: string | null // 循环原始日期 (YYYY-MM-DD)
}

/**
 * ProjectCard (项目卡片视图模型)
 *
 * 用途: 在项目列表中显示的项目卡片
 */
/**
 * ProjectCard (项目卡片视图模型)
 *
 * 注意：任务统计由前端基于 task store 实时计算
 * 使用 projectStore.getProjectStatsRealtime(projectId) 获取统计信息
 */
export interface ProjectCard {
  id: string
  name: string
  description: string | null
  status: 'ACTIVE' | 'COMPLETED'
  due_date: string | null // YYYY-MM-DD
  completed_at: string | null // ISO 8601 UTC
  area_id: string | null
  created_at: string // ISO 8601 UTC
  updated_at: string // ISO 8601 UTC
}

/**
 * ProjectSection (项目章节视图模型)
 *
 * 用途: 在项目详情中显示的章节
 */
export interface ProjectSection {
  id: string
  project_id: string
  title: string
  description: string | null
  sort_order: string | null
  created_at: string // ISO 8601 UTC
  updated_at: string // ISO 8601 UTC
}

/**
 * Template (模板)
 *
 * 用途: 快速创建具有相同结构的任务
 */
export interface Template {
  id: string
  title: string
  glance_note_template: string | null
  detail_note_template: string | null
  estimated_duration_template: number | null
  subtasks_template: Array<{
    id: string
    title: string
    is_completed: boolean
    sort_order: string
  }> | null
  area_id: string | null
  category: TemplateCategory
  sort_rank: string | null
  created_at: string
  updated_at: string
}

/**
 * TaskRecurrence (循环任务规则)
 *
 * 用途: 定义循环任务的规则，系统将根据 RRULE 标准规则自动生成任务实例
 */
export interface TaskRecurrence {
  id: string
  template_id: string
  rule: string // RRULE 标准字符串，如 "FREQ=DAILY" 或 "FREQ=WEEKLY;BYDAY=MO,WE,FR"
  time_type: TimeType // 时间类型
  start_date: string | null // 生效起始日期 (YYYY-MM-DD)
  end_date: string | null // 生效结束日期 (YYYY-MM-DD)
  timezone: string | null // 时区（仅 FIXED 类型使用）
  expiry_behavior: 'CARRYOVER_TO_STAGING' | 'EXPIRE' // 过期行为
  is_active: boolean // 是否激活
  created_at: string
  updated_at: string
}

/**
 * CreateTaskRecurrencePayload (创建循环规则的请求载荷)
 */
export interface CreateTaskRecurrencePayload {
  template_id: string
  rule: string
  time_type?: TimeType
  start_date?: string | null
  end_date?: string | null
  timezone?: string | null
  expiry_behavior?: 'CARRYOVER_TO_STAGING' | 'EXPIRE' // 过期行为
  is_active?: boolean
  source_task_id?: string // 源任务ID - 如果提供，将其作为第一个循环实例（避免重复创建）
}

/**
 * UpdateTaskRecurrencePayload (更新循环规则的请求载荷)
 *
 * 注意：后端使用三态字段 Option<Option<T>>，需要区分：
 * - undefined: 不更新该字段
 * - null: 清空该字段
 * - value: 更新为指定值
 */
export interface UpdateTaskRecurrencePayload {
  template_id?: string
  rule?: string
  time_type?: TimeType
  start_date?: string | null // 三态：undefined=不更新, null=清空, string=设置值
  end_date?: string | null // 三态：undefined=不更新, null=清空, string=设置值
  timezone?: string | null // 三态：undefined=不更新, null=清空, string=设置值
  expiry_behavior?: 'CARRYOVER_TO_STAGING' | 'EXPIRE' // 过期行为
  is_active?: boolean
}

// --- Time Block Recurrence Types ---

/**
 * TimeBlockRecurrence (时间块循环规则)
 *
 * 用途: 定义时间块循环规则，系统将根据 RRULE 标准规则自动生成时间块实例
 */
export interface TimeBlockRecurrence {
  id: string
  template_id: string
  rule: string // RRULE 标准字符串
  time_type: TimeType
  start_date: string | null
  end_date: string | null
  timezone: string | null
  is_active: boolean
  created_at: string
  updated_at: string
  template?: TimeBlockTemplateInfo | null
}

/**
 * TimeBlockTemplateInfo (时间块模板简要信息)
 */
export interface TimeBlockTemplateInfo {
  id: string
  title: string | null
  glance_note_template?: string | null
  detail_note_template?: string | null
  duration_minutes: number
  start_time_local: string // HH:MM:SS
  is_all_day: boolean
  area_id: string | null
}

/**
 * CreateTimeBlockRecurrencePayload (创建时间块循环规则的请求载荷)
 */
export interface CreateTimeBlockRecurrencePayload {
  // 模板信息
  title?: string | null
  glance_note_template?: string | null
  detail_note_template?: string | null
  duration_minutes: number
  start_time_local: string // HH:MM:SS
  time_type?: TimeType
  is_all_day?: boolean
  area_id?: string | null

  // 循环规则信息
  rule: string
  start_date?: string | null
  end_date?: string | null
  timezone?: string | null

  // 源时间块（可选）
  source_time_block_id?: string
}

/**
 * UpdateTimeBlockRecurrencePayload (更新时间块循环规则的请求载荷)
 */
export interface UpdateTimeBlockRecurrencePayload {
  rule?: string
  time_type?: TimeType
  start_date?: string | null
  end_date?: string | null
  timezone?: string | null
  is_active?: boolean
}

/**
 * EditTimeBlockRecurrencePayload (编辑时间块循环规则的请求载荷)
 */
export interface EditTimeBlockRecurrencePayload {
  id: string
  rule?: string
  end_date?: string | null
  timezone?: string | null
  time_type?: TimeType
  title?: string | null
  glance_note_template?: string | null
  detail_note_template?: string | null
  duration_minutes?: number
  is_all_day?: boolean
  area_id?: string | null
  local_now: string
  delete_future_instances?: boolean
}

/**
 * TimeBlockRecurrenceEditResult (编辑循环规则响应)
 */
export interface TimeBlockRecurrenceEditResult {
  recurrence: TimeBlockRecurrence
  deleted_time_block_ids: string[]
  deleted_count: number
}

// --- Daily Shutdown Ritual Types ---

export interface ShutdownRitualStep {
  id: string
  title: string
  order_rank: string
  created_at: string
  updated_at: string
}

export interface ShutdownRitualProgress {
  step_id: string
  date: string // YYYY-MM-DD
  completed_at: string | null
}

export interface ShutdownRitualState {
  date: string // YYYY-MM-DD
  /** NULL = use frontend fallback/i18n */
  title: string | null
  steps: ShutdownRitualStep[]
  progress: ShutdownRitualProgress[]
}

export interface ShutdownRitualSettings {
  /** NULL = use frontend fallback/i18n */
  title: string | null
  updated_at: string
}

export interface UpdateShutdownRitualStepSortResponse {
  step_id: string
  new_rank: string
}

// --- Drag & Drop Type System ---

/**
 * DragObjectType (拖放对象类型)
 *
 * 定义系统中可以被拖放的对象类型
 */
export type DragObjectType = 'task' | 'template' | 'project' | 'area' | 'time-block' | 'other'

/**
 * DragObject (拖放对象联合类型)
 *
 * 所有可以被拖放的对象的联合类型，用于泛型约束
 */
export type DragObject = TaskCard | Template | TimeBlockView

/**
 * 类型守卫：检查是否为 TaskCard
 * 通过检查 TaskCard 特有的字段来判断（schedules 字段是 TaskCard 特有的）
 */
export function isTaskCard(obj: DragObject): obj is TaskCard {
  return obj && typeof obj === 'object' && 'schedules' in obj
}

/**
 * 类型守卫：检查是否为 Template
 */
export function isTemplate(obj: DragObject): obj is Template {
  return obj && typeof obj === 'object' && 'estimated_duration_template' in obj
}

/**
 * 类型守卫：检查是否为 TimeBlockView
 */
export function isTimeBlockView(obj: DragObject): obj is TimeBlockView {
  return obj && typeof obj === 'object' && 'time_type' in obj && 'is_all_day' in obj
}
