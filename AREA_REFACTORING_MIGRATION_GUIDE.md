# Area 字段重构迁移指南

## 背景

为解决 N+1 查询问题，我们将 TaskCard 中的完整 area 对象改为只传递 area_id，前端在应用启动时一次性加载所有 areas，然后在组件中通过 area_id 从 area store 获取完整信息。

## 架构变更

### 后端变更

**TaskCardDto 结构变更：**

```rust
// ❌ 旧版本
pub struct TaskCardDto {
    pub area: Option<AreaSummary>,  // 包含 id, name, color
    // ...
}

// ✅ 新版本
pub struct TaskCardDto {
    pub area_id: Option<Uuid>,  // 只传递 ID
    // ...
}
```

### 前端变更

**TaskCard interface 变更：**

```typescript
// ❌ 旧版本
export interface TaskCard {
  area: {
    id: string
    name: string
    color: string
  } | null
  // ...
}

// ✅ 新版本
export interface TaskCard {
  area_id: string | null // 只存储 ID
  // ...
}
```

**应用启动时加载所有 areas：**

```typescript
// src/main.ts
initializeApiConfig().then(async () => {
  // ✅ 在应用启动时加载所有 areas
  const { useAreaStore } = await import('@/stores/area')
  const areaStore = useAreaStore()
  await areaStore.fetchAreas()
  console.log('✅ All areas loaded')
})
```

## 组件迁移指南

### 方式 1: 使用 computed 属性（推荐）

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useAreaStore } from '@/stores/area'
import type { TaskCard } from '@/types/dtos'

const props = defineProps<{
  task: TaskCard
}>()

const areaStore = useAreaStore()

// ✅ 通过 area_id 从 store 获取完整 area 信息
const area = computed(() => {
  return props.task.area_id ? areaStore.getAreaById(props.task.area_id) : null
})
</script>

<template>
  <!-- ❌ 旧版本 -->
  <!-- <div v-if="task.area" :style="{ color: task.area.color }">
    {{ task.area.name }}
  </div> -->

  <!-- ✅ 新版本 -->
  <div v-if="area" :style="{ color: area.color }">
    {{ area.name }}
  </div>
</template>
```

### 方式 2: 直接在模板中调用（适用于简单场景）

```vue
<script setup lang="ts">
import { useAreaStore } from '@/stores/area'

const areaStore = useAreaStore()
</script>

<template>
  <!-- ✅ 直接在模板中调用 -->
  <div v-if="task.area_id" :style="{ color: areaStore.getAreaById(task.area_id)?.color }">
    {{ areaStore.getAreaById(task.area_id)?.name }}
  </div>
</template>
```

### 方式 3: 批量处理（适用于列表渲染）

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { useAreaStore } from '@/stores/area'
import type { TaskCard } from '@/types/dtos'

const props = defineProps<{
  tasks: TaskCard[]
}>()

const areaStore = useAreaStore()

// ✅ 为每个任务创建 area 映射
const tasksWithAreas = computed(() => {
  return props.tasks.map((task) => ({
    ...task,
    area: task.area_id ? areaStore.getAreaById(task.area_id) : null,
  }))
})
</script>

<template>
  <div v-for="task in tasksWithAreas" :key="task.id">
    <div v-if="task.area" :style="{ color: task.area.color }">
      {{ task.area.name }}
    </div>
  </div>
</template>
```

## 需要修改的典型模式

### 模式 1: 显示 Area 标签

```vue
<!-- ❌ 旧版本 -->
<div v-if="task.area" class="area-tag" :style="{ backgroundColor: task.area.color }">
  {{ task.area.name }}
</div>

<!-- ✅ 新版本 -->
<div
  v-if="task.area_id && areaStore.getAreaById(task.area_id)"
  class="area-tag"
  :style="{ backgroundColor: areaStore.getAreaById(task.area_id)?.color }"
>
  {{ areaStore.getAreaById(task.area_id)?.name }}
</div>
```

### 模式 2: 按 Area 分组

```typescript
// ❌ 旧版本
const tasksByArea = computed(() => {
  const groups = new Map<string, TaskCard[]>()
  for (const task of tasks.value) {
    const areaId = task.area?.id || 'no-area'
    // ...
  }
  return groups
})

// ✅ 新版本
const tasksByArea = computed(() => {
  const groups = new Map<string, TaskCard[]>()
  for (const task of tasks.value) {
    const areaId = task.area_id || 'no-area'
    // ...
  }
  return groups
})
```

### 模式 3: Area 筛选

```typescript
// ❌ 旧版本
const filteredTasks = computed(() => {
  if (!selectedAreaId.value) return tasks.value
  return tasks.value.filter((task) => task.area?.id === selectedAreaId.value)
})

// ✅ 新版本
const filteredTasks = computed(() => {
  if (!selectedAreaId.value) return tasks.value
  return tasks.value.filter((task) => task.area_id === selectedAreaId.value)
})
```

## 性能优势

### 后端优化

- **消除 N+1 查询**：之前每个任务都要查询一次 area，现在完全不查询
- **减少数据库压力**：查询任务时不再 JOIN areas 表
- **减少响应体积**：每个任务节省 ~50 字节（area 对象）

### 前端优化

- **一次性加载**：应用启动时一次加载所有 areas（通常 < 50 个）
- **本地查询**：组件中通过 Map 查询，时间复杂度 O(1)
- **自动更新**：area 更新时所有使用它的组件自动响应

## 注意事项

1. **Area 不存在的情况**：使用可选链操作符 `?.` 处理

   ```typescript
   const area = areaStore.getAreaById(task.area_id)?.name ?? '无分类'
   ```

2. **性能考虑**：避免在模板中多次调用 `getAreaById`，使用 computed 缓存

   ```vue
   <!-- ❌ 不推荐：每次渲染都查询 3 次 -->
   <div>
     <span>{{ areaStore.getAreaById(task.area_id)?.name }}</span>
     <span :style="{ color: areaStore.getAreaById(task.area_id)?.color }">●</span>
     <span>{{ areaStore.getAreaById(task.area_id)?.parent_area_id }}</span>
   </div>

   <!-- ✅ 推荐：使用 computed 缓存 -->
   <script setup>
   const area = computed(() => (task.area_id ? areaStore.getAreaById(task.area_id) : null))
   </script>
   <div v-if="area">
     <span>{{ area.name }}</span>
     <span :style="{ color: area.color }">●</span>
     <span>{{ area.parent_area_id }}</span>
   </div>
   ```

3. **响应式更新**：area store 是响应式的，area 更新时组件会自动重新渲染

## 已修改的文件清单

### 后端（11 个文件）

- ✅ `src-tauri/src/entities/task/response_dtos.rs` - TaskCardDto 定义
- ✅ `src-tauri/src/features/tasks/shared/assembler.rs` - TaskAssembler
- ✅ `src-tauri/src/features/tasks/endpoints/create_task.rs`
- ✅ `src-tauri/src/features/tasks/endpoints/get_task.rs`
- ✅ `src-tauri/src/features/tasks/endpoints/update_task.rs`
- ✅ `src-tauri/src/features/tasks/endpoints/complete_task.rs`
- ✅ `src-tauri/src/features/tasks/endpoints/reopen_task.rs`
- ✅ `src-tauri/src/features/time_blocks/endpoints/create_from_task.rs`
- ✅ `src-tauri/src/features/views/shared/task_card_assembler.rs`

### 前端（2 个文件）

- ✅ `src/types/dtos.ts` - TaskCard interface
- ✅ `src/main.ts` - 应用启动时加载 areas

## 测试验证

1. **后端编译**：`cd src-tauri && cargo check`
2. **前端类型检查**：`npm run type-check`（如果有）
3. **功能测试**：
   - 创建任务时 area 颜色正确显示
   - 更新任务 area 时界面立即更新
   - 按 area 筛选任务功能正常
   - Area 管理（创建/更新/删除）后任务显示正确

## 后续优化建议

1. **批量加载优化**：如果 areas 数量超过 100 个，考虑分页或按需加载
2. **缓存策略**：添加 areas 的本地存储缓存，减少启动时的网络请求
3. **增量更新**：通过 SSE 监听 area 的变更事件，实时更新 store

---

**迁移完成标志**：所有使用 `task.area.xxx` 的地方都改为使用 `areaStore.getAreaById(task.area_id)?.xxx`
