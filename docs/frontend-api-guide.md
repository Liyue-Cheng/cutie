# Cutie 前端API使用指南

## 概述

本文档为前端开发者提供Cutie后端API的快速使用指南，包含最常用的API端点和使用示例。

## 基础配置

### API基础URL
```typescript
const API_BASE_URL = 'http://localhost:3030/api'
```

### 通用请求头
```typescript
const headers = {
  'Content-Type': 'application/json',
  'X-Request-ID': generateRequestId(), // 可选，用于请求追踪
}
```

## 核心API使用示例

### 1. 任务管理

#### 1.1 获取未安排任务（Staging区）
```typescript
// GET /api/tasks/unscheduled
async function fetchUnscheduledTasks(): Promise<Task[]> {
  const response = await fetch(`${API_BASE_URL}/tasks/unscheduled`)
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 1.2 创建任务
```typescript
// POST /api/tasks
async function createTask(title: string): Promise<Task> {
  const payload = {
    title,
    context: {
      context_type: 'MISC',
      context_id: 'floating', // 创建到浮动区域
    },
  }
  
  const response = await fetch(`${API_BASE_URL}/tasks`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  
  if (!response.ok) {
    const errorData = await response.json()
    throw new Error(errorData.message)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 1.3 完成任务
```typescript
// POST /api/tasks/{id}/completion
async function completeTask(taskId: string): Promise<Task> {
  const response = await fetch(`${API_BASE_URL}/tasks/${taskId}/completion`, {
    method: 'POST',
  })
  
  if (!response.ok) {
    const errorData = await response.json()
    throw new Error(errorData.message)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 1.4 重新打开任务
```typescript
// POST /api/tasks/{id}/reopen
async function reopenTask(taskId: string): Promise<Task> {
  const response = await fetch(`${API_BASE_URL}/tasks/${taskId}/reopen`, {
    method: 'POST',
  })
  
  if (!response.ok) {
    const errorData = await response.json()
    throw new Error(errorData.message)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 1.5 更新任务
```typescript
// PUT /api/tasks/{id}
async function updateTask(taskId: string, updates: Partial<Task>): Promise<Task> {
  const payload = {
    title: updates.title,
    glance_note: updates.glance_note,
    detail_note: updates.detail_note,
    estimated_duration: updates.estimated_duration,
    area_id: updates.area_id,
    due_date: updates.due_date,
    due_date_type: updates.due_date_type,
  }
  
  const response = await fetch(`${API_BASE_URL}/tasks/${taskId}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  
  if (!response.ok) {
    const errorData = await response.json()
    throw new Error(errorData.message)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 1.6 删除任务
```typescript
// DELETE /api/tasks/{id}
async function deleteTask(taskId: string): Promise<void> {
  const response = await fetch(`${API_BASE_URL}/tasks/${taskId}`, {
    method: 'DELETE',
  })
  
  if (!response.ok) {
    const errorData = await response.json()
    throw new Error(errorData.message)
  }
}
```

### 2. 日程管理

#### 2.1 安排任务到某一天
```typescript
// POST /api/schedules
async function scheduleTask(taskId: string, targetDay: string): Promise<TaskSchedule> {
  const payload = {
    task_id: taskId,
    target_day: targetDay, // "2024-09-29T00:00:00Z"
    mode: 'link', // 创建额外日程
  }
  
  const response = await fetch(`${API_BASE_URL}/schedules`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  
  if (!response.ok) {
    const errorData = await response.json()
    throw new Error(errorData.message)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 2.2 获取某一天的日程
```typescript
// GET /api/schedules?date=2024-09-29T00:00:00Z
async function getSchedulesForDay(date: string): Promise<TaskSchedule[]> {
  const response = await fetch(`${API_BASE_URL}/schedules?date=${encodeURIComponent(date)}`)
  
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 2.3 记录努力
```typescript
// POST /api/schedules/{id}/presence
async function logPresence(scheduleId: string): Promise<TaskSchedule> {
  const response = await fetch(`${API_BASE_URL}/schedules/${scheduleId}/presence`, {
    method: 'POST',
  })
  
  if (!response.ok) {
    const errorData = await response.json()
    throw new Error(errorData.message)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

### 3. 时间块管理

#### 3.1 创建时间块
```typescript
// POST /api/time-blocks
async function createTimeBlock(
  startTime: string,
  endTime: string,
  taskIds: string[] = []
): Promise<TimeBlock> {
  const payload = {
    title: null,
    start_time: startTime,
    end_time: endTime,
    task_ids: taskIds,
  }
  
  const response = await fetch(`${API_BASE_URL}/time-blocks`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  
  if (!response.ok) {
    const errorData = await response.json()
    throw new Error(errorData.message)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 3.2 检查时间冲突
```typescript
// GET /api/time-blocks/conflicts
async function checkTimeConflict(
  startTime: string,
  endTime: string,
  excludeId?: string
): Promise<boolean> {
  const params = new URLSearchParams({
    start_time: startTime,
    end_time: endTime,
  })
  
  if (excludeId) {
    params.append('exclude_id', excludeId)
  }
  
  const response = await fetch(`${API_BASE_URL}/time-blocks/conflicts?${params}`)
  
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data.has_conflict
}
```

#### 3.3 查找空闲时间
```typescript
// GET /api/time-blocks/free-slots
async function findFreeTimeSlots(
  startTime: string,
  endTime: string,
  minDurationMinutes: number
): Promise<FreeTimeSlot[]> {
  const params = new URLSearchParams({
    start_time: startTime,
    end_time: endTime,
    min_duration_minutes: minDurationMinutes.toString(),
  })
  
  const response = await fetch(`${API_BASE_URL}/time-blocks/free-slots?${params}`)
  
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

### 4. 模板系统

#### 4.1 获取所有模板
```typescript
// GET /api/templates
async function getAllTemplates(): Promise<Template[]> {
  const response = await fetch(`${API_BASE_URL}/templates`)
  
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 4.2 基于模板创建任务
```typescript
// POST /api/templates/{id}/tasks
async function createTaskFromTemplate(
  templateId: string,
  context: CreationContext
): Promise<Task> {
  const payload = { context }
  
  const response = await fetch(`${API_BASE_URL}/templates/${templateId}/tasks`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  
  if (!response.ok) {
    const errorData = await response.json()
    throw new Error(errorData.message)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

### 5. 领域管理

#### 5.1 获取所有领域
```typescript
// GET /api/areas
async function getAllAreas(): Promise<Area[]> {
  const response = await fetch(`${API_BASE_URL}/areas`)
  
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

#### 5.2 获取根领域
```typescript
// GET /api/areas?roots_only=true
async function getRootAreas(): Promise<Area[]> {
  const response = await fetch(`${API_BASE_URL}/areas?roots_only=true`)
  
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}: ${response.statusText}`)
  }
  
  const apiResponse = await response.json()
  return apiResponse.data
}
```

## 类型定义

### TypeScript接口

```typescript
// 核心实体类型
interface Task {
  id: string
  title: string
  glance_note: string | null
  detail_note: string | null
  estimated_duration: number | null
  subtasks: Subtask[] | null
  project_id: string | null
  area_id: string | null
  due_date: string | null
  due_date_type: 'SOFT' | 'HARD' | null
  completed_at: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
  // ... 其他字段
}

interface Subtask {
  id: string
  title: string
  is_completed: boolean
  sort_order: string
}

interface TaskSchedule {
  id: string
  task_id: string
  scheduled_day: string
  outcome: 'PLANNED' | 'PRESENCE_LOGGED' | 'COMPLETED_ON_DAY' | 'CARRIED_OVER'
  created_at: string
  updated_at: string
}

interface TimeBlock {
  id: string
  title: string | null
  glance_note: string | null
  detail_note: string | null
  start_time: string
  end_time: string
  area_id: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
}

interface Template {
  id: string
  name: string
  title_template: string
  glance_note_template: string | null
  detail_note_template: string | null
  estimated_duration_template: number | null
  subtasks_template: Subtask[] | null
  area_id: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
}

interface Area {
  id: string
  name: string
  color: string
  parent_area_id: string | null
  created_at: string
  updated_at: string
  is_deleted: boolean
}

interface FreeTimeSlot {
  start_time: string
  end_time: string
  duration_minutes: number
}

// 请求载荷类型
interface CreateTaskPayload {
  title: string
  glance_note?: string | null
  detail_note?: string | null
  estimated_duration?: number | null
  subtasks?: Subtask[] | null
  area_id?: string | null
  due_date?: string | null
  due_date_type?: 'SOFT' | 'HARD' | null
  context: CreationContext
}

interface CreationContext {
  context_type: 'MISC' | 'DAILY_KANBAN' | 'PROJECT_LIST' | 'AREA_FILTER'
  context_id: string
}

// API响应类型
interface ApiResponse<T> {
  data: T
  timestamp: string
  request_id?: string
}

interface ErrorResponse {
  error_type: string
  message: string
  details?: any
  code?: string
  timestamp: string
  request_id?: string
}
```

## 错误处理

### 统一错误处理函数
```typescript
async function handleApiRequest<T>(request: Promise<Response>): Promise<T> {
  try {
    const response = await request
    
    if (!response.ok) {
      const errorData = await response.json() as ErrorResponse
      throw new ApiError(errorData.message, response.status, errorData)
    }
    
    const apiResponse = await response.json() as ApiResponse<T>
    return apiResponse.data
  } catch (error) {
    if (error instanceof ApiError) {
      throw error
    }
    
    // 网络错误或其他错误
    throw new ApiError(`Network error: ${error}`, 0)
  }
}

class ApiError extends Error {
  constructor(
    message: string,
    public statusCode: number,
    public errorResponse?: ErrorResponse
  ) {
    super(message)
    this.name = 'ApiError'
  }
}
```

### 错误处理示例
```typescript
// 在Pinia store中使用
async function createTask(payload: CreateTaskPayload) {
  isLoading.value = true
  error.value = null
  
  try {
    const newTask = await handleApiRequest<Task>(
      fetch(`${API_BASE_URL}/tasks`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      })
    )
    
    tasks.value.set(newTask.id, newTask)
    console.log('Task created successfully:', newTask)
  } catch (e) {
    if (e instanceof ApiError) {
      error.value = e.message
      
      // 处理特定错误类型
      if (e.statusCode === 422) {
        console.error('Validation error:', e.errorResponse?.details)
      }
    } else {
      error.value = `Unexpected error: ${e}`
    }
    
    console.error('Error creating task:', e)
  } finally {
    isLoading.value = false
  }
}
```

## 常用业务场景

### 场景1: 任务的完整生命周期
```typescript
// 1. 创建任务
const newTask = await createTask('完成项目文档')

// 2. 更新任务详情
const updatedTask = await updateTask(newTask.id, {
  glance_note: '需要包含API文档',
  estimated_duration: 120, // 2小时
})

// 3. 安排到今天
const today = new Date()
today.setHours(0, 0, 0, 0)
const schedule = await scheduleTask(newTask.id, today.toISOString())

// 4. 记录努力
await logPresence(schedule.id)

// 5. 完成任务
const completedTask = await completeTask(newTask.id)
```

### 场景2: 时间块管理
```typescript
// 1. 检查时间冲突
const hasConflict = await checkTimeConflict(
  '2024-09-29T09:00:00Z',
  '2024-09-29T11:00:00Z'
)

if (!hasConflict) {
  // 2. 创建时间块
  const timeBlock = await createTimeBlock(
    '2024-09-29T09:00:00Z',
    '2024-09-29T11:00:00Z',
    [taskId] // 关联任务
  )
}

// 3. 查找空闲时间
const freeSlots = await findFreeTimeSlots(
  '2024-09-29T08:00:00Z',
  '2024-09-29T18:00:00Z',
  60 // 至少1小时
)
```

### 场景3: 模板使用
```typescript
// 1. 获取所有模板
const templates = await getAllTemplates()

// 2. 基于模板创建任务
const taskFromTemplate = await createTaskFromTemplate(templateId, {
  context_type: 'DAILY_KANBAN',
  context_id: '1729555200', // Unix时间戳
})
```

## 响应数据结构

### 成功响应
```json
{
  "data": {
    // 实际数据内容
  },
  "timestamp": "2024-09-29T10:00:00Z",
  "request_id": "req-123e4567-e89b-12d3-a456-426614174000"
}
```

### 错误响应
```json
{
  "error_type": "ValidationError",
  "message": "输入数据验证失败",
  "details": {
    "validation_errors": [
      {
        "field": "title",
        "message": "Title cannot be empty",
        "code": "TITLE_EMPTY"
      }
    ]
  },
  "code": "VALIDATION_FAILED",
  "timestamp": "2024-09-29T10:00:00Z",
  "request_id": "req-123e4567-e89b-12d3-a456-426614174000"
}
```

## 最佳实践

### 1. 请求ID追踪
```typescript
function generateRequestId(): string {
  return `req-${crypto.randomUUID()}`
}

// 在请求头中包含请求ID
const headers = {
  'Content-Type': 'application/json',
  'X-Request-ID': generateRequestId(),
}
```

### 2. 乐观更新
```typescript
async function optimisticTaskUpdate(taskId: string, updates: Partial<Task>) {
  // 1. 立即更新UI
  const currentTask = tasks.value.get(taskId)
  if (currentTask) {
    const optimisticTask = { ...currentTask, ...updates, updated_at: new Date().toISOString() }
    tasks.value.set(taskId, optimisticTask)
  }
  
  try {
    // 2. 发送API请求
    const actualTask = await updateTask(taskId, updates)
    
    // 3. 用实际结果替换乐观更新
    tasks.value.set(taskId, actualTask)
  } catch (error) {
    // 4. 回滚乐观更新
    if (currentTask) {
      tasks.value.set(taskId, currentTask)
    }
    throw error
  }
}
```

### 3. 缓存策略
```typescript
// 简单的内存缓存
const cache = new Map<string, { data: any; timestamp: number }>()
const CACHE_TTL = 5 * 60 * 1000 // 5分钟

async function getCachedData<T>(key: string, fetcher: () => Promise<T>): Promise<T> {
  const cached = cache.get(key)
  const now = Date.now()
  
  if (cached && (now - cached.timestamp) < CACHE_TTL) {
    return cached.data
  }
  
  const data = await fetcher()
  cache.set(key, { data, timestamp: now })
  return data
}
```

### 4. 批量操作
```typescript
// 批量创建任务
async function createMultipleTasks(titles: string[]): Promise<Task[]> {
  const promises = titles.map(title => 
    createTask(title)
  )
  
  // 并发执行，但限制并发数
  const results = await Promise.allSettled(promises)
  
  const successTasks: Task[] = []
  const errors: string[] = []
  
  results.forEach((result, index) => {
    if (result.status === 'fulfilled') {
      successTasks.push(result.value)
    } else {
      errors.push(`Failed to create "${titles[index]}": ${result.reason}`)
    }
  })
  
  if (errors.length > 0) {
    console.warn('Some tasks failed to create:', errors)
  }
  
  return successTasks
}
```

## 开发调试

### 1. API调试工具
```typescript
// 开发环境下的API调试
if (import.meta.env.DEV) {
  // 拦截所有API请求进行日志记录
  const originalFetch = window.fetch
  window.fetch = async (input, init) => {
    console.log(`[API] ${init?.method || 'GET'} ${input}`, init?.body)
    
    const response = await originalFetch(input, init)
    
    if (!response.ok) {
      console.error(`[API] Error ${response.status}:`, await response.clone().text())
    }
    
    return response
  }
}
```

### 2. 健康检查
```typescript
// 定期检查后端健康状态
async function checkBackendHealth(): Promise<boolean> {
  try {
    const response = await fetch(`${API_BASE_URL.replace('/api', '')}/health`)
    return response.ok
  } catch {
    return false
  }
}

// 在应用启动时检查
onMounted(async () => {
  const isHealthy = await checkBackendHealth()
  if (!isHealthy) {
    console.warn('Backend server is not responding')
    // 显示离线提示或重试逻辑
  }
})
```

### 3. 开发环境配置
```typescript
// 环境变量配置
const API_CONFIG = {
  development: {
    baseUrl: 'http://localhost:3030/api',
    timeout: 10000,
    retries: 3,
  },
  production: {
    baseUrl: 'https://api.cutie.app/v1',
    timeout: 5000,
    retries: 1,
  },
}

const config = API_CONFIG[import.meta.env.MODE] || API_CONFIG.development
```

## 性能优化建议

### 1. 请求去重
```typescript
// 防止重复请求
const pendingRequests = new Map<string, Promise<any>>()

async function deduplicatedRequest<T>(key: string, request: () => Promise<T>): Promise<T> {
  if (pendingRequests.has(key)) {
    return pendingRequests.get(key)
  }
  
  const promise = request()
  pendingRequests.set(key, promise)
  
  try {
    const result = await promise
    return result
  } finally {
    pendingRequests.delete(key)
  }
}
```

### 2. 分页加载
```typescript
// 分页获取任务（如果后端支持）
async function getTasksPaginated(page: number = 1, pageSize: number = 20): Promise<Task[]> {
  const params = new URLSearchParams({
    page: page.toString(),
    page_size: pageSize.toString(),
  })
  
  const response = await fetch(`${API_BASE_URL}/tasks?${params}`)
  const apiResponse = await response.json()
  return apiResponse.data
}
```

### 3. 增量更新
```typescript
// 只获取更新的任务
async function getTasksUpdatedSince(lastUpdateTime: string): Promise<Task[]> {
  const params = new URLSearchParams({
    updated_since: lastUpdateTime,
  })
  
  // 注意：这个端点需要后端支持
  const response = await fetch(`${API_BASE_URL}/tasks/updated?${params}`)
  const apiResponse = await response.json()
  return apiResponse.data
}
```

## 总结

这个API设计遵循RESTful原则，提供了完整的任务管理功能。前端开发者可以：

1. **使用标准HTTP方法**进行CRUD操作
2. **遵循统一的数据格式**进行请求和响应处理
3. **利用类型安全**的TypeScript接口
4. **实现优雅的错误处理**
5. **支持高级功能**如时间块、模板、领域管理

如有任何问题或需要新功能，请参考完整的OpenAPI文档或联系后端开发团队。
