/**
 * 命令总线类型定义
 *
 * 定义所有可以通过事件总线发送的命令类型
 */

// ============================================================
// 任务相关命令
// ============================================================

export type TaskCommand =
  | {
      type: 'task.create'
      payload: {
        title: string
        area_id?: string | null
        glance_note?: string | null
        detail_note?: string | null
      }
    }
  | {
      type: 'task.create_with_schedule'
      payload: {
        title: string
        scheduled_day: string
        area_id?: string | null
        glance_note?: string | null
        detail_note?: string | null
      }
    }
  | {
      type: 'task.update'
      payload: {
        id: string
        updates: {
          title?: string
          area_id?: string | null
          glance_note?: string | null
          detail_note?: string | null
          estimated_duration?: number | null
          due_date?: string | null
          due_date_type?: 'SOFT' | 'HARD' | null
          subtasks?: any[] | null
        }
      }
    }
  | {
      type: 'task.complete'
      payload: {
        id: string
      }
    }
  | {
      type: 'task.reopen'
      payload: {
        id: string
      }
    }
  | {
      type: 'task.delete'
      payload: {
        id: string
      }
    }
  | {
      type: 'task.archive'
      payload: {
        id: string
      }
    }
  | {
      type: 'task.unarchive'
      payload: {
        id: string
      }
    }
  | {
      type: 'task.return_to_staging'
      payload: {
        id: string
      }
    }

// ============================================================
// 日程相关命令
// ============================================================

export type ScheduleCommand =
  | {
      type: 'schedule.create'
      payload: {
        task_id: string
        scheduled_day: string
      }
    }
  | {
      type: 'schedule.update'
      payload: {
        task_id: string
        scheduled_day: string
        updates: {
          new_date?: string
          outcome?: 'presence_logged' | undefined
        }
      }
    }
  | {
      type: 'schedule.delete'
      payload: {
        task_id: string
        scheduled_day: string
      }
    }

// ============================================================
// 时间块相关命令
// ============================================================

export type TimeBlockCommand =
  | {
      type: 'time_block.create_from_task'
      payload: {
        task_id: string
        start_time: string // ISO 8601
        end_time: string // ISO 8601
        start_time_local?: string | null // HH:mm:ss (可选)
        end_time_local?: string | null // HH:mm:ss (可选)
        time_type?: 'FLOATING' | 'FIXED' // 可选，默认 FLOATING
        creation_timezone?: string | null // 可选
        is_all_day?: boolean // 可选，默认 false
      }
    }
  | {
      type: 'time_block.create'
      payload: {
        title: string
        start_time: string // ISO 8601
        end_time: string // ISO 8601
        start_time_local?: string | null // HH:mm:ss (可选)
        end_time_local?: string | null // HH:mm:ss (可选)
        time_type?: 'FLOATING' | 'FIXED' // 可选，默认 FLOATING
        creation_timezone?: string | null // 可选
        is_all_day?: boolean // 可选，默认 false
      }
    }
  | {
      type: 'time_block.update'
      payload: {
        id: string
        updates: {
          title?: string
          start_time?: string
          end_time?: string
          start_time_local?: string | null
          end_time_local?: string | null
          time_type?: 'FLOATING' | 'FIXED'
          is_all_day?: boolean
          task_id?: string
        }
      }
    }
  | {
      type: 'time_block.delete'
      payload: {
        id: string
      }
    }

// ============================================================
// 模板相关命令
// ============================================================

export type TemplateCommand =
  | {
      type: 'template.create'
      payload: {
        name: string
        title: string
        area_id?: string | null
        glance_note?: string | null
        detail_note?: string | null
        estimated_duration?: number | null
      }
    }
  | {
      type: 'template.update'
      payload: {
        id: string
        updates: {
          name?: string
          title?: string
          area_id?: string | null
          glance_note?: string | null
          detail_note?: string | null
          estimated_duration?: number | null
        }
      }
    }
  | {
      type: 'template.delete'
      payload: {
        id: string
      }
    }
  | {
      type: 'template.create_task'
      payload: {
        template_id: string
        scheduled_day?: string
      }
    }

// ============================================================
// 循环规则相关命令
// ============================================================

export type RecurrenceCommand =
  | {
      type: 'recurrence.create'
      payload: {
        template_id: string
        rule: string
        start_date: string
        end_date?: string | null
      }
    }
  | {
      type: 'recurrence.update'
      payload: {
        id: string
        updates: {
          rule?: string
          end_date?: string | null
          is_active?: boolean
        }
      }
    }
  | {
      type: 'recurrence.delete'
      payload: {
        id: string
      }
    }
  | {
      type: 'recurrence.stop_repeating'
      payload: {
        recurrence_id: string
        original_date: string
      }
    }

// ============================================================
// 垃圾桶相关命令
// ============================================================

export type TrashCommand =
  | {
      type: 'trash.restore'
      payload: {
        id: string
      }
    }
  | {
      type: 'trash.delete_permanently'
      payload: {
        id: string
      }
    }
  | {
      type: 'trash.empty'
      payload: Record<string, never>
    }

// ============================================================
// 视图偏好相关命令
// ============================================================

export type ViewPreferenceCommand = {
  type: 'view.update_sorting'
  payload: {
    view_key: string
    sorted_task_ids: string[]
    /**
     * 🔥 乐观更新支持：用于失败回滚
     * - original_sorted_task_ids: 原始顺序（用于回滚）
     */
    original_sorted_task_ids?: string[]
  }
}

// ============================================================
// 联合类型
// ============================================================

export type Command =
  | TaskCommand
  | ScheduleCommand
  | TimeBlockCommand
  | TemplateCommand
  | RecurrenceCommand
  | TrashCommand
  | ViewPreferenceCommand

/**
 * 命令处理器函数签名
 */
export type CommandHandler<T extends Command = Command> = (payload: T['payload']) => Promise<void>

/**
 * 命令处理器注册表
 */
export type CommandHandlerMap = {
  [K in Command['type']]: CommandHandler<Extract<Command, { type: K }>>
}
