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
            v-for="(item, index) in sortableTaskList"
            :key="item.id"
            class="scrollable-task-item"
            :class="{
              'is-dragging': dragState.isDragging && dragState.draggedItemId === item.id,
              'drag-over-before':
                dragState.insertPosition === 'before' && dragState.targetIndex === index,
              'drag-over-after':
                dragState.insertPosition === 'after' && dragState.targetIndex === index,
            }"
            :data-index="index"
            @pointerdown="handleItemPointerDown($event, item, index)"
          >
            <span class="task-order">{{ index + 1 }}</span>
            <span class="item-icon">📝</span>
            <span class="task-title">{{ item.title }}</span>
            <span class="task-priority" :class="`priority-${item.priority}`">
              {{ item.priority === 'high' ? '🔴' : item.priority === 'medium' ? '🟡' : '🟢' }}
            </span>
          </div>

          <!-- 拖拽预览元素 -->
          <div
            v-if="dragState.showPreview"
            class="scrollable-task-item preview-item"
            :style="{
              transform: `translateY(${dragState.previewPosition}px)`,
            }"
          >
            <span class="task-order">{{ dragState.previewItem?.newIndex || 0 }}</span>
            <span class="item-icon">📝</span>
            <span class="task-title">{{ dragState.previewItem?.title }}</span>
            <span class="task-priority" :class="`priority-${dragState.previewItem?.priority}`">
              {{
                dragState.previewItem?.priority === 'high'
                  ? '🔴'
                  : dragState.previewItem?.priority === 'medium'
                    ? '🟡'
                    : '🟢'
              }}
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
import { ref } from 'vue'
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

// 新的拖拽排序系统
const scrollableListRef = ref<HTMLElement>()

// 任务列表数据
const sortableTaskList = ref(
  Array.from({ length: 50 }, (_, i) => ({
    id: 1000 + i,
    title: `任务 ${i + 1}: ${['完成UI设计', '代码重构', '性能优化', 'bug修复', '功能测试', '文档更新', '部署上线'][i % 7]}`,
    priority: ['high', 'medium', 'low'][i % 3] as 'high' | 'medium' | 'low',
    completed: false,
  }))
)

// 拖拽状态
const dragState = ref({
  isDragging: false,
  draggedItemId: null as number | null,
  draggedItemIndex: -1,
  targetIndex: -1,
  insertPosition: 'after' as 'before' | 'after',
  showPreview: false,
  previewPosition: 0,
  previewItem: null as any,
  startY: 0,
  currentY: 0,
  itemHeight: 0,
})

// 拖拽阈值
const DRAG_THRESHOLD = 5

// 新的拖拽事件处理器
const handleItemPointerDown = (event: PointerEvent, item: any, index: number) => {
  event.preventDefault()

  const target = event.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()

  // 初始化拖拽状态
  dragState.value = {
    isDragging: false,
    draggedItemId: item.id,
    draggedItemIndex: index,
    targetIndex: -1,
    insertPosition: 'after',
    showPreview: false,
    previewPosition: 0,
    previewItem: { ...item },
    startY: event.clientY,
    currentY: event.clientY,
    itemHeight: rect.height,
  }

  // 添加全局事件监听器
  document.addEventListener('pointermove', handlePointerMove)
  document.addEventListener('pointerup', handlePointerUp)
}

const handlePointerMove = (event: PointerEvent) => {
  const deltaY = event.clientY - dragState.value.startY

  // 检查是否达到拖拽阈值
  if (!dragState.value.isDragging && Math.abs(deltaY) > DRAG_THRESHOLD) {
    dragState.value.isDragging = true
    dragState.value.showPreview = true
  }

  if (!dragState.value.isDragging) return

  dragState.value.currentY = event.clientY

  // 计算目标位置
  updateDropTarget(event)
}

const updateDropTarget = (event: PointerEvent) => {
  if (!scrollableListRef.value) return

  const listRect = scrollableListRef.value.getBoundingClientRect()
  const mouseY = event.clientY - listRect.top + scrollableListRef.value.scrollTop
  const itemHeight = dragState.value.itemHeight

  // 计算鼠标位置对应的项目索引
  let targetIndex = Math.floor(mouseY / itemHeight)
  targetIndex = Math.max(0, Math.min(targetIndex, sortableTaskList.value.length - 1))

  // 确定插入位置（before 或 after）
  const itemY = targetIndex * itemHeight
  const mouseRelativeY = mouseY - itemY
  const insertPosition = mouseRelativeY < itemHeight / 2 ? 'before' : 'after'

  // 更新状态
  dragState.value.targetIndex = targetIndex
  dragState.value.insertPosition = insertPosition

  // 计算预览位置
  let previewY = targetIndex * itemHeight
  if (insertPosition === 'after') {
    previewY += itemHeight
  }
  dragState.value.previewPosition = previewY

  // 更新预览项的索引
  let newIndex = targetIndex
  if (insertPosition === 'after') {
    newIndex++
  }
  // 如果拖拽的项在目标位置之前，需要调整索引
  if (dragState.value.draggedItemIndex < newIndex) {
    newIndex--
  }
  dragState.value.previewItem.newIndex = newIndex + 1
}

const handlePointerUp = () => {
  // 清理事件监听器
  document.removeEventListener('pointermove', handlePointerMove)
  document.removeEventListener('pointerup', handlePointerUp)

  // 如果正在拖拽，执行排序
  if (dragState.value.isDragging) {
    performSort()
  }

  // 重置状态
  resetDragState()
}

const performSort = () => {
  const { draggedItemIndex, targetIndex, insertPosition } = dragState.value

  if (draggedItemIndex === -1 || targetIndex === -1) return

  const items = [...sortableTaskList.value]
  const draggedItem = items[draggedItemIndex]

  if (!draggedItem) return

  // 移除拖拽的项
  items.splice(draggedItemIndex, 1)

  // 计算新的插入位置
  let insertIndex = targetIndex
  if (insertPosition === 'after') {
    insertIndex++
  }
  // 如果拖拽的项在目标位置之前，插入位置需要减1
  if (draggedItemIndex < insertIndex) {
    insertIndex--
  }

  // 插入到新位置
  items.splice(insertIndex, 0, draggedItem)

  // 更新列表
  sortableTaskList.value = items
}

const resetDragState = () => {
  dragState.value = {
    isDragging: false,
    draggedItemId: null,
    draggedItemIndex: -1,
    targetIndex: -1,
    insertPosition: 'after',
    showPreview: false,
    previewPosition: 0,
    previewItem: null,
    startY: 0,
    currentY: 0,
    itemHeight: 0,
  }
}

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
  position: relative; /* 为预览元素提供定位上下文 */
}

.scrollable-task-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
  cursor: grab;
  transition: all 0.2s ease;
  background: white;
  user-select: none;
}

.scrollable-task-item:active {
  cursor: grabbing;
}

.scrollable-task-item:hover {
  background: #f8f9fa;
}

/* 拖拽中的元素样式 */
.scrollable-task-item.is-dragging {
  opacity: 0.3;
  transform: scale(0.95);
  pointer-events: none;
  transition: none !important;
}

/* 拖拽目标指示器 */
.scrollable-task-item.drag-over-before {
  border-top: 3px solid #2196f3;
  border-radius: 8px 8px 0 0;
}

.scrollable-task-item.drag-over-after {
  border-bottom: 3px solid #2196f3;
  border-radius: 0 0 8px 8px;
}

/* 预览元素样式 */
.scrollable-task-item.preview-item {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  opacity: 0.8;
  background: #e3f2fd !important;
  border: 2px solid #2196f3 !important;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgb(33 150 243 / 30%);
  pointer-events: none;
  z-index: 1000;
  transition: none !important;
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
