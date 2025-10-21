<template>
  <div class="home-view">
    <!-- 左栏 -->
    <div class="left-column" :style="{ width: leftPaneWidth + '%' }">
      <TwoRowLayout>
        <template #top>
          <div class="column-header">
            <h2>任务列表</h2>
          </div>
        </template>
        <template #bottom>
          <div class="task-list">
            <!-- 今日任务栏 -->
            <TaskBar
              title="今日任务"
              :tasks="[
                {
                  id: '1',
                  title: '完成项目文档',
                  note: '需要更新 API 文档和用户指南',
                  subtasks: [
                    { id: '1-1', title: '更新 API 文档', completed: true },
                    { id: '1-2', title: '编写用户指南', completed: false },
                    { id: '1-3', title: '添加示例代码', completed: false },
                  ],
                  completed: false,
                },
                {
                  id: '2',
                  title: '准备团队会议',
                  completed: false,
                },
              ]"
            />

            <!-- 进行中任务栏 -->
            <TaskBar
              title="进行中"
              :tasks="[
                {
                  id: '3',
                  title: '代码审查',
                  note: '审查 PR #123 和 PR #124',
                  subtasks: [
                    { id: '3-1', title: '审查 PR #123', completed: true },
                    { id: '3-2', title: '审查 PR #124', completed: false },
                  ],
                  completed: false,
                },
              ]"
            />

            <!-- 已完成任务栏 -->
            <TaskBar
              title="已完成"
              :tasks="[
                {
                  id: '4',
                  title: '已完成的任务示例',
                  note: '这是一个已完成的任务',
                  completed: true,
                },
              ]"
              :default-collapsed="true"
            />
          </div>
        </template>
      </TwoRowLayout>
    </div>

    <!-- 可拖动的分割线 -->
    <div class="divider" @mousedown="startDragging" @dblclick="resetPaneWidth"></div>

    <!-- 右栏 -->
    <div class="right-column">
      <TwoRowLayout>
        <template #top>
          <div class="column-header">
            <h2>日历</h2>
          </div>
        </template>
        <template #bottom>
          <div class="column-content">
            <p class="placeholder-text">Calendar will be here</p>
          </div>
        </template>
      </TwoRowLayout>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import TwoRowLayout from '@/components/templates/TwoRowLayout.vue'
import TaskBar from '@/components/parts/TaskBar.vue'
import { useRegisterStore } from '@/stores/register'
import { logger, LogTags } from '@/infra/logging/logger'

const registerStore = useRegisterStore()

// 初始化
onMounted(() => {
  logger.info(LogTags.VIEW_HOME, 'Initializing brand new home view...')
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_VIEW, 'home')
})

// ==================== 可拖动分割线逻辑 ====================
const leftPaneWidth = ref(33.33) // 默认比例 1:2，左栏占 33.33%
const isDragging = ref(false)

function startDragging(e: MouseEvent) {
  isDragging.value = true
  document.addEventListener('mousemove', onDragging)
  document.addEventListener('mouseup', stopDragging)
  e.preventDefault()
}

function onDragging(e: MouseEvent) {
  if (!isDragging.value) return

  const container = document.querySelector('.home-view') as HTMLElement
  if (!container) return

  const containerRect = container.getBoundingClientRect()
  const containerWidth = containerRect.width
  const mouseX = e.clientX - containerRect.left

  // 计算新的左栏宽度百分比
  let newWidth = (mouseX / containerWidth) * 100

  // 限制最小和最大宽度（20% - 80%）
  newWidth = Math.max(20, Math.min(80, newWidth))

  leftPaneWidth.value = newWidth
}

function stopDragging() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)
}

// 双击重置为默认比例
function resetPaneWidth() {
  leftPaneWidth.value = 33.33
}

// 清理事件监听器
onBeforeUnmount(() => {
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)
})
</script>

<style scoped>
.home-view {
  width: 100%;
  height: 100%;
  display: flex;
  overflow: hidden;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
}

/* 左右栏 */
.left-column,
.right-column {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: transparent;
}

.left-column {
  flex-shrink: 0;
  position: relative;
}

.right-column {
  flex: 1;
  min-width: 0;
  position: relative;
}

/* 分割线 */
.divider {
  width: 1px;
  height: 100%;
  background-color: var(--color-border-default);
  cursor: col-resize;
  flex-shrink: 0;
  transition: background-color 0.2s;
  position: relative;
  z-index: 10;
}

/* 扩大可点击区域 */
.divider::before {
  content: '';
  position: absolute;
  inset: 0 -4px;
  cursor: col-resize;
}

.divider:hover {
  background-color: var(--color-border-hover, var(--color-border-default));
}

/* 列头部 */
.column-header {
  width: 100%;
  display: flex;
  align-items: center;
}

.column-header h2 {
  font-size: 1.8rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

/* 任务列表 */
.task-list {
  padding: 1.6rem;
  height: 100%;
  overflow-y: auto;
}

/* 列内容 */
.column-content {
  padding: 2rem;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.placeholder-text {
  font-size: 1.6rem;
  color: var(--color-text-secondary);
}
</style>
