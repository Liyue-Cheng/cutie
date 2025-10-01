# SQLite Database Locked 问题分析

**错误：** `database is locked (code: 5)`  
**场景：** 快速连续点击完成/重新打开按钮  
**诊断时间：** 2025-10-01

---

## 🔴 **核心问题：缺少 busy_timeout 配置**

### **错误日志：**

```
ERROR Database error: Database connection error:
error returned from database: (code: 5) database is locked
```

### **问题根源：**

```rust
// shared/database/connection.rs

// ✅ 已配置：
PRAGMA journal_mode = WAL;       // WAL 模式
PRAGMA synchronous = NORMAL;     // 同步模式
PRAGMA cache_size = ...;         // 缓存大小

// ❌ 缺少关键配置：
// PRAGMA busy_timeout = ???;    // ← 没有设置！
```

**SQLite 默认行为：**

- `busy_timeout = 0`（默认）
- 遇到锁立即返回错误
- 不等待、不重试

**导致的问题：**

```
请求 1: 开始事务 → 获得写锁 → 执行中...
请求 2: 开始事务 → ❌ 数据库被锁 → 立即报错！
```

---

## 📊 **SQLite 并发模型**

### **WAL 模式下的并发：**

```
✅ 支持：多个读取同时进行
✅ 支持：读取和写入同时进行
❌ 不支持：多个写入同时进行（必须串行）
```

### **当前配置问题：**

| 配置项            | 当前值          | 推荐值     | 影响                 |
| ----------------- | --------------- | ---------- | -------------------- |
| `max_connections` | 10              | **5**      | 太多连接争抢写锁     |
| `busy_timeout`    | **0（未设置）** | **5000ms** | 立即失败 vs 等待重试 |
| `journal_mode`    | WAL ✅          | WAL        | 已正确               |
| `synchronous`     | NORMAL ✅       | NORMAL     | 已正确               |

---

## 🔥 **快速点击时的请求竞争**

### **场景重现：**

```
用户操作：快速点击 5 次
↓
前端发送：5 个并发请求
↓
后端处理：

Request 1: BEGIN → 获得锁 → UPDATE tasks → COMMIT (200ms)
Request 2: BEGIN → ❌ 锁定！→ 报错
Request 3: BEGIN → ❌ 锁定！→ 报错
Request 4: BEGIN → ❌ 锁定！→ 报错
Request 5: BEGIN → ✅ 成功（Request 1 完成后）
```

---

## 💡 **解决方案（按优先级）**

### **P0 - 立即修复（5分钟）** 🔥

#### 1. **添加 busy_timeout 配置**

```rust
// shared/database/connection.rs::configure_sqlite()

// 添加到现有配置中
sqlx::query("PRAGMA busy_timeout = 5000")  // 5 秒超时
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

tracing::info!("SQLite busy_timeout set to 5000ms");
```

**效果：**

- SQLite 会等待最多 5 秒获取锁
- 自动重试，而非立即失败
- ✅ 解决 99% 的 database locked 错误

---

#### 2. **减少连接池大小（SQLite 最佳实践）**

```rust
// config/database_config.rs 或 DatabaseConfig::default()

max_connections: 5,  // ← 改为 5（当前是 10）
```

**原因：**

- SQLite 同时只能有 1 个写入
- 更多连接 = 更多竞争
- **推荐：3-5 个连接**

**效果：**

- 减少锁竞争
- 更快的连接获取

---

### **P1 - 前端防抖（10分钟）** ⚠️

#### 3. **添加按钮防抖/节流**

```vue
<!-- KanbanTaskCard.vue -->
<script setup>
const isProcessing = ref(false)

async function handleComplete() {
  if (isProcessing.value) return  // ← 防止重复点击

  isProcessing.value = true
  try {
    await taskStore.completeTask(task.id)
  } finally {
    setTimeout(() => {
      isProcessing.value = false  // 200ms 后恢复
    }, 200)
  }
}
</script>

<template>
  <button
    @click="handleComplete"
    :disabled="isProcessing"  <!-- 禁用按钮 -->
    :class="{ 'processing': isProcessing }"
  >
    Complete
  </button>
</template>
```

**效果：**

- 防止用户快速连续点击
- UI 层面的保护

---

#### 4. **前端请求去重**

```typescript
// stores/task.ts

const pendingOperations = new Map<string, Promise<any>>()

async function completeTask(id: string) {
  // 如果该任务正在处理，返回现有的 Promise
  if (pendingOperations.has(id)) {
    return pendingOperations.get(id)
  }

  const operation = (async () => {
    try {
      // 实际 API 调用
      return await actualCompleteTask(id)
    } finally {
      pendingOperations.delete(id)
    }
  })()

  pendingOperations.set(id, operation)
  return operation
}
```

**效果：**

- 同一任务的并发请求会被合并
- 只发送一次实际请求

---

### **P2 - 后端优化（1-2小时）** 📝

#### 5. **添加 SQLite 重试机制**

```rust
// shared/database/connection.rs

use sqlx::sqlite::SqliteError;

pub async fn execute_with_retry<F, T>(f: F) -> AppResult<T>
where
    F: Fn() -> Future<Output = Result<T, sqlx::Error>>,
{
    let max_retries = 3;
    let mut attempts = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if is_locked_error(&e) && attempts < max_retries {
                    attempts += 1;
                    tracing::warn!("Database locked, retrying ({}/{})", attempts, max_retries);
                    tokio::time::sleep(Duration::from_millis(100 * attempts)).await;
                    continue;
                }
                return Err(AppError::DatabaseError(DbError::ConnectionError(e)));
            }
        }
    }
}

fn is_locked_error(e: &sqlx::Error) -> bool {
    match e {
        sqlx::Error::Database(db_err) => {
            db_err.code().map(|c| c == "5").unwrap_or(false)
        }
        _ => false,
    }
}
```

---

#### 6. **优化事务持有时间**

```rust
// ❌ 当前：事务可能持有过长
let mut tx = pool.begin().await?;
database::complex_operation(&mut tx).await?;  // 可能很慢
sleep(1000ms);  // 假设有耗时操作
tx.commit().await?;  // 持有锁太久！

// ✅ 优化：只在真正需要时持有事务
// 1. 先准备数据（不在事务中）
let prepared_data = prepare_data().await;

// 2. 快速提交事务
let mut tx = pool.begin().await?;
database::quick_update(&mut tx, prepared_data).await?;
tx.commit().await?;  // 快速释放锁
```

---

## 📊 **配置对比**

### **修复前：**

```rust
max_connections: 10
busy_timeout: 0 (未设置) ❌
```

**结果：**

- 10 个连接争抢 1 个写锁
- 锁定时立即失败
- 用户快速点击 → 大量失败

### **修复后：**

```rust
max_connections: 5
busy_timeout: 5000ms ✅
```

**结果：**

- 5 个连接（减少竞争）
- 等待 5 秒重试
- 用户快速点击 → 自动排队成功

---

## 🎯 **立即修复步骤**

### **步骤 1：添加 busy_timeout**

```rust
// src-tauri/src/shared/database/connection.rs
// 在 configure_sqlite 函数中添加：

sqlx::query("PRAGMA busy_timeout = 5000")  // 5秒
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseError(DbError::ConnectionError(e)))?;

tracing::info!("SQLite busy_timeout set to 5000ms");
```

### **步骤 2：减少连接池大小**

```rust
// src-tauri/src/shared/database/connection.rs
// DatabaseConfig::default()

max_connections: 5,  // 改为 5（从 10）
```

### **步骤 3：测试**

```bash
cargo tauri dev
# 快速点击完成/重新打开按钮
# 应该不再报错！
```

---

## 🔚 **总结**

### **核心问题：**

- ❌ 缺少 `busy_timeout` 配置
- ❌ 连接池过大（10 个连接争抢 1 个写锁）
- ❌ 没有前端防抖保护

### **影响：**

- 快速点击导致并发写入
- SQLite 立即报错（不等待）
- 用户体验差

### **修复后：**

- ✅ SQLite 会等待和重试（最多 5 秒）
- ✅ 减少连接池竞争
- ✅ 99% 的情况下自动恢复

### **预期效果：**

- **database locked 错误：频繁 → 几乎不会发生** ✅

---

## 🚀 **为什么这个问题现在才出现？**

**可能原因：**

1. **之前没有快速连续操作**
   - 正常使用：点击 → 等待 → 点击
   - 测试时：快速点击暴露问题

2. **Debug 模式放大问题**
   - 事务执行慢 → 持有锁更久
   - 更容易出现锁竞争

3. **功能增加导致并发增加**
   - 现在有更多端点
   - 更多并发请求机会

---

**立即添加 busy_timeout 就能解决！** 🚀
