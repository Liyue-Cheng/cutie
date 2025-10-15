# 整个看板作为接收区域 - 实现总结

## 🎯 实现方法（3 步）

### 1️⃣ 创建两个 ref

```typescript
const kanbanContainerRef = ref<HTMLElement | null>(null) // dropzone
const taskListRef = ref<HTMLElement | null>(null) // 预留
```

### 2️⃣ 在模板中添加 wrapper

```vue
<template>
  <CutePane class="kanban-column">
    <div ref="kanbanContainerRef" class="kanban-content-wrapper">
      <!-- 所有内容 -->
    </div>
  </CutePane>
</template>

<style>
.kanban-content-wrapper {
  display: flex;
  flex-direction: column;
  height: 100%;
  width: 100%;
}
</style>
```

### 3️⃣ 传入整个容器

```typescript
useInteractDrag({
  containerRef: kanbanContainerRef, // 使用整个容器
  // ...
})
```

---

## ✅ 为什么有效？

- **Dropzone 范围**: 整个看板都可以接收拖放
- **DropIndex 计算**: `querySelectorAll('.task-card-wrapper')` 穿透查找所有卡片
- **空看板**: 自动返回 `dropIndex = 0`

---

## ⚠️ 关键点

1. **不要**直接给 `<CutePane ref="xxx">` 加 ref（组件不转发）
2. **必须**在内部添加 `<div ref="xxx">` wrapper
3. **必须**让 wrapper 占满 `height: 100%; width: 100%`

---

完整文档：[FULL_DROPZONE_GUIDE.md](./FULL_DROPZONE_GUIDE.md)
