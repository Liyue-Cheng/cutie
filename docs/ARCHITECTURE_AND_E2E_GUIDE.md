# Cutie 系统整体架构与端到端功能开发指南

## 一、系统架构总览

### 1.1 技术栈

**前端**：

- Vue 3 (Composition API)
- TypeScript
- Pinia (状态管理)
- Interact.js (拖放)
- FullCalendar (日历)

**后端**：

- Rust + Axum (Web框架)
- SQLite (数据库)
- SQLx (数据库查询)
- Tokio (异步运行时)
- Server-Sent Events (SSE，实时通信)

### 1.2 前后端通信流程

```
前端组件
  ↓
CPU Pipeline (指令调度器)
  ↓
HTTP Request (标准化的请求格式)
  ↓
后端 Axum Router
  ↓
单文件端点组件
  ↓
业务逻辑 → 数据库事务 → Event Outbox
  ↓
HTTP Response (标准化的响应格式)
  ↓
CPU Pipeline Commit (更新前端状态)
  ↓
SSE Event (实时推送到其他客户端)
```

### 1.3 核心设计模式

#### CPU指令集架构 (ISA)

- **目的**：将所有前后端交互统一为"指令"，实现声明式编程
- **优势**：
  - 统一错误处理
  - 自动重试和超时管理
  - 乐观更新支持
  - 请求去重和防抖
  - 指令优先级调度

#### RTL Store 架构

- **Register (寄存器)**：纯响应式状态，只读
- **Transmission (传输线)**：Getters，计算属性
- **Logic (逻辑门)**：Mutations，纯数据操作（`_mut` 后缀）
- **DMA (直接内存访问)**：绕过指令系统的数据加载

#### 拖放策略系统

- **目的**：将复杂的拖放业务逻辑解耦为独立策略
- **组成**：
  - 条件匹配 (source/target viewKey)
  - 优先级排序
  - 多步骤操作链
  - 灵活的上下文传递

---

## 二、端到端新增功能开发流程

以"模板拖放功能"为例，说明完整的开发流程。

### 阶段 1：需求分析

**功能目标**：

1. 将模板拖动到日程看板 → 从模板创建任务并添加日程
2. 将任务拖动到模板区 → 从任务创建模板

**数据流**：

- 模板 → 创建任务 → 添加日程 → 更新排序
- 任务 → 创建模板 → 更新排序

### 阶段 2：数据库设计

**检查是否需要新表**：

- 模板表已存在，无需修改

**检查是否需要迁移**：

- 如果数据库结构有问题，创建迁移脚本
- 确保所有 ID 字段为 TEXT 类型（UUID字符串）

### 阶段 3：后端端点开发

#### 3.1 创建单文件组件端点

**文件命名**：`src-tauri/src/features/endpoints/{domain}/{action}.rs`

**结构模板**：

```rust
/// 端点描述 - 单文件组件
/// POST /api/{domain}/{action}

// ==================== CABC 文档 ====================
/*
1. 端点签名：POST /api/...
2. 用户故事：作为用户，我想要...
3. 输入输出：Request Body / Response
4. 验证规则：必填字段、格式要求
5. 业务逻辑：步骤说明
6. 边界情况：错误场景
7. 副作用：数据库操作、SSE事件
8. 契约：前置/后置条件
*/

// ==================== 依赖引入 ====================
use axum::{extract::{Path, State}, response::{IntoResponse, Response}, Json};
use serde::{Deserialize, Serialize};

// ==================== 请求/响应结构 ====================
#[derive(Deserialize)]
pub struct MyRequest { /* 字段 */ }

// ==================== HTTP 处理器 ====================
pub async fn handle(...) -> Response {
    match logic::execute(...).await {
        Ok(dto) => success_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== 业务逻辑层 ====================
mod logic {
    pub async fn execute(...) -> AppResult<MyDto> {
        // 1. 验证
        // 2. 获取依赖 (id_generator, clock)
        // 3. 获取写入许可 (acquire_write_permit)
        // 4. 开启事务
        // 5. 执行数据库操作
        // 6. 写入 Event Outbox (事务内)
        // 7. 提交事务
        // 8. 返回 DTO
    }
}

// ==================== 数据库层 ====================
mod database {
    pub async fn operation(...) -> AppResult<...> {
        sqlx::query("...")
            .bind(...)
            .execute(tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))
    }
}
```

**关键点**：

- ✅ 使用 `success_response(data)` 包装响应
- ✅ 在事务内写入 Event Outbox（用于 SSE）
- ✅ 获取写入许可（防止并发冲突）
- ✅ 完整的 CABC 文档

#### 3.2 注册路由

**文件**：`src-tauri/src/features/{domain}.rs`

```rust
pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/{id}/action", post(endpoints::my_endpoint))
}
```

### 阶段 4：前端指令集定义

#### 4.1 创建 ISA 文件

**文件**：`src/cpu/isa/{domain}-isa.ts`

```typescript
import type { ISADefinition } from './types'
import { pipeline } from '@/cpu'
import { useMyStore } from '@/stores/my'

export const MyISA: ISADefinition = {
  'domain.action': {
    meta: {
      description: '操作描述',
      category: 'domain',
      resourceIdentifier: (payload) => [`resource:${payload.id}`],
      priority: 5,
      timeout: 10000,
    },
    request: {
      method: 'POST',
      url: '/endpoint',
      // 或动态 URL
      url: (payload) => `/endpoint/${payload.id}`,
      body: (payload) => ({ ...payload }),
    },
    commit: async (result, payload) => {
      const store = useMyStore()
      store.addOrUpdate_mut(result)
    },
  },
}
```

**字段说明**：

- `resourceIdentifier`：资源唯一标识，用于请求去重
- `priority`：优先级（数字越大越优先）
- `request.url`：可以是字符串或函数
- `commit`：成功后的状态更新

#### 4.2 注册 ISA

**文件**：`src/cpu/isa/index.ts`

```typescript
import { MyISA } from './my-isa'

export const ISA: ISADefinition = {
  ...TaskISA,
  ...ScheduleISA,
  ...MyISA, // 新增
}
```

### 阶段 5：Store 重构

#### 5.1 Store 结构

**文件**：`src/stores/{domain}/index.ts`

```typescript
export const useMyStore = defineStore('my', () => {
  return {
    // ========== STATE (寄存器) ==========
    items: core.items,

    // ========== GETTERS (多路复用器) ==========
    allItems: core.allItems,
    getItemById: core.getItemById,

    // ========== MUTATIONS (寄存器写入) ==========
    addOrUpdate_mut: core.addOrUpdate_mut,
    remove_mut: core.remove_mut,

    // ========== DMA (数据加载) ==========
    fetchAll: view.fetchAll,

    // ========== EVENT HANDLING ==========
    initEventSubscriptions: events.initEventSubscriptions,
  }
})
```

#### 5.2 核心状态文件

**文件**：`src/stores/{domain}/core.ts`

```typescript
import { ref, computed } from 'vue'

// State
export const items = ref(new Map<string, MyItem>())

// Getters
export const allItems = computed(() => Array.from(items.value.values()))
export const getItemById = computed(() => (id: string) => items.value.get(id))

// Mutations (必须 _mut 后缀)
export function addOrUpdate_mut(item: MyItem) {
  const newMap = new Map(items.value)
  newMap.set(item.id, item)
  items.value = newMap
}

export function remove_mut(id: string) {
  const newMap = new Map(items.value)
  newMap.delete(id)
  items.value = newMap
}
```

#### 5.3 事件处理

**文件**：`src/stores/{domain}/event-handlers.ts`

```typescript
export function initEventSubscriptions() {
  const { eventBus } = useEventBus()

  eventBus.on('domain.created', (data: MyItem) => {
    core.addOrUpdate_mut(data)
  })

  eventBus.on('domain.updated', (data: MyItem) => {
    core.addOrUpdate_mut(data)
  })

  eventBus.on('domain.deleted', (data: { id: string }) => {
    core.remove_mut(data.id)
  })
}
```

**注意**：必须调用 `_mut` 版本的函数！

### 阶段 6：拖放策略开发

#### 6.1 创建策略文件

**文件**：`src/infra/drag/strategies/{feature}-scheduling.ts`

```typescript
import type { Strategy } from '../types'
import { pipeline } from '@/cpu'

export const myStrategy: Strategy = {
  id: 'my-strategy',
  name: '策略名称',

  conditions: {
    source: {
      viewKey: 'misc::source', // 或正则 /^daily::\d{4}-\d{2}-\d{2}$/
      objectType: 'task', // 可选
    },
    target: {
      viewKey: 'misc::target',
    },
    priority: 90, // 优先级
  },

  action: {
    name: 'action_name',
    description: '操作描述',

    async execute(ctx) {
      const operations = []

      try {
        // 步骤 1: 调用指令
        const result = await pipeline.dispatch('domain.action', {
          /* payload */
        })
        operations.push(createOperationRecord('action', ctx.targetViewId, payload))

        // 步骤 2: 更新排序
        await pipeline.dispatch('viewpreference.update_sorting', {
          view_key: ctx.targetViewId,
          sorted_task_ids: newSorting,
          original_sorted_task_ids: oldSorting,
        })

        return {
          success: true,
          message: '✅ 操作成功',
          operations,
          affectedViews: [ctx.targetViewId],
        }
      } catch (error) {
        return {
          success: false,
          message: `❌ 操作失败: ${error.message}`,
          operations,
        }
      }
    },
  },

  tags: ['domain', 'scheduling'],
}
```

#### 6.2 注册策略

**文件**：`src/infra/drag/strategies/index.ts`

```typescript
export { myStrategy } from './my-scheduling'
```

**自动注册**：策略会在 `src/main.ts` 中通过 `initializeDragStrategies()` 自动注册。

### 阶段 7：组件集成

#### 7.1 看板列组件

**关键点**：

```typescript
// 1. 设置 viewKey
const VIEW_KEY = 'misc::myview'

// 2. 注册拖放
const kanbanContainerRef = ref<HTMLElement | null>(null)

const { displayTasks } = useInteractDrag({
  viewMetadata,
  tasks: myTasks,
  containerRef: kanbanContainerRef,
  draggableSelector: `.task-card-wrapper-${VIEW_KEY.replace(/::/g, '--')}`,
  onDrop: async (session) => {
    await dragStrategy.executeDrop(session, VIEW_KEY, {
      sourceContext: session.metadata?.sourceContext || {},
      targetContext: {
        taskIds: myTasks.value.map(t => t.id),
        displayTasks: myTasks.value,
        dropIndex: dragPreviewState.value?.computed.dropIndex,
        viewKey: VIEW_KEY,
      },
    })
  },
})

// 3. 模板结构
<template>
  <CutePane>
    <div ref="kanbanContainerRef" class="kanban-dropzone-wrapper">
      <div
        v-for="task in displayTasks"
        :key="task.id"
        :class="`task-card-wrapper task-card-wrapper-${VIEW_KEY.replace(/::/g, '--')}`"
        :data-task-id="task.id"
      >
        <TaskCard :task="task" />
      </div>
    </div>
  </CutePane>
</template>
```

#### 7.2 调用指令

**在组件中**：

```typescript
import { pipeline } from '@/cpu'

async function handleAction() {
  try {
    const result = await pipeline.dispatch('domain.action', {
      /* payload */
    })
    // 指令成功后会自动调用 commit，更新 store
  } catch (error) {
    // 错误处理
  }
}
```

### 阶段 8：测试与验证

**检查清单**：

- ✅ 后端编译通过 (`cargo check`)
- ✅ 前端无 linter 错误
- ✅ 数据库迁移成功
- ✅ 指令可以正常调用
- ✅ Store 状态正确更新
- ✅ 拖放功能正常工作
- ✅ SSE 事件正确接收

---

## 三、数据流全景图

### 创建操作流程

```
用户点击"创建"按钮
  ↓
组件调用 pipeline.dispatch('domain.create', payload)
  ↓
CPU Pipeline 调度指令
  ↓
发送 HTTP POST /api/domain
  ↓
后端处理请求 → 数据库插入 → Event Outbox 写入
  ↓
返回 201 Created + DTO
  ↓
Pipeline 调用 commit(result)
  ↓
Store mutation 更新状态 (addOrUpdate_mut)
  ↓
Vue 响应式系统更新视图
  ↓ (并行)
SSE Dispatcher 推送事件
  ↓
其他客户端接收 domain.created 事件
  ↓
EventBus 触发事件处理器
  ↓
Store mutation 更新状态
  ↓
所有客户端视图同步
```

### 拖放操作流程

```
用户开始拖动
  ↓
Interact.js 触发 dragstart
  ↓
DragController 创建 DragSession
  ↓
用户移动到目标区域
  ↓
DragController 更新预览状态
  ↓
useInteractDrag 计算 displayTasks（显示预览）
  ↓
用户释放鼠标
  ↓
DragController 调用 onDrop
  ↓
DragStrategy 查找匹配的策略
  ↓
策略执行多步指令 (dispatch × N)
  ↓
每个指令独立完成 HTTP → Commit 流程
  ↓
所有指令成功 → 返回成功结果
  ↓
预览状态清除 → 视图更新完成
```

---

## 四、关键设计原则

### 4.1 单一职责

- **Store**: 只负责状态管理，不包含业务逻辑
- **Mutation**: 只负责数据变更，不调用 API
- **ISA**: 只负责声明，不包含实现
- **Strategy**: 只负责拖放逻辑，调用现有指令

### 4.2 声明式编程

- 所有 API 调用通过指令集声明
- 所有拖放操作通过策略声明
- 避免命令式的 fetch/axios 调用

### 4.3 不可变性

- Store 状态使用 Map（创建新 Map 而非修改）
- 避免直接修改对象属性

### 4.4 类型安全

- 所有 Payload 和 DTO 有明确的 TypeScript 类型
- 后端 Rust 类型自动生成前端绑定

### 4.5 错误处理

- 后端统一错误响应格式
- 前端统一错误捕获和日志
- 用户友好的错误提示

---

## 五、常见问题与解决方案

### 问题 1: 响应格式不一致

**症状**：前端报错 `Cannot read properties of undefined`

**原因**：后端直接返回 `Json(data)`，未包装为标准响应

**解决**：使用 `success_response(data)`

### 问题 2: SSE 事件未触发

**症状**：创建数据后其他客户端未更新

**原因**：未在事务内写入 Event Outbox

**解决**：在业务逻辑中添加 Event Outbox 写入

### 问题 3: 拖放预览不显示

**症状**：拖动时看不到预览元素

**原因**：容器 ref 绑定错误或选择器不匹配

**解决**：

- ref 绑定到 HTMLElement（非组件）
- 确保 `data-task-id` 属性存在
- 选择器使用正确的类名

### 问题 4: Mutation 函数未定义

**症状**：`core.addOrUpdate is not a function`

**原因**：使用了旧的函数名（未加 `_mut` 后缀）

**解决**：

- 所有 mutation 必须以 `_mut` 结尾
- 在事件处理器中调用 `_mut` 版本

### 问题 5: 数据库类型不匹配

**症状**：`mismatched types; Rust type String is not compatible with SQL type BLOB`

**原因**：数据库字段类型与代码期望不一致

**解决**：创建迁移脚本修复表结构

---

## 六、开发规范速查

### 命名规范

- **指令名**：`domain.action`（小写，点分隔）
- **Mutation**：`actionName_mut`（驼峰，`_mut` 后缀）
- **ViewKey**：`type::identifier`（小写，双冒号分隔）
- **文件名**：`kebab-case.ts/rs`（短横线）

### 代码组织

```
src/
  cpu/isa/          # 前端指令集
  stores/           # 状态管理
  infra/drag/       # 拖放系统
  components/       # Vue 组件

src-tauri/src/
  features/
    endpoints/      # API 端点
    {domain}.rs     # 路由注册
  entities/         # 数据实体
  infra/            # 基础设施
```

### 开发检查清单

**后端**：

- [ ] CABC 文档完整
- [ ] 使用 `success_response` 包装
- [ ] 事务内写入 Event Outbox
- [ ] 获取写入许可
- [ ] 路由已注册

**前端**：

- [ ] ISA 定义完整
- [ ] ISA 已注册
- [ ] Mutation 使用 `_mut` 后缀
- [ ] 事件处理器调用正确的 mutation
- [ ] 拖放策略已导出
- [ ] 组件使用 `pipeline.dispatch`

**测试**：

- [ ] 后端编译通过
- [ ] 前端无 linter 错误
- [ ] 功能正常工作
- [ ] SSE 事件正常推送
