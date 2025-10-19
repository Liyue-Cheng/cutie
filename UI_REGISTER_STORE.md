# UI Register Store 架构重构

## 🎯 目标

彻底消除组件之间的 **props drilling**（透传）和**组件钻探**问题，提供一个简洁、类型安全的全局 UI 状态管理方案。

## 📦 核心实现

### Store 位置

`src/stores/register.ts`

### 核心 API

```typescript
const registerStore = useRegisterStore()

// 写入寄存器
registerStore.writeRegister<T>(key: string, value: T)

// 读取寄存器
registerStore.readRegister<T>(key: string, defaultValue?: T)

// 删除寄存器
registerStore.deleteRegister(key: string)

// 检查是否存在
registerStore.hasRegister(key: string)

// 清空所有
registerStore.clearAllRegisters()
```

### 预定义键名

```typescript
registerStore.RegisterKeys = {
  CURRENT_CALENDAR_DATE_HOME: 'currentCalendarDate_Home',
  KANBAN_SCROLL_POSITION: 'kanbanScrollPosition',
  CURRENT_VIEW: 'currentView',
  // 可以继续添加...
}
```

## 🔄 重构示例：日历日期同步

### ❌ 之前（Props Drilling）

```
CuteCalendar
  ↓ emit('date-change')
HomeView (监听事件，更新 ref)
  ↓ :current-calendar-date="currentCalendarDate"
InfiniteDailyKanban
  ↓ :is-calendar-date="isCalendarDate(date)"
SimpleKanbanColumn
```

**问题：**

- 需要在 3 层组件中传递同一个状态
- HomeView 作为中间层必须处理不相关的状态
- 添加新组件需要继续传递 props

### ✅ 之后（Register Store）

```
CuteCalendar
  ↓ registerStore.writeRegister(...)

[Register Store - 全局状态]

SimpleKanbanColumn
  ↑ registerStore.readRegister(...)
```

**优势：**

- 直接在源头写入，直接在终点读取
- 中间组件无需关心状态传递
- 完全解耦，易于维护

## 📝 实际代码修改

### 1. 写入方（CuteCalendar.vue）

```typescript
import { useRegisterStore } from '@/stores/register'

const registerStore = useRegisterStore()

const handleDatesSet = (dateInfo: { start: Date; end: Date }) => {
  const dateStr = formatDate(dateInfo.start)

  // ✅ 直接写入寄存器
  registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME, dateStr)
}
```

### 2. 读取方（InfiniteDailyKanban.vue）

```typescript
import { useRegisterStore } from '@/stores/register'

const registerStore = useRegisterStore()

function isCalendarDate(date: Date): boolean {
  // ✅ 直接从寄存器读取
  const currentCalendarDate = registerStore.readRegister<string>(
    registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME
  )

  if (!currentCalendarDate) return false

  const dateStr = formatDate(date)
  return dateStr === currentCalendarDate
}
```

### 3. 移除中间层（HomeView.vue）

```typescript
// ❌ 删除
// const currentCalendarDate = ref<string>('')
// function handleCalendarDateChange(dateStr: string) { ... }

// ❌ 删除
// <CuteCalendar @date-change="handleCalendarDateChange" />
// <InfiniteDailyKanban :current-calendar-date="currentCalendarDate" />

// ✅ 简化
<CuteCalendar />
<InfiniteDailyKanban />
```

## 🚀 未来扩展

### 添加新的全局状态

1. **定义键名**（可选，用于类型提示）

```typescript
// 在 register.ts 中
const RegisterKeys = {
  // ... 现有键名
  MY_NEW_STATE: 'myNewState',
}
```

2. **写入**

```typescript
registerStore.writeRegister('myNewState', someValue)
```

3. **读取**

```typescript
const value = registerStore.readRegister<Type>('myNewState', defaultValue)
```

### 响应式监听

```typescript
import { watch } from 'vue'

const registerStore = useRegisterStore()

// 监听整个寄存器的变化
watch(
  () => registerStore.readRegister<string>('myKey'),
  (newValue, oldValue) => {
    console.log('Value changed:', oldValue, '->', newValue)
  }
)
```

## 💡 最佳实践

1. **使用预定义键名**：避免拼写错误

   ```typescript
   // ✅ 推荐
   registerStore.writeRegister(registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME, date)

   // ❌ 不推荐
   registerStore.writeRegister('currentCalendarDate_Home', date)
   ```

2. **提供默认值**：避免 undefined

   ```typescript
   const date = registerStore.readRegister<string>(
     registerStore.RegisterKeys.CURRENT_CALENDAR_DATE_HOME,
     getTodayDateString() // 默认值
   )
   ```

3. **类型安全**：始终指定泛型类型

   ```typescript
   // ✅ 有类型推断
   const count = registerStore.readRegister<number>('count')

   // ❌ 失去类型安全
   const count = registerStore.readRegister('count')
   ```

4. **及时清理**：不再使用的状态应该删除
   ```typescript
   onBeforeUnmount(() => {
     registerStore.deleteRegister('temporaryState')
   })
   ```

## 📊 收益

- ✅ **消除 Props Drilling**：3 层组件传递 → 直接读写
- ✅ **降低耦合**：组件之间无需知道彼此的存在
- ✅ **易于维护**：状态集中管理，易于追踪和调试
- ✅ **类型安全**：完整的 TypeScript 支持
- ✅ **响应式**：基于 Vue 3 响应式系统，自动更新视图

## 🔍 调试

所有寄存器操作都会记录日志：

```
[STORE_UI] Register write: { key: 'currentCalendarDate', oldValue: '2025-10-18', newValue: '2025-10-19' }
[STORE_UI] Register deleted: { key: 'temporaryState' }
```

可以在浏览器控制台查看完整的操作历史。
