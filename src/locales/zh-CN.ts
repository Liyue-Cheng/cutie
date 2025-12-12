export default {
  // ==================== 导航与布局 ====================
  nav: {
    recent: '最近',
    staging: '暂存区',
    upcoming: '即将到来',
    calendar: '日历',
    projects: '项目',
    templates: '模板',
    dailyOverview: '每日规划',
    dailyShutdown: '每日收尾',
    stagingKanban: '暂存区看板',
    timelineKanban: '时间轴看板',
    calendarKanban: '日历看板',
    areas: '区域',
    recurrence: '循环任务',
    settings: '设置',
    section: {
      dailyRoutines: '每日例行',
      kanban: '看板',
    },
    quickAddTask: '快速添加任务',
  },

  // ==================== 侧边栏（兼容旧结构） ====================
  sidebar: {
    header: 'Cutie .',
    timeline: '时间轴',
    planning: '计划',
    upcoming: '即将到来',
    allTasks: '所有任务',
    projects: '项目',
    experience: '经历',
    settings: '设置',
  },

  // ==================== 工具栏视图 ====================
  toolbar: {
    calendar: '日历',
    timeline: '时间线',
    staging: '暂存区',
    upcoming: '即将到来',
    dailyTasks: '当天任务',
    templates: '模板',
    projects: '项目',
    polling: '轮询',
    completed: '已完成',
    archive: '已归档',
    deleted: '最近删除',
    deadline: '截止日期',
  },

  // ==================== 任务相关 ====================
  task: {
    action: {
      edit: '编辑任务',
      delete: '删除任务',
      archive: '归档任务',
      unarchive: '取消归档',
      returnToStaging: '返回暂存区',
      cancelTodaySchedule: '取消今日排期',
      complete: '完成',
      reopen: '重新打开',
      addTask: '+ 添加任务',
      addNewTask: '添加新任务...',
      sort: '排序',
    },
    placeholder: {
      description: '任务描述...',
      addSubtask: '添加子任务...',
      addSubtaskTemplate: '添加子任务模板...',
      detailNote: '详细笔记...',
      detailNoteTemplate: '详细笔记模板...',
      glanceNoteTemplate: '快速概览笔记模板...',
      title: '输入任务标题...',
    },
    label: {
      subtasks: '子任务',
      subtaskTemplate: '子任务模板',
      noArea: '无区域',
      unscheduled: '未安排的任务',
      archived: '已归档的任务',
      scheduled: '待安排',
      noTasks: '暂无任务',
      noDeadlineTasks: '没有设置截止日期的任务',
      noLinkedTasks: '暂无链接任务',
      linkedTasks: '链接的任务',
      recentCarryover: '最近结转',
    },
    button: {
      done: '完成',
    },
    duration: {
      tiny: 'tiny',
    },
    status: {
      completed: '已完成',
      incomplete: '未完成',
      inProgress: '进行中',
    },
    count: {
      overdue: '{n} 个已逾期',
      tasks: '任务',
    },
    empty: {
      noStagingTasks: '暂无暂存区任务',
    },
  },

  // ==================== 截止日期 ====================
  dueDate: {
    setDueDate: '设置截止日期',
    label: {
      date: '日期',
      type: '类型',
    },
    type: {
      soft: '软截止',
      hard: '硬截止',
    },
  },

  // ==================== 循环任务 ====================
  recurrence: {
    title: {
      manager: '循环任务管理',
      config: '配置循环规则',
      taskConfig: '为任务 "{title}" 设置循环',
      edit: '编辑循环规则',
      timeBlockConfig: '配置时间块循环',
      timeBlockConfigDesc: '为时间块 "{title}" 设置循环规则',
      timeBlockEdit: '编辑时间块循环',
      timeBlockEditDesc: '调整循环规则并处理未来实例',
    },
    label: {
      frequency: '重复频率',
      selectWeekday: '选择星期',
      expiryBehavior: '过期后的处理方式',
      expiryBehaviorHint: '未完成的任务在当天结束后的处理方式',
      endDate: '结束日期（可选）',
      startDate: '开始日期',
      startDateHint: '留空表示立即生效',
      endDateHint: '留空表示永久有效',
      interval: '每',
      intervalSuffix: '周',
      monthDay: '每月几号',
      monthDaySuffix: '号',
      month: '月',
      start: '开始',
      end: '结束',
      conflictBehavior: '遇到时间冲突时',
      isRecurring: '循环时间块',
      templateTitle: '模板标题',
      shortNote: '快览笔记模板',
      detailNote: '详细笔记模板',
      futureInstances: '未来实例处理',
      deleteFutureInstancesDesc: '从当前时间起删除所有尚未开始的实例',
    },
    freq: {
      daily: '每天',
      weekly: '每周',
      monthly: '每月特定日期',
      yearly: '每年',
      weekdays: '工作日（周一至周五）',
    },
    weekday: {
      mon: '周一',
      tue: '周二',
      wed: '周三',
      thu: '周四',
      fri: '周五',
      sat: '周六',
      sun: '周日',
    },
    description: {
      daily: '每日循环',
      weekly: '每周循环',
      monthly: '每月循环',
      yearly: '每年循环',
    },
    timeType: {
      floating: '浮动时间',
      fixed: '固定时间',
    },
    expiry: {
      carryover: '转入暂存',
      carryoverFull: '结转到暂存区',
      carryoverDesc: '如果今天忘记完成，任务会进入暂存区等待处理（如：交水电费）',
      expire: '自动过期',
      expireDesc: '如果今天没完成，任务自动失效，不再提醒（如：每日签到、游戏日常）',
    },
    conflict: {
      skip: '跳过冲突时段',
      skipDesc: '如果该时间段已有其他时间块，则跳过不创建',
      error: '报错阻止创建',
      errorDesc: '如果遇到时间冲突，则停止创建并提示错误',
    },
    status: {
      active: '激活',
      expired: '过期',
      inactive: '已停用',
    },
    action: {
      stopRepeating: '停止重复',
      stop: '停止重复',
      continue: '继续循环',
      deleteRule: '删除规则',
      changeFrequency: '修改重复频率',
      updateAll: '更新所有未完成实例以匹配此任务',
      deleteAll: '删除所有未完成实例并停止重复',
      pause: '停用',
      resume: '启用',
      setRecurrence: '设置循环',
      deleteFutureInstances: '删除未来实例',
    },
    menuSection: '任务循环设置：',
    empty: {
      title: '暂无循环任务规则',
      hint: '在任务编辑器中可以为任务设置循环规则',
    },
    unknownTask: '未知任务',
  },

  // ==================== 回收站 ====================
  trash: {
    title: '回收站',
    action: {
      empty: '清空回收站',
      restore: '恢复',
      permanentDelete: '彻底删除',
    },
    empty: {
      message: '回收站是空的',
    },
    label: {
      deletedAt: '删除于 {time}',
    },
  },

  // ==================== 时间格式化 ====================
  time: {
    unknown: '未知时间',
    justNow: '刚刚',
    minutesAgo: '{n} 分钟前',
    hoursAgo: '{n} 小时前',
    daysAgo: '{n} 天前',
    yesterday: '昨天',
    today: '今天',
    tomorrow: '明天',
    thisWeek: '本周',
    nextWeek: '下周',
    thisMonth: '本月',
    farFuture: '更远',
    overdue: '逾期',
  },

  // ==================== 日历 ====================
  calendar: {
    view: {
      week: '周视图',
      month: '月视图',
      day: '日视图',
    },
    action: {
      previous: '上一周/月',
      next: '下一周/月',
      switchTo3Days: '切换到3天视图',
      switchTo1Day: '切换到1天视图',
    },
  },

  // ==================== 设置 ====================
  settings: {
    title: '设置',
    category: {
      ai: 'AI',
      appearance: '外观',
      behavior: '行为',
      data: '数据',
      account: '账号',
      debug: '调试',
      system: '系统',
    },
    ai: {
      title: 'AI 设置',
      description: '配置对话模型与快速模型的接入信息',
      conversation: {
        title: '对话模型（用于 AI 对话）',
        description: '应用于 AI 助手聊天的模型配置，需填写可用的请求地址、密钥与模型名称。',
      },
      quick: {
        title: '快速模型（用于任务分类等快速调用）',
        description: '用于自动分类等快速任务的轻量模型，推荐配置高性能、低延迟的模型。',
      },
      field: {
        apiBaseUrl: 'API 地址',
        apiBaseUrlDesc: '例如：https://api.openai.com/v1',
        apiKey: 'API 密钥',
        apiKeyDesc: '用于访问模型的密钥',
        model: '模型',
        modelDesc: '模型名称，例如 gpt-4o、glm-4.5 等',
      },
    },
    appearance: {
      title: '外观设置',
      description: '外观和主题相关的设置选项',
      theme: '主题',
      themeDesc: '选择应用程序的外观主题',
    },
    behavior: {
      title: '行为设置',
      description: '任务和应用行为相关的设置选项',
      taskCompletion: {
        title: '任务完成',
        description: '控制任务完成时的行为',
        createSchedule: '完成时创建日程',
        createScheduleDesc: '在非当日视图完成任务时，自动在今天创建一条已完成的日程记录',
      },
    },
    debug: {
      title: '调试设置',
      description: '开发和调试相关的设置选项',
    },
    theme: {
      rosePine: '暮色庭院',
      rosePineDawn: '晨光花园',
      rosePineMoon: '月夜花园',
      cutie: '甜心梦境',
      business: '简约办公',
    },
    action: {
      resetAll: '重置全部',
      close: '关闭',
    },
    empty: {
      title: '暂无设置项',
      comingSoon: '该分类的设置即将推出',
    },
  },

  // ==================== Area 管理 ====================
  area: {
    title: {
      manager: '区域管理器',
      create: '创建新区域',
      all: '所有区域',
    },
    placeholder: {
      name: '输入区域名称...',
    },
    action: {
      selectColor: '选择颜色',
      aiColor: 'AI 自动染色',
      add: '添加',
      clearArea: '清除区域',
    },
    count: '{n} 个',
    empty: {
      title: '还没有创建任何区域',
      hint: '区域可以帮助你组织和分类任务',
    },
    message: {
      enterName: '请先输入区域名称',
    },
  },

  // ==================== 快速添加任务 ====================
  quickAdd: {
    title: '快速添加任务',
    placeholder: '输入任务标题...',
    button: {
      addTo: '添加到 {date}',
      add: '添加任务',
    },
  },

  // ==================== 视图标题 ====================
  view: {
    staging: {
      title: '暂存区',
      areaCount: '{n} 个区域',
      empty: {
        selectCategory: '选择一个分类查看任务',
        notExist: '分类不存在',
      },
      desc: {
        recentCarryover: '过去 5 天内有排期但目前属于暂存区的任务',
        noArea: '未分配区域的暂存区任务',
      },
    },
    upcoming: {
      title: '即将到来',
    },
    dailyOverview: {
      greeting: '今天想做些什么有趣的事？',
      weather: '天气',
      weatherHint: '天气数据暂为示意，后续可接入真实天气 API',
      dailyRecurring: '每日循环',
      noDailyRecurring: '今天没有每日循环任务',
      todaysTasks: '今日任务',
    },
    dailyPlanning: {
      dailyTasksNav: {
        prev: '上一天（跳过今天）',
        next: '下一天（跳过今天）',
        todayJump: '今天（实际上会跳到明天）',
      },
      step1: {
        title: '想做些什么有趣的事？',
        subtitle: '从暂存区挑选任务吧～',
        hint: '不用太多，三五件刚刚好～',
      },
      step2: {
        title: '想什么时候做呢？',
        subtitle: '把任务拖到右边的日历里～',
        hint: '留点空隙，别把自己塞太满～',
      },
      next: '下一步',
      done: '完成',
      back: '返回',
    },
    dailyShutdown: {
      title: '每日收尾',
      todayIncomplete: '今日 · 未完成',
      todayCompleted: '今日 · 已完成',
      tomorrow: '明天',
      ritual: '今日小仪式',
      ritualDesc: '收尾小仪式正在设计中，未来会在这里帮助你温柔结束这一天。',
    },
    projects: {
      title: '项目',
    },
  },

  // ==================== 通用操作/按钮 ====================
  common: {
    action: {
      save: '保存',
      cancel: '取消',
      confirm: '确定',
      clear: '清除',
      close: '关闭',
      refresh: '刷新',
      edit: '编辑',
      delete: '删除',
      add: '添加',
      create: '创建',
      filter: '筛选',
      enable: '启用',
      disable: '停用',
      select: '请选择',
      selectMonth: '选择月份',
      selectDate: '选择日期',
    },
    state: {
      loading: '加载中...',
      saving: '保存中...',
      underConstruction: '功能开发中...',
    },
    label: {
      required: '*',
    },
    unit: {
      minutes: '分钟',
    },
  },

  // ==================== 确认对话框 ====================
  confirm: {
    restoreTask: '确定要恢复任务"{title}"吗？',
    permanentDeleteTask: '确定要彻底删除任务"{title}"吗？此操作不可恢复！',
    emptyTrash: '确定要清空回收站吗？这将彻底删除所有任务，此操作不可恢复！',
    deleteArea: '确定要删除这个区域吗？这将影响所有关联的任务。',
    deleteAllRecurrenceInstances:
      '确定删除所有未完成的循环任务实例并停止重复吗？\n这将删除所有未来的"{title}"任务。\n此操作不可撤销。',
    pauseRecurrence:
      '确定要暂停此循环任务吗？\n\n将会删除今天之后的所有未完成任务实例。\n今天及之前的任务不受影响。',
    resumeRecurrence: '确定继续此循环吗？将清除结束日期，继续生成新任务。',
    deleteRecurrence: '确定要删除这个循环规则吗？\n\n{rule}\n\n将会删除所有未完成的任务实例。',
    resetSettings: '确定要将所有设置重置为默认值吗？',
  },

  // ==================== 错误与提示 ====================
  message: {
    success: {
      trashEmptied: '已清空回收站，删除了 {count} 个任务',
      taskRestored: '任务已恢复',
      taskDeleted: '任务已删除',
      updateRecurrence: '循环规则已更新',
    },
    error: {
      restoreFailed: '恢复失败，请重试',
      deleteFailed: '删除失败，请重试',
      emptyTrashFailed: '清空失败，请重试',
      createAreaFailed: '创建区域失败',
      updateAreaFailed: '更新区域失败',
      deleteAreaFailed: '删除区域失败',
      aiColorFailed: 'AI 染色失败',
      operationFailed: '操作失败，请重试',
      createRecurrenceFailed: '创建循环规则失败，请检查配置',
      updateRecurrenceFailed: '更新循环规则失败，请重试',
      loadRecurrenceFailed: '加载循环规则失败，请重试',
      createProjectFailed: '创建项目失败，请重试',
      updateProjectFailed: '更新项目失败，请重试',
      createSectionFailed: '创建章节失败，请重试',
      updateSectionFailed: '更新章节失败，请重试',
      deleteSectionFailed: '删除章节失败，请重试',
    },
  },

  // ==================== AI 助手 ====================
  ai: {
    floatingButton: 'AI 助手',
    title: 'AI 助手',
    placeholder: '输入消息... (Enter 发送, Shift+Enter 换行)',
    role: {
      you: '你',
      ai: 'AI',
    },
    action: {
      clearHistory: '清空记录',
      close: '关闭',
      upload: '上传图片',
      send: '发送',
    },
    empty: {
      title: '开始与 AI 对话吧！',
    },
    confirm: {
      clearHistory: '确定要清空聊天记录吗？',
    },
    message: {
      selectImageFile: '请选择图片文件',
      imageMaxSize: '图片文件不能超过 5MB',
      sendFailed: '发送消息失败',
    },
    image: {
      alt: '上传的图片',
      altPending: '待发送图片',
    },
    meta: {
      tokens: '{n} tokens',
    },
  },

  // ==================== 项目管理 ====================
  project: {
    title: {
      list: '项目列表',
      create: '新建项目',
      edit: '编辑项目',
      detail: '项目详情',
      noProject: '无项目',
    },
    action: {
      create: '新建项目',
      edit: '编辑项目',
      delete: '删除项目',
      addSection: '新增节',
      editSection: '编辑章节',
      createSection: '添加章节',
    },
    field: {
      name: '项目名称',
      description: '项目描述',
      area: '所属区域',
      dueDate: '截止日期',
      status: '项目状态',
      sectionTitle: '章节标题',
      sectionDescription: '章节描述',
    },
    placeholder: {
      name: '输入项目名称...',
      description: '输入项目描述...',
      sectionTitle: '输入章节标题...',
      sectionDescription: '输入章节描述...',
    },
    status: {
      active: '进行中',
      completed: '已完成',
    },
    label: {
      noArea: '无区域',
      noProjectTasks: '未分配到项目的任务',
      uncategorized: '未分类任务',
      unassignedToProject: '显示所有未分配到任何项目的任务',
    },
    empty: {
      noProjects: '暂无项目',
      noProjectsHint: '点击"新建项目"开始创建',
      selectProject: '请选择一个项目',
      noTasks: '暂无任务',
      noTasksHint: '从其他视图拖动任务到此项目，或点击"添加章节"组织任务',
      notExist: '项目不存在',
    },
    button: {
      creating: '创建中...',
      create: '创建项目',
      saving: '保存中...',
      save: '保存',
      createSection: '创建章节',
    },
    confirm: {
      delete: '确定要删除这个项目吗？',
      deleteSection: '确定要删除这个章节吗？章节内的任务将移至"未分类"。',
    },
  },

  // ==================== 模板管理 ====================
  template: {
    title: {
      templates: '模板',
      subtitle: '模板库',
      create: '新建模板',
      edit: '编辑模板',
    },
    placeholder: {
      name: '输入模板名称，按回车创建...',
      title: '模板标题',
    },
    label: {
      noTemplates: '暂无模板',
    },
    button: {
      create: '创建',
    },
    message: {
      enterTitle: '请输入模板标题',
      createFailed: '创建模板失败',
    },
  },

  // ==================== 时间块管理 ====================
  timeBlock: {
    title: {
      detail: '时间块详情',
      create: '创建时间块',
    },
    type: {
      task: '任务',
      event: '事件',
    },
    label: {
      allDay: '全天',
      timeType: {
        floating: '浮动时间',
        fixed: '固定时间',
      },
      startTime: '开始时间',
      duration: '时长',
      untitled: '无标题',
    },
    placeholder: {
      title: '输入标题...',
      taskTitle: '输入任务标题...',
      eventTitle: '输入事件标题...',
      description: '添加备注...',
    },
    button: {
      confirm: '确认',
    },
    action: {
      delete: '删除事件',
    },
  },

  // ==================== Upcoming 视图 ====================
  upcoming: {
    title: {
      main: '即将到来',
      horizontal: '即将到来（横向）',
    },
    timeRange: {
      overdue: '逾期',
      today: '今天',
      thisWeek: '本周',
      nextWeek: '下周',
      thisMonth: '本月',
      farFuture: '更远',
    },
    taskType: {
      dueDate: '截止日期',
      recurrence: '循环任务',
      scheduled: '排期任务',
    },
    empty: {
      message: '暂无即将到来的任务',
    },
  },

  // ==================== 项目相关（占位） ====================
  projects: {
    alpha: '项目 Alpha',
    beta: '项目 Beta',
  },

  // ==================== 经历相关（占位） ====================
  experience: {
    acme: '在 Acme 实习',
    opensource: '开源志愿者',
  },
}
