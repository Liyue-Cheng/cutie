# 开发经验教训

本文档记录在项目开发过程中遇到的重要问题和解决方案，供未来参考。

---

## 2025-10-07: 硬编码端口导致 API 连接失败

### 问题: 拖拽链接功能无法连接后端

**现象：**

- 拖动任务到已有时间块时，浏览器报错 `net::ERR_CONNECTION_REFUSED`
- 控制台显示请求地址为 `http://127.0.0.1:3538/api/...`
- 其他 API 调用都正常工作

**根本原因：**

- 在 `useCalendarDrag.ts` 中硬编码了端口号 `3538`
- Tauri 的 sidecar 使用动态端口，每次启动可能不同
- 其他代码都使用 `apiBaseUrl` 动态获取正确端口
- 只有这个新功能硬编码了错误的端口

**错误代码：**

```typescript
// ❌ 错误：硬编码端口
const response = await fetch(
  `http://127.0.0.1:3538/api/time-blocks/${eventIdToLink}/link-task`,
  {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ task_id: currentDraggedTask.value.id }),
  }
)
```

**解决方案：**

```typescript
// ✅ 正确：使用动态端口
import { apiBaseUrl } from '@/composables/useApiConfig'

const response = await fetch(
  `${apiBaseUrl.value}/time-blocks/${eventIdToLink}/link-task`,
  {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ task_id: currentDraggedTask.value.id }),
  }
)
```

**经验教训：**

> **永远不要硬编码 API 端口！** 在 Tauri 项目中，sidecar 服务器使用动态端口以避免端口冲突。所有 API 调用都必须使用 `apiBaseUrl` 或 `useApiConfig()` 获取正确的端口。

**检查清单：**

在编写新的 API 调用时，务必检查：

- [ ] 是否导入了 `apiBaseUrl` 或 `useApiConfig`
- [ ] 是否使用 `apiBaseUrl.value` 构建 URL
- [ ] 是否有任何硬编码的端口号（3030、3538 等）
- [ ] 是否使用了 `http://127.0.0.1:${port}` 这种拼接方式

**相关代码：**

- `src/composables/useApiConfig.ts` - 端口发现和管理
- `src/stores/*.ts` - 所有 Store 都使用 `apiBaseUrl`
- `src/composables/calendar/useCalendarDrag.ts` - 修复示例

**架构原则：**

- Tauri sidecar 启动时监听随机可用端口
- 前端通过 Tauri event `sidecar-port-discovered` 获取端口
- `useApiConfig` 提供响应式的 `apiBaseUrl` 供全局使用
- 所有 HTTP 请求都必须使用这个动态 URL

---

## 2025-10-07: 归档功能实现中的两个关键问题

### 问题 1: 前端过滤遗漏导致 UI 不一致

**现象：**

- 归档任务后，任务不会立即从日期看板移除
- 任务会出现在归档栏（正确）
- 刷新后任务才从日期看板消失

**根本原因：**

- `getTasksByDate` 计算属性没有过滤归档任务（`is_archived === true`）
- 导致归档的任务仍然显示在日期看板中

**解决方案：**

```typescript
// src/stores/task/core.ts
const getTasksByDate = computed(() => (date: string) => {
  return allTasksArray.value.filter((task) => {
    // ✅ 添加归档任务过滤
    if (task.is_archived) {
      return false
    }
    // 检查任务是否有该日期的 schedule
    return task.schedules?.some((schedule) => schedule.scheduled_day === date) ?? false
  })
})
```

**经验教训：**

> 在实现状态过滤功能（如归档、删除、完成等）时，必须检查所有使用该数据的计算属性和过滤器，确保一致性。前端的多个视图可能使用不同的过滤逻辑，需要逐一审查。

---

### 问题 2: 后端查询过滤导致数据丢失

**现象：**

- 归档任务后，任务出现在归档栏（正确）
- 刷新页面后，归档栏被清空（错误）
- 归档任务在数据库中存在，但前端获取不到

**根本原因：**

- 后端 `/views/all` 端点的 SQL 查询包含 `WHERE archived_at IS NULL`
- `fetchAllTasks()` 无法加载归档任务到前端
- 前端的 `archivedTasks` 计算属性依赖于 `allTasks`，但数据源为空

**解决方案：**

```rust
// src-tauri/src/features/views/endpoints/get_all.rs
pub async fn find_all_tasks(pool: &sqlx::SqlitePool) -> AppResult<Vec<Task>> {
    let query = r#"
        SELECT ...
        FROM tasks
        WHERE is_deleted = false
        -- ❌ 移除: AND archived_at IS NULL
        ORDER BY created_at DESC
    "#;
    // ...
}
```

**经验教训：**

> 当实现"获取所有数据"的端点时，要特别注意不要过度过滤。"all" 应该包含所有状态的数据（已完成、未完成、归档等），让前端根据需要进行过滤。后端的过滤会导致前端的计算属性无法工作，因为数据源本身就不完整。

**架构原则：**

- **后端**: `/views/all` 应返回所有任务（只排除 `is_deleted = true`）
- **前端**: 使用计算属性在客户端进行状态过滤（`archivedTasks`、`completedTasks` 等）
- **好处**: 前端可以灵活切换视图，无需重复请求后端

---

### 问题 3: 归档任务未清理未来日程

**现象：**

- 归档任务后，任务的未来日程仍然保留
- 取消归档后，任务会重新出现在未来的日期看板中
- 用户预期：归档的任务不应该有未来的待办事项

**根本原因：**

- 归档任务时只更新了 `archived_at` 字段
- 没有删除当天及之后的日程
- 归档的业务语义是"暂时放下、不再处理"，应该清理所有未来安排

**解决方案：**

```rust
// src-tauri/src/features/tasks/endpoints/archive_task.rs

// 3. 删除当天及之后的所有日程（包括关联的时间块）
let today = now.date_naive();
database::delete_today_and_future_schedules_in_tx(&mut tx, task_id, today).await?;

// 4. 更新任务的 archived_at 字段
database::set_archived_in_tx(&mut tx, task_id, now).await?;
```

删除日程的步骤：

1. 查找所有当天及之后的日程日期
2. 对每个日期：
   - 查找该日期的所有时间块
   - 删除任务到时间块的链接（`task_time_block_links`）
   - 软删除"孤儿"时间块（没有其他任务关联的时间块）
3. 删除所有当天及之后的日程记录（`task_schedules`）

**经验教训：**

> **状态转换时要考虑关联数据的清理**：实现状态变更功能（如归档、删除、完成等）时，要仔细思考该状态变更对关联数据的影响。不仅要更新主实体的状态字段，还要考虑：
>
> 1. 哪些关联数据应该保留（如过去的日程，用于历史记录）
> 2. 哪些关联数据应该删除（如未来的日程，因为任务已不再进行）
> 3. 删除关联数据时要级联清理（时间块链接、孤儿时间块等）

**业务语义分析：**

- **归档** = "暂时放下"：删除未来日程 ✅，保留过去日程（历史）✅
- **删除** = "不再需要"：删除所有日程和关联数据 ✅
- **完成** = "已完成"：保留所有日程（记录完成过程）✅

---

## 通用原则

### 状态过滤的最佳实践

1. **后端**: 提供完整数据，只排除物理删除的记录
2. **前端**: 使用计算属性进行逻辑过滤
3. **一致性检查**: 新增状态字段时，检查所有过滤器和计算属性

### 状态转换与数据清理

1. **语义分析**: 理解状态变更的业务含义
2. **关联数据**: 明确哪些数据应保留、哪些应删除
3. **级联清理**: 删除时要处理所有依赖和关联（链接表、孤儿记录等）
4. **历史保留**: 考虑是否需要保留历史数据用于审计和分析

### 调试工具的重要性

- 为复杂功能添加调试按钮（如"加载全部"）能快速验证数据完整性
- 在开发阶段保留详细的日志输出
- 使用 alert 或 toast 显示操作结果的统计信息

---

## 2025-10-07: 任务卡片按钮交互实现中的关键问题

### 问题 1: 前后端枚举格式不一致导致按钮状态不更新

**现象：**

- 点击在场按钮（星星）后，按钮不会变色（保持灰色）
- API 请求成功，但前端状态没有反映更新
- 刷新页面后状态才正确显示

**根本原因：**

后端有两个不同的枚举定义，使用不同的序列化格式：

1. **`Outcome` 枚举**（用于 API 输入）：

   ```rust
   #[serde(rename_all = "UPPERCASE")]
   pub enum Outcome {
       Planned,              // → "PLANNED"
       PresenceLogged,       // → "PRESENCE_LOGGED"
       CompletedOnDay,       // → "COMPLETED_ON_DAY"
       CarriedOver,          // → "CARRIED_OVER"
   }
   ```

2. **`DailyOutcome` 枚举**（用于 DTO 输出）：
   ```rust
   #[serde(rename_all = "snake_case")]
   pub enum DailyOutcome {
       Planned,              // → "planned"
       PresenceLogged,       // → "presence_logged"
       Completed,            // → "completed"
       CarriedOver,          // → "carried_over"
   }
   ```

初始实现错误地在所有地方使用了大写格式：

```typescript
// ❌ 错误：判断时使用大写
const isPresenceLogged = computed(() => {
  return currentScheduleOutcome.value === 'PRESENCE_LOGGED' // 永远不会匹配
})

// ❌ 错误：发送时也使用大写
const newOutcome = newCheckedValue ? 'PRESENCE_LOGGED' : 'PLANNED' // 这个是对的
await taskStore.updateSchedule(props.task.id, kanbanDate, { outcome: newOutcome })
```

**解决方案：**

区分接收和发送的数据格式：

```typescript
// ✅ 正确：接收数据时使用小写（来自后端 DTO）
const isPresenceLogged = computed(() => {
  return currentScheduleOutcome.value === 'presence_logged' // snake_case
})

// ✅ 正确：发送数据时使用大写（发送给后端 API）
const newOutcome = newCheckedValue ? 'PRESENCE_LOGGED' : 'PLANNED' // UPPERCASE
await taskStore.updateSchedule(props.task.id, kanbanDate, { outcome: newOutcome })
```

**经验教训：**

> **前后端数据格式要一致，或明确区分接收/发送格式**：
>
> 1. **理想方案**：前后端使用统一的枚举格式（如都用 snake_case 或都用 UPPERCASE）
> 2. **当前方案**：后端输入输出使用不同格式时，前端必须区分：
>    - 从后端接收：使用 DTO 格式（`snake_case`）
>    - 发送给后端：使用 API 输入格式（`UPPERCASE`）
> 3. **调试技巧**：枚举值不匹配时，使用 `console.log` 打印实际接收的值，不要猜测
> 4. **类型安全**：前端可以定义 TypeScript 类型来强制区分输入/输出格式

---

### 问题 2: 事件处理函数使用当前状态而非事件参数

**现象：**

- 点击在场按钮后，按钮状态切换不正确
- 有时会连续点击两次才生效
- 状态更新逻辑不稳定

**根本原因：**

初始实现在事件处理函数中使用当前状态来计算新状态：

```typescript
// ❌ 错误：使用当前状态计算
async function handlePresenceToggle() {
  // 这里读取的是点击前的旧状态
  const newOutcome = isPresenceLogged.value ? 'PLANNED' : 'PRESENCE_LOGGED'
  await taskStore.updateSchedule(props.task.id, kanbanDate, { outcome: newOutcome })
}
```

问题在于：

1. `CuteCheckbox` 组件触发 `@update:checked` 事件时，已经传递了新的 checked 值
2. 但我们忽略了这个参数，而是从 computed 属性读取旧值
3. 如果状态更新慢于点击，会导致状态计算错误

**解决方案：**

使用事件参数中的新值：

```typescript
// ✅ 正确：使用事件参数
async function handlePresenceToggle(newCheckedValue: boolean) {
  // 直接使用事件传递的新状态
  const newOutcome = newCheckedValue ? 'PRESENCE_LOGGED' : 'PLANNED'
  await taskStore.updateSchedule(props.task.id, kanbanDate, { outcome: newOutcome })
}
```

**经验教训：**

> **事件处理函数应该依赖事件参数而非当前状态**：
>
> 1. **优先使用事件参数**：UI 组件触发的事件通常包含新值，这是最准确的数据源
> 2. **避免状态竞争**：从 computed 属性读取状态可能获取到旧值（如果响应式更新还没完成）
> 3. **明确意图**：`newCheckedValue` 的命名清楚表明这是"新值"，而不是"当前值"
> 4. **单向数据流**：事件 → 处理函数 → API → 状态更新 → UI 刷新，不要在中间插入状态读取

---

### 问题 3: 防误触交互的实现细节

**需求：**

点击在场按钮后：

- 在场按钮立即移到左边并长显
- 完成按钮移到右边（原本在场按钮的位置）
- 但完成按钮暂时不显示（防止用户误触）
- 用户移出鼠标后，下次悬浮才显示完成按钮

**实现方案：**

```typescript
// 1. 添加防误触状态
const justToggledPresence = ref(false)

// 2. 点击在场按钮时设置标志
async function handlePresenceToggle(newCheckedValue: boolean) {
  justToggledPresence.value = true  // 标记刚点击过
  await taskStore.updateSchedule(...)
}

// 3. 鼠标离开卡片时重置
function handleMouseLeave() {
  justToggledPresence.value = false
}

// 4. 模板中条件渲染
<CuteCheckbox
  v-if="shouldShowCompleteButton && !justToggledPresence"
  class="main-checkbox hover-visible"
  ...
/>
```

**关键点：**

1. **时机控制**：使用 `@mouseleave` 事件而非 `setTimeout`，确保用户完全离开后才重置
2. **条件组合**：`v-if` 同时检查业务规则和防误触标志
3. **用户体验**：避免按钮"跳来跳去"，给用户充足的反应时间

**经验教训：**

> **防误触设计要考虑用户操作流程**：
>
> 1. **识别误触场景**：按钮位置切换时，新按钮正好出现在用户刚点击的位置
> 2. **合理的延迟**：不是简单的时间延迟，而是等待用户"重新开始"操作（鼠标离开再回来）
> 3. **状态管理**：使用简单的 boolean flag，而非复杂的状态机
> 4. **可测试性**：防误触逻辑独立于业务逻辑，易于调试和修改

---

## 2025-10-07: 新增字段时的前后端同步问题

### 问题：预期时间字段显示 "NaNmin" 且无法持久化

**现象：**

- 前端显示预期时间时出现 "NaNmin"
- 用户选择时间后无法保存到数据库
- 刷新页面后数据丢失

**根本原因：**

虽然数据库中已经有 `estimated_duration` 字段，但在从数据库到前端的整个数据流中缺少关键的映射步骤：

1. ❌ **后端 DTO 缺少字段**：`TaskCardDto` 没有 `estimated_duration` 字段
2. ❌ **Assembler 未映射**：从 `Task` 实体转换为 DTO 时没有映射该字段
3. ❌ **Update 端点未处理**：`update_task` 端点虽然文档提到了该字段，但实际没有处理

**数据流中的断点：**

```
数据库 (tasks.estimated_duration)
    ↓ ✅ Task 实体有字段
    ↓ ❌ TaskCardDto 缺少字段 ← 第一个断点
    ↓ ❌ Assembler 未映射 ← 第二个断点
    ↓ ✅ 前端 DTO 有字段
    ↓ ✅ UI 显示 (但收到 undefined，显示 NaN)
    ↓ ✅ 用户修改
    ↓ ❌ Update 端点未处理 ← 第三个断点
    ✗ 无法写回数据库
```

**解决方案：**

#### 1. 后端 DTO 添加字段

```rust
// src-tauri/src/entities/task/response_dtos.rs
pub struct TaskCardDto {
    // ...
    pub estimated_duration: Option<i32>, // 预期时长（分钟）
    // ...
}
```

#### 2. Assembler 映射字段

```rust
// src-tauri/src/features/tasks/shared/assembler.rs
pub fn task_to_card_basic(task: &Task) -> TaskCardDto {
    TaskCardDto {
        // ...
        estimated_duration: task.estimated_duration, // ✅ 添加映射
        // ...
    }
}
```

#### 3. Update 端点处理字段

```rust
// src-tauri/src/features/tasks/shared/repositories/task_repository.rs

// A. 添加到 set_clauses
if request.estimated_duration.is_some() {
    set_clauses.push("estimated_duration = ?");
}

// B. 添加到参数绑定
if let Some(estimated_duration) = &request.estimated_duration {
    q = q.bind(estimated_duration.clone());
}
```

**经验教训：**

> **新增字段必须打通完整数据流**：
>
> 1. **数据流检查清单**：添加新字段时，必须检查以下所有环节：
>    - [ ] 数据库 Schema（表结构）
>    - [ ] 后端实体（Entity/Model）
>    - [ ] 后端请求 DTO（Request DTO）
>    - [ ] 后端响应 DTO（Response DTO）
>    - [ ] Assembler/Mapper（实体到 DTO 的转换）
>    - [ ] Repository（数据库读写逻辑）
>    - [ ] 端点处理（API endpoint）
>    - [ ] 前端类型定义（TypeScript types）
>    - [ ] 前端 UI 显示
> 2. **从后往前验证**：最容易遗漏的是中间层（DTO、Assembler、Repository）
>    - 数据库有字段 ≠ API 会返回字段
>    - API 文档说支持 ≠ 代码实际处理了
>    - 前端能显示 ≠ 能保存成功
> 3. **端到端测试**：新增字段后必须进行完整的读写测试：
>    - **读取测试**：从数据库 → API → 前端显示
>    - **写入测试**：前端修改 → API → 数据库保存 → 刷新验证
> 4. **类型安全的价值**：
>    - Rust 的类型系统会在编译时捕获 Entity 和 Request DTO 的不匹配
>    - 但 DTO 的字段缺失不会报错（可选字段合法）
>    - Assembler 的遗漏也不会报错（所有字段都是独立设置）
>    - 只有运行时测试才能发现数据流断点
> 5. **文档与代码一致性**：
>    - API 文档提到 `estimated_duration` 但代码未实现
>    - 这种不一致容易误导开发，以为已经实现
>    - 应该：代码先行，文档跟随；或者用文档驱动开发时确保实现

**调试技巧：**

- **前端收到 `undefined`**：检查后端 Response DTO 和 Assembler
- **后端收到但不处理**：检查 Repository 的 UPDATE 语句
- **数据库没有字段**：最容易发现，会有 SQL 错误
- **中间层缺失**：最难发现，需要端到端测试

**预防措施：**

在项目中建立"新字段清单"，每次添加字段时对照检查所有环节，避免遗漏。

---

## 2025-10-07: 任务编辑器 Modal 开发中的问题

### 问题 1: Modal 按钮状态不同步

**现象：**

- 在场按钮和完成按钮的勾选状态与实际数据不一致
- 点击按钮后状态不更新
- 刷新页面后才显示正确状态

**根本原因：**

在场按钮的状态被硬编码为 `false`，没有从任务数据中读取：

```vue
<!-- ❌ 错误：硬编码状态 -->
<CuteCheckbox
  :checked="false"
  size="large"
  variant="star"
  @update:checked="handlePresenceToggle"
/>
```

同时，`handlePresenceToggle` 函数只是一个空的 `console.log`，没有真正调用 API。

**解决方案：**

1. 添加状态计算逻辑：

```typescript
// 获取今天的日期
const todayDate = computed(() => {
  const now = new Date()
  return now.toISOString().split('T')[0]
})

// 获取今天的 schedule outcome
const currentScheduleOutcome = computed(() => {
  if (!task.value?.schedules || !todayDate.value) return null
  const todaySchedule = task.value.schedules.find((s) => s.scheduled_day === todayDate.value)
  return todaySchedule?.outcome || null
})

// 今天是否已记录在场
const isPresenceLogged = computed(() => {
  return currentScheduleOutcome.value === 'presence_logged'
})
```

2. 实现在场切换功能：

```typescript
async function handlePresenceToggle(isChecked: boolean) {
  if (!props.taskId || !todayDate.value) return
  const newOutcome = isChecked ? 'presence_logged' : undefined
  await taskStore.updateSchedule(props.taskId, todayDate.value, { outcome: newOutcome })
}
```

3. 修复模板绑定：

```vue
<!-- ✅ 正确：绑定实际状态 -->
<CuteCheckbox
  :checked="isPresenceLogged"
  size="large"
  variant="star"
  @update:checked="handlePresenceToggle"
/>
```

**经验教训：**

> **复用组件代码时要完整实现所有功能**：
>
> 1. **不要遗留 TODO 或 console.log**：事件处理函数必须有真实的业务逻辑
> 2. **状态绑定要准确**：`:checked` 必须绑定到实际的计算属性，不能硬编码
> 3. **测试完整性**：测试时要验证：显示状态 → 点击 → API 调用 → 状态更新 → UI 刷新
> 4. **复用与修改的平衡**：从其他组件复制代码时，要仔细审查每一行，确保适配新场景

---

### 问题 2: Textarea 自动延展与滚动条冲突

**需求：**

笔记区域的 textarea 应该：

- 根据内容自动增高，不显示滚动条
- 卡片整体高度随内容延展
- 当卡片超出屏幕时，在 Modal 卡片上显示滚动条

**初始实现的问题：**

1. Textarea 有固定的 `rows` 属性，内容多时出现内部滚动条
2. Textarea 的 `resize: vertical` 显示右下角调整图标
3. 卡片没有高度限制，超长内容会溢出屏幕

**解决方案：**

#### 1. 自动调整 Textarea 高度

```typescript
// 自动调整 textarea 高度
function autoResizeTextarea(textarea: HTMLTextAreaElement) {
  textarea.style.height = 'auto'
  textarea.style.height = textarea.scrollHeight + 'px'
}

// 在 input 事件中调用
<textarea
  v-model="glanceNote"
  rows="1"
  @input="autoResizeTextarea($event.target as HTMLTextAreaElement)"
/>
```

#### 2. 初始化高度

```typescript
// 加载任务后初始化所有 textarea 高度
onMounted(async () => {
  // ... 加载任务数据
  await nextTick()
  initTextareaHeights()
})
```

#### 3. 优化样式

```css
.note-textarea {
  resize: none; /* 移除右下角调整图标 */
  overflow: hidden; /* 防止出现滚动条 */
  min-height: 2rem;
}

.editor-card {
  max-height: 90vh; /* 限制最大高度 */
  overflow-y: auto; /* 超出时显示滚动条 */
}
```

**经验教训：**

> **自动延展 Textarea 的实现要点**：
>
> 1. **动态高度计算**：
>    - 先设置 `height: auto` 让浏览器计算内容高度
>    - 再设置 `height: scrollHeight` 应用计算结果
>    - 必须在 `@input` 事件中实时调用
> 2. **初始化时机**：
>    - 使用 `nextTick()` 确保 DOM 已更新
>    - 在数据加载和任务切换时都要重新初始化
> 3. **样式配合**：
>    - `resize: none` - 移除调整图标
>    - `overflow: hidden` - 防止内部滚动条
>    - `rows="1"` - 最小行数，避免初始空白
> 4. **滚动容器的选择**：
>    - Textarea 本身不滚动（`overflow: hidden`）
>    - 外层容器负责滚动（`.editor-card`）
>    - 这样用户体验更好，不会出现"滚动条套滚动条"
> 5. **隐形设计**：
>    - `hover` 和 `focus` 都不改变背景色
>    - 没有边框和视觉变化
>    - 看起来像普通文本区域，但可以编辑

---

### 问题 3: Modal 点击关闭的误触 Bug

**现象：**

- 用户在卡片内按下鼠标
- 拖动到卡片外释放鼠标
- Modal 被意外关闭

**根本原因：**

初始实现使用 `@click` 事件监听 overlay：

```vue
<!-- ❌ 问题代码 -->
<div class="modal-overlay" @click="handleClose">
  <CuteCard class="editor-card" @click.stop>
    <!-- ... -->
  </CuteCard>
</div>
```

`click` 事件的触发条件是：在同一元素上 `mousedown` 和 `mouseup`。但是：

1. 用户在卡片内 `mousedown`
2. 移动到 overlay 上 `mouseup`
3. 由于 `mousedown` 位置不同，不会在卡片上触发 `click`
4. 但会在 overlay 上触发 `click`（因为 `mouseup` 在 overlay 上）
5. 导致 Modal 被关闭

**尝试 1: 使用 mousedown 跟踪（失败）**

```typescript
// ❌ 问题：事件冒泡导致误判
const mouseDownOnOverlay = ref(false)

function handleOverlayMouseDown() {
  mouseDownOnOverlay.value = true // 卡片内的 mousedown 也会冒泡到这里
}

function handleCardMouseDown() {
  mouseDownOnOverlay.value = false // 试图阻止，但事件已经冒泡
}
```

这个方案失败是因为事件冒泡：即使在卡片上按下，事件也会冒泡到 overlay。

**最终方案: 使用 .self 修饰符**

```vue
<!-- ✅ 正确：使用 .self 修饰符 -->
<div
  class="modal-overlay"
  @mousedown.self="handleOverlayMouseDown"
  @click.self="handleOverlayClick"
>
  <CuteCard class="editor-card" @mousedown="handleCardMouseDown" @click.stop>
    <!-- ... -->
  </CuteCard>
</div>
```

`.self` 修饰符确保事件处理函数只在事件直接发生在该元素上时才触发，忽略冒泡的事件。

配合 `mouseDownOnOverlay` 标志：

```typescript
const mouseDownOnOverlay = ref(false)

// 只有在 overlay 自身上按下才设置标志
function handleOverlayMouseDown() {
  mouseDownOnOverlay.value = true
}

// 只有在 overlay 自身上点击，且之前在 overlay 上按下过，才关闭
function handleOverlayClick() {
  if (mouseDownOnOverlay.value) {
    emit('close')
  }
  mouseDownOnOverlay.value = false
}

// 在卡片上按下时清除标志
function handleCardMouseDown() {
  mouseDownOnOverlay.value = false
}
```

**经验教训：**

> **Modal 关闭交互的正确实现**：
>
> 1. **不要只用 `@click`**：
>    - `click` 事件只检测释放位置，不检测按下位置
>    - 容易导致"拖出误触"问题
> 2. **结合 `mousedown` 和 `click`**：
>    - 用 `mousedown` 记录按下位置
>    - 用 `click` 检测释放位置
>    - 只有两者都在 overlay 上才关闭
> 3. **`.self` 修饰符很关键**：
>    - 防止事件冒泡导致误判
>    - 确保只响应直接点击，不响应子元素的冒泡
>    - Vue 的 `.self` 比手动检查 `event.target === event.currentTarget` 更简洁
> 4. **用户体验考虑**：
>    - 滚动、选择文本、拖动等操作可能跨越卡片边界
>    - 这些操作不应该触发关闭
>    - 只有明确的"点击外部空白区域"才应该关闭
> 5. **测试场景**：
>    - 卡片内按下 → 卡片内释放 → 不关闭 ✅
>    - 卡片内按下 → 卡片外释放 → 不关闭 ✅
>    - 卡片外按下 → 卡片外释放 → 关闭 ✅
>    - 卡片外按下 → 卡片内释放 → 不关闭 ✅
> 6. **Vue 事件修饰符总结**：
>    - `.stop` - 阻止事件冒泡
>    - `.prevent` - 阻止默认行为
>    - `.self` - 只在元素自身触发（忽略冒泡）
>    - `.capture` - 使用捕获模式
>    - 组合使用：`@click.self.stop` 同时应用多个修饰符

---

## 通用原则补充

### UI 交互设计

1. **防误触机制**：按钮位置变化时要考虑用户可能的误操作
2. **自动适应**：输入框、文本域等应该根据内容自动调整，而不是固定尺寸
3. **滚动层级**：合理选择滚动容器，避免"滚动条套滚动条"
4. **关闭交互**：Modal 关闭要同时考虑按下和释放位置，防止误触

### 状态同步

1. **完整实现**：不要遗留 TODO、console.log 或空函数
2. **计算属性**：状态应该从数据计算得出，不要硬编码
3. **端到端测试**：测试完整的交互流程，不只是单个函数

### Vue 事件处理

1. **优先使用事件参数**：`@update:checked="fn"` 的参数比读取当前状态更可靠
2. **事件修饰符**：熟练使用 `.stop`、`.self`、`.prevent` 等修饰符
3. **事件冒泡**：理解冒泡机制，在需要时使用 `.self` 或 `.stop` 阻止