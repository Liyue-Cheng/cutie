# Cutie API 端点总览

## 服务器信息

- **基础URL**: `http://localhost:3030/api`
- **API版本**: v1.0.0
- **总端点数**: 52个
- **支持的HTTP方法**: GET, POST, PUT, DELETE

## API端点分类

### 🎯 任务管理 (Tasks) - 11个端点

| 方法   | 端点                     | 描述           | 主要用途                   |
| ------ | ------------------------ | -------------- | -------------------------- |
| POST   | `/tasks`                 | 创建任务       | 在指定上下文中创建新任务   |
| GET    | `/tasks/search`          | 搜索任务       | 根据关键词搜索任务         |
| GET    | `/tasks/unscheduled`     | 获取未安排任务 | 获取Staging区的任务        |
| GET    | `/tasks/stats`           | 获取任务统计   | 任务数量、状态分布等统计   |
| GET    | `/tasks/{id}`            | 获取任务详情   | 根据ID获取单个任务         |
| PUT    | `/tasks/{id}`            | 更新任务       | 修改任务的属性             |
| DELETE | `/tasks/{id}`            | 删除任务       | 软删除任务                 |
| POST   | `/tasks/{id}/completion` | 完成任务       | 标记任务为已完成           |
| POST   | `/tasks/{id}/reopen`     | 重新打开任务   | 重新激活已完成的任务       |
| GET    | `/tasks/{id}/schedules`  | 获取任务日程   | 查看任务的所有日程安排     |
| GET    | `/tasks/{id}/ordering`   | 获取任务排序   | 查看任务在各上下文中的排序 |

### 📅 日程管理 (Schedules) - 6个端点

| 方法   | 端点                        | 描述             | 主要用途                 |
| ------ | --------------------------- | ---------------- | ------------------------ |
| POST   | `/schedules`                | 安排任务         | 将任务安排到特定日期     |
| GET    | `/schedules`                | 获取日程列表     | 查看特定日期或范围的日程 |
| GET    | `/schedules/stats`          | 获取日程统计     | 日程完成率、分布等统计   |
| DELETE | `/schedules/{id}`           | 删除日程         | 删除单个日程安排         |
| POST   | `/schedules/{id}/presence`  | 记录努力         | 标记已付出努力           |
| DELETE | `/schedules/tasks/{taskId}` | 取消任务所有日程 | 将任务移回Staging区      |

### 🔄 排序管理 (Ordering) - 5个端点

| 方法   | 端点                  | 描述           | 主要用途                     |
| ------ | --------------------- | -------------- | ---------------------------- |
| PUT    | `/ordering`           | 更新排序       | 修改任务在上下文中的排序位置 |
| GET    | `/ordering`           | 获取上下文排序 | 查看特定上下文的排序记录     |
| DELETE | `/ordering`           | 清理上下文排序 | 清除上下文中的所有排序       |
| PUT    | `/ordering/batch`     | 批量更新排序   | 一次性更新多个排序记录       |
| GET    | `/ordering/calculate` | 计算排序位置   | 计算两个位置之间的排序值     |

### ⏰ 时间块管理 (Time Blocks) - 12个端点

| 方法   | 端点                               | 描述             | 主要用途                       |
| ------ | ---------------------------------- | ---------------- | ------------------------------ |
| POST   | `/time-blocks`                     | 创建时间块       | 创建新的时间块                 |
| GET    | `/time-blocks`                     | 获取时间块列表   | 查询时间块（按日期/任务/领域） |
| GET    | `/time-blocks/{id}`                | 获取时间块详情   | 查看单个时间块                 |
| PUT    | `/time-blocks/{id}`                | 更新时间块       | 修改时间块属性                 |
| DELETE | `/time-blocks/{id}`                | 删除时间块       | 删除时间块及其关联             |
| POST   | `/time-blocks/{id}/tasks`          | 链接任务到时间块 | 建立任务-时间块关联            |
| DELETE | `/time-blocks/{id}/tasks/{taskId}` | 取消任务关联     | 解除任务-时间块关联            |
| POST   | `/time-blocks/{id}/truncate`       | 截断时间块       | 缩短正在进行的时间块           |
| POST   | `/time-blocks/{id}/extend`         | 扩展时间块       | 延长时间块的结束时间           |
| POST   | `/time-blocks/{id}/split`          | 分割时间块       | 将时间块分割为两个             |
| GET    | `/time-blocks/conflicts`           | 检查时间冲突     | 验证时间范围是否冲突           |
| GET    | `/time-blocks/free-slots`          | 查找空闲时间段   | 找到可用的时间段               |

### 📝 模板管理 (Templates) - 8个端点

| 方法   | 端点                    | 描述             | 主要用途                  |
| ------ | ----------------------- | ---------------- | ------------------------- |
| POST   | `/templates`            | 创建模板         | 创建新的任务模板          |
| GET    | `/templates`            | 获取模板列表     | 查询模板（支持搜索/筛选） |
| GET    | `/templates/stats`      | 获取模板统计     | 模板使用情况统计          |
| GET    | `/templates/{id}`       | 获取模板详情     | 查看单个模板              |
| PUT    | `/templates/{id}`       | 更新模板         | 修改模板内容              |
| DELETE | `/templates/{id}`       | 删除模板         | 删除模板                  |
| POST   | `/templates/{id}/clone` | 克隆模板         | 复制现有模板              |
| POST   | `/templates/{id}/tasks` | 基于模板创建任务 | 使用模板生成任务          |

### 🏷️ 领域管理 (Areas) - 10个端点

| 方法   | 端点                     | 描述           | 主要用途                 |
| ------ | ------------------------ | -------------- | ------------------------ |
| POST   | `/areas`                 | 创建领域       | 创建新的领域分类         |
| GET    | `/areas`                 | 获取领域列表   | 查询领域（支持层级查询） |
| GET    | `/areas/stats`           | 获取领域统计   | 领域使用情况统计         |
| GET    | `/areas/{id}`            | 获取领域详情   | 查看单个领域             |
| PUT    | `/areas/{id}`            | 更新领域       | 修改领域属性             |
| DELETE | `/areas/{id}`            | 删除领域       | 删除领域（含使用检查）   |
| GET    | `/areas/{id}/path`       | 获取领域路径   | 获取从根到当前领域的路径 |
| POST   | `/areas/{id}/move`       | 移动领域       | 改变领域的层级位置       |
| POST   | `/areas/{id}/restore`    | 恢复领域       | 恢复已删除的领域         |
| GET    | `/areas/{id}/can-delete` | 检查是否可删除 | 验证删除安全性           |

## 上下文类型说明

### Context Types

- **MISC**: 杂项上下文
  - `floating`: 浮动任务
  - `staging_all`: 所有暂存任务
- **DAILY_KANBAN**: 日程看板
  - Context ID: Unix时间戳字符串（如 `1729555200`）
- **PROJECT_LIST**: 项目列表
  - Context ID: `project::{project_id}`
- **AREA_FILTER**: 领域筛选
  - Context ID: `area::{area_id}`

## 业务流程示例

### 1. 创建并安排任务的完整流程

```
1. POST /tasks (创建任务)
   ↓
2. POST /schedules (安排到某一天)
   ↓
3. POST /time-blocks (创建时间块)
   ↓
4. POST /time-blocks/{id}/tasks (关联任务到时间块)
   ↓
5. POST /schedules/{id}/presence (记录努力)
   ↓
6. POST /tasks/{id}/completion (完成任务)
```

### 2. 模板驱动的任务创建流程

```
1. GET /templates (获取可用模板)
   ↓
2. POST /templates/{id}/tasks (基于模板创建任务)
   ↓
3. PUT /tasks/{id} (根据需要调整任务)
   ↓
4. POST /schedules (安排任务)
```

### 3. 时间管理流程

```
1. GET /time-blocks/free-slots (查找空闲时间)
   ↓
2. GET /time-blocks/conflicts (检查冲突)
   ↓
3. POST /time-blocks (创建时间块)
   ↓
4. POST /time-blocks/{id}/tasks (关联任务)
```

## 错误码对照表

| HTTP状态码 | 错误类型              | 描述           | 常见原因                     |
| ---------- | --------------------- | -------------- | ---------------------------- |
| 400        | Bad Request           | 请求格式错误   | JSON格式错误、参数类型错误   |
| 404        | Not Found             | 资源未找到     | 任务/领域/模板不存在         |
| 409        | Conflict              | 资源冲突       | 时间冲突、状态冲突、依赖冲突 |
| 422        | Unprocessable Entity  | 验证失败       | 字段验证失败、业务规则违反   |
| 500        | Internal Server Error | 服务器内部错误 | 数据库错误、系统异常         |
| 503        | Service Unavailable   | 服务不可用     | 外部依赖失败、系统过载       |

## 数据一致性保证

### 1. 事务保证

- 所有写操作都在数据库事务中执行
- 复合操作（如完成任务）保证原子性
- 失败时自动回滚

### 2. 并发控制

- 乐观锁定（基于updated_at字段）
- 幂等操作设计
- 冲突检测和处理

### 3. 数据完整性

- 外键约束保证引用完整性
- 检查约束保证数据有效性
- 软删除保证历史数据完整性

## 性能特征

### 响应时间 (P95)

- **简单查询**: < 10ms
- **复杂查询**: < 50ms
- **写操作**: < 100ms
- **统计计算**: < 200ms

### 并发能力

- **最大并发连接**: 100
- **请求队列**: 1000
- **超时设置**: 30秒

### 缓存策略

- **内存缓存**: 热点数据10MB
- **查询缓存**: 复杂查询结果缓存5分钟
- **CDN缓存**: 静态资源永久缓存

## 开发工具

### 1. API文档浏览

```bash
# 使用Swagger UI查看API文档
# 访问 http://localhost:3030/docs (如果后端支持)
```

### 2. API测试

```bash
# 使用curl测试API
curl -X GET "http://localhost:3030/api/tasks/unscheduled" \
  -H "Content-Type: application/json"

# 创建任务
curl -X POST "http://localhost:3030/api/tasks" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "测试任务",
    "context": {
      "context_type": "MISC",
      "context_id": "floating"
    }
  }'
```

### 3. 健康检查

```bash
# 检查服务器状态
curl -X GET "http://localhost:3030/health"

# 获取服务器信息
curl -X GET "http://localhost:3030/info"

# 简单连通性测试
curl -X GET "http://localhost:3030/api/ping"
```

## 前端集成建议

### 1. Pinia Store结构

```typescript
// 推荐的store结构
export const useTaskStore = defineStore('task', () => {
  // State
  const tasks = ref(new Map<string, Task>())
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const unscheduledTasks = computed(() =>
    Array.from(tasks.value.values()).filter((t) => !t.is_deleted && !t.completed_at)
  )

  // Actions
  const fetchTasks = async () => {
    /* API调用 */
  }
  const createTask = async (payload: CreateTaskPayload) => {
    /* API调用 */
  }
  const completeTask = async (id: string) => {
    /* API调用 */
  }

  return { tasks, isLoading, error, unscheduledTasks, fetchTasks, createTask, completeTask }
})
```

### 2. 错误处理策略

```typescript
// 统一的错误处理
class ApiService {
  private async request<T>(url: string, options?: RequestInit): Promise<T> {
    try {
      const response = await fetch(url, {
        ...options,
        headers: {
          'Content-Type': 'application/json',
          ...options?.headers,
        },
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new ApiError(errorData.message, response.status, errorData)
      }

      const apiResponse = await response.json()
      return apiResponse.data
    } catch (error) {
      // 统一错误处理逻辑
      this.handleError(error)
      throw error
    }
  }

  private handleError(error: any) {
    // 记录错误、显示通知等
    console.error('API Error:', error)
  }
}
```

### 3. 实时更新策略

```typescript
// 轮询更新（简单方案）
const pollInterval = setInterval(async () => {
  await taskStore.fetchTasks()
}, 30000) // 每30秒更新一次

// WebSocket更新（未来版本）
// const ws = new WebSocket('ws://localhost:3030/ws')
// ws.onmessage = (event) => {
//   const update = JSON.parse(event.data)
//   handleRealTimeUpdate(update)
// }
```

## 版本兼容性

### 当前版本 (v1.0.0)

- ✅ 所有核心功能已实现
- ✅ 完整的CRUD操作
- ✅ 高级功能（时间块、模板、排序）
- ❌ 用户认证（单用户版本）
- ❌ 实时推送（使用轮询）
- ❌ 文件上传（暂未支持）

### 未来版本规划

- **v1.1.0**: WebSocket实时更新
- **v1.2.0**: 文件上传和附件
- **v2.0.0**: 多用户支持和认证
- **v2.1.0**: 高级搜索和筛选
- **v3.0.0**: 云同步和协作功能

## 快速开始示例

### 基础任务操作

```typescript
// 1. 初始化API服务
const api = new ApiService('http://localhost:3030/api')

// 2. 获取未安排任务
const unscheduledTasks = await api.get<Task[]>('/tasks/unscheduled')

// 3. 创建新任务
const newTask = await api.post<Task>('/tasks', {
  title: '学习Vue 3',
  glance_note: 'Composition API和新特性',
  estimated_duration: 180,
  context: {
    context_type: 'MISC',
    context_id: 'floating',
  },
})

// 4. 完成任务
const completedTask = await api.post<Task>(`/tasks/${newTask.id}/completion`)

console.log('任务已完成:', completedTask.completed_at)
```

### 日程安排示例

```typescript
// 1. 安排任务到今天
const today = new Date()
today.setHours(0, 0, 0, 0)

const schedule = await api.post<TaskSchedule>('/schedules', {
  task_id: taskId,
  target_day: today.toISOString(),
  mode: 'link',
})

// 2. 获取今天的所有日程
const todaySchedules = await api.get<TaskSchedule[]>(
  `/schedules?date=${encodeURIComponent(today.toISOString())}`
)

// 3. 记录努力
await api.post(`/schedules/${schedule.id}/presence`)
```

## 注意事项

### 1. 时间格式

- 所有时间戳使用ISO 8601格式（RFC 3339）
- 日期使用零点时间戳（如 `2024-09-29T00:00:00Z`）
- 时区统一使用UTC

### 2. ID格式

- 所有ID使用UUID v4格式
- 示例: `123e4567-e89b-12d3-a456-426614174000`

### 3. 排序值

- 使用LexoRank算法生成排序值
- 排序值为字符串（如 `"n"`, `"a"`, `"z"`）
- 支持在任意两个位置之间插入

### 4. 上下文ID规范

- **DAILY_KANBAN**: Unix时间戳字符串
- **PROJECT_LIST**: `project::{uuid}`格式
- **AREA_FILTER**: `area::{uuid}`格式
- **MISC**: 小写蛇形命名

### 5. 软删除

- 大部分实体使用软删除（`is_deleted: true`）
- 删除操作不会立即清除数据
- 可通过restore端点恢复（如果支持）

---

**文档版本**: v1.0.0  
**最后更新**: 2024年9月29日  
**维护者**: Cutie开发团队
