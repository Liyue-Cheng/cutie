# 整个看板作为拖放接收区域 - 实现指南

## 📋 需求

将整个看板容器（而不仅仅是任务列表区域）作为有效的拖放接收区域，提升用户体验：

- ✅ 用户可以在看板的任何位置释放鼠标
- ✅ 标题区、空白区、底部区域都是有效接收点
- ✅ 空看板也能正常接收任务
- ✅ dropIndex 仍然基于任务卡片位置精确计算

---

## 🎯 核心思路

### 传统实现的问题

```vue
<!-- ❌ 传统方案：只有 task-list 可以接收 -->
<div class="kanban-column">
  <div class="header">...</div>
  <div ref="taskListRef" class="task-list">  <!-- 只有这里可以 drop -->
    <TaskCard v-for="task in tasks" :key="task.id" />
  </div>
</div>
```

**缺点**：

- 用户必须精确瞄准任务列表区域
- 空看板难以命中
- 标题区、底部区域无法接收

### 新实现方案

```vue
<!-- ✅ 新方案：整个看板都可以接收 -->
<div class="kanban-column">
  <div ref="kanbanContainerRef" class="content-wrapper">  <!-- 整个区域可以 drop -->
    <div class="header">...</div>
    <div ref="taskListRef" class="task-list">  <!-- 仅用于计算 dropIndex -->
      <TaskCard v-for="task in tasks" :key="task.id" />
    </div>
  </div>
</div>
```

**优点**：

- 整个看板都是有效接收区域
- 空看板接收面积大
- 用户体验更友好

---

## 🛠️ 实现步骤

### 步骤 1：创建两个 ref

```typescript
// 整个看板容器 - 用于注册 dropzone
const kanbanContainerRef = ref<HTMLElement | null>(null)

// 任务列表区域 - 用于计算 dropIndex（保留，但不使用）
const taskListRef = ref<HTMLElement | null>(null)
```

**说明**：

- `kanbanContainerRef`: interact.js 的 dropzone 目标
- `taskListRef`: 预留，可用于其他用途（如自动滚动边界计算）

---

### 步骤 2：在模板中添加 wrapper

```vue
<template>
  <CutePane class="kanban-column">
    <!-- 🔥 关键：在 CutePane 内部添加一个 ref wrapper -->
    <div ref="kanbanContainerRef" class="kanban-content-wrapper">
      <div class="header">
        <h2>{{ title }}</h2>
        <span class="count">{{ tasks.length }}</span>
      </div>

      <div class="add-task-input">...</div>

      <div ref="taskListRef" class="task-list">
        <TaskCard v-for="task in displayTasks" :key="task.id" :task="task" />
      </div>

      <div class="debug-info">...</div>
    </div>
  </CutePane>
</template>
```

**要点**：

1. ⚠️ **不要直接给 `<CutePane>` 加 ref**，因为它不支持 ref 转发
2. ✅ **在 CutePane 内部添加一个 `<div>` wrapper**
3. ✅ **所有内容都放在这个 wrapper 里面**

---

### 步骤 3：添加样式让 wrapper 占满

```css
/* 包装器占满整个看板 */
.kanban-content-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}
```

**作用**：

- `height: 100%` 和 `width: 100%` 确保 wrapper 占满父容器
- `display: flex` 和 `flex-direction: column` 保持原有布局

---

### 步骤 4：在 composable 中使用整个容器

```typescript
const { displayTasks, isDragging, isReceiving } = useInteractDrag({
  viewMetadata: effectiveViewMetadata,
  tasks: computed(() => effectiveTasks.value),

  // 🔥 使用整个看板容器作为 dropzone
  containerRef: kanbanContainerRef,

  draggableSelector: '.task-card-wrapper',
  onDrop: async (session) => {
    // 处理拖放
  },
})
```

---

### 步骤 5：dropIndex 自动计算

**无需修改** - `useInteractDrag` 内部会自动处理：

```typescript
// 在 drag-controller.ts 中
private calculateDropIndexForZone(
  pointerY: number,
  element: HTMLElement,  // 这是整个看板容器
  useLastIndex: boolean = false
): number {
  // 🔥 在整个容器内查找所有任务卡片
  const wrappers = Array.from(
    element.querySelectorAll('.task-card-wrapper')
  ) as HTMLElement[]

  // 🔥 空看板自动返回 0
  if (wrappers.length === 0) {
    return 0
  }

  // 根据鼠标 Y 坐标计算插入位置
  return calculateDropIndex(pointerY, wrappers, lastDropIndex)
}
```

**工作原理**：

1. `element` 是整个看板容器
2. `querySelectorAll('.task-card-wrapper')` 在容器内查找所有卡片
3. 即使卡片在 `task-list` 内部，也能被正确找到（CSS 选择器穿透）
4. 空看板返回 `0`（插入第一个位置）

---

## 🎨 完整示例

### 完整的组件代码

```vue
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useInteractDrag } from '@/composables/drag/useInteractDrag'
import CutePane from '@/components/alias/CutePane.vue'
import TaskCard from './TaskCard.vue'

const props = defineProps<{
  title: string
  viewKey: string
  tasks: TaskCard[]
}>()

// 🔥 两个 ref：一个用于 dropzone，一个预留
const kanbanContainerRef = ref<HTMLElement | null>(null)
const taskListRef = ref<HTMLElement | null>(null)

// 使用 interact 拖放系统
const { displayTasks, isDragging, isReceiving } = useInteractDrag({
  viewMetadata: computed(() => ({
    id: props.viewKey,
    type: 'status',
    label: props.title,
  })),
  tasks: computed(() => props.tasks),
  containerRef: kanbanContainerRef, // 整个看板
  draggableSelector: '.task-card-wrapper',
  onDrop: async (session) => {
    console.log('Drop:', session)
  },
})
</script>

<template>
  <CutePane class="kanban-column">
    <!-- 🔥 关键：wrapper div -->
    <div ref="kanbanContainerRef" class="kanban-content-wrapper">
      <div class="header">
        <h2>{{ title }}</h2>
        <span class="count">{{ tasks.length }}</span>
      </div>

      <div ref="taskListRef" class="task-list">
        <TaskCard
          v-for="task in displayTasks"
          :key="task.id"
          :task="task"
          class="task-card-wrapper"
        />
      </div>
    </div>
  </CutePane>
</template>

<style scoped>
.kanban-column {
  display: flex;
  flex-direction: column;
  height: 100%;
}

/* 🔥 关键：wrapper 样式 */
.kanban-content-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}

.header {
  padding: 1rem;
  border-bottom: 1px solid #ddd;
}

.task-list {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}
</style>
```

---

## 🔍 技术细节

### 1. 为什么不能直接给 CutePane 加 ref？

```vue
<!-- ❌ 这样不行 -->
<CutePane ref="kanbanContainerRef">
  ...
</CutePane>
```

**原因**：

- `CutePane` 是一个 Vue 组件，不是原生 DOM 元素
- Vue 3 的 ref 在组件上会得到组件实例，而不是 DOM 元素
- interact.js 需要真实的 DOM 元素来注册 dropzone

**解决方案**：

- 在 CutePane 内部添加一个 `<div>`
- 给这个 `<div>` 加 ref
- 或者让 CutePane 支持 ref 转发（使用 `defineExpose`）

---

### 2. 为什么 querySelector 能找到嵌套的卡片？

```typescript
// element 是 kanban-content-wrapper
const wrappers = element.querySelectorAll('.task-card-wrapper')
```

**原因**：

- `querySelectorAll` 会查找元素**内部所有匹配的后代元素**
- 不管卡片嵌套多深，只要在容器内，都能被找到

**DOM 结构**：

```
<div ref="kanbanContainerRef">           <!-- dropzone -->
  <div class="header">...</div>
  <div class="task-list">                <!-- 中间层 -->
    <div class="task-card-wrapper">...</div>  <!-- ✅ 能被找到 -->
    <div class="task-card-wrapper">...</div>  <!-- ✅ 能被找到 -->
  </div>
</div>
```

---

### 3. 空看板如何处理？

```typescript
if (wrappers.length === 0) {
  return 0 // 插入到第一个位置
}
```

**效果**：

- 空看板没有任何卡片
- `querySelectorAll` 返回空数组
- 自动返回 `dropIndex = 0`
- 任务会被插入到第一个位置

---

### 4. dropIndex 计算是否受影响？

**不受影响**，计算逻辑完全相同：

```typescript
for (let i = 0; i < wrappers.length; i++) {
  const wrapper = wrappers[i]
  const rect = wrapper.getBoundingClientRect()
  const centerY = rect.top + height / 2

  // 鼠标在这个卡片上方 → 插入到这个位置
  if (mouseY < centerY) {
    return i
  }
}

// 鼠标在所有卡片下方 → 插入到末尾
return wrappers.length
```

**说明**：

- `getBoundingClientRect()` 获取卡片的屏幕坐标
- 与容器的 DOM 结构无关
- 只要能找到卡片元素，计算就是准确的

---

## 📊 效果对比

| 维度               | 传统方案       | 新方案         |
| ------------------ | -------------- | -------------- |
| **接收区域**       | 仅 task-list   | 整个看板       |
| **空看板接收**     | 困难（区域小） | 容易（区域大） |
| **标题区**         | ❌ 不可接收    | ✅ 可接收      |
| **底部空白**       | ❌ 不可接收    | ✅ 可接收      |
| **dropIndex 精度** | 高             | 高（相同）     |
| **实现复杂度**     | 简单           | 简单           |
| **性能**           | 好             | 好（相同）     |

---

## ⚠️ 注意事项

### 1. Wrapper 必须占满父容器

```css
/* ✅ 正确 */
.kanban-content-wrapper {
  height: 100%;
  width: 100%;
}

/* ❌ 错误：高度不够，底部无法接收 */
.kanban-content-wrapper {
  height: auto; /* 只会包裹内容 */
}
```

---

### 2. 不要给 wrapper 添加 pointer-events: none

```css
/* ❌ 错误：会导致 dropzone 无法响应 */
.kanban-content-wrapper {
  pointer-events: none;
}
```

---

### 3. 确保卡片的 class 正确

```vue
<!-- ✅ 正确 -->
<TaskCard class="task-card-wrapper" />

<!-- ❌ 错误：找不到卡片 -->
<TaskCard />
<!-- 缺少 class -->
```

---

### 4. 多个看板时的 class 冲突

如果页面有多个看板，使用唯一的 draggable 选择器：

```typescript
const { displayTasks } = useInteractDrag({
  // ...
  draggableSelector: `.task-card-wrapper-${viewKey.replace(/:/g, '-')}`,
})
```

```vue
<TaskCard :class="`task-card-wrapper task-card-wrapper-${viewKey.replace(/:/g, '-')}`" />
```

---

## 🔧 调试技巧

### 1. 检查 ref 是否正确绑定

```vue
<script setup>
onMounted(() => {
  console.log('kanbanContainerRef:', kanbanContainerRef.value)
  // 应该输出 HTMLDivElement，而不是 undefined 或组件实例
})
</script>
```

---

### 2. 检查 dropzone 是否注册成功

在浏览器控制台：

```javascript
// 查看已注册的 dropzone
console.log(interactManager.getDropzones())

// 应该看到你的看板容器
```

---

### 3. 检查卡片是否能被找到

```javascript
const container = document.querySelector('.kanban-content-wrapper')
const wrappers = container.querySelectorAll('.task-card-wrapper')
console.log('Found', wrappers.length, 'cards')
```

---

### 4. 查看 dropIndex 计算

拖动时查看控制台：

```
🎯 Drag Strategy: ...
  📦 Context Data:
    Drop Index: 2  ← 这个值应该正确
```

---

## 📚 相关文档

- [interact.js 拖放系统架构](./ARCHITECTURE.md)
- [Schmitt Trigger 防抖实现](./SCHMITT_TRIGGER.md)
- [混合检测策略](./HYBRID_APPROACH.md)

---

## 🎉 总结

通过添加一个简单的 wrapper div，我们成功地将整个看板变成了有效的拖放接收区域，同时保持了：

✅ **精确的 dropIndex 计算**  
✅ **空看板正常接收**  
✅ **代码简洁清晰**  
✅ **性能无损耗**  
✅ **用户体验提升**

核心原理就是：**dropzone 范围大，但 dropIndex 计算仍然基于卡片位置**。
