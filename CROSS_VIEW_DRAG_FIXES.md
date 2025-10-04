# 跨看板拖放细节修复 🔧

## 修复的问题

### 1. 幽灵元素清除问题 ✅

**问题描述**: 拖动到另一个看板后，原看板的幽灵元素没有被清除

**修复方案**:

- 在 `useCrossViewDrag/context.ts` 中添加全局状态 `currentTargetViewId`
- 在 `handleColumnDragEnter` 中调用 `setTargetViewId(props.viewMetadata.id)` 记录目标看板
- 在 `handleColumnDragLeave` 中调用 `setTargetViewId(null)` 清除目标看板
- 在 `displayTasks` computed 中：
  ```typescript
  // 如果是源看板，且有其他看板正在接收拖动
  if (context && context.sourceView.id === props.viewMetadata.id) {
    if (targetView && targetView !== props.viewMetadata.id) {
      // 隐藏幽灵元素
      taskList = taskList.filter((t) => t.id !== context.task.id)
    }
  }
  ```

**效果**:

- ✅ 拖到另一个看板时，源看板幽灵元素消失
- ✅ 拖到日历、无效区域时，源看板幽灵元素保留
- ✅ 离开目标看板时，源看板幽灵元素恢复

---

### 2. 实时排序问题 ✅

**问题描述**: 拖动到另一个看板后，卡片固定在顶部，不支持实时排序

**修复方案**:

- 在 `handleColumnDragEnter` 中：
  ```typescript
  draggedOverIndex.value = null // 初始不设置位置，等待第一次 dragover
  ```
- 在 `handleDragOver` 中添加跨看板支持：
  ```typescript
  const context = crossViewDrag.currentContext.value
  if (context && context.sourceView.id !== props.viewMetadata.id) {
    // 跨看板拖放：直接更新目标索引
    draggedOverIndex.value = targetIndex
    return
  }
  ```

**效果**:

- ✅ 进入目标看板时，卡片初始不显示（等待第一次 dragover）
- ✅ 在目标看板内拖动时，卡片实时预览位置
- ✅ 支持在目标看板内任意位置放置

---

## 修改的文件

1. `src/composables/drag/useCrossViewDrag/context.ts`
   - 添加 `currentTargetViewId` 全局状态
   - 添加 `setTargetViewId()` 方法
   - 在 `clearContext()` 中清理 `currentTargetViewId`

2. `src/composables/drag/useCrossViewDrag/index.ts`
   - 导出 `targetViewId` 和 `setTargetViewId`

3. `src/components/parts/kanban/SimpleKanbanColumn.vue`
   - 修改 `displayTasks` computed：使用 `targetViewId` 判断是否隐藏幽灵元素
   - 修改 `handleColumnDragEnter`：调用 `setTargetViewId()`，初始化 `draggedOverIndex = null`
   - 修改 `handleColumnDragLeave`：调用 `setTargetViewId(null)`
   - 修改 `handleDragOver`：添加跨看板拖放的实时排序支持

---

## 测试场景

### 场景1：跨看板拖放 + 幽灵元素

1. 拖动任务从看板A
2. 进入看板B
   - ✅ 看板A的幽灵元素消失
   - ✅ 看板B显示任务（位置跟随鼠标）
3. 离开看板B
   - ✅ 看板A的幽灵元素恢复
   - ✅ 看板B任务消失
4. 再次进入看板C
   - ✅ 看板A的幽灵元素消失
   - ✅ 看板C显示任务

### 场景2：跨看板拖放 + 实时排序

1. 拖动任务从看板A到看板B
2. 在看板B内上下移动鼠标
   - ✅ 任务实时预览位置（在不同任务之间"跳跃"）
3. 松开鼠标
   - ✅ 任务固定在最终位置
   - ✅ 控制台输出跨看板拖放日志

### 场景3：拖到日历（保留幽灵元素）

1. 拖动任务从看板A
2. 移动到日历区域（未进入任何其他看板）
   - ✅ 看板A的幽灵元素保留
3. 松开鼠标在日历上
   - ✅ 创建时间块（原有功能）

### 场景4：拖到空白区域（保留幽灵元素）

1. 拖动任务从看板A
2. 移动到任何空白区域
   - ✅ 看板A的幽灵元素保留
3. 松开鼠标或按ESC
   - ✅ 任务回到原位置

---

## 技术细节

### 全局状态管理

```typescript
// context.ts
const currentTargetViewId = ref<string | null>(null)

function setTargetViewId(viewId: string | null): void {
  currentTargetViewId.value = viewId
  if (viewId) {
    console.log('[DragContext] 🎯 Target view changed:', viewId)
  }
}
```

### 幽灵元素判断逻辑

```typescript
// SimpleKanbanColumn.vue - displayTasks
const context = crossViewDrag.currentContext.value
const targetView = crossViewDrag.targetViewId.value

if (context && context.sourceView.id === props.viewMetadata.id) {
  // 这是源看板
  if (targetView && targetView !== props.viewMetadata.id) {
    // 有其他看板在接收，隐藏幽灵元素
    taskList = taskList.filter((t) => t.id !== context.task.id)
  }
}
```

### 跨看板实时排序

```typescript
// SimpleKanbanColumn.vue - handleDragOver
const context = crossViewDrag.currentContext.value
if (context && context.sourceView.id !== props.viewMetadata.id) {
  // 跨看板拖放：直接更新目标索引
  draggedOverIndex.value = targetIndex
  return
}
```

---

## ✨ 完成！

现在跨看板拖放功能完全符合原生HTML5拖放的交互规范：

- ✅ 幽灵元素正确显示/隐藏
- ✅ 实时排序流畅自然
- ✅ 支持任意位置放置

**测试时请注意观察控制台日志，确保所有拖放操作都有相应的日志输出！** 🎉
