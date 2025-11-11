# 循环任务功能使用指南

## 功能概述

循环任务功能允许用户创建自动重复的任务，系统会根据 RRULE 标准规则在每天的看板中自动生成任务实例。

### 关键特性

- ✅ **标准 RRULE 支持**：使用国际标准 iCalendar RRULE 格式
- ✅ **智能实例化**：只在查询某一天时才创建该天的任务实例
- ✅ **用户可调整**：已生成的任务可以被用户自由修改、删除或移动
- ✅ **链接追踪**：系统追踪每个循环规则在每一天生成的任务实例

---

## 数据库结构

### 核心表

1. **`templates`** - 模板表
   - 新增 `category` 字段：`'GENERAL'` | `'RECURRENCE'`
   - 循环任务必须关联到 `category='RECURRENCE'` 的模板

2. **`task_recurrences`** - 循环规则表
   - `rule`: RRULE 标准字符串（如 `"FREQ=DAILY"`, `"FREQ=WEEKLY;BYDAY=MO,WE,FR"`）
   - `template_id`: 关联的模板
   - `time_type`: `'FLOATING'` | `'FIXED'`（暂时只用 FLOATING）
   - `start_date` / `end_date`: 生效时间范围（可选）
   - `is_active`: 软删除标记（`true`=正常使用，`false`=已删除）

3. **`task_recurrence_links`** - 实例链接表
   - 记录每个循环规则在某一天生成的任务ID
   - 联合主键：`(recurrence_id, instance_date)`
   - 唯一约束：`task_id`（一个任务只能是一个循环实例）

4. **`tasks`** - 任务表
   - 新增 `recurrence_id`: 关联到循环规则
   - 新增 `recurrence_original_date`: 原始日期（YYYY-MM-DD）

---

## 后端实现

### 核心服务

**RecurrenceInstantiationService** - 循环实例化服务

```rust
pub async fn instantiate_for_date(
    pool: &SqlitePool,
    id_generator: &dyn IdGenerator,
    clock: &dyn Clock,
    target_date: &NaiveDate,
) -> AppResult<Vec<Uuid>>
```

**工作流程**：

1. 查询在该日期生效的所有循环规则
2. 使用 `rrule` crate 解析规则并判断是否匹配该日期
3. 检查链接表是否已有实例：
   - 有 → 验证任务是否仍属于该日期
   - 无 → 创建新任务实例并记录链接

### API 端点

- `POST /api/recurrences` - 创建循环规则
- `GET /api/recurrences` - 查询所有激活的规则
- `GET /api/recurrences?template_id=xxx` - 查询某个模板的规则
- `PATCH /api/recurrences/:id` - 更新规则
- `DELETE /api/recurrences/:id` - 删除规则（标记为不激活）

### 集成点

**get_daily_tasks** 端点已集成循环任务实例化：

```rust
// 在返回任务列表前，自动实例化该天的循环任务
RecurrenceInstantiationService::instantiate_for_date(
    pool, id_generator, clock, &target_date
).await?;
```

---

## 前端实现

### Store

`useRecurrenceStore()` 提供：

```typescript
// State
recurrences: Map<string, TaskRecurrence>
allRecurrences: TaskRecurrence[]
activeRecurrences: TaskRecurrence[]

// Actions
createRecurrence(payload): Promise<TaskRecurrence>
updateRecurrence(id, payload): Promise<TaskRecurrence>
deleteRecurrence(id): Promise<void>
fetchAllRecurrences(): Promise<void>
```

### UI 组件

1. **RecurrenceConfigDialog.vue** - 循环配置对话框
   - 提供友好的 UI 选择循环规则
   - 支持：每天、工作日、每周、每月、每年
   - 自动将用户选择转换为标准 RRULE 字符串

2. **RecurrenceRuleCard.vue** - 循环规则卡片
   - 显示规则的人类可读描述
   - 支持暂停/激活和删除操作

3. **RecurrenceBoard.vue** - 循环看板
   - 显示所有循环规则
   - 类似模板看板的实现方式

### 使用方式

#### 方式1：从任务设置循环

```vue
<!-- 在任务卡片菜单中 -->
<button @click="showRecurrenceDialog = true">
  🔄 设置为循环
</button>

<RecurrenceConfigDialog
  v-if="showRecurrenceDialog"
  :task="task"
  :open="showRecurrenceDialog"
  @close="showRecurrenceDialog = false"
  @success="handleSuccess"
/>
```

#### 方式2：查看循环看板

在右边栏添加"循环"按钮，点击显示 `RecurrenceBoard.vue`。

---

## RRULE 示例

### 常见规则

```
FREQ=DAILY
→ 每天

FREQ=WEEKLY;BYDAY=MO,WE,FR
→ 每周一、三、五

FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR
→ 工作日

FREQ=MONTHLY;BYMONTHDAY=1
→ 每月1号

FREQ=YEARLY;BYMONTH=1;BYMONTHDAY=1
→ 每年1月1日
```

### 前端生成 RRULE

```typescript
import { RRule } from 'rrule'

// 每周一、三、五
const rule = new RRule({
  freq: RRule.WEEKLY,
  byweekday: [RRule.MO, RRule.WE, RRule.FR],
})
console.log(rule.toString())
// → "RRULE:FREQ=WEEKLY;BYDAY=MO,WE,FR"
```

### 前端解析 RRULE

```typescript
import { RRule } from 'rrule'

const rule = RRule.fromString('FREQ=WEEKLY;BYDAY=MO,WE,FR')
console.log(rule.toText())
// → "每周一、周三、周五"
```

---

## 用户行为处理

### 1. 删除循环规则

- 软删除规则（`is_active = false`）
- 清除所有任务的循环字段（已完成的任务也清除）
- 删除未来的未完成任务实例
- 已完成的任务保留（作为历史记录）

### 2. 修改已生成的任务

- 用户可以自由修改任务的标题、日期、状态等
- 如果用户将任务移到其他日期或删除，下次查询该日期时系统会识别并不再返回该任务

### 3. 循环例外（跳过特定日期）

- 用户手动删除某天的循环任务实例
- 该任务的 `task_recurrence_links` 记录被删除
- 下次查询该日期时，系统识别到"用户已排除"，不再生成
- 其他日期的实例不受影响

---

## 注意事项

### 后端

1. **必须使用 `rrule` crate**：禁止自定义解析器
2. **事务安全**：实例化过程在事务中进行
3. **幂等性**：重复查询同一天不会重复创建实例

### 前端

1. **必须使用 `rrule.js` 库**：禁止自定义规则格式
2. **提取响应数据**：记得从 `responseData.data` 提取数据
3. **人类可读**：使用 `RRule.toText()` 显示规则描述

---

## 测试示例

### 创建每日循环任务

```bash
# 1. 创建循环模板
POST /api/templates
{
  "title": "每日站会",
  "category": "RECURRENCE",
  "estimated_duration_template": 15
}

# 2. 创建循环规则
POST /api/recurrences
{
  "template_id": "xxx",
  "rule": "FREQ=DAILY",
  "time_type": "FLOATING",
  "is_active": true
}

# 3. 查询某一天的任务（会自动生成实例）
GET /api/views/daily/2025-10-10
```

---

## 未来扩展

- [ ] 支持时区（FIXED 类型）
- [ ] 支持 RRULE 的 COUNT 和 UNTIL 参数
- [ ] 支持循环任务的批量操作
- [ ] 支持排除特定日期（EXDATE）
- [ ] 支持编辑循环规则

---

## 相关文件

### 后端

- `src-tauri/src/entities/task_recurrence/` - 实体定义
- `src-tauri/src/entities/recurrence_link/` - 链接实体
- `src-tauri/src/features/recurrences/` - 循环功能模块
- `src-tauri/src/features/recurrences/shared/recurrence_instantiation_service.rs` - 核心服务
- `src-tauri/Cargo.toml` - 依赖 `rrule = "0.14"`

### 前端

- `src/types/dtos.ts` - 类型定义
- `src/stores/recurrence/` - Store
- `src/components/parts/recurrence/` - UI 组件
- `package.json` - 依赖 `rrule = "2.8.1"`
