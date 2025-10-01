# Pinia 最佳实践 - 避免状态不同步 Bug

## ⚠️ 核心原则

**所有数据必须只有一个真理来源（Single Source of Truth）- Pinia Store**

---

## 🐛 常见 Bug 及解决方案

### Bug 1: 操作后 UI 不更新

#### **错误示例：**
```vue
<script setup>
// ❌ 错误：缓存数据到本地
const localTimeBlocks = ref([])

onMounted(async () => {
  const blocks = await timeBlockStore.fetchTimeBlocksForDate('2024-10-28')
  localTimeBlocks.value = blocks  // ❌ 创建了数据副本
})

// ❌ 问题：当 store 中的数据变化时（删除/更新），localTimeBlocks 不会更新
const events = computed(() => {
  return localTimeBlocks.value.map(block => ...)  // UI 不会响应 store 变化
})
</script>
```

#### **正确示例：**
```vue
<script setup>
// ✅ 正确：直接使用 store 的 getter
const timeBlockStore = useTimeBlockStore()

onMounted(async () => {
  await timeBlockStore.fetchTimeBlocksForDate('2024-10-28')
  // ✅ 不缓存，数据已存储在 store 中
})

// ✅ 正确：直接从 store 读取
const events = computed(() => {
  return timeBlockStore.allTimeBlocks.map(block => ...)  // 自动响应更新
})
</script>
```

---

### Bug 2: 删除操作后日历仍显示旧数据

#### **原因分析：**
```typescript
// ❌ 错误的删除流程
async function deleteTimeBlock(id: string) {
  await fetch(`/api/time-blocks/${id}`, { method: 'DELETE' })
  // ❌ 忘记更新本地 store！
}

// 结果：后端已删除，但前端 store 中仍有数据，UI 继续显示
```

#### **正确流程：**
```typescript
// ✅ 正确的删除流程（在 TimeBlockStore 中）
async function deleteTimeBlock(id: string): Promise<boolean> {
  try {
    const apiBaseUrl = await waitForApiReady()
    const response = await fetch(`${apiBaseUrl}/time-blocks/${id}`, {
      method: 'DELETE'
    })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    
    // ✅ 关键：删除本地 store 中的数据
    removeTimeBlock(id)  // 更新 state
    return true
  } catch (e) {
    console.error('Error deleting time block:', e)
    return false
  }
}
```

---

### Bug 3: 更新操作后显示过时数据

#### **错误模式：**
```typescript
// ❌ 在组件中直接修改数据
function updateBlockTitle(blockId: string, newTitle: string) {
  const block = timeBlockStore.getTimeBlockById(blockId)
  if (block) {
    block.title = newTitle  // ❌ 直接修改会破坏响应性！
  }
}
```

#### **正确模式：**
```typescript
// ✅ 通过 store action 更新
async function updateBlockTitle(blockId: string, newTitle: string) {
  await timeBlockStore.updateTimeBlock(blockId, { title: newTitle })
  // ✅ store 内部会创建新的 Map，触发响应式更新
}
```

---

## ✅ 正确的 Store 使用模式

### 1. 组件中只读取，不修改

```vue
<script setup>
const timeBlockStore = useTimeBlockStore()

// ✅ 使用 computed 读取
const todayBlocks = computed(() => {
  return timeBlockStore.getTimeBlocksForDate('2024-10-28')
})

// ✅ 使用 getter 函数
const getBlock = (id: string) => {
  return timeBlockStore.getTimeBlockById(id)
}

// ❌ 不要这样做
const blocks = ref(timeBlockStore.allTimeBlocks)  // 失去响应性
</script>
```

### 2. 所有修改通过 Action

```vue
<script setup>
const timeBlockStore = useTimeBlockStore()

// ✅ 创建
async function handleCreate() {
  await timeBlockStore.createTimeBlock(payload)
  // 自动触发 UI 更新
}

// ✅ 更新
async function handleUpdate(id: string) {
  await timeBlockStore.updateTimeBlock(id, payload)
  // 自动触发 UI 更新
}

// ✅ 删除
async function handleDelete(id: string) {
  await timeBlockStore.deleteTimeBlock(id)
  // 自动触发 UI 更新
}
</script>
```

### 3. Store Action 内部必须创建新对象

```typescript
// ❌ 错误：直接修改 Map
function removeTimeBlock(id: string) {
  timeBlocks.value.delete(id)  // ❌ 不会触发响应式更新！
}

// ✅ 正确：创建新 Map
function removeTimeBlock(id: string) {
  const newMap = new Map(timeBlocks.value)
  newMap.delete(id)
  timeBlocks.value = newMap  // ✅ 触发响应式更新
}
```

---

## 📋 检查清单

在实现涉及 Pinia 的功能时，检查：

- [ ] 组件是否直接使用 store 的 getters？
- [ ] 是否使用 `computed` 包装数据读取？
- [ ] 是否避免了在组件中缓存 store 数据？
- [ ] 所有 CRUD 操作是否通过 store actions？
- [ ] Store actions 是否创建了新对象（而非直接修改）？
- [ ] 是否避免了直接修改从 getter 返回的对象？

---

## 🎯 实际案例：CuteCalendar

### ✅ 正确实现

```vue
<script setup>
import { useTimeBlockStore } from '@/stores/timeblock'

const timeBlockStore = useTimeBlockStore()

// ✅ 响应式事件列表
const calendarEvents = computed(() => {
  // 直接从 store.allTimeBlocks 读取
  // 当 store 中的时间块被删除/更新时，这里会自动重新计算
  return timeBlockStore.allTimeBlocks.map(block => ({
    id: block.id,
    title: block.title,
    start: block.start_time,
    end: block.end_time,
    color: block.area?.color ?? '#4a90e2'
  }))
})

// ✅ 删除时间块
async function handleDelete(blockId: string) {
  // 通过 store action 删除
  await timeBlockStore.deleteTimeBlock(blockId)
  // ✅ store 内部会更新 timeBlocks Map
  // ✅ allTimeBlocks getter 自动重新计算
  // ✅ calendarEvents computed 自动重新计算
  // ✅ FullCalendar 自动重新渲染
}
</script>

<template>
  <!-- FullCalendar 绑定到响应式的 calendarEvents -->
  <FullCalendar :options="{ events: calendarEvents }" />
</template>
```

---

## 🔄 响应式更新链路

```
用户操作
   ↓
Store Action (deleteTimeBlock)
   ↓
更新 State (创建新 Map)
   ↓
触发 Store Getter (allTimeBlocks)
   ↓
触发 Component Computed (calendarEvents)
   ↓
Vue 自动重新渲染
   ↓
UI 更新完成 ✅
```

**关键点：** 这个链路的每一步都是响应式的，任何一步断裂都会导致 UI 不更新！

---

## 🛠️ 调试技巧

### 如果 UI 没有更新，检查：

1. **Store State 是否更新？**
```typescript
// 在浏览器控制台
$pinia.state.value.timeblock.timeBlocks
// 检查数据是否确实被删除/更新
```

2. **Getter 是否重新计算？**
```typescript
// 添加调试日志
const allTimeBlocks = computed(() => {
  console.log('[TimeBlockStore] allTimeBlocks getter called')
  return Array.from(timeBlocks.value.values())
})
```

3. **Component Computed 是否重新计算？**
```typescript
const calendarEvents = computed(() => {
  console.log('[Calendar] calendarEvents computed, blocks count:', timeBlockStore.allTimeBlocks.length)
  return ...
})
```

4. **是否创建了新对象？**
```typescript
// ❌ 这不会触发更新
timeBlocks.value.set(id, newBlock)

// ✅ 这会触发更新
timeBlocks.value = new Map(timeBlocks.value).set(id, newBlock)
```

---

## 📚 参考资料

- Vue 3 响应性原理：https://vuejs.org/guide/essentials/reactivity-fundamentals.html
- Pinia Getters：https://pinia.vuejs.org/core-concepts/getters.html
- Map 响应性注意事项：https://vuejs.org/guide/essentials/reactivity-fundamentals.html#limitations-of-reactive

---

## 💡 总结

**记住一句话：**
> "组件从 Store 读，操作通过 Store 做，Store 内部创建新对象。"

遵循这个原则，就不会出现状态不同步的 bug！
