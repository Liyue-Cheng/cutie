# CuteDropdown 使用指南

Cutie 项目的统一下拉菜单组件系统。

## 组件列表

- **CuteDropdown**: 下拉菜单容器组件
- **CuteDropdownItem**: 下拉菜单项
- **CuteDropdownDivider**: 下拉菜单分割线

---

## 基础用法

### 1. 简单下拉选择（使用 options 属性）

```vue
<template>
  <CuteDropdown
    v-model="selectedValue"
    :options="options"
    trigger-text="选择水果"
    @change="handleChange"
  />
</template>

<script setup>
import { ref } from 'vue'
import CuteDropdown from '@/components/parts/CuteDropdown.vue'

const selectedValue = ref(null)
const options = [
  { value: 'apple', label: '苹果', icon: 'Apple' },
  { value: 'banana', label: '香蕉', icon: 'Banana' },
  { value: 'orange', label: '橙子', icon: 'Circle' },
]

function handleChange(value) {
  console.log('选中:', value)
}
</script>
```

---

### 2. 自定义触发器

```vue
<template>
  <CuteDropdown v-model="selectedArea">
    <!-- 自定义触发器 -->
    <template #trigger>
      <button class="custom-trigger">
        <AreaTag v-if="selectedArea" :area-id="selectedArea" />
        <span v-else>选择 Area</span>
        <CuteIcon name="ChevronDown" :size="16" />
      </button>
    </template>

    <!-- 选项列表 -->
    <CuteDropdownItem
      v-for="area in areas"
      :key="area.id"
      :label="area.name"
      :active="selectedArea === area.id"
      @click="selectedArea = area.id"
    />
  </CuteDropdown>
</template>
```

---

### 3. 使用分组和分割线

```vue
<template>
  <CuteDropdown trigger-text="操作">
    <CuteDropdownItem icon="Edit" label="编辑" @click="handleEdit" />
    <CuteDropdownItem icon="Copy" label="复制" @click="handleCopy" />

    <CuteDropdownDivider />

    <CuteDropdownItem icon="Archive" label="归档" @click="handleArchive" />

    <CuteDropdownDivider label="危险操作" />

    <CuteDropdownItem
      icon="Trash2"
      label="删除"
      variant="danger"
      @click="handleDelete"
    />
  </CuteDropdown>
</template>
```

---

### 4. 可搜索下拉框

```vue
<template>
  <CuteDropdown
    v-model="selectedTask"
    :options="tasks"
    searchable
    search-placeholder="搜索任务..."
    title="选择任务"
    value-key="id"
    label-key="title"
  />
</template>

<script setup>
const tasks = [
  { id: '1', title: '完成项目文档' },
  { id: '2', title: '修复 Bug #123' },
  { id: '3', title: '代码审查' },
]
</script>
```

---

### 5. 多选模式

```vue
<template>
  <CuteDropdown
    v-model="selectedTags"
    :options="tags"
    multiple
    :close-on-select="false"
    trigger-text="选择标签"
  />
</template>

<script setup>
const selectedTags = ref([])
const tags = [
  { value: 'urgent', label: '紧急' },
  { value: 'important', label: '重要' },
  { value: 'personal', label: '个人' },
]
</script>
```

---

### 6. 自定义底部操作

```vue
<template>
  <CuteDropdown v-model="selectedDate">
    <CuteDropdownItem
      v-for="date in quickDates"
      :key="date.value"
      :label="date.label"
      :active="selectedDate === date.value"
      @click="selectedDate = date.value"
    />

    <!-- 底部自定义操作 -->
    <template #footer>
      <button class="custom-date-btn" @click="openDatePicker">
        <CuteIcon name="Calendar" :size="16" />
        <span>选择其他日期...</span>
      </button>
    </template>
  </CuteDropdown>
</template>

<style scoped>
.custom-date-btn {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.8rem;
  padding: 1rem 1.2rem;
  font-size: 1.3rem;
  color: var(--color-text-accent);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: background-color 0.15s;
}

.custom-date-btn:hover {
  background-color: var(--color-background-hover);
}
</style>
```

---

### 7. 带图标和后缀的高级选项

```vue
<template>
  <CuteDropdown trigger-text="选择视图">
    <CuteDropdownItem
      icon="List"
      label="列表视图"
      suffix="⌘1"
      :active="currentView === 'list'"
      @click="currentView = 'list'"
    />
    <CuteDropdownItem
      icon="LayoutGrid"
      label="看板视图"
      suffix="⌘2"
      :active="currentView === 'kanban'"
      @click="currentView = 'kanban'"
    />
    <CuteDropdownItem
      icon="Calendar"
      label="日历视图"
      suffix="⌘3"
      :active="currentView === 'calendar'"
      @click="currentView = 'calendar'"
    />
  </CuteDropdown>
</template>
```

---

## API 参考

### CuteDropdown Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `modelValue` | `any \| any[]` | - | 选中的值（v-model） |
| `options` | `any[]` | - | 选项列表 |
| `valueKey` | `string` | `'value'` | 选项值字段名 |
| `labelKey` | `string` | `'label'` | 选项标签字段名 |
| `iconKey` | `string` | `'icon'` | 选项图标字段名 |
| `disabledKey` | `string` | `'disabled'` | 选项禁用字段名 |
| `triggerText` | `string` | `'选择'` | 默认触发器文本 |
| `title` | `string` | - | 下拉框标题 |
| `searchable` | `boolean` | `false` | 是否可搜索 |
| `searchPlaceholder` | `string` | `'搜索...'` | 搜索框占位符 |
| `emptyText` | `string` | `'无数据'` | 空状态文本 |
| `multiple` | `boolean` | `false` | 是否多选 |
| `maxHeight` | `string` | `'32rem'` | 最大高度 |
| `alignRight` | `boolean` | `false` | 是否右对齐 |
| `disabled` | `boolean` | `false` | 是否禁用 |
| `closeOnSelect` | `boolean` | `true` | 选择后是否自动关闭 |

### CuteDropdown Events

| 事件 | 参数 | 说明 |
|------|------|------|
| `update:modelValue` | `value: any` | 选中值变化 |
| `change` | `value: any` | 选中值变化（同上） |
| `open` | - | 下拉框打开 |
| `close` | - | 下拉框关闭 |

### CuteDropdown Slots

| 插槽 | 说明 |
|------|------|
| `trigger` | 自定义触发器 |
| `default` | 自定义下拉内容 |
| `footer` | 底部操作区 |

---

### CuteDropdownItem Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `label` | `string` | - | 选项标签 |
| `icon` | `string` | - | 左侧图标名称 |
| `iconSize` | `number` | `16` | 图标大小 |
| `suffix` | `string` | - | 右侧后缀文本 |
| `active` | `boolean` | `false` | 是否激活（选中） |
| `disabled` | `boolean` | `false` | 是否禁用 |
| `showCheck` | `boolean` | `true` | 是否显示选中标记 |
| `variant` | `'default' \| 'danger'` | `'default'` | 变体样式 |

### CuteDropdownItem Events

| 事件 | 参数 | 说明 |
|------|------|------|
| `click` | - | 点击事件 |

### CuteDropdownItem Slots

| 插槽 | 说明 |
|------|------|
| `default` | 主内容 |
| `suffix` | 右侧后缀内容 |

---

### CuteDropdownDivider Props

| 属性 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `label` | `string` | - | 分割线标签 |
| `variant` | `'default' \| 'strong'` | `'default'` | 变体样式 |

---

## 设计原则

1. **统一风格**: 使用项目全局 CSS 变量，确保与其他组件视觉一致
2. **灵活性**: 支持 options 属性快速创建，也支持 slot 自定义
3. **可访问性**: 支持键盘导航（ESC 关闭）
4. **响应式**: 自动计算位置，避免超出视口
5. **性能**: 使用 Teleport 渲染到 body，避免层级问题

---

## 与 CuteContextMenu 的区别

| 特性 | CuteDropdown | CuteContextMenu |
|------|--------------|-----------------|
| **触发方式** | 点击按钮 | 右键点击 |
| **定位** | 相对触发器 | 固定鼠标位置 |
| **用途** | 选择器、操作菜单 | 右键操作菜单 |
| **支持搜索** | ✅ | ❌ |
| **支持多选** | ✅ | ❌ |
| **v-model** | ✅ | ❌ |
