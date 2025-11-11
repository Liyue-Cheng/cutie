# 循环任务数据不一致问题 - 后端审计报告

## 🚨 严重Bug清单

### ~~Bug #1: 删除任务时未清理 task_recurrence_links~~ ✅ 这是设计特性（循环例外）

**说明**: 删除任务时**保留** `task_recurrence_links` 记录是设计功能，称为"循环例外"：

- 用户手动删除某天的循环任务实例
- `task_recurrence_links` 保留，标记"用户已排除此日期"
- 该日期永远不会再生成新实例
- 其他日期的实例不受影响

**使用场景**: 用户需要跳过某几天的循环任务（如度假、出差）

---

### Bug #2: 更新循环规则时只软删除任务，未清除循环字段

**位置**:

- `src-tauri/src/features/endpoints/recurrences/update_recurrence.rs:303-317`
- `src-tauri/src/features/endpoints/recurrences/update_recurrence.rs:442-443`

**问题**:

```rust
// cleanup_future_instances()
for task_id_str in task_ids {
    TaskRepository::soft_delete_in_tx(tx, task_id, chrono::Utc::now()).await?;
    // ❌ 缺失：没有清除 recurrence_id 和 recurrence_original_date
}

// cleanup_mismatched_instances()
TaskRepository::soft_delete_in_tx(tx, task_id, chrono::Utc::now()).await?;
// ❌ 缺失：没有清除循环字段
```

**对比**: `delete_recurrence.rs` 中正确地做了清除：

```rust
// ✅ 正确做法
UPDATE tasks
SET recurrence_id = NULL,
    recurrence_original_date = NULL,
    updated_at = ?
WHERE id = ?
```

**后果**:

1. 修改循环规则后，被删除的任务实例仍保留 recurrence_id
2. 任务有 recurrence_id，但循环规则可能已失效或被删除
3. 前端右键菜单显示为循环任务，但编辑框找不到循环规则

**影响范围**:

- 修改循环规则的结束日期（`PATCH /recurrences/:id` with `end_date`）
- 修改循环规则本身（`PATCH /recurrences/:id` with `rule`）

---

### Bug #3: 删除循环规则时遗留已完成任务的数据

**位置**: `src-tauri/src/features/endpoints/recurrences/delete_recurrence.rs:104-112`

**问题**:

```rust
SELECT trl.task_id, trl.instance_date
FROM task_recurrence_links trl
JOIN tasks t ON t.id = trl.task_id
WHERE trl.recurrence_id = ?
  AND trl.instance_date >= ?
  AND t.completed_at IS NULL  // ❌ 只处理未完成任务
  AND t.deleted_at IS NULL
```

然后在代码末尾：

```rust
// 3. 删除所有链接记录（包括已完成的）
DELETE FROM task_recurrence_links
WHERE recurrence_id = ?  // ✅ 这里删除了所有链接
```

**后果**:

1. **未完成的任务**: 清除循环字段 ✅，软删除 ✅，删除链接 ✅
2. **已完成的任务**: ❌ 保留循环字段，❌ 不删除，✅ 删除链接

导致：

- 已完成的任务保留 `recurrence_id` 和 `recurrence_original_date`
- 但 `task_recurrence_links` 中找不到对应记录
- 循环规则已 `is_active = false`，`find_all_active()` 找不到

**影响范围**: 删除循环规则时的已完成任务

---

## 📊 数据不一致的产生路径

### 场景 1: 用户手动删除循环任务

```
用户操作: DELETE /tasks/:id
    ↓
后端处理:
  ✅ 软删除任务 (deleted_at = now)
  ✅ 删除 task_schedules
  ✅ 删除 task_time_block_links
  ❌ 未删除 task_recurrence_links  <-- Bug #1
    ↓
结果:
  - task_recurrence_links 仍指向已删除的任务
  - 下次实例化时，系统认为该日期"已有实例"
  - 该日期永远不会再生成新实例
```

### 场景 2: 修改循环规则（设置结束日期）

```
用户操作: PATCH /recurrences/:id { end_date: "2025-11-10" }
    ↓
后端处理 cleanup_future_instances():
  1. 查询 recurrence_original_date > "2025-11-10" 的未完成任务
  2. ❌ 只软删除，未清除循环字段  <-- Bug #2
  3. ✅ 删除链接记录
    ↓
结果:
  - 任务有 recurrence_id 但已被软删除
  - 前端可能在回收站中看到循环任务
  - 恢复任务后，数据不一致
```

### 场景 3: 删除循环规则

```
用户操作: DELETE /recurrences/:id
    ↓
后端处理 cleanup_all_future_instances():
  1. 只处理未完成任务 (completed_at IS NULL)
     - ✅ 清除循环字段
     - ✅ 软删除
  2. 删除所有 task_recurrence_links (包括已完成的)
    ↓
结果:
  - 未完成任务：正确清理 ✅
  - 已完成任务：
    * ❌ 保留 recurrence_id, recurrence_original_date  <-- Bug #3
    * ✅ 删除链接记录
    * ✅ 循环规则 is_active = false
```

---

## ✅ 修复方案

### 修复 Bug #1: 删除任务时清理循环链接

```rust
// src-tauri/src/features/endpoints/tasks/delete_task.rs

// 5. 删除任务的所有链接和日程
TaskTimeBlockLinkRepository::delete_all_for_task_in_tx(&mut tx, task_id).await?;
TaskScheduleRepository::delete_all_in_tx(&mut tx, task_id).await?;

// ✅ 新增：删除循环链接
use crate::features::shared::repositories::TaskRecurrenceLinkRepository;
TaskRecurrenceLinkRepository::delete_by_task_id_in_tx(&mut tx, task_id).await?;
```

### 修复 Bug #2: 更新循环规则时清除循环字段

```rust
// src-tauri/src/features/endpoints/recurrences/update_recurrence.rs

// cleanup_future_instances()
for task_id_str in task_ids {
    let task_id = Uuid::parse_str(&task_id_str)?;

    // ✅ 新增：清除循环参数
    let clear_params_query = r#"
        UPDATE tasks
        SET recurrence_id = NULL,
            recurrence_original_date = NULL,
            updated_at = ?
        WHERE id = ?
    "#;
    sqlx::query(clear_params_query)
        .bind(now)
        .bind(task_id.to_string())
        .execute(&mut **tx)
        .await?;

    // 然后再软删除
    TaskRepository::soft_delete_in_tx(tx, task_id, now).await?;
}

// cleanup_mismatched_instances() 同理
```

### 修复 Bug #3: 删除循环规则时处理所有任务

**方案 A**: 清除所有任务的循环字段（推荐）

```rust
// src-tauri/src/features/endpoints/recurrences/delete_recurrence.rs

// 清理所有任务实例的循环字段（包括已完成的）
let clear_all_query = r#"
    UPDATE tasks
    SET recurrence_id = NULL,
        recurrence_original_date = NULL,
        updated_at = ?
    WHERE recurrence_id = ?
      AND deleted_at IS NULL  -- 只清理未删除的
"#;
sqlx::query(clear_all_query)
    .bind(now)
    .bind(recurrence_id.to_string())
    .execute(&mut **tx)
    .await?;

// 只软删除未来的未完成任务
let query = r#"
    SELECT id
    FROM tasks
    WHERE recurrence_id = ?
      AND recurrence_original_date >= ?
      AND completed_at IS NULL
      AND deleted_at IS NULL
"#;
// ... 软删除逻辑
```

**方案 B**: 保留已完成任务的循环信息（作为历史记录）

如果要保留已完成任务的循环信息，则不删除其 `task_recurrence_links`：

```rust
// 只删除未完成任务的链接
DELETE FROM task_recurrence_links
WHERE recurrence_id = ?
  AND task_id IN (
      SELECT id FROM tasks
      WHERE completed_at IS NULL
         OR deleted_at IS NOT NULL
  )
```

---

## 🔒 数据一致性规则（设计规范）

### 规则 1: 循环任务的三元组完整性

一个任务是循环任务，**当且仅当**以下三者同时存在：

1. `tasks.recurrence_id` (指向 task_recurrences.id)
2. `tasks.recurrence_original_date` (YYYY-MM-DD)
3. `task_recurrence_links` 记录 (recurrence_id, instance_date, task_id)

**循环例外**：如果用户手动删除循环任务实例，`task_recurrence_links` 被删除，该日期不再生成新实例

### 规则 2: 删除任务时的清理职责

```sql
DELETE /tasks/:id 必须清理：
1. task_schedules (WHERE task_id = ?)
2. task_time_block_links (WHERE task_id = ?)
3. task_recurrence_links (WHERE task_id = ?)  ← ✅ 设计为循环例外功能
```

**注意**：删除循环任务实例时**不清理** `task_recurrence_links` 是设计特性（循环例外），该日期不再生成新实例

### 规则 3: 软删除任务时的循环字段处理

如果任务被软删除 (`deleted_at IS NOT NULL`)：

- **选项 A** (推荐): 清除 recurrence_id 和 recurrence_original_date
- **选项 B**: 保留字段，但确保不会被实例化服务处理

### 规则 4: 循环规则失效时的任务处理

当循环规则被删除或修改时：

- **未完成且未来的任务**: 清除循环字段 + 软删除
- **已完成的任务**:
  - 清除循环字段（断开关联）✅
  - 保留任务本身（作为历史记录）✅

---

## 🧪 测试用例

### 测试 Bug #1

```rust
#[tokio::test]
async fn test_delete_task_should_cleanup_recurrence_links() {
    // 1. 创建循环规则和任务实例
    let recurrence_id = create_recurrence(...);
    let task_id = instantiate_for_date(...);

    // 2. 验证链接存在
    let link = find_recurrence_link(recurrence_id, date);
    assert!(link.is_some());

    // 3. 删除任务
    delete_task(task_id);

    // 4. 验证链接已被删除
    let link = find_recurrence_link(recurrence_id, date);
    assert!(link.is_none());  // ❌ 当前失败
}
```

### 测试 Bug #2

```rust
#[tokio::test]
async fn test_update_recurrence_should_clear_deleted_task_fields() {
    // 1. 创建循环规则和任务实例
    let recurrence_id = create_recurrence(...);
    let task_id = instantiate_for_date("2025-11-15");

    // 2. 设置结束日期（早于任务日期）
    update_recurrence(recurrence_id, { end_date: "2025-11-10" });

    // 3. 任务应该被软删除
    let task = find_task(task_id);
    assert!(task.deleted_at.is_some());

    // 4. 循环字段应该被清除
    assert!(task.recurrence_id.is_none());  // ❌ 当前失败
    assert!(task.recurrence_original_date.is_none());  // ❌ 当前失败
}
```

### 测试 Bug #3

```rust
#[tokio::test]
async fn test_delete_recurrence_should_cleanup_completed_tasks() {
    // 1. 创建循环规则和任务实例
    let recurrence_id = create_recurrence(...);
    let task_id = instantiate_for_date(...);

    // 2. 完成任务
    complete_task(task_id);

    // 3. 删除循环规则
    delete_recurrence(recurrence_id);

    // 4. 已完成任务的循环字段应该被清除
    let task = find_task(task_id);
    assert!(task.recurrence_id.is_none());  // ❌ 当前失败
    assert!(task.recurrence_original_date.is_none());  // ❌ 当前失败

    // 5. 任务本身应该保留（不应被删除）
    assert!(task.deleted_at.is_none());  // ✅ 当前通过
}
```

---

## 📝 修复优先级

| Bug | 严重性 | 影响范围                   | 优先级        |
| --- | ------ | -------------------------- | ------------- |
| #1  | 🔴 高  | 所有循环任务删除操作       | P0 (立即修复) |
| #2  | 🟠 中  | 修改循环规则时             | P1 (本周修复) |
| #3  | 🟡 低  | 删除循环规则时的已完成任务 | P2 (下周修复) |

---

## 🎯 修复后的一致性保证

修复后，系统将保证：

✅ **任务删除**: 清理所有关联记录（schedules, time_block_links, recurrence_links）
✅ **循环规则修改**: 清除不再匹配的任务的循环字段
✅ **循环规则删除**: 清除所有任务的循环字段（或保留已完成任务但断开关联）
✅ **前后端一致**: 右键菜单和编辑框使用相同的判断逻辑
✅ **数据完整性**: recurrence_id ⇔ recurrence_original_date ⇔ task_recurrence_links 三者同步

生成时间: 2025-11-10
审计人员: Claude (AI Assistant)
