<template>
  <div class="upcoming-list">
    <!-- 标题栏（不可折叠） -->
    <div class="upcoming-header">
      <div class="header-left">
        <h3 class="upcoming-title">即将到期</h3>
        <span class="task-count">{{ allTasks.length }}</span>
      </div>
    </div>

    <!-- 内容区 -->
    <div class="upcoming-content">
      <!-- 逾期 -->
      <div v-if="groupedTasks.overdue.length > 0" class="task-group">
        <div class="group-header" @click="toggleGroup('overdue')">
          <span>逾期</span>
          <CuteIcon
            name="ChevronDown"
            :size="14"
            class="group-collapse-icon"
            :class="{ rotated: collapsedGroups.overdue }"
          />
        </div>
        <div v-if="!collapsedGroups.overdue" class="group-tasks">
          <TaskStrip
            v-for="task in groupedTasks.overdue"
            :key="task.id"
            :task="task"
            view-key="misc::upcoming"
          />
        </div>
      </div>

      <!-- 今天 -->
      <div v-if="groupedTasks.today.length > 0" class="task-group">
        <div class="group-header" @click="toggleGroup('today')">
          <span>今天</span>
          <CuteIcon
            name="ChevronDown"
            :size="14"
            class="group-collapse-icon"
            :class="{ rotated: collapsedGroups.today }"
          />
        </div>
        <div v-if="!collapsedGroups.today" class="group-tasks">
          <TaskStrip
            v-for="task in groupedTasks.today"
            :key="task.id"
            :task="task"
            view-key="misc::upcoming"
          />
        </div>
      </div>

      <!-- 三天内 -->
      <div v-if="groupedTasks.threeDays.length > 0" class="task-group">
        <div class="group-header" @click="toggleGroup('threeDays')">
          <span>三天内</span>
          <CuteIcon
            name="ChevronDown"
            :size="14"
            class="group-collapse-icon"
            :class="{ rotated: collapsedGroups.threeDays }"
          />
        </div>
        <div v-if="!collapsedGroups.threeDays" class="group-tasks">
          <TaskStrip
            v-for="task in groupedTasks.threeDays"
            :key="task.id"
            :task="task"
            view-key="misc::upcoming"
          />
        </div>
      </div>

      <!-- 本周 -->
      <div v-if="groupedTasks.thisWeek.length > 0" class="task-group">
        <div class="group-header" @click="toggleGroup('thisWeek')">
          <span>本周</span>
          <CuteIcon
            name="ChevronDown"
            :size="14"
            class="group-collapse-icon"
            :class="{ rotated: collapsedGroups.thisWeek }"
          />
        </div>
        <div v-if="!collapsedGroups.thisWeek" class="group-tasks">
          <TaskStrip
            v-for="task in groupedTasks.thisWeek"
            :key="task.id"
            :task="task"
            view-key="misc::upcoming"
          />
        </div>
      </div>

      <!-- 本月 -->
      <div v-if="groupedTasks.thisMonth.length > 0" class="task-group">
        <div class="group-header" @click="toggleGroup('thisMonth')">
          <span>本月</span>
          <CuteIcon
            name="ChevronDown"
            :size="14"
            class="group-collapse-icon"
            :class="{ rotated: collapsedGroups.thisMonth }"
          />
        </div>
        <div v-if="!collapsedGroups.thisMonth" class="group-tasks">
          <TaskStrip
            v-for="task in groupedTasks.thisMonth"
            :key="task.id"
            :task="task"
            view-key="misc::upcoming"
          />
        </div>
      </div>

      <!-- 更远 -->
      <div v-if="groupedTasks.later.length > 0" class="task-group">
        <div class="group-header" @click="toggleGroup('later')">
          <span>更远</span>
          <CuteIcon
            name="ChevronDown"
            :size="14"
            class="group-collapse-icon"
            :class="{ rotated: collapsedGroups.later }"
          />
        </div>
        <div v-if="!collapsedGroups.later" class="group-tasks">
          <TaskStrip
            v-for="task in groupedTasks.later"
            :key="task.id"
            :task="task"
            view-key="misc::upcoming"
          />
        </div>
      </div>

      <!-- 空状态 -->
      <div v-if="allTasks.length === 0" class="empty-state">
        <p>暂无即将到期的任务</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive } from 'vue'
import type { TaskCard } from '@/types/dtos'
import { useTaskStore } from '@/stores/task'
import { getTodayDateString } from '@/infra/utils/dateUtils'
import TaskStrip from './TaskStrip.vue'
import CuteIcon from '@/components/parts/CuteIcon.vue'

const taskStore = useTaskStore()

// 各分组的折叠状态
const collapsedGroups = reactive({
  overdue: false,
  today: false,
  threeDays: false,
  thisWeek: false,
  thisMonth: false,
  later: false,
})

function toggleGroup(group: keyof typeof collapsedGroups) {
  collapsedGroups[group] = !collapsedGroups[group]
}

// 获取所有即将到期的任务
const allTasks = computed(() => taskStore.upcomingTasks)

// 按时间范围分组
const groupedTasks = computed(() => {
  const today = getTodayDateString()
  const todayDate = new Date(today)

  // 计算各个时间节点
  const threeDaysLater = new Date(todayDate)
  threeDaysLater.setDate(threeDaysLater.getDate() + 3)

  // 本周末（周日）
  const endOfWeek = new Date(todayDate)
  const daysUntilSunday = 7 - endOfWeek.getDay()
  endOfWeek.setDate(endOfWeek.getDate() + daysUntilSunday)

  // 本月末
  const endOfMonth = new Date(todayDate.getFullYear(), todayDate.getMonth() + 1, 0)

  const groups = {
    overdue: [] as TaskCard[],
    today: [] as TaskCard[],
    threeDays: [] as TaskCard[],
    thisWeek: [] as TaskCard[],
    thisMonth: [] as TaskCard[],
    later: [] as TaskCard[],
  }

  for (const task of allTasks.value) {
    if (!task.due_date) continue

    // ✅ due_date.date 现在是 YYYY-MM-DD 格式，直接使用字符串比较
    const dueDateStr = task.due_date.date

    // 为了与 Date 对象比较，需要创建一个 Date 对象
    const dueDate = new Date(dueDateStr + 'T00:00:00')

    // 逾期（截止日期 < 今天）
    if (dueDateStr < today) {
      groups.overdue.push(task)
    }
    // 今天
    else if (dueDateStr === today) {
      groups.today.push(task)
    }
    // 三天内（不包括今天）
    else if (dueDate > todayDate && dueDate <= threeDaysLater) {
      groups.threeDays.push(task)
    }
    // 本周（不包括前三天）
    else if (dueDate > threeDaysLater && dueDate <= endOfWeek) {
      groups.thisWeek.push(task)
    }
    // 本月（不包括本周）
    else if (dueDate > endOfWeek && dueDate <= endOfMonth) {
      groups.thisMonth.push(task)
    }
    // 更远
    else if (dueDate > endOfMonth) {
      groups.later.push(task)
    }
  }

  return groups
})
</script>

<style scoped>
.upcoming-list {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--color-background-content);
}

/* 标题栏 */
.upcoming-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.2rem 1.6rem;
  background-color: var(--color-background-content);
  border-bottom: 1px solid var(--color-border-default);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 0.8rem;
}

.upcoming-title {
  font-size: 1.6rem;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.task-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 2.4rem;
  height: 2.4rem;
  padding: 0 0.6rem;
  font-size: 1.2rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  background-color: var(--color-background-secondary, #f0f0f0);
  border-radius: 1.2rem;
}

/* 内容区 */
.upcoming-content {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}

.task-group {
  display: flex;
  flex-direction: column;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1.2rem 1.6rem 0.8rem;
  font-size: 1.3rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  background-color: var(--color-background-content);
  position: sticky;
  top: 0;
  z-index: 10;
  border-bottom: 1px solid var(--color-border-default);
  cursor: pointer;
  user-select: none;
  transition: background-color 0.2s ease;
}

.group-header:hover {
  background-color: var(--color-background-hover, rgb(0 0 0 / 2%));
}

.group-collapse-icon {
  transition: transform 0.2s ease;
  color: var(--color-text-secondary);
}

.group-collapse-icon.rotated {
  transform: rotate(-90deg);
}

.group-tasks {
  display: flex;
  flex-direction: column;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4rem 2rem;
  color: var(--color-text-secondary);
  font-size: 1.4rem;
}
</style>
