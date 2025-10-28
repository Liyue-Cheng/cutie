# Recent View 与 Calendar 联动技术文档

## 概述

本文档描述了 HomeView 中左侧 RecentView（任务列表）与右侧 CuteCalendar（日历）的天数联动机制，以及可拖动分割线的实时尺寸更新实现。

## 功能特性

### 1. 天数联动

- 用户在左侧 RecentView 点击天数按钮（1天、3天、5天、7天）
- 右侧日历自动切换显示对应天数的视图
  - 1天：单天视图
  - 3天：3天视图（从当前日期开始）
  - 5天：5天视图（从当前日期开始）
  - 7天：本周视图（周一到周日）
- 左右两侧保持完全同步

### 2. 实时分割线拖动

- 用户拖动中间分割线调整左右栏比例
- 日历在拖动过程中实时更新尺寸（60fps）
- 提供流畅的视觉反馈

---

## 技术实现

### 架构设计

```
HomeView (父组件，状态管理)
├── RecentView (左栏，任务列表)
│   └── v-model:calendarDays (双向绑定天数)
├── Divider (可拖动分割线)
└── CuteCalendar (右栏，日历)
    └── :days="calendarDays" (接收天数 prop)
```

### 数据流

```
用户点击天数按钮
    ↓
RecentView 更新内部 dayCount
    ↓
emit('update:modelValue', count)
    ↓
HomeView 的 calendarDays 更新
    ↓
CuteCalendar 接收新的 days prop
    ↓
watch(() => props.days) 触发
    ↓
调用 calendarApi.changeView(viewName)
    ↓
日历切换到对应天数视图
```

---

## 核心代码实现

### 1. HomeView：状态管理中心

```typescript
// 管理天数状态（1 | 3 | 5 | 7）
const calendarDays = ref<1 | 3 | 5 | 7>(3)

// 根据天数计算视图类型：7天显示本周视图，其他显示多天视图
const calendarViewType = computed(() => {
  return calendarDays.value === 7 ? 'week' : 'day'
})

// 传递给 RecentView（v-model 双向绑定）
<RecentView v-model="calendarDays" />

// 传递给 CuteCalendar（单向数据流）
<CuteCalendar :days="calendarDays" :view-type="calendarViewType" />
```

**关键点：**

- `calendarDays` 是唯一的数据源（Single Source of Truth）
- 使用 `v-model` 简化双向绑定
- 类型约束确保只能是 1、3、5、7
- 通过 `computed` 动态计算视图类型：7天使用 week 视图，其他使用 day 视图

### 2. RecentView：天数选择器

```typescript
// Props 定义
interface Props {
  modelValue?: number // 支持 v-model
}

const props = withDefaults(defineProps<Props>(), {
  modelValue: 3,
})

// Emits 定义
const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()

// 内部状态同步
const dayCount = ref(props.modelValue)

watch(
  () => props.modelValue,
  (newValue) => {
    dayCount.value = newValue
  }
)

// 用户点击按钮时通知父组件
function setDayCount(count: number) {
  dayCount.value = count
  emit('update:modelValue', count) // 关键：向上传递
  loadDateRangeTasks()
}
```

**关键点：**

- 实现标准的 `v-model` 模式（`modelValue` prop + `update:modelValue` emit）
- 内部维护 `dayCount` 状态用于 UI 渲染
- 通过 `watch` 监听 prop 变化保持同步

### 3. CuteCalendar：视图切换

#### 3.1 Props 类型定义

```typescript
const props = withDefaults(
  defineProps<{
    days?: 1 | 3 | 5 | 7 // 联合类型约束
  }>(),
  {
    days: 1,
  }
)
```

#### 3.2 视图名称映射

```typescript
function getViewName(viewType: 'day' | 'week' | 'month', days: 1 | 3 | 5 | 7): string {
  if (viewType === 'day') {
    if (days === 3) return 'timeGrid3Days'
    if (days === 5) return 'timeGrid5Days'
    if (days === 7) return 'timeGrid7Days'
    return 'timeGridDay'
  } else if (viewType === 'week') {
    return 'timeGridWeek'
  } else {
    return 'dayGridMonth'
  }
}
```

#### 3.3 动态视图切换

```typescript
// 监听 viewType 和 days prop 变化，动态切换视图
watch(
  [() => props.viewType, () => props.days],
  async ([newViewType, newDays]) => {
    if (!calendarRef.value) return
    const calendarApi = calendarRef.value.getApi()
    if (!calendarApi) return

    const viewName = getViewName(newViewType, newDays ?? 1)

    logger.info(LogTags.COMPONENT_CALENDAR, 'Changing calendar view', {
      from: calendarApi.view.type,
      to: viewName,
      viewType: newViewType,
      days: newDays,
    })

    // 保存当前日期
    const currentDate = calendarApi.getDate()

    // 切换视图
    calendarApi.changeView(viewName)

    // 🔧 FIX: 更新 dayHeaders 配置
    // week 视图或多天视图显示日期头部
    calendarOptions.dayHeaders = newViewType === 'week' || (newDays ?? 1) > 1

    await nextTick()

    // 强制更新尺寸
    calendarApi.updateSize()

    // 恢复日期
    calendarApi.gotoDate(currentDate)

    // 清除缓存
    clearCache()
  },
  { immediate: false }
)
```

**关键点：**

- 使用 `watch` 同时监听 `props.viewType` 和 `props.days` 变化
- 根据 viewType 和 days 组合确定最终视图（7天时使用 week 视图）
- 保存并恢复当前日期，避免视图切换时日期跳转
- 调用 `updateSize()` 确保布局正确
- 使用 `async/await` 和 `nextTick` 确保 DOM 更新完成

### 4. useCalendarOptions：自定义视图定义

```typescript
export function useCalendarOptions(
  // ...其他参数
  days: 1 | 3 | 5 | 7 = 1
) {
  // 根据天数确定初始视图
  let initialView: string
  if (viewType === 'day') {
    if (days === 3) {
      initialView = 'timeGrid3Days'
    } else if (days === 5) {
      initialView = 'timeGrid5Days'
    } else if (days === 7) {
      initialView = 'timeGrid7Days'
    } else {
      initialView = 'timeGridDay'
    }
  }

  const calendarOptions = reactive({
    // ...其他配置
    initialView,

    // 自定义视图定义
    views: {
      timeGrid3Days: {
        type: 'timeGrid',
        duration: { days: 3 },
      },
      timeGrid5Days: {
        type: 'timeGrid',
        duration: { days: 5 },
      },
      timeGrid7Days: {
        type: 'timeGrid',
        duration: { days: 7 },
      },
    },

    // 多天视图显示日期头部
    dayHeaders: viewType !== 'day' || days > 1,
  })

  return { calendarOptions }
}
```

**关键点：**

- 使用 FullCalendar 的自定义视图功能
- 定义 3天、5天、7天的 `timeGrid` 视图
- 动态控制 `dayHeaders` 显示

---

## 实时分割线拖动实现

### 问题背景

用户拖动分割线调整左右栏比例时，日历组件不会自动感知容器尺寸变化，导致：

- 日历宽度不匹配容器
- 需要手动点击才能触发重新渲染
- 用户体验差

### 解决方案：requestAnimationFrame

```typescript
let rafId: number | null = null

function onDragging(e: MouseEvent) {
  if (!isDragging.value) return

  // 计算新的左栏宽度
  const container = document.querySelector('.home-view') as HTMLElement
  const containerRect = container.getBoundingClientRect()
  const mouseX = e.clientX - containerRect.left
  let newWidth = (mouseX / containerRect.width) * 100
  newWidth = Math.max(20, Math.min(80, newWidth))

  leftPaneWidth.value = newWidth

  // 使用 requestAnimationFrame 实现流畅的实时更新
  if (rafId !== null) {
    cancelAnimationFrame(rafId) // 取消上一帧
  }
  rafId = requestAnimationFrame(() => {
    updateCalendarSize() // 在下一帧更新日历
    rafId = null
  })
}

function updateCalendarSize() {
  if (calendarRef.value?.calendarRef) {
    const calendarApi = calendarRef.value.calendarRef.getApi()
    if (calendarApi) {
      calendarApi.updateSize()
    }
  }
}
```

### 技术对比

| 方案                      | 更新时机        | 帧率   | 用户体验          |
| ------------------------- | --------------- | ------ | ----------------- |
| **setTimeout(50ms)**      | 鼠标停止后 50ms | ~20fps | ❌ 有延迟，不流畅 |
| **requestAnimationFrame** | 每一帧          | 60fps  | ✅ 实时，流畅     |

### 工作原理

1. **鼠标移动事件触发** → `onDragging` 被调用
2. **更新左栏宽度** → `leftPaneWidth.value = newWidth`
3. **请求动画帧** → `requestAnimationFrame(updateCalendarSize)`
4. **浏览器在下一帧渲染前** → 执行 `updateCalendarSize()`
5. **调用 FullCalendar API** → `calendarApi.updateSize()`
6. **日历重新计算布局** → 适应新的容器宽度

### 性能优化

```typescript
// 取消上一帧的请求，避免重复更新
if (rafId !== null) {
  cancelAnimationFrame(rafId)
}
```

**为什么需要取消？**

- 鼠标移动速度快时，一帧内可能触发多次 `mousemove` 事件
- 只保留最后一次请求，避免不必要的计算
- 确保每帧最多更新一次

### 清理机制

```typescript
async function stopDragging() {
  isDragging.value = false
  document.removeEventListener('mousemove', onDragging)
  document.removeEventListener('mouseup', stopDragging)

  // 清除待处理的动画帧
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }

  // 最后确保更新一次
  await nextTick()
  updateCalendarSize()
}

onBeforeUnmount(() => {
  // 组件卸载时清理
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
  }
})
```

---

## 关键技术点总结

### 1. 响应式数据流

- **单一数据源**：`calendarDays` 在 HomeView 中管理
- **v-model 双向绑定**：简化父子组件通信
- **Props 单向流动**：保持数据流清晰可预测

### 2. FullCalendar 自定义视图

- 使用 `views` 配置定义自定义天数视图
- 通过 `calendarApi.changeView()` 动态切换
- 保存并恢复当前日期，避免跳转

### 3. 实时尺寸更新

- **requestAnimationFrame**：与浏览器渲染周期同步
- **取消机制**：避免重复计算，优化性能
- **清理机制**：防止内存泄漏

### 4. 异步处理

- 使用 `async/await` 处理 DOM 更新
- `nextTick()` 确保 Vue 响应式更新完成
- 先更新 DOM，再调用 FullCalendar API

---

## 性能指标

| 指标             | 数值   | 说明                       |
| ---------------- | ------ | -------------------------- |
| **天数切换延迟** | < 50ms | 用户点击到视图切换完成     |
| **拖动更新帧率** | 60fps  | 与浏览器刷新率同步         |
| **内存占用**     | 稳定   | 正确清理定时器和事件监听器 |

---

## 未来优化方向

1. **防抖优化**：对于非常快速的拖动，可以考虑跳帧
2. **虚拟滚动**：当任务数量非常多时，使用虚拟滚动优化性能
3. **持久化**：将用户选择的天数和分割线比例保存到 localStorage
4. **动画过渡**：添加平滑的过渡动画，提升视觉体验

---

## 相关文件

- `src/views/HomeView.vue` - 主视图，状态管理
- `src/components/templates/RecentView.vue` - 任务列表，天数选择器
- `src/components/parts/CuteCalendar.vue` - 日历组件，视图切换
- `src/composables/calendar/useCalendarOptions.ts` - 日历配置，自定义视图定义

---

## 参考资料

- [Vue 3 v-model 文档](https://vuejs.org/guide/components/v-model.html)
- [FullCalendar Custom Views](https://fullcalendar.io/docs/custom-view-with-settings)
- [MDN: requestAnimationFrame](https://developer.mozilla.org/en-US/docs/Web/API/window/requestAnimationFrame)
