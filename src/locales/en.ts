export default {
  // ==================== Navigation & Layout ====================
  nav: {
    recent: 'Recent',
    staging: 'Staging',
    upcoming: 'Upcoming',
    calendar: 'Calendar',
    projects: 'Projects',
    templates: 'Templates',
    dailyOverview: 'Daily Overview',
    dailyShutdown: 'Daily Shutdown',
    timelineKanban: 'Timeline Kanban',
    calendarKanban: 'Calendar Kanban',
    areas: 'Areas',
    recurrence: 'Recurring Tasks',
    settings: 'Settings',
    section: {
      dailyRoutines: 'DAILY ROUTINES',
      kanban: 'KANBAN',
    },
    quickAddTask: 'Quick Add Task',
  },

  // ==================== Sidebar (legacy compatibility) ====================
  sidebar: {
    header: 'Cutie .',
    timeline: 'Timeline',
    planning: 'Planning',
    upcoming: 'Upcoming',
    allTasks: 'All Tasks',
    projects: 'Projects',
    experience: 'Experience',
    settings: 'Settings',
  },

  // ==================== Toolbar Views ====================
  toolbar: {
    calendar: 'Calendar',
    timeline: 'Timeline',
    staging: 'Staging',
    upcoming: 'Upcoming',
    dailyTasks: 'Daily Tasks',
    templates: 'Templates',
    projects: 'Projects',
    polling: 'Polling',
    completed: 'Completed',
    archive: 'Archived',
    deleted: 'Recently Deleted',
    deadline: 'Deadline',
  },

  // ==================== Task Related ====================
  task: {
    action: {
      edit: 'Edit Task',
      delete: 'Delete Task',
      archive: 'Archive Task',
      unarchive: 'Unarchive',
      returnToStaging: 'Return to Staging',
      cancelTodaySchedule: 'Cancel Today Schedule',
      complete: 'Complete',
      reopen: 'Reopen',
      addTask: '+ Add Task',
      addNewTask: 'Add new task...',
      sort: 'Sort',
    },
    placeholder: {
      description: 'Task description...',
      addSubtask: 'Add subtask...',
      addSubtaskTemplate: 'Add subtask template...',
      detailNote: 'Detailed notes...',
      detailNoteTemplate: 'Detailed notes template...',
      glanceNoteTemplate: 'Quick overview notes template...',
      title: 'Enter task title...',
    },
    label: {
      subtasks: 'Subtasks',
      subtaskTemplate: 'Subtask Template',
      noArea: 'No Area',
      unscheduled: 'Unscheduled tasks',
      archived: 'Archived tasks',
      scheduled: 'Scheduled',
      noTasks: 'No tasks',
      noDeadlineTasks: 'No tasks with due dates',
      noLinkedTasks: 'No linked tasks',
      linkedTasks: 'Linked Tasks',
      recentCarryover: 'Recent Carryover',
    },
    button: {
      done: 'Done',
    },
    duration: {
      tiny: 'tiny',
    },
    status: {
      completed: 'Completed',
      incomplete: 'Incomplete',
      inProgress: 'In Progress',
    },
    count: {
      overdue: '{n} overdue',
      tasks: 'tasks',
    },
  },

  // ==================== Due Date ====================
  dueDate: {
    setDueDate: 'Set Due Date',
    label: {
      date: 'Date',
      type: 'Type',
    },
    type: {
      soft: 'Soft Deadline',
      hard: 'Hard Deadline',
    },
  },

  // ==================== Recurrence ====================
  recurrence: {
    title: {
      manager: 'Recurring Task Manager',
      config: 'Configure Recurrence Rule',
      taskConfig: 'Set recurrence for task "{title}"',
      edit: 'Edit Recurrence Rule',
      timeBlockConfig: 'Configure Time Block Recurrence',
      timeBlockConfigDesc: 'Set recurrence rule for time block "{title}"',
      timeBlockEdit: 'Edit Time Block Recurrence',
      timeBlockEditDesc: 'Adjust the rule and handle future instances',
    },
    label: {
      frequency: 'Repeat Frequency',
      selectWeekday: 'Select Days',
      expiryBehavior: 'Expiry Behavior',
      endDate: 'End Date (Optional)',
      interval: 'Every',
      intervalSuffix: 'week(s)',
      monthDay: 'Day of Month',
      monthDaySuffix: '',
      month: 'month',
      start: 'Start',
      end: 'End',
      conflictBehavior: 'When time conflict occurs',
      isRecurring: 'Recurring Time Block',
      templateTitle: 'Template Title',
      shortNote: 'Quick note template',
      detailNote: 'Detailed note template',
      futureInstances: 'Future instances',
      deleteFutureInstancesDesc: 'Remove all instances that have not started yet',
    },
    freq: {
      daily: 'Daily',
      weekly: 'Weekly',
      monthly: 'Monthly on specific date',
      yearly: 'Yearly',
      weekdays: 'Weekdays (Mon-Fri)',
    },
    weekday: {
      mon: 'Mon',
      tue: 'Tue',
      wed: 'Wed',
      thu: 'Thu',
      fri: 'Fri',
      sat: 'Sat',
      sun: 'Sun',
    },
    description: {
      daily: 'Daily',
      weekly: 'Weekly',
      monthly: 'Monthly',
      yearly: 'Yearly',
    },
    timeType: {
      floating: 'Floating Time',
      fixed: 'Fixed Time',
    },
    expiry: {
      carryover: 'Carryover',
      carryoverFull: 'Carryover to Staging',
      carryoverDesc:
        'If you forget to complete it today, the task will be moved to staging for later (e.g., pay utility bills)',
      expire: 'Auto Expire',
      expireDesc:
        'If not completed today, the task automatically expires (e.g., daily check-in, game dailies)',
    },
    conflict: {
      skip: 'Skip Conflicting Time Slots',
      skipDesc: 'If the time slot already has other time blocks, skip and do not create',
      error: 'Block Creation on Conflict',
      errorDesc: 'If there is a time conflict, stop creation and show error',
    },
    status: {
      active: 'Active',
      expired: 'Expired',
      inactive: 'Inactive',
    },
    action: {
      stopRepeating: 'Stop Repeating',
      stop: 'Stop repeating',
      continue: 'Continue Recurrence',
      deleteRule: 'Delete Rule',
      changeFrequency: 'Change repeat frequency',
      updateAll: 'Update all incomplete instances to match this task',
      deleteAll: 'Delete all incomplete instances and stop repeating',
      setRecurrence: 'Set Recurrence',
      deleteFutureInstances: 'Delete future instances',
    },
    menuSection: 'Task recurrence:',
    empty: {
      title: 'No recurring task rules',
      hint: 'You can set up recurring rules in the task editor',
    },
    unknownTask: 'Unknown Task',
  },

  // ==================== Trash ====================
  trash: {
    title: 'Trash',
    action: {
      empty: 'Empty Trash',
      restore: 'Restore',
      permanentDelete: 'Delete Permanently',
    },
    empty: {
      message: 'Trash is empty',
    },
    label: {
      deletedAt: 'Deleted {time}',
    },
  },

  // ==================== Time Formatting ====================
  time: {
    unknown: 'Unknown time',
    justNow: 'Just now',
    minutesAgo: '{n} minute(s) ago',
    hoursAgo: '{n} hour(s) ago',
    daysAgo: '{n} day(s) ago',
    today: 'Today',
    tomorrow: 'Tomorrow',
    thisWeek: 'This Week',
    nextWeek: 'Next Week',
    thisMonth: 'This Month',
    farFuture: 'Later',
    overdue: 'Overdue',
  },

  // ==================== Calendar ====================
  calendar: {
    view: {
      week: 'Week View',
      month: 'Month View',
      day: 'Day View',
    },
    action: {
      previous: 'Previous',
      next: 'Next',
      switchTo3Days: 'Switch to 3-day view',
      switchTo1Day: 'Switch to 1-day view',
    },
  },

  // ==================== Settings ====================
  settings: {
    title: 'Settings',
    category: {
      ai: 'AI',
      appearance: 'Appearance',
      behavior: 'Behavior',
      data: 'Data',
      account: 'Account',
      debug: 'Debug',
      system: 'System',
    },
    ai: {
      title: 'AI Settings',
      description: 'Configure conversation and quick model access',
      conversation: {
        title: 'Conversation Model (for AI chat)',
        description:
          'Model configuration for AI assistant chat. Fill in the available request URL, key, and model name.',
      },
      quick: {
        title: 'Quick Model (for task classification, etc.)',
        description:
          'Lightweight model for auto-classification and quick tasks. Recommend high-performance, low-latency models.',
      },
      field: {
        apiBaseUrl: 'API Base URL',
        apiBaseUrlDesc: 'e.g., https://api.openai.com/v1',
        apiKey: 'API Key',
        apiKeyDesc: 'Key for accessing the model',
        model: 'Model',
        modelDesc: 'Model name, e.g., gpt-4o, glm-4.5, etc.',
      },
    },
    appearance: {
      title: 'Appearance Settings',
      description: 'Appearance and theme settings',
      theme: 'Theme',
      themeDesc: 'Select application theme',
    },
    behavior: {
      title: 'Behavior Settings',
      description: 'Task and application behavior settings',
      taskCompletion: {
        title: 'Task Completion',
        description: 'Control behavior when completing tasks',
        createSchedule: 'Create schedule on completion',
        createScheduleDesc:
          'Automatically create a completed schedule record for today when completing tasks from non-today views',
      },
    },
    debug: {
      title: 'Debug Settings',
      description: 'Development and debugging settings',
    },
    theme: {
      rosePine: 'Rose Pine',
      rosePineDawn: 'Rose Pine Dawn',
      rosePineMoon: 'Rose Pine Moon',
      cutie: 'Cutie',
      business: 'Business',
    },
    action: {
      resetAll: 'Reset All',
      close: 'Close',
    },
    empty: {
      title: 'No settings available',
      comingSoon: 'Settings for this category coming soon',
    },
  },

  // ==================== Area Management ====================
  area: {
    title: {
      manager: 'Area Manager',
      create: 'Create New Area',
      all: 'All Areas',
    },
    placeholder: {
      name: 'Enter area name...',
    },
    action: {
      selectColor: 'Select Color',
      aiColor: 'AI Color',
      add: 'Add',
      clearArea: 'Clear Area',
    },
    count: '{n} area(s)',
    empty: {
      title: 'No areas created yet',
      hint: 'Areas help you organize and categorize tasks',
    },
    message: {
      enterName: 'Please enter an area name first',
    },
  },

  // ==================== Quick Add Task ====================
  quickAdd: {
    title: 'Quick Add Task',
    placeholder: 'Enter task title...',
    button: {
      addTo: 'Add to {date}',
      add: 'Add Task',
    },
  },

  // ==================== View Titles ====================
  view: {
    staging: {
      title: 'Unscheduled Tasks',
      areaCount: '{n} area(s)',
    },
    upcoming: {
      title: 'Upcoming',
    },
    dailyOverview: {
      greeting: 'What fun things do you want to do today?',
      weather: 'Weather',
      weatherHint: 'Weather data is for demo purposes. Real weather API integration coming soon.',
      dailyRecurring: 'Daily recurring',
      noDailyRecurring: 'No daily recurring tasks today',
      todaysTasks: "Today's tasks",
    },
    dailyShutdown: {
      title: 'Daily Shutdown',
      todayIncomplete: 'Today · Incomplete',
      todayCompleted: 'Today · Completed',
      tomorrow: 'Tomorrow',
      ritual: 'Daily Ritual',
      ritualDesc:
        'The shutdown ritual is being designed. It will help you gently end your day here.',
    },
    projects: {
      title: 'Projects',
    },
  },

  // ==================== Common Actions/Buttons ====================
  common: {
    action: {
      save: 'Save',
      cancel: 'Cancel',
      confirm: 'Confirm',
      clear: 'Clear',
      close: 'Close',
      refresh: 'Refresh',
      edit: 'Edit',
      delete: 'Delete',
      add: 'Add',
      create: 'Create',
      filter: 'Filter',
      enable: 'Enable',
      disable: 'Disable',
      select: 'Please select',
      selectMonth: 'Select month',
      selectDate: 'Select date',
    },
    state: {
      loading: 'Loading...',
      saving: 'Saving...',
      underConstruction: 'Under construction...',
    },
    label: {
      required: '*',
    },
    unit: {
      minutes: 'minutes',
    },
  },

  // ==================== Confirmation Dialogs ====================
  confirm: {
    restoreTask: 'Are you sure you want to restore task "{title}"?',
    permanentDeleteTask:
      'Are you sure you want to permanently delete task "{title}"? This cannot be undone!',
    emptyTrash:
      'Are you sure you want to empty the trash? All tasks will be permanently deleted. This cannot be undone!',
    deleteArea: 'Are you sure you want to delete this Area? This will affect all associated tasks.',
    deleteAllRecurrenceInstances:
      'Are you sure you want to delete all incomplete recurring task instances and stop repeating?\nThis will delete all future "{title}" tasks.\nThis action cannot be undone.',
    pauseRecurrence:
      'Are you sure you want to pause this recurring task?\n\nAll incomplete task instances after today will be deleted.\nTasks on or before today are not affected.',
    resumeRecurrence:
      'Are you sure you want to continue this recurrence? The end date will be cleared and new tasks will be generated.',
    deleteRecurrence:
      'Are you sure you want to delete this recurrence rule?\n\n{rule}\n\nAll incomplete task instances will be deleted.',
    resetSettings: 'Reset all settings to defaults?',
  },

  // ==================== Messages ====================
  message: {
    success: {
      trashEmptied: 'Trash emptied, {count} task(s) deleted',
      taskRestored: 'Task restored',
      taskDeleted: 'Task deleted',
      updateRecurrence: 'Recurrence updated successfully',
    },
    error: {
      restoreFailed: 'Restore failed, please try again',
      deleteFailed: 'Delete failed, please try again',
      emptyTrashFailed: 'Empty trash failed, please try again',
      createAreaFailed: 'Failed to create Area',
      updateAreaFailed: 'Failed to update Area',
      deleteAreaFailed: 'Failed to delete Area',
      aiColorFailed: 'AI color suggestion failed',
      operationFailed: 'Operation failed, please try again',
      createRecurrenceFailed: 'Failed to create recurrence rule, please check configuration',
      updateRecurrenceFailed: 'Failed to update recurrence, please try again',
      loadRecurrenceFailed: 'Failed to load recurrence, please try again',
      createProjectFailed: 'Failed to create project, please try again',
      updateProjectFailed: 'Failed to update project, please try again',
      createSectionFailed: 'Failed to create section, please try again',
      updateSectionFailed: 'Failed to update section, please try again',
      deleteSectionFailed: 'Failed to delete section, please try again',
    },
  },

  // ==================== AI Assistant ====================
  ai: {
    floatingButton: 'AI Assistant',
    title: 'AI Assistant',
    placeholder: 'Type a message... (Enter to send, Shift+Enter for new line)',
    role: {
      you: 'You',
      ai: 'AI',
    },
    action: {
      clearHistory: 'Clear History',
      close: 'Close',
      upload: 'Upload Image',
      send: 'Send',
    },
    empty: {
      title: 'Start chatting with AI!',
    },
    confirm: {
      clearHistory: 'Are you sure you want to clear the chat history?',
    },
    message: {
      selectImageFile: 'Please select an image file',
      imageMaxSize: 'Image file cannot exceed 5MB',
      sendFailed: 'Failed to send message',
    },
    image: {
      alt: 'Uploaded image',
      altPending: 'Image to send',
    },
    meta: {
      tokens: '{n} tokens',
    },
  },

  // ==================== Project Management ====================
  project: {
    title: {
      list: 'Project List',
      create: 'Create Project',
      edit: 'Edit Project',
      detail: 'Project Detail',
      noProject: 'No Project',
    },
    action: {
      create: 'Create Project',
      edit: 'Edit Project',
      delete: 'Delete Project',
      addSection: 'Add Section',
      editSection: 'Edit Section',
      createSection: 'Create Section',
    },
    field: {
      name: 'Project Name',
      description: 'Project Description',
      area: 'Area',
      dueDate: 'Due Date',
      status: 'Status',
      sectionTitle: 'Section Title',
      sectionDescription: 'Section Description',
    },
    placeholder: {
      name: 'Enter project name...',
      description: 'Enter project description...',
      sectionTitle: 'Enter section title...',
      sectionDescription: 'Enter section description...',
    },
    status: {
      active: 'Active',
      completed: 'Completed',
    },
    label: {
      noArea: 'No Area',
      noProjectTasks: 'Tasks not assigned to any project',
      uncategorized: 'Uncategorized Tasks',
    },
    empty: {
      noProjects: 'No projects',
      noProjectsHint: 'Click "Create Project" to get started',
      selectProject: 'Please select a project',
      noTasks: 'No tasks',
      noTasksHint:
        'Drag tasks from other views to this project, or click "Add Section" to organize tasks',
      notExist: 'Project does not exist',
    },
    button: {
      creating: 'Creating...',
      create: 'Create Project',
      saving: 'Saving...',
      save: 'Save',
      createSection: 'Create Section',
    },
    confirm: {
      delete: 'Are you sure you want to delete this project?',
      deleteSection:
        'Are you sure you want to delete this section? Tasks in this section will be moved to "Uncategorized".',
    },
  },

  // ==================== Template Management ====================
  template: {
    title: {
      templates: '模板',
      subtitle: 'Templates',
      create: 'Create Template',
      edit: 'Edit Template',
    },
    placeholder: {
      name: 'Enter template name and press Enter to create...',
      title: 'Template Title',
    },
    label: {
      noTemplates: 'No templates',
    },
    button: {
      create: 'Create',
    },
    message: {
      enterTitle: 'Please enter template title',
      createFailed: 'Failed to create template',
    },
  },

  // ==================== Time Block Management ====================
  timeBlock: {
    title: {
      detail: 'Time Block Detail',
      create: 'Create Time Block',
    },
    type: {
      task: 'Task',
      event: 'Event',
    },
    label: {
      allDay: 'All Day',
      timeType: {
        floating: 'Floating Time',
        fixed: 'Fixed Time',
      },
      startTime: 'Start Time',
      duration: 'Duration',
      untitled: 'Untitled',
    },
    placeholder: {
      title: 'Enter title...',
      taskTitle: 'Enter task title...',
      eventTitle: 'Enter event title...',
      description: 'Add notes...',
    },
    button: {
      confirm: 'Confirm',
    },
    action: {
      delete: 'Delete Event',
    },
  },

  // ==================== Upcoming View ====================
  upcoming: {
    title: {
      main: 'Upcoming',
      horizontal: 'Upcoming (Horizontal)',
    },
    timeRange: {
      overdue: 'Overdue',
      today: 'Today',
      thisWeek: 'This Week',
      nextWeek: 'Next Week',
      thisMonth: 'This Month',
      farFuture: 'Later',
    },
    taskType: {
      dueDate: 'Due Date',
      recurrence: 'Recurrence',
      scheduled: 'Scheduled',
    },
    empty: {
      message: 'No upcoming tasks',
    },
  },

  // ==================== Projects (placeholder) ====================
  projects: {
    alpha: 'Project Alpha',
    beta: 'Project Beta',
  },

  // ==================== Experience (placeholder) ====================
  experience: {
    acme: 'Internship at Acme',
    opensource: 'Volunteer at OpenSource',
  },
}
