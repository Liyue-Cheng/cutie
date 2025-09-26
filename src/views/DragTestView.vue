<template>
  <div class="drag-test-view">
    <h1>Vue-Draxis 拖放测试</h1>

    <div class="test-section">
      <h2>场景1: 拖动现有项目</h2>
      <div class="drag-source-area">
        <h3>拖拽源（可拖拽的项目）</h3>
        <div class="draggable-items">
          <div
            v-for="item in draggableItems"
            :key="item.id"
            v-c-draggable="{ data: item, dataType: 'task' }"
            class="draggable-item"
          >
            <span class="item-icon">📋</span>
            {{ item.title }}
          </div>
        </div>
      </div>

      <div class="drop-zones">
        <h3>放置区域</h3>
        <div class="drop-zone-container">
          <div
            v-c-droppable="{
              acceptedDataTypes: ['task'],
              onDrop: handleDrop,
              onDragEnter: handleDragEnter,
              onDragLeave: handleDragLeave,
            }"
            class="drop-zone"
            :class="{ 'drop-zone-active': isDropZoneActive }"
          >
            <h4>任务放置区</h4>
            <p v-if="droppedItems.length === 0" class="drop-hint">将任务拖拽到这里</p>
            <div v-else class="dropped-items">
              <div v-for="item in droppedItems" :key="`dropped-${item.id}`" class="dropped-item">
                <span class="item-icon">✅</span>
                {{ item.title }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="test-section">
      <h2>场景2: 程序化创建并拖拽</h2>
      <div class="creator-area">
        <h3>工具栏（点击创建新任务）</h3>
        <button @click="createNewTask" class="create-button">
          <span class="item-icon">➕</span>
          创建新任务
        </button>
      </div>
    </div>

    <div class="test-section">
      <h2>拖拽状态信息</h2>
      <div class="debug-info">
        <p><strong>是否正在拖拽:</strong> {{ dragManager.state.value.isDragging ? '是' : '否' }}</p>
        <p><strong>拖拽数据类型:</strong> {{ dragManager.state.value.dataType || '无' }}</p>
        <p>
          <strong>鼠标位置:</strong> X: {{ dragManager.state.value.currentPosition.x }}, Y:
          {{ dragManager.state.value.currentPosition.y }}
        </p>
        <p><strong>拖拽数据:</strong> {{ JSON.stringify(dragManager.state.value.dragData) }}</p>
      </div>
    </div>

    <!-- 占位空间，确保页面有足够高度需要滚动 -->
    <div class="spacer-section">
      <p>滚动到下方查看更多放置区域...</p>
    </div>

    <!-- 可滚动任务列表区域 -->
    <div class="test-section">
      <h2>可滚动任务列表（支持排序）</h2>
      <p class="section-description">
        这个列表有滚动条，拖拽任务到列表边缘会自动滚动，支持任务排序
      </p>

      <div class="scrollable-list-container">
        <div class="scrollable-task-list" ref="scrollableListRef">
          <div
            v-for="item in displayScrollableTaskList"
            :key="item.isPreview ? `preview-${item.id}` : item.id"
            v-c-draggable="!item.isPreview ? { data: item, dataType: 'scrollable-task' } : null"
            v-c-droppable="
              !item.isPreview
                ? {
                    acceptedDataTypes: ['scrollable-task', 'task'],
                    onDrop: (data: any) => handleScrollableListDrop(data, item.displayIndex),
                    onDragEnter: () => handleScrollableListDragEnter(item.displayIndex),
                    onDragOver: (_data: any, _dataType: string, event?: PointerEvent) =>
                      event && handleScrollableListDragOver(event, item.displayIndex),
                    onDragLeave: handleScrollableListDragLeave,
                  }
                : null
            "
            class="scrollable-task-item"
            :class="{
              'is-preview': item.isPreview,
              'is-hidden': item.isHidden,
            }"
          >
            <span class="task-order">{{ item.displayIndex + 1 }}</span>
            <span class="item-icon">📝</span>
            <span class="task-title">{{ item.title }}</span>
            <span class="task-priority" :class="`priority-${item.priority}`">
              {{ item.priority === 'high' ? '🔴' : item.priority === 'medium' ? '🟡' : '🟢' }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- 下方的放置区域（需要滚动才能看到） -->
    <div class="test-section bottom-section">
      <h2>下方任务收集区</h2>
      <p class="section-description">将任务拖拽到页面底部边缘，页面会自动滚动显示此区域</p>

      <div class="drop-zones">
        <div class="drop-zone-container">
          <div
            v-c-droppable="{
              acceptedDataTypes: ['task', 'scrollable-task'],
              onDrop: handleBottomDrop,
              onDragEnter: handleBottomDragEnter,
              onDragLeave: handleBottomDragLeave,
            }"
            class="drop-zone bottom-drop-zone"
            :class="{ 'drop-zone-active': isBottomDropZoneActive }"
          >
            <h4>🎯 底部任务收集区</h4>
            <p v-if="bottomDroppedItems.length === 0" class="drop-hint">
              拖拽到页面底部边缘，页面会自动滚动到这里
            </p>
            <div v-else class="dropped-items">
              <div
                v-for="item in bottomDroppedItems"
                :key="`bottom-dropped-${item.id}`"
                class="dropped-item bottom-dropped-item"
              >
                <span class="item-icon">🎯</span>
                {{ item.title }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { dragManager, useDragCreator } from '@/composables/drag'
import type { DragData } from '@/composables/drag'
import NewTaskGhost from '@/components/NewTaskGhost.vue'

// 可拖拽的项目数据
const draggableItems = ref([
  { id: 1, title: '任务 1: 完成项目文档' },
  { id: 2, title: '任务 2: 代码审查' },
  { id: 3, title: '任务 3: 单元测试' },
  { id: 4, title: '任务 4: 部署准备' },
])

// 已放置的项目
const droppedItems = ref<any[]>([])
const bottomDroppedItems = ref<any[]>([])

// 放置区状态
const isDropZoneActive = ref(false)
const isBottomDropZoneActive = ref(false)

// 可滚动列表相关
const scrollableListRef = ref<HTMLElement>()
const scrollableListDragOverIndex = ref(-1)
const scrollableListInsertPosition = ref<'before' | 'after'>('after')
const scrollableListDragSourceIndex = ref(-1)
const scrollableListPreviewData = ref<any>(null)

// 生成大量任务数据用于滚动测试
const scrollableTaskList = ref(
  Array.from({ length: 50 }, (_, i) => ({
    id: 1000 + i,
    title: `任务 ${i + 1}: ${['完成UI设计', '代码重构', '性能优化', 'bug修复', '功能测试', '文档更新', '部署上线'][i % 7]}`,
    priority: ['high', 'medium', 'low'][i % 3] as 'high' | 'medium' | 'low',
    completed: false,
  }))
)

// 计算带预览的显示列表
const displayScrollableTaskList = computed(() => {
  const sourceIndex = scrollableListDragSourceIndex.value
  const targetIndex = scrollableListDragOverIndex.value
  const hasPreview = targetIndex !== -1 && scrollableListPreviewData.value
  const insertBefore = scrollableListInsertPosition.value === 'before'

  if (!hasPreview) {
    // 没有预览时，只标记隐藏源元素（如果在拖动中）
    return scrollableTaskList.value.map((item, index) => ({
      ...item,
      isPreview: false,
      isHidden: sourceIndex === index && sourceIndex !== -1,
      displayIndex: index,
    }))
  }

  const result: any[] = []
  let displayIndex = 0

  for (let i = 0; i < scrollableTaskList.value.length; i++) {
    const item = scrollableTaskList.value[i]
    const isSourceItem = i === sourceIndex

    // 在目标位置之前插入预览元素
    if (insertBefore && i === targetIndex) {
      result.push({
        ...scrollableListPreviewData.value,
        isPreview: true,
        isHidden: false,
        displayIndex: displayIndex++,
      })
    }

    // 添加当前元素（源元素设为隐藏）
    result.push({
      ...item,
      isPreview: false,
      isHidden: isSourceItem,
      displayIndex: displayIndex++,
    })

    // 在目标位置之后插入预览元素
    if (!insertBefore && i === targetIndex) {
      result.push({
        ...scrollableListPreviewData.value,
        isPreview: true,
        isHidden: false,
        displayIndex: displayIndex++,
      })
    }
  }

  return result
})

// 拖放事件处理
const handleDrop = (data: DragData) => {
  console.log('放置事件:', data)

  // 添加到已放置列表（如果还没有）
  const existingItem = droppedItems.value.find((item) => item.id === data.id)
  if (!existingItem) {
    droppedItems.value.push({ ...data })
  }

  isDropZoneActive.value = false
}

const handleDragEnter = (data: DragData) => {
  console.log('拖拽进入:', data)
  isDropZoneActive.value = true
}

const handleDragLeave = () => {
  console.log('拖拽离开')
  isDropZoneActive.value = false
}

// 底部放置区事件处理
const handleBottomDrop = (data: DragData) => {
  console.log('底部放置事件:', data)

  // 添加到底部已放置列表（如果还没有）
  const existingItem = bottomDroppedItems.value.find((item) => item.id === data.id)
  if (!existingItem) {
    bottomDroppedItems.value.push({ ...data })
  }

  isBottomDropZoneActive.value = false
}

const handleBottomDragEnter = (data: DragData) => {
  console.log('底部拖拽进入:', data)
  isBottomDropZoneActive.value = true
}

const handleBottomDragLeave = () => {
  console.log('底部拖拽离开')
  isBottomDropZoneActive.value = false
}

// 可滚动列表事件处理
const handleScrollableListDrop = (data: DragData, targetIndex: number) => {
  console.log('可滚动列表放置:', data, '目标索引:', targetIndex)

  // 如果是从其他地方拖入的新任务
  if (data.dataType !== 'scrollable-task') {
    // 检查是否已经存在相同的任务（避免重复）
    const existingTask = scrollableTaskList.value.find((item) => item.id === data.id)
    if (!existingTask) {
      const newTask = {
        id: data.id || Date.now(),
        title: data.title || `新任务: ${data.dataType}`,
        priority: 'medium' as const,
        completed: false,
      }

      // 根据预览位置插入
      const actualIndex =
        scrollableListInsertPosition.value === 'before' ? targetIndex : targetIndex + 1
      scrollableTaskList.value.splice(actualIndex, 0, newTask)
    }
  } else {
    // 如果是列表内部的排序
    const realSourceIndex = scrollableTaskList.value.findIndex((item) => item.id === data.id)

    if (realSourceIndex !== -1) {
      console.log('内部排序:', {
        realSourceIndex,
        targetIndex,
        insertPosition: scrollableListInsertPosition.value,
      })

      // 移动任务到新位置
      const movedItem = scrollableTaskList.value[realSourceIndex]
      if (movedItem) {
        // 先移除源元素
        scrollableTaskList.value.splice(realSourceIndex, 1)

        // 计算插入位置
        let insertIndex = targetIndex
        if (scrollableListInsertPosition.value === 'after') {
          insertIndex = realSourceIndex < targetIndex ? targetIndex : targetIndex + 1
        } else {
          insertIndex = realSourceIndex < targetIndex ? targetIndex - 1 : targetIndex
        }

        // 确保插入索引不超出范围
        insertIndex = Math.max(0, Math.min(insertIndex, scrollableTaskList.value.length))

        // 插入到新位置
        scrollableTaskList.value.splice(insertIndex, 0, movedItem)

        console.log('排序完成:', {
          insertIndex,
          newLength: scrollableTaskList.value.length,
          newOrder: scrollableTaskList.value.map((item) => item.title),
        })
      }
    }
  }

  // 清理状态
  clearScrollableListDragState()
}

const handleScrollableListDragEnter = (index: number) => {
  if (scrollableListDragOverIndex.value !== index) {
    scrollableListDragOverIndex.value = index
    scrollableListInsertPosition.value = 'after'
  }
}

const handleScrollableListDragOver = (event: PointerEvent, index: number) => {
  // 根据鼠标在元素中的位置决定插入位置
  const element = event.currentTarget as HTMLElement
  const rect = element.getBoundingClientRect()
  const mouseY = event.clientY
  const elementMiddle = rect.top + rect.height / 2
  const newInsertPosition = mouseY < elementMiddle ? 'before' : 'after'

  // 只有当位置真正发生变化时才更新状态
  if (
    scrollableListDragOverIndex.value !== index ||
    scrollableListInsertPosition.value !== newInsertPosition
  ) {
    scrollableListDragOverIndex.value = index
    scrollableListInsertPosition.value = newInsertPosition
  }
}

const handleScrollableListDragLeave = () => {
  // 延迟清理，避免在相邻元素间移动时闪烁
  setTimeout(() => {
    scrollableListDragOverIndex.value = -1
    scrollableListInsertPosition.value = 'after'
  }, 100) // 增加延迟时间，减少闪烁
}

const clearScrollableListDragState = () => {
  scrollableListDragOverIndex.value = -1
  scrollableListInsertPosition.value = 'after'
  scrollableListDragSourceIndex.value = -1
  scrollableListPreviewData.value = null
}

// 程序化拖拽创建器
const taskCreator = useDragCreator({
  createData: () => {
    const timestamp = Date.now()
    return {
      id: timestamp,
      title: `新任务 ${new Date().toLocaleTimeString()}`,
      createdAt: new Date().toISOString(),
    }
  },
  dataType: 'task',
  ghostComponent: NewTaskGhost,
  ghostProps: (data) => ({
    // 动态生成属性，基于创建的数据
    title: data.title,
    id: data.id,
    createdAt: data.createdAt,
  }),
})

const createNewTask = (event: MouseEvent) => {
  console.log('创建新任务')
  taskCreator.startDragFromEvent(event)
}

// 监听拖拽状态变化
watch(
  () => dragManager.state.value.isDragging,
  (isDragging, wasIsDragging) => {
    if (isDragging && !wasIsDragging) {
      // 拖拽开始
      const dragData = dragManager.state.value.dragData
      const dataType = dragManager.state.value.dataType

      if (dataType === 'scrollable-task' && dragData) {
        // 如果是可滚动列表内的任务开始拖拽
        const sourceIndex = scrollableTaskList.value.findIndex((item) => item.id === dragData.id)
        if (sourceIndex !== -1) {
          scrollableListDragSourceIndex.value = sourceIndex
          scrollableListPreviewData.value = { ...dragData }
        }
      }
    } else if (!isDragging && wasIsDragging) {
      // 拖拽结束
      clearScrollableListDragState()
    }
  }
)
</script>

<style scoped>
.drag-test-view {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.test-section {
  margin-bottom: 40px;
  padding: 20px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  background: #fafafa;
}

.test-section h2 {
  margin-top: 0;
  color: #333;
  border-bottom: 2px solid #007acc;
  padding-bottom: 8px;
}

/* 拖拽源区域 */
.drag-source-area {
  margin-bottom: 30px;
}

.draggable-items {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 12px;
}

.draggable-item {
  padding: 12px 16px;
  background: linear-gradient(135deg, #007acc, #005a9e);
  color: white;
  border-radius: 8px;
  cursor: move;
  user-select: none;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 10px;
  box-shadow: 0 2px 8px rgb(0 0 0 / 15%);
  border: 1px solid rgb(255 255 255 / 20%);
  font-weight: 500;
  min-width: 180px;
}

.draggable-item:hover {
  background: linear-gradient(135deg, #005a9e, #004080);
  transform: translateY(-2px);
  box-shadow: 0 6px 16px rgb(0 0 0 / 25%);
}

/* 放置区域 */
.drop-zones {
  margin-top: 20px;
}

.drop-zone-container {
  margin-top: 12px;
}

.drop-zone {
  min-height: 150px;
  border: 2px dashed #ccc;
  border-radius: 8px;
  padding: 20px;
  text-align: center;
  transition: all 0.3s ease;
  background: #f9f9f9;
}

.drop-zone-active {
  border-color: #007acc;
  background: #e3f2fd;
  border-style: solid;
}

.drop-zone:global(.drag-valid-target) {
  border-color: #4caf50;
  background: #e8f5e8;
}

.drop-zone:global(.drag-over) {
  border-color: #ff9800;
  background: #fff3e0;
  transform: scale(1.02);
}

.drop-hint {
  color: #666;
  font-style: italic;
  margin: 0;
}

.dropped-items {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
}

.dropped-item {
  padding: 8px 12px;
  background: #4caf50;
  color: white;
  border-radius: 4px;
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 创建器区域 */
.creator-area {
  text-align: center;
}

.create-button {
  padding: 12px 24px;
  background: #ff9800;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  transition: all 0.2s ease;
  box-shadow: 0 2px 4px rgb(0 0 0 / 10%);
}

.create-button:hover {
  background: #f57c00;
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgb(0 0 0 / 15%);
}

/* 调试信息 */
.debug-info {
  background: #f5f5f5;
  padding: 16px;
  border-radius: 4px;
  font-family: 'Courier New', monospace;
  font-size: 14px;
}

.debug-info p {
  margin: 8px 0;
}

.item-icon {
  font-size: 16px;
}

/* 占位空间 */
.spacer-section {
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  color: #666;
  font-size: 18px;
  font-style: italic;
}

/* 可滚动列表样式 */
.scrollable-list-container {
  margin-top: 20px;
  border: 1px solid #ddd;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgb(0 0 0 / 10%);
}

.scrollable-task-list {
  max-height: 400px;
  overflow-y: auto;
  background: #fff;
}

.scrollable-task-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
  cursor: move;
  transition: all 0.2s ease;
  background: white;
  user-select: none;
}

.scrollable-task-item:hover {
  background: #f8f9fa;
}

.scrollable-task-item.drag-over {
  background: #e3f2fd;
  border-color: #2196f3;
}

.scrollable-task-item.insert-before {
  border-top: 3px solid #2196f3;
}

.scrollable-task-item.insert-after {
  border-bottom: 3px solid #2196f3;
}

.scrollable-task-item.is-preview {
  opacity: 0.6;
  background: #e3f2fd !important;
  border: 2px dashed #2196f3 !important;
  transform: scale(0.98);
  transition: none; /* 移除过渡动画，减少闪烁 */
  pointer-events: none; /* 防止鼠标事件干扰 */
}

.scrollable-task-item.is-hidden {
  opacity: 0;
  transform: scale(0.95);
  transition:
    opacity 0.15s ease,
    transform 0.15s ease; /* 只对关键属性添加过渡 */

  pointer-events: none;
}

.task-order {
  min-width: 30px;
  text-align: center;
  font-weight: bold;
  color: #666;
  background: #f5f5f5;
  border-radius: 50%;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
}

.task-title {
  flex: 1;
  font-weight: 500;
}

.task-priority {
  font-size: 16px;
}

.priority-high {
  animation: pulse-red 2s infinite;
}

.priority-medium {
  opacity: 0.8;
}

.priority-low {
  opacity: 0.6;
}

@keyframes pulse-red {
  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.6;
  }
}

/* 底部区域 */
.bottom-section {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  margin-bottom: 0;
}

.bottom-section h2 {
  color: white;
  border-bottom-color: rgb(255 255 255 / 30%);
}

.section-description {
  color: rgb(255 255 255 / 80%);
  font-style: italic;
  margin-bottom: 20px;
}

/* 底部放置区样式 */
.bottom-drop-zone {
  background: rgb(255 255 255 / 10%);
  border-color: rgb(255 255 255 / 30%);
  color: white;
}

.bottom-drop-zone.drop-zone-active {
  border-color: #ffd700;
  background: rgb(255 215 0 / 20%);
}

.bottom-drop-zone:global(.drag-valid-target) {
  border-color: #0f8;
  background: rgb(0 255 136 / 20%);
}

.bottom-drop-zone:global(.drag-over) {
  border-color: #ff6b6b;
  background: rgb(255 107 107 / 20%);
  transform: scale(1.02);
}

.bottom-dropped-item {
  background: #ffd700;
  color: #333;
}

/* 全局拖拽样式 */
:global(.draggable) {
  cursor: move !important;
}

:global(.droppable) {
  transition: all 0.2s ease;
}
</style>
