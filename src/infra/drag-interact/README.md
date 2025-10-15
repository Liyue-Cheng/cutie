# Interact.js 拖放系统

基于 interact.js 的新一代拖放系统，支持双重视觉元素、非破坏性预览和越界即时回弹。

## 核心特性

- ✅ **双重视觉元素**: 幽灵元素 + 实体元素预览
- ✅ **非破坏性预览**: 所有预览通过响应式计算实现
- ✅ **越界即时回弹**: 拖出有效区域自动恢复原位
- ✅ **单一 Composable**: 替代原有的多个 composable
- ✅ **完全兼容**: 不破坏现有代码

## 快速开始

### 1. 基础用法

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import type { TaskCard } from '@/types/dtos'

const props = defineProps<{ viewKey: string }>()
const tasks = ref<TaskCard[]>([])
const taskListRef = ref<HTMLElement | null>(null)

const viewMetadata = computed(() => ({
  type: 'daily' as const,
  id: props.viewKey,
  config: { date: '2025-10-14' },
}))

// 🔥 使用新的拖放系统
const { displayTasks, isDragging, isReceiving } = useInteractDrag({
  viewMetadata,
  tasks,
  containerRef: taskListRef,
})
</script>

<template>
  <div
    ref="taskListRef"
    class="task-list"
    :class="{
      'is-dragging': isDragging,
      'is-receiving': isReceiving,
    }"
  >
    <div
      v-for="task in displayTasks"
      :key="task.id"
      class="task-card-wrapper"
      :class="{ 'is-preview': task._isPreview }"
      :data-task-id="task.id"
    >
      <TaskCard :task="task" />
    </div>
  </div>
</template>

<style scoped>
/* 预览样式 */
.task-card-wrapper.is-preview {
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.15);
  transform: translateY(-2px) scale(1.02);
  transition: all 0.2s ease;
  border: 2px solid var(--primary-color);
}

/* 拖动状态样式 */
.task-list.is-dragging {
  background-color: rgba(59, 130, 246, 0.05);
}

.task-list.is-receiving {
  background-color: rgba(16, 185, 129, 0.05);
  border: 2px dashed var(--success-color);
}
</style>
```

### 2. 日历拖放

```vue
<script setup lang="ts">
import { useInteractDrag } from '@/composables/drag/useInteractDrag'

const { displayTasks } = useInteractDrag({
  viewMetadata,
  tasks,
  containerRef: calendarRef,
  dropzoneType: 'calendar', // 🔥 日历类型
  onDrop: async (session) => {
    // 自定义日历放置逻辑
    await handleCalendarDrop(session)
  },
})
</script>
```

## 架构说明

### 数据流

```
用户拖动任务
    ↓
interact.js 检测事件
    ↓
DragController 更新状态
    ↓
dragPreviewState 响应式更新
    ↓
组件 displayTasks computed 重新计算
    ↓
Vue 自动重新渲染
```

### 核心文件

```
src/infra/drag-interact/
├── types.ts              # 类型定义
├── preview-state.ts       # 响应式预览状态
├── drag-controller.ts     # 拖放控制器
├── utils.ts              # 工具函数
└── index.ts              # 统一导出

src/composables/drag/
└── useInteractDrag.ts    # Vue Composable
```

## 迁移指南

### 从旧系统迁移

```vue
<!-- 旧系统 -->
<script setup lang="ts">
const sameViewDrag = useSameViewDrag(getTasksFn)
const crossViewDrag = useCrossViewDrag()
const crossViewTarget = useCrossViewDragTarget(viewMetadata)

const displayTasks = computed(() => {
  // 复杂的逻辑...
})
</script>

<!-- 新系统 -->
<script setup lang="ts">
const { displayTasks } = useInteractDrag({
  viewMetadata,
  tasks,
  containerRef: taskListRef,
})
</script>
```

### 渐进式迁移

```vue
<script setup lang="ts">
// 条件使用新系统
const USE_NEW_DRAG = props.viewKey === 'staging'

const dragSystem = USE_NEW_DRAG
  ? useInteractDrag({ viewMetadata, tasks, containerRef })
  : useLegacyDrag({ viewMetadata, tasks })

const displayTasks = dragSystem.displayTasks
</script>
```

## 调试

### 获取调试信息

```typescript
const { getDebugInfo } = useInteractDrag({ ... })

console.log(getDebugInfo())
// {
//   viewId: 'daily::2025-10-14',
//   taskCount: 5,
//   displayTaskCount: 6,
//   isDragging: true,
//   isReceiving: false,
//   previewState: { ... }
// }
```

### 控制器调试

```typescript
import { interactManager } from '@/infra/drag-interact'

console.log(interactManager.getDebugInfo())
// {
//   phase: 'OVER_TARGET',
//   hasSession: true,
//   targetZone: 'daily::2025-10-15',
//   validZones: ['staging', 'daily::2025-10-14', ...]
// }
```

## 注意事项

1. **DOM 结构要求**: 可拖拽元素必须有 `data-task-id` 属性
2. **CSS 类名**: 容器内的任务包装元素需要 `.task-card-wrapper` 类
3. **生命周期**: Composable 会自动处理初始化和清理
4. **兼容性**: 完全兼容现有的策略系统

## 故障排除

### 常见问题

1. **拖拽不工作**: 检查 `data-task-id` 属性是否存在
2. **预览不显示**: 检查 `containerRef` 是否正确绑定
3. **样式问题**: 确保 CSS 类名正确

### 错误日志

系统会在控制台输出详细的调试信息，标签为 `[DragController]`。
