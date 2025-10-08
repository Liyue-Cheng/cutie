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
const response = await fetch(`http://127.0.0.1:3538/api/time-blocks/${eventIdToLink}/link-task`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ task_id: currentDraggedTask.value.id }),
})
```

**解决方案：**

```typescript
// ✅ 正确：使用动态端口
import { apiBaseUrl } from '@/composables/useApiConfig'

const response = await fetch(`${apiBaseUrl.value}/time-blocks/${eventIdToLink}/link-task`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ task_id: currentDraggedTask.value.id }),
})
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
<CuteCheckbox :checked="false" size="large" variant="star" @update:checked="handlePresenceToggle" />
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

---

## 2025-10-08: 孤儿时间块删除逻辑的致命缺陷与 SSE 事件链完整性问题

### 问题群：实时更新功能完全失效

**现象：**

1. 链接任务到时间块后，时间块不继承任务的 area（颜色不变）
2. 链接任务后，任务卡片不显示时间指示器
3. 拖拽任务到日历后，卡片只闪一下就消失
4. 所有情况都需要手动刷新才能看到正确结果

**用户报告：**

> "任务那边修好了，但是时间片还没有修好，拖动连接到时间片之后，还是要刷新"

### 根本原因分析（7层问题叠加）

#### 问题1：孤儿时间块删除逻辑的业务缺陷

**错误设计：** 基于 `time_block.title == deleted_task.title` 判断是否删除

**Bug 场景：**

```
1. 任务 A 创建时间块 K (title="任务A")
2. 链接任务 B 到时间块 K
3. 删除任务 A → K 保留（还有任务 B）✅
4. 删除任务 B → K 保留（title "任务A" ≠ "任务B"）❌
   结果：孤儿时间块！
```

**另一个问题：**

```
手动创建"会议"时间块 K
链接任务 A
删除任务 A → 如果恰好 task.title == "会议"，则会误删！❌
```

**根本缺陷：** 标题是易变的、不可靠的，不能作为业务逻辑判断依据

**解决方案：** 使用命名空间化的 `source_info.source_type`

```rust
// SourceInfo 扩展
pub struct SourceInfo {
    pub source_type: String,        // "native::from_task" | "native::manual" | "external::*"
    pub created_by_task_id: Option<Uuid>,  // 记录创建来源
}

// 创建时设置
// create_from_task: source_type = "native::from_task"
// create_time_block: source_type = "native::manual"

// 删除时判断
if source_info.source_type == "native::from_task" {
    return Ok(true);  // 孤儿 + 自动创建 = 删除
}
Ok(false)  // 其他来源（manual、external::*）一律保留
```

**经验教训：**

- ❌ 不要使用易变的业务数据（如标题）作为逻辑判断依据
- ✅ 使用明确的元数据（source_type）标记来源和意图
- ✅ 设计时考虑边界情况和多任务链接场景
- ✅ 采用命名空间化设计，便于未来扩展（external::google, external::outlook）

#### 问题2：TimeBlockStore 完全缺失 SSE 订阅

**错误：** `timeblock.ts` 没有任何事件订阅代码

```typescript
// ❌ timeblock.ts 中完全没有这些代码：
// import { getEventSubscriber } from '@/services/events'
// function initEventSubscriptions() { ... }
// subscriber.on('time_blocks.created', ...)
// subscriber.on('time_blocks.linked', ...)
```

**后果：**

- 后端发送 SSE 事件 ✅
- EventSource 接收事件 ✅
- Store 完全不知道发生了什么 ❌
- UI 永远不更新 ❌

**解决方案：** 完整实现 Store 的 SSE 订阅

```typescript
function initEventSubscriptions() {
  const subscriber = getEventSubscriber()
  if (!subscriber) return

  subscriber.on('time_blocks.created', handleTimeBlockCreatedEvent)
  subscriber.on('time_blocks.updated', handleTimeBlockUpdatedEvent)
  subscriber.on('time_blocks.deleted', handleTimeBlockDeletedEvent)
  subscriber.on('time_blocks.linked', handleTimeBlockLinkedEvent)
}

// 在 useApiConfig.ts 中初始化
timeBlockStore.initEventSubscriptions()
```

#### 问题3：create_from_task 端点无 SSE 事件

**错误：** 端点只返回 HTTP 响应，没有发送 SSE 事件

```rust
// ❌ 只有这行
Ok(CreateFromTaskResponse {
    time_block: time_block_view,
    updated_task,
})

// ❌ 缺少整个事件发布逻辑
```

**后果：**

- 拖拽操作的发起者看到结果（HTTP 响应）✅
- 其他视图（Kanban、其他日期）不知道发生了什么 ❌
- 必须手动刷新才能看到 ❌

**解决方案：** 添加 SSE 事件发布

```rust
// 13. 发送 SSE 事件
let payload = serde_json::json!({
    "time_block_id": block_id,
    "task_id": request.task_id,
    "time_block": time_block_view,
    "updated_task": updated_task,
});

let event = DomainEvent::new(
    "time_blocks.created",
    "TimeBlock",
    block_id.to_string(),
    payload,
);

outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
```

#### 问题4：EventSubscriber 未注册事件监听器

**错误：** `events.ts` 中没有 `addEventListener`

```typescript
// ❌ events.ts 中完全没有这两行：
// this.eventSource.addEventListener('time_blocks.created', ...)
// this.eventSource.addEventListener('time_blocks.linked', ...)
```

**SSE 事件链断裂：**

```
Backend → SSE Stream → EventSource ❌ 事件被丢弃
                                   ↓
                          Store handler 永远不会被调用
```

**调试过程：**

用户提供 SSE 响应：

```json
{"event_type":"time_blocks.linked","payload":{...}}
```

证明后端发送成功，但前端没有反应。检查后发现 EventSource 根本没有监听这个事件类型！

**解决方案：**

```typescript
// src/services/events.ts
this.eventSource.addEventListener('time_blocks.created', (e: MessageEvent) => {
  this.handleEvent('time_blocks.created', e.data)
})

this.eventSource.addEventListener('time_blocks.linked', (e: MessageEvent) => {
  this.handleEvent('time_blocks.linked', e.data)
})
```

#### 问题5：link_task 未更新时间块 area_id

**错误：** 只创建链接，不更新时间块属性

```rust
// ❌ 只有这行
TaskTimeBlockLinkRepository::link_in_tx(&mut tx, task_id, block_id).await?;

// ❌ 没有更新 time_block.area_id
```

**时间块颜色决定逻辑：**

```typescript
// useCalendarEvents.ts
const area = timeBlock.area_id ? areaStore.getAreaById(timeBlock.area_id) : null
if (area) {
  color = area.color // ← 颜色由 area_id 决定
}
```

**后果：**

- 时间块没有 area_id → 无法获取 area → 颜色保持默认灰色 ❌

**解决方案：** 继承任务的 area_id

```rust
// 5.5. 如果时间块没有 area，则继承任务的 area
let should_update_area = time_block.area_id.is_none() && task.area_id.is_some();
if should_update_area {
    let update_request = UpdateTimeBlockRequest {
        area_id: Some(task.area_id),
        ..Default::default()
    };
    TimeBlockRepository::update_in_tx(&mut tx, block_id, &update_request, now).await?;
}
```

#### 问题6：SSE Payload 只含 ID，无完整数据

**错误：** 事件载荷只包含 ID

```rust
// ❌ 只有 ID
let payload = serde_json::json!({
    "time_block_id": block_id,
    "linked_task_id": task_id,
    "affected_task_ids": vec![task_id],
});
```

**前端尝试获取数据：**

```typescript
// ❌ 前端试图通过 ID 查询
const response = await fetch(`http://localhost:${port}/api/time-blocks?ids=${timeBlockId}`)
```

**问题：** 后端 API 不支持 `ids` 参数！只支持 `start_date` 和 `end_date`

**解决方案：** 在 SSE payload 中包含完整数据

```rust
// ✅ 包含完整的 time_block 数据
let time_block_view = TimeBlockViewDto {
    id: updated_time_block.id,
    area_id: updated_time_block.area_id,  // ← 包含更新后的 area_id
    // ... 所有字段
};

let payload = serde_json::json!({
    "time_block_id": block_id,
    "time_block": time_block_view,  // ← 完整数据
});
```

```typescript
// ✅ 前端直接使用
function handleTimeBlockLinkedEvent(event: any) {
  const timeBlock = event.payload?.time_block
  if (timeBlock) {
    addOrUpdateTimeBlock(timeBlock) // 不需要 API 调用
  }
}
```

#### 问题7：前端调用不存在的 API

**错误：** 尝试调用 `/api/time-blocks?ids=X`

```typescript
// ❌ 这个 API 不存在
const response = await fetch(`http://localhost:${port}/api/time-blocks?ids=${timeBlockId}`)
```

**实际 API：**

```rust
// ✅ 实际只支持这个
GET /api/time-blocks?start_date=...&end_date=...
```

**根本问题：** 前后端 API 契约不一致

**解决方案：**

1. 方案 A：SSE payload 包含完整数据（已采用）✅
2. 方案 B：添加 `GET /api/time-blocks/:id` 端点
3. 方案 C：支持 `ids` 批量查询参数

### 完整的问题链和修复流程

```
业务逻辑缺陷（标题判断）
    ↓
SSE 订阅缺失（Store 层）
    ↓
SSE 事件缺失（create_from_task）
    ↓
EventSource 未注册监听器
    ↓
时间块未继承 area_id
    ↓
SSE payload 数据不完整
    ↓
API 调用失败（不存在的端点）
    ↓
所有实时更新功能失效 ❌
```

**修复后的完整流程：**

```
用户拖动任务到时间块
    ↓
Backend: 创建链接 + 更新 area_id
    ↓
Backend: 发送 time_blocks.linked SSE（包含完整数据）
    ↓
EventSource: addEventListener 接收事件
    ↓
EventSubscriber: handleEvent 解析 JSON
    ↓
TimeBlockStore: handleTimeBlockLinkedEvent
    ↓
Store: addOrUpdateTimeBlock(payload.time_block)
    ↓
Calendar: 响应式更新，颜色正确 ✅
```

### 关键经验教训

#### 1. 业务逻辑设计原则

- ❌ **不要依赖易变数据**：标题、描述等用户可编辑的字段
- ✅ **使用明确的元数据**：source_type、created_by、flags 等
- ✅ **考虑边界情况**：多对多关系、删除顺序、并发操作
- ✅ **命名空间化设计**：`native::`, `external::` 便于扩展

#### 2. SSE 事件链完整性检查清单

实现新功能时，必须检查以下**所有环节**：

**后端（Rust）：**

```
[ ] 端点发送 SSE 事件（EventOutbox）
[ ] SSE payload 包含**完整数据**，不只是 ID
[ ] 事件类型命名一致（如 time_blocks.linked）
```

**中间层（events.ts）：**

```
[ ] EventSource.addEventListener 注册了该事件类型
[ ] handleEvent 正确解析和分发
```

**前端 Store：**

```
[ ] Store 实现了 initEventSubscriptions
[ ] Store 订阅了所有相关事件
[ ] Event handler 正确处理数据
[ ] useApiConfig.ts 中调用了 initEventSubscriptions
```

**测试验证：**

```
[ ] 控制台可以看到 SSE 事件日志
[ ] Store handler 被正确调用
[ ] UI 实时更新，无需手动刷新
```

#### 3. SSE Payload 设计原则

**❌ 错误（只发 ID）：**

```json
{
  "entity_id": "uuid",
  "affected_ids": ["uuid1", "uuid2"]
}
```

**问题：**

- 前端需要额外 API 调用
- 增加延迟
- API 可能不支持
- 竞态条件（数据可能还没写入）

**✅ 正确（发完整数据）：**

```json
{
  "entity_id": "uuid",
  "entity": {
    "id": "uuid",
    "all_fields": "...",
    "computed_fields": "..."
  },
  "affected_ids": ["uuid1"],
  "side_effects": {
    "updated_entities": [...]
  }
}
```

**优势：**

- 前端直接使用，无需额外请求
- 零延迟
- 避免 API 不匹配问题
- 数据完整性保证（事务后发送）

#### 4. 调试 SSE 问题的步骤

1. **检查后端是否发送**：查看后端日志、数据库 event_outbox 表
2. **检查网络传输**：浏览器 DevTools → Network → EventStream
3. **检查 EventSource 接收**：查看 `addEventListener` 是否注册
4. **检查 Store 订阅**：`initEventSubscriptions` 是否调用
5. **检查 Handler 执行**：添加 console.log 确认被调用
6. **检查数据处理**：验证 payload 结构和内容

**本次调试关键点：**

用户提供 SSE 响应证明后端发送成功 → 快速定位到 EventSource 未注册监听器

#### 5. 跨模块状态同步策略

**问题：** 时间块属性（如 area_id）变化时，多个 Store 需要同步

**方案：**

**A. SSE 事件广播（已采用）：**

```
Backend 更新 time_block.area_id
    ↓
发送 time_blocks.updated SSE（包含完整数据）
    ↓
TimeBlockStore 接收并更新
    ↓
TaskStore 监听同一事件（如果需要）
```

**优势：**

- 解耦
- 实时
- 可靠

**B. Store 间直接调用：**

```typescript
// ❌ 不推荐
timeBlockStore.updateTimeBlock(...)
taskStore.updateRelatedTasks(...)  // 紧耦合
```

**C. Pinia subscriptions：**

```typescript
// ❌ 复杂且难以追踪
watch(() => timeBlockStore.timeBlocks, ...)
```

#### 6. 数据继承和传播规则

**场景：** 链接两个实体时，哪些属性应该继承？

**设计原则：**

1. **优先级判断：**

   ```rust
   // ✅ 只在目标没有时才继承
   if target.area_id.is_none() && source.area_id.is_some() {
       target.area_id = source.area_id;
   }
   ```

2. **单向继承：**

   ```
   Task → TimeBlock  ✅ 任务的属性传递给时间块
   TimeBlock → Task  ❌ 时间块不影响任务属性
   ```

3. **显式记录：**
   ```rust
   tracing::info!(
       "Updated time block {} area_id to {:?} (inherited from task)",
       block_id, task.area_id
   );
   ```

### 防范措施和检查清单

#### 新增 SSE 事件时必须：

```
Backend:
[ ] 创建端点时同时添加 SSE 发布代码
[ ] Payload 包含完整数据，不仅仅是 ID
[ ] 在 API_SPEC.md 中记录事件类型和 payload 结构

Frontend (events.ts):
[ ] 添加 addEventListener
[ ] 测试事件能否被接收

Frontend (Store):
[ ] 实现 handler function
[ ] 在 initEventSubscriptions 中订阅
[ ] 添加 console.log 用于调试
[ ] 在 useApiConfig.ts 中初始化

测试:
[ ] 手动触发操作
[ ] 检查控制台是否有事件日志
[ ] 验证 UI 是否实时更新
[ ] 打开多个浏览器标签，验证跨标签同步
```

#### 删除/更新逻辑实现时必须：

```
[ ] 考虑多对多关系（不止一个关联实体）
[ ] 考虑删除顺序（A删除后B还存在的情况）
[ ] 使用稳定的元数据做判断（source_type, flags）
[ ] 避免使用易变数据（title, description）
[ ] 编写边界情况测试用例
[ ] 记录业务规则到 CABC 文档
```

### 总结

这次 bug 修复涉及**7层问题叠加**，从业务逻辑设计到 SSE 事件链的每一个环节都有问题。这反映了：

1. **系统复杂性管理的重要性**：实时更新功能涉及多个层次，任何一环出错都会导致整体失效
2. **完整性检查的必要性**：新增功能时必须检查完整的数据流路径
3. **调试技巧的价值**：用户提供的 SSE 响应帮助快速定位问题
4. **设计原则的重要性**：使用稳定的元数据、包含完整数据的 payload、单一职责的事件

**核心教训：**

> SSE 实时更新功能像一条完整的链条，从后端发送 → 网络传输 → EventSource 接收 → Store 处理 → UI 更新，任何一环断裂都会导致功能失效。新增功能时必须验证整条链路的完整性。

> 业务逻辑判断应该基于稳定的、明确的元数据（如 source_type），而不是易变的业务数据（如 title）。

**修复成果：**

✅ 所有实时更新功能正常工作
✅ 时间块正确继承任务属性
✅ 无需手动刷新
✅ 跨标签同步正常
✅ 孤儿时间块正确清理

---

## 2025-10-08: 回收站功能实现 - deleted_at 字段与 SSE 事件数据一致性

### 问题：回收站显示"删除于未知时间"

**现象：**

- 删除任务后，任务进入回收站 ✅
- 但显示"删除于未知时间" ❌
- 前端收到的 `deleted_at` 字段为 `null`

**根本原因：**

在 `delete_task.rs` 端点中，数据组装的时机错误：

```rust
// ❌ 错误：在软删除之前组装
let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id).await?;
let task_card = TaskAssembler::task_to_card_basic(&task);  // ← task.deleted_at 还是 None

let now = app_state.clock().now_utc();
TaskRepository::soft_delete_in_tx(&mut tx, task_id, now).await?;  // ← 数据库更新了

// SSE 事件
let payload = serde_json::json!({
    "task": task_card,  // ← deleted_at = None ❌
    "deleted_at": now.to_rfc3339(),  // ← 重复字段，但 task_card 内部还是 None
});
```

**问题分析：**

1. **时序问题**：先组装 DTO，后更新数据库
2. **数据不一致**：
   - `task_card.deleted_at = None`（从旧的 task 实体组装）
   - `payload.deleted_at = now`（手动添加的字段）
3. **前端解析**：前端读取 `event.payload.task.deleted_at`，得到 `null`

**解决方案：**

在软删除**之后**组装 DTO，或手动设置字段：

```rust
// ✅ 正确：在软删除之后组装
let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id).await?;

let now = app_state.clock().now_utc();
TaskRepository::soft_delete_in_tx(&mut tx, task_id, now).await?;

// 组装 task_card 并手动设置 deleted_at
let mut task_card = TaskAssembler::task_to_card_basic(&task);
task_card.deleted_at = Some(now);  // ← 手动设置
task_card.is_deleted = true;

// SSE 事件
let payload = serde_json::json!({
    "task": task_card,  // ← deleted_at = Some(now) ✅
    "deleted_at": now.to_rfc3339(),
});
```

**经验教训：**

> **SSE 事件数据必须反映数据库的最终状态**：
>
> 1. **数据组装时机**：
>    - ❌ 在数据库更新之前组装 → 数据不一致
>    - ✅ 在数据库更新之后组装 → 数据一致
>    - ✅ 或者手动设置变更的字段
> 2. **状态字段的特殊性**：
>    - `deleted_at`、`completed_at`、`archived_at` 等状态字段
>    - 在状态转换时才被设置
>    - 不能从"转换前"的实体中读取
> 3. **HTTP 响应与 SSE 一致性**：
>    - HTTP 响应和 SSE payload 必须包含相同的数据
>    - 不要在 payload 中添加"额外的"顶层字段（如单独的 `deleted_at`）
>    - 所有数据应该在 DTO 对象内部
> 4. **验证方法**：
>    - 检查 SSE payload 的 JSON 结构
>    - 确认前端读取的字段路径正确
>    - 使用 console.log 打印实际接收的数据
> 5. **类似场景**：
>    - `complete_task` → 设置 `completed_at`
>    - `archive_task` → 设置 `archived_at`
>    - `restore_task` → 清除 `deleted_at`
>    - 所有这些端点都要注意数据组装时机

**防范措施：**

在实现状态转换端点时，遵循以下模式：

```rust
// 1. 查询原始数据
let entity = Repository::find_by_id_in_tx(&mut tx, id).await?;

// 2. 执行状态转换
let now = app_state.clock().now_utc();
Repository::update_state_in_tx(&mut tx, id, now).await?;

// 3. 组装 DTO（在状态转换之后）
let mut dto = Assembler::entity_to_dto(&entity);
dto.state_field = Some(now);  // ← 手动设置新状态
dto.is_state = true;

// 4. 发送 SSE 事件（使用更新后的 DTO）
let payload = serde_json::json!({ "entity": dto });
```

**相关代码：**

- `src-tauri/src/features/tasks/endpoints/delete_task.rs` - 修复示例
- `src-tauri/src/features/tasks/endpoints/restore_task.rs` - 恢复逻辑
- `src-tauri/src/features/tasks/endpoints/complete_task.rs` - 类似场景
- `src-tauri/src/features/tasks/shared/assembler.rs` - DTO 组装

**架构原则：**

- **SSE First**：SSE 事件的数据质量直接影响实时更新体验
- **数据完整性**：状态转换后的数据必须完整且一致
- **时序正确性**：先改数据，后组装 DTO，再发事件

---

## 2025-10-08: 前后端响应格式不一致导致 undefined 数据

### 问题：回收站操作返回 undefined

**现象：**

1. 清空回收站后显示"删除了 undefined 个任务"
2. 恢复任务可能失败
3. 前端无法正确读取后端返回的数据

**用户报告的实际响应：**

```json
{
  "data": {
    "deleted_count": 0
  },
  "timestamp": "2025-10-08T02:26:38.187841100Z",
  "request_id": null
}
```

**根本原因：**

后端使用 `success_response()` 包装器，它会将所有响应包装在 `data` 字段中，但前端代码没有正确解包：

```typescript
// ❌ 错误：直接解析顶层
const data: { deleted_count: number } = await response.json()
return data.deleted_count // undefined！因为实际结构是 result.data.deleted_count
```

**问题分析：**

1. **后端包装器**：`success_response()` 自动添加 `data` 包装层
2. **前端假设**：前端代码假设响应直接是业务数据
3. **类型不匹配**：TypeScript 类型定义与实际结构不符
4. **系统性问题**：所有使用 `success_response()` 的端点都有这个问题

**影响范围：**

回收站功能的所有 API 调用：

- `GET /api/trash` - 获取回收站列表
- `POST /api/trash/empty` - 清空回收站
- `PATCH /api/tasks/:id/restore` - 恢复任务
- `DELETE /api/tasks/:id/permanently` - 彻底删除任务

**解决方案：**

修正所有前端响应解析代码，正确解包 `data` 字段：

#### 1. 清空回收站

```typescript
// ❌ 错误
const data: { deleted_count: number } = await response.json()
return data.deleted_count

// ✅ 正确
const result: { data: { deleted_count: number } } = await response.json()
return result.data.deleted_count
```

#### 2. 获取回收站列表

```typescript
// ❌ 错误
const data: { tasks: TaskCard[]; total: number } = await response.json()
setTrashedTasks(data.tasks)

// ✅ 正确
const result: { data: { tasks: TaskCard[]; total: number } } = await response.json()
setTrashedTasks(result.data.tasks)
```

#### 3. 恢复任务

```typescript
// ❌ 错误
const task: TaskCard = await response.json()
return task

// ✅ 正确
const result: { data: TaskCard } = await response.json()
return result.data
```

**经验教训：**

> **前后端响应格式必须明确约定并严格遵守**：
>
> 1. **后端包装器的影响**：
>    - `success_response()` → 返回 `{ data: T, timestamp, request_id }`
>    - `created_response()` → 返回 `{ data: T, timestamp, request_id }`
>    - 直接返回 JSON → 返回 `T`
> 2. **前端解析规则**：
>    - 检查后端使用的响应包装器
>    - 使用正确的类型定义：`{ data: T }` 而不是 `T`
>    - 访问 `result.data` 而不是 `result`
> 3. **类型安全的局限性**：
>    - TypeScript 类型定义可以骗过编译器
>    - 错误的类型定义 + 错误的访问路径 = 运行时 `undefined`
>    - 必须通过实际测试验证数据流
> 4. **调试方法**：
>    - 检查实际的 HTTP 响应（DevTools Network）
>    - 打印 `response.json()` 的完整结果
>    - 不要猜测数据结构，要验证
> 5. **系统性排查**：
>    - 发现一个问题后，检查所有类似的代码
>    - 使用 grep 搜索相同的模式
>    - 批量修复，避免遗漏

**防范措施：**

#### 1. 建立统一的响应解析工具

```typescript
// src/utils/api.ts
export async function parseSuccessResponse<T>(response: Response): Promise<T> {
  const result: { data: T } = await response.json()
  return result.data
}

// 使用
const deletedCount = await parseSuccessResponse<{ deleted_count: number }>(response)
return deletedCount.deleted_count
```

#### 2. 后端响应格式文档

在 API 文档中明确标注响应格式：

````rust
/// **响应格式：**
/// ```json
/// {
///   "data": {
///     "deleted_count": 0
///   },
///   "timestamp": "2025-10-08T02:26:38Z",
///   "request_id": null
/// }
/// ```
````

#### 3. 前端类型定义

创建标准的响应类型：

```typescript
// src/types/api.ts
export interface SuccessResponse<T> {
  data: T
  timestamp: string
  request_id: string | null
}

// 使用
const result: SuccessResponse<{ deleted_count: number }> = await response.json()
return result.data.deleted_count
```

#### 4. 检查清单

新增 API 调用时必须检查：

```
[ ] 确认后端使用的响应包装器（success_response/created_response/直接返回）
[ ] 使用正确的类型定义（SuccessResponse<T> vs T）
[ ] 正确访问数据（result.data vs result）
[ ] 实际测试验证数据能正确读取
[ ] 检查是否有其他类似的 API 调用需要修复
```

**相关代码：**

- `src/stores/trash/view-operations.ts` - 修复示例
- `src/stores/trash/crud-operations.ts` - 修复示例
- `src-tauri/src/shared/http/error_handler.rs` - 响应包装器定义
- `src-tauri/src/features/trash/endpoints/*.rs` - 后端端点

**架构建议：**

1. **统一响应格式**：所有端点使用相同的包装器
2. **前端工具函数**：封装响应解析逻辑
3. **类型安全**：使用泛型类型确保编译时检查
4. **文档先行**：API 文档中明确标注响应格式

**核心教训：**

> 不要用 workaround（如 `?? 0`）掩盖问题！`undefined` 的出现一定有根本原因，必须追查到底并从源头修复。类型定义可以骗过编译器，但骗不过运行时。

---

## 2025-10-08: JavaScript Falsy 值陷阱导致功能失效

### 问题：清空回收站功能完全不生效

**现象：**

1. 点击"清空回收站"按钮
2. 提示"已清空回收站，删除了 0 个任务"
3. 但回收站中的任务仍然存在
4. 后端返回 `deleted_count: 0`

**调试过程：**

1. 检查后端逻辑 → 发现时间过滤逻辑正确
2. 添加日志 → 发现后端收到的 `older_than_days = 30` 而不是 `0`
3. 检查前端代码 → 发现使用了 `||` 运算符

**根本原因：**

前端使用了 `||` 运算符设置默认值，但 `0` 是 JavaScript 的 falsy 值：

```typescript
// ❌ 错误：0 是 falsy 值
older_than_days: options?.olderThanDays || 30
// 当 olderThanDays = 0 时：0 || 30 → 30 ❌

// ✅ 正确：使用 ?? 运算符
older_than_days: options?.olderThanDays ?? 30
// 当 olderThanDays = 0 时：0 ?? 30 → 0 ✅
```

**问题分析：**

1. **JavaScript Falsy 值**：
   - `||` 运算符：左侧为 falsy（`0`, `""`, `false`, `null`, `undefined`, `NaN`）时返回右侧
   - `??` 运算符（空值合并）：左侧为 `null` 或 `undefined` 时才返回右侧

2. **数据流错误**：

   ```
   前端调用：emptyTrash({ olderThanDays: 0 })
       ↓
   前端发送：{ older_than_days: 0 || 30 } = { older_than_days: 30 }
       ↓
   后端接收：older_than_days = 30
       ↓
   后端逻辑：cutoff_time = now - 30天
       ↓
   结果：只删除 30 天前的任务（没有任务符合条件）
       ↓
   返回：deleted_count = 0
   ```

3. **业务语义丢失**：
   - 用户意图：`0` 表示"删除所有任务，不限制天数"
   - 实际效果：`0` 被转换为 `30`，变成"删除 30 天前的任务"
   - 完全违背了用户意图

**解决方案：**

使用 `??` 运算符替代 `||` 运算符：

```typescript
// src/stores/trash/view-operations.ts

export async function emptyTrash(options?: {
  olderThanDays?: number
  limit?: number
}): Promise<number> {
  const response = await fetch(`${apiBaseUrl.value}/trash/empty`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      older_than_days: options?.olderThanDays ?? 30, // ✅ 使用 ??
      limit: options?.limit ?? 100, // ✅ 使用 ??
    }),
  })
  // ...
}
```

**行为对比：**

| 输入值      | `\|\|` 运算符 | `??` 运算符 | 正确性    |
| ----------- | ------------- | ----------- | --------- |
| `0`         | `30` ❌       | `0` ✅      | `??` 正确 |
| `undefined` | `30` ✅       | `30` ✅     | 都正确    |
| `null`      | `30` ✅       | `30` ✅     | 都正确    |
| `""`        | `30` ❌       | `""` ✅     | `??` 正确 |
| `false`     | `30` ❌       | `false` ✅  | `??` 正确 |

**经验教训：**

> **设置默认值时，必须区分"无值"和"有效的零值"**：
>
> 1. **运算符选择**：
>    - `||` 运算符：用于布尔逻辑，不适合设置默认值
>    - `??` 运算符：专门用于空值合并，只处理 `null` 和 `undefined`
>    - **规则**：设置默认值时，永远使用 `??` 而不是 `||`
> 2. **零值的业务意义**：
>    - `0` 可能是有效的业务值（如"删除所有"、"无限制"、"立即执行"）
>    - `""` 可能是有效的业务值（如"清空字段"、"无标题"）
>    - `false` 可能是有效的业务值（如"禁用"、"关闭"）
>    - 这些值不应该被当作"缺失"而使用默认值
> 3. **类型安全的局限**：
>    - TypeScript 无法检测 `||` 和 `??` 的语义差异
>    - 两者都能通过类型检查
>    - 只有运行时才能发现问题
> 4. **调试技巧**：
>    - 添加日志打印实际发送的请求体
>    - 对比前端发送和后端接收的值
>    - 检查所有使用 `||` 设置默认值的地方
> 5. **系统性排查**：
>    - 搜索代码中所有 `|| 数字` 的模式
>    - 检查是否应该改为 `?? 数字`
>    - 特别注意参数、配置、选项等场景

**防范措施：**

#### 1. ESLint 规则

配置 ESLint 规则，警告可疑的 `||` 用法：

```json
{
  "rules": {
    "prefer-nullish-coalescing": [
      "warn",
      {
        "ignoreTernaryTests": false,
        "ignoreConditionalTests": false
      }
    ]
  }
}
```

#### 2. 代码审查清单

设置默认值时必须检查：

```
[ ] 是否使用了 || 运算符？
[ ] 左侧的值是否可能为 0、""、false？
[ ] 这些值是否有业务意义？
[ ] 是否应该改为 ?? 运算符？
```

#### 3. 明确的类型定义

使用类型定义明确"可选"和"可为零"的区别：

```typescript
// ✅ 明确：undefined 表示未提供，0 是有效值
interface Options {
  olderThanDays?: number // undefined = 使用默认值，0 = 删除所有
  limit?: number // undefined = 使用默认值，0 = 无限制
}

// 使用时
const value = options?.olderThanDays ?? 30 // 只有 undefined 才用 30
```

#### 4. 单元测试覆盖边界值

```typescript
describe('emptyTrash', () => {
  it('should delete all tasks when olderThanDays is 0', async () => {
    const result = await emptyTrash({ olderThanDays: 0 })
    expect(mockFetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        body: JSON.stringify({ older_than_days: 0, limit: 100 }),
      })
    )
  })

  it('should use default value when olderThanDays is undefined', async () => {
    const result = await emptyTrash({})
    expect(mockFetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        body: JSON.stringify({ older_than_days: 30, limit: 100 }),
      })
    )
  })
})
```

**相关代码：**

- `src/stores/trash/view-operations.ts` - 修复示例
- `src/views/TrashView.vue` - 调用处

**常见的 Falsy 陷阱场景：**

1. **数字默认值**：

   ```typescript
   // ❌ 错误
   const count = input || 10 // input=0 时返回 10
   // ✅ 正确
   const count = input ?? 10 // input=0 时返回 0
   ```

2. **字符串默认值**：

   ```typescript
   // ❌ 错误
   const name = input || 'default' // input="" 时返回 'default'
   // ✅ 正确
   const name = input ?? 'default' // input="" 时返回 ""
   ```

3. **布尔默认值**：
   ```typescript
   // ❌ 错误
   const enabled = input || true // input=false 时返回 true
   // ✅ 正确
   const enabled = input ?? true // input=false 时返回 false
   ```

**核心教训：**

> **`||` 是逻辑运算符，不是默认值运算符！** 在 JavaScript/TypeScript 中设置默认值时，永远使用 `??` 而不是 `||`。`0`、`""`、`false` 都是有效的业务值，不应该被当作"缺失"。

**架构建议：**

1. **团队规范**：在代码规范中明确禁止使用 `||` 设置默认值
2. **自动化检查**：配置 ESLint 规则自动检测
3. **代码审查**：重点审查参数处理、配置读取等场景
4. **文档说明**：在 API 文档中明确说明零值的业务含义
