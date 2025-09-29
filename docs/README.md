# Cutie API 文档

欢迎使用Cutie任务管理系统的API文档！

## 📚 文档目录

### 核心文档

- **[OpenAPI规范](./openapi.yaml)** - 完整的API规范文档（可用于生成客户端代码）
- **[前端API使用指南](./frontend-api-guide.md)** - 面向前端开发者的实用指南
- **[API端点总览](./api-endpoints-overview.md)** - 所有52个API端点的快速参考

### 架构文档

- **[完整开发过程总结](../ai-doc/complete-development-process-summary.md)** - 整个后端重构的详细记录
- **[业务逻辑修改指南](../ai-doc/business-logic-modification-guide.md)** - 如何安全地修改业务逻辑
- **[数据库字段修改指南](../ai-doc/database-schema-modification-guide.md)** - 数据库变更的完整流程

## 🚀 快速开始

### 1. 启动后端服务器

#### 开发环境

```bash
# 方式1: 与Tauri应用一起启动（推荐）
pnpm tauri dev

# 方式2: 单独启动Sidecar服务器
cargo run --manifest-path src-tauri/Cargo.toml -- --sidecar
```

#### 验证启动

```bash
# 检查服务器是否启动
curl http://localhost:3030/health

# 测试API连通性
curl http://localhost:3030/api/ping
```

### 2. 基础API测试

#### 创建第一个任务

```bash
curl -X POST "http://localhost:3030/api/tasks" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "我的第一个任务",
    "glance_note": "这是一个测试任务",
    "context": {
      "context_type": "MISC",
      "context_id": "floating"
    }
  }'
```

#### 获取未安排任务

```bash
curl -X GET "http://localhost:3030/api/tasks/unscheduled"
```

#### 完成任务

```bash
# 使用上面创建任务返回的ID
curl -X POST "http://localhost:3030/api/tasks/{task_id}/completion"
```

### 3. 前端集成

#### 在Vue项目中使用

```typescript
// 1. 定义API服务
import { setApiBaseUrl } from '@/stores/task'

// 2. 配置API基础URL（如果需要）
setApiBaseUrl('http://localhost:3030/api')

// 3. 在组件中使用
import { useTaskStore } from '@/stores/task'

const taskStore = useTaskStore()

// 获取任务
await taskStore.fetchTasks()

// 创建任务
await taskStore.createTask({
  title: '新任务',
  context: {
    context_type: 'MISC',
    context_id: 'floating',
  },
})

// 完成任务
await taskStore.completeTask(taskId)
```

## 📖 API使用模式

### 1. RESTful资源操作

```typescript
// 标准CRUD模式
GET / api / tasks // 查询（带参数）
POST / api / tasks // 创建
GET / api / tasks / { id } // 获取单个
PUT / api / tasks / { id } // 更新
DELETE / api / tasks / { id } // 删除
```

### 2. 业务操作端点

```typescript
// 业务特定操作
POST / api / tasks / { id } / completion // 完成任务
POST / api / tasks / { id } / reopen // 重新打开
POST / api / schedules / { id } / presence // 记录努力
```

### 3. 查询和筛选

```typescript
// 参数化查询
GET /api/tasks/search?q=关键词&limit=20
GET /api/schedules?date=2024-09-29T00:00:00Z
GET /api/areas?parent_id=uuid&include_descendants=true
```

## 🔧 开发工具

### 1. API文档查看器

#### 使用Swagger UI（推荐）

```bash
# 安装swagger-ui-serve
npm install -g swagger-ui-serve

# 启动文档服务器
swagger-ui-serve docs/openapi.yaml
```

#### 在线编辑器

- [Swagger Editor](https://editor.swagger.io/) - 在线查看和编辑OpenAPI文档
- [Redoc](https://redocly.github.io/redoc/) - 美观的API文档展示

### 2. API测试工具

#### Postman集合

```json
{
  "info": {
    "name": "Cutie API",
    "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json"
  },
  "item": [
    {
      "name": "获取未安排任务",
      "request": {
        "method": "GET",
        "header": [],
        "url": {
          "raw": "{{baseUrl}}/tasks/unscheduled",
          "host": ["{{baseUrl}}"],
          "path": ["tasks", "unscheduled"]
        }
      }
    }
  ],
  "variable": [
    {
      "key": "baseUrl",
      "value": "http://localhost:3030/api"
    }
  ]
}
```

#### VS Code REST Client

```http
### 变量定义
@baseUrl = http://localhost:3030/api

### 获取未安排任务
GET {{baseUrl}}/tasks/unscheduled

### 创建任务
POST {{baseUrl}}/tasks
Content-Type: application/json

{
  "title": "测试任务",
  "context": {
    "context_type": "MISC",
    "context_id": "floating"
  }
}

### 完成任务
POST {{baseUrl}}/tasks/{{taskId}}/completion
```

### 3. 代码生成工具

#### OpenAPI Generator

```bash
# 安装OpenAPI Generator
npm install -g @openapitools/openapi-generator-cli

# 生成TypeScript客户端
openapi-generator-cli generate \
  -i docs/openapi.yaml \
  -g typescript-fetch \
  -o src/api/generated
```

#### 生成的客户端使用示例

```typescript
import { TasksApi, Configuration } from '@/api/generated'

const config = new Configuration({
  basePath: 'http://localhost:3030/api',
})

const tasksApi = new TasksApi(config)

// 使用生成的客户端
const tasks = await tasksApi.getUnscheduledTasks()
const newTask = await tasksApi.createTask({
  createTaskPayload: {
    title: '新任务',
    context: {
      context_type: 'MISC',
      context_id: 'floating',
    },
  },
})
```

## 🐛 故障排除

### 常见问题

#### 1. 连接失败

```
错误: fetch failed / Connection refused
解决: 确保后端服务器已启动在3030端口
检查: curl http://localhost:3030/health
```

#### 2. CORS错误

```
错误: CORS policy blocked
解决: 确保前端域名在后端CORS配置中
配置: cors_origins: ["http://localhost:1420"]
```

#### 3. 验证错误

```
错误: 422 Unprocessable Entity
原因: 请求数据不符合验证规则
检查: 查看错误响应中的validation_errors字段
```

#### 4. 任务未找到

```
错误: 404 Not Found
原因: 任务ID不存在或已被删除
检查: 确认ID格式正确，检查is_deleted字段
```

### 调试技巧

#### 1. 启用详细日志

```bash
# 设置环境变量启用详细日志
export RUST_LOG=debug
export CUTIE_SERVER_REQUEST_LOGGING=true

# 重启服务器
cargo run --manifest-path src-tauri/Cargo.toml -- --sidecar
```

#### 2. 数据库检查

```bash
# 连接到SQLite数据库
sqlite3 ~/.local/share/cutie/cutie.db

# 查看任务表
.schema tasks
SELECT * FROM tasks LIMIT 5;

# 检查日程表
SELECT * FROM task_schedules LIMIT 5;
```

#### 3. 网络抓包

```bash
# 使用tcpdump抓包（Linux/macOS）
sudo tcpdump -i lo0 port 3030

# 使用Wireshark抓包（Windows）
# 筛选器: tcp.port == 3030
```

## 📊 监控和指标

### 1. 健康检查端点

```bash
# 系统健康状态
curl http://localhost:3030/health

# 响应示例
{
  "status": "healthy",
  "timestamp": "2024-09-29T10:00:00Z",
  "version": "1.0.0",
  "details": {
    "database": "healthy",
    "memory": "healthy",
    "disk": "healthy"
  }
}
```

### 2. 性能指标

```bash
# 获取服务器信息
curl http://localhost:3030/info

# 响应示例
{
  "name": "Cutie API Server",
  "version": "1.0.0",
  "build_time": "2024-09-29T08:00:00Z",
  "rust_version": "1.70.0",
  "features": [
    "task_management",
    "schedule_management",
    "time_blocking",
    "template_system"
  ]
}
```

### 3. 统计端点

```bash
# 任务统计
curl http://localhost:3030/api/tasks/stats

# 日程统计
curl http://localhost:3030/api/schedules/stats

# 模板统计
curl http://localhost:3030/api/templates/stats

# 领域统计
curl http://localhost:3030/api/areas/stats
```

## 🔐 安全注意事项

### V1.0版本（单用户）

- ❌ 无身份验证要求
- ❌ 无权限检查
- ✅ 输入验证和清理
- ✅ SQL注入防护（参数化查询）
- ✅ 请求大小限制

### 未来版本安全规划

- 🔄 JWT身份验证
- 🔄 基于角色的权限控制
- 🔄 API速率限制
- 🔄 审计日志
- 🔄 数据加密

## 📞 支持和反馈

### 技术支持

- **GitHub Issues**: [项目Issues页面]
- **开发文档**: `ai-doc/`目录下的详细文档
- **API规范**: `docs/openapi.yaml`

### 贡献指南

1. 阅读[业务逻辑修改指南](../ai-doc/business-logic-modification-guide.md)
2. 遵循[数据库字段修改指南](../ai-doc/database-schema-modification-guide.md)
3. 确保所有测试通过
4. 更新相关文档

---

**Happy Coding! 🎉**

_Cutie API - 让任务管理变得简单而优雅_
