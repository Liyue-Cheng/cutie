# 循环任务功能重构报告

## 📅 重构时间

2025-11-10

## 🎯 重构目标

移除 `is_active` 字段的暂停/激活功能，简化架构设计，避免回溯生成问题。

---

## ❌ 移除的功能：暂停/激活循环规则

### 原设计的问题

**业务场景问题**：

```
用户场景：11月10-15日度假，暂停"每日站会"规则

操作：11月10日点击"暂停" → is_active = false
结果：11月11-15日不生成实例 ✅

操作：11月16日点击"激活" → is_active = true
结果：查询11月11-15日时，系统检测到：
  - is_active = true ✅
  - RRULE 匹配这些日期 ✅
  - 没有 task_recurrence_links 记录 ✅
  → 自动生成度假期间的6个站会任务！❌

问题：回溯生成，违背用户预期
```

**架构问题**：

- 前端缓存策略不完整（只缓存 is_active=true 的规则）
- 验证逻辑错误（依赖本地缓存而非后端数据）
- API 语义混乱（GET /recurrences 隐式过滤）

---

## ✅ 新的设计

### `is_active` 字段重新定义

```rust
/// is_active: 循环规则的软删除标记
/// - true: 规则正常使用中
/// - false: 规则已被删除（类似任务的 deleted_at）
pub struct TaskRecurrence {
    pub is_active: bool,
    // ...
}
```

### 三种场景的解决方案

| 需求         | 解决方案                | 说明                                            |
| ------------ | ----------------------- | ----------------------------------------------- |
| **跳过几天** | 循环例外                | 手动删除特定日期的实例，该日期永远不再生成      |
| **永久停止** | 设置 end_date           | 从某天之后永久停止生成新实例                    |
| **删除规则** | DELETE /recurrences/:id | 软删除规则（is_active=false），清理所有相关数据 |

---

## 🔧 前端修改清单

### 1. ✅ RecurrenceRuleCard.vue

- 移除"暂停/激活"按钮
- 移除 `status-badge` 显示
- 移除 `.inactive` 样式
- 移除 `toggle-active` 事件

### 2. ✅ RecurrenceEditDialog.vue

- 移除 `isActive` 字段
- 移除"激活此循环规则" checkbox
- 更新 payload 不包含 `is_active`

### 3. ✅ RecurrenceBoard.vue

- 移除 `handleToggleActive` 函数
- 移除 `@toggle-active` 事件绑定

### 4. ✅ KanbanTaskEditorModal.vue

- 移除循环规则的激活状态显示
- 移除"暂停/激活"按钮
- 移除 `handleToggleRecurrenceActive` 函数

### 5. ✅ recurrence-isa.ts

- 简化 validate 逻辑：只验证参数，不验证数据存在性
- `recurrence.update`: 移除本地缓存检查
- `recurrence.delete`: 移除本地缓存检查
- `recurrence.update_template_and_instances`: 移除本地缓存检查

---

## 🔒 后端保持不变

### `is_active` 仍然用于：

1. **实例化过滤**：

   ```rust
   // find_effective_for_date()
   WHERE is_active = 1  // ✅ 只实例化未删除的规则
   ```

2. **列表查询过滤**：

   ```rust
   // find_all_active()
   WHERE is_active = 1  // ✅ 只返回未删除的规则
   ```

3. **软删除标记**：
   ```rust
   // deactivate_in_tx()
   UPDATE task_recurrences SET is_active = 0  // ✅ 标记为已删除
   ```

---

## 📊 重构效果

### 修复前

- ❌ 暂停规则后重新激活会回溯生成
- ❌ 前端缓存不完整导致操作失败
- ❌ 验证逻辑依赖本地缓存
- ❌ UI 复杂（暂停/激活/停止/删除）

### 修复后

- ✅ 无回溯生成问题（移除暂停/激活功能）
- ✅ 验证逻辑简单（只验证参数，让后端验证数据）
- ✅ 无需维护完整缓存
- ✅ UI 简洁（停止/删除/编辑）
- ✅ 循环例外功能清晰（删除实例=排除日期）

---

## 🎓 设计教训

### 1. 单一数据源原则

**错误**：前端验证依赖本地缓存

```typescript
validate: async (payload) => {
  const recurrence = recurrenceStore.getRecurrenceById(payload.id)
  if (!recurrence) return false // ❌ 缓存可能不完整
}
```

**正确**：让后端验证

```typescript
validate: async (payload) => {
  if (!payload.id) return false // ✅ 只验证参数
  // 后端会返回 404 如果规则不存在
}
```

### 2. 避免回溯生成

**错误**：暂停后激活会回溯生成历史日期的实例

**正确**：

- 永久停止用 `end_date`
- 临时跳过用"循环例外"（删除实例）

### 3. 清晰的业务语义

**模糊**：`is_active` 既表示"激活状态"又表示"软删除"

**清晰**：`is_active` 只表示"软删除标记"

- `true` = 规则存在
- `false` = 规则已删除

---

## 📝 更新的文档

- ✅ `docs/RECURRENCE_FEATURE_GUIDE.md`：更新用户行为处理章节
- ✅ `RECURRENCE_DATA_CONSISTENCY_BUGS.md`：标注 Bug #1 为设计特性

---

## ✨ 重构成果

- **代码行数减少**: ~50行
- **复杂度降低**: 移除4个函数、3个UI元素、多处验证逻辑
- **Bug 修复**: Bug #2 和 Bug #3 彻底解决
- **架构优化**: 验证逻辑简化，缓存策略清晰
- **业务清晰**: 循环例外功能明确

生成时间: 2025-11-10
重构人员: Claude (AI Assistant)



