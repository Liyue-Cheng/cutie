<template>
  <div class="cute-dropdown" v-bind="$attrs">
    <!-- 触发器插槽 -->
    <div ref="triggerRef" @click="toggleDropdown">
      <slot name="trigger">
        <button class="dropdown-trigger" type="button">
          <span>{{ triggerText }}</span>
          <CuteIcon :name="show ? 'ChevronUp' : 'ChevronDown'" :size="16" />
        </button>
      </slot>
    </div>

    <!-- 下拉内容 -->
    <Teleport to="body">
      <div
        v-if="show"
        ref="dropdownRef"
        class="dropdown-content"
        :class="{ 'align-right': alignRight }"
        :style="dropdownStyle"
        @click.stop
      >
        <!-- 标题（可选） -->
        <div v-if="title" class="dropdown-header">
          <span class="dropdown-title">{{ title }}</span>
        </div>

        <!-- 搜索框（可选） -->
        <div v-if="searchable" class="dropdown-search">
          <input
            ref="searchInputRef"
            v-model="searchQuery"
            type="text"
            class="search-input"
            :placeholder="searchPlaceholder"
            @click.stop
          />
        </div>

        <!-- 选项列表 -->
        <div class="dropdown-body" :style="{ maxHeight: maxHeight }" @click="handleSlotClick">
          <slot>
            <!-- 默认选项渲染 -->
            <div v-if="filteredOptions.length === 0" class="dropdown-empty">
              {{ emptyText }}
            </div>
            <button
              v-for="option in filteredOptions"
              :key="getOptionValue(option)"
              class="dropdown-item"
              :class="{
                active: isSelected(option),
                disabled: isDisabled(option),
              }"
              :disabled="isDisabled(option)"
              @click="handleSelect(option)"
            >
              <!-- 图标（可选） -->
              <CuteIcon
                v-if="getOptionIcon(option)"
                :name="getOptionIcon(option)"
                :size="16"
                class="item-icon"
              />

              <!-- 标签文本 -->
              <span class="item-label">{{ getOptionLabel(option) }}</span>

              <!-- 选中标记 -->
              <CuteIcon
                v-if="isSelected(option)"
                name="Check"
                :size="16"
                class="item-check"
              />
            </button>
          </slot>
        </div>

        <!-- 底部操作区（可选） -->
        <div v-if="$slots.footer" class="dropdown-footer">
          <slot name="footer"></slot>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue'
import CuteIcon from './CuteIcon.vue'

interface Props {
  /** 选中的值（单选模式）或值数组（多选模式） */
  modelValue?: any | any[]
  /** 选项列表 */
  options?: any[]
  /** 选项的值字段名 */
  valueKey?: string
  /** 选项的标签字段名 */
  labelKey?: string
  /** 选项的图标字段名 */
  iconKey?: string
  /** 选项的禁用字段名 */
  disabledKey?: string
  /** 触发器文本（使用默认触发器时） */
  triggerText?: string
  /** 下拉框标题 */
  title?: string
  /** 是否可搜索 */
  searchable?: boolean
  /** 搜索框占位符 */
  searchPlaceholder?: string
  /** 空状态文本 */
  emptyText?: string
  /** 是否多选 */
  multiple?: boolean
  /** 最大高度 */
  maxHeight?: string
  /** 是否右对齐 */
  alignRight?: boolean
  /** 是否禁用 */
  disabled?: boolean
  /** 点击选项后是否自动关闭 */
  closeOnSelect?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  valueKey: 'value',
  labelKey: 'label',
  iconKey: 'icon',
  disabledKey: 'disabled',
  triggerText: '选择',
  searchPlaceholder: '搜索...',
  emptyText: '无数据',
  multiple: false,
  maxHeight: '32rem',
  alignRight: false,
  disabled: false,
  closeOnSelect: true,
})

const emit = defineEmits<{
  'update:modelValue': [value: any]
  change: [value: any]
  open: []
  close: []
}>()

// Refs
const triggerRef = ref<HTMLElement | null>(null)
const dropdownRef = ref<HTMLElement | null>(null)
const searchInputRef = ref<HTMLInputElement | null>(null)
const show = ref(false)
const searchQuery = ref('')

// 下拉框定位样式
const dropdownStyle = ref({
  top: '0px',
  left: '0px',
  minWidth: '0px',
})

// 获取选项的值
function getOptionValue(option: any): any {
  if (typeof option === 'object' && option !== null) {
    return option[props.valueKey]
  }
  return option
}

// 获取选项的标签
function getOptionLabel(option: any): string {
  if (typeof option === 'object' && option !== null) {
    return option[props.labelKey] || String(option[props.valueKey])
  }
  return String(option)
}

// 获取选项的图标
function getOptionIcon(option: any): string | undefined {
  if (typeof option === 'object' && option !== null) {
    return option[props.iconKey]
  }
  return undefined
}

// 判断选项是否禁用
function isDisabled(option: any): boolean {
  if (typeof option === 'object' && option !== null) {
    return option[props.disabledKey] === true
  }
  return false
}

// 判断选项是否选中
function isSelected(option: any): boolean {
  const value = getOptionValue(option)
  if (props.multiple) {
    return Array.isArray(props.modelValue) && props.modelValue.includes(value)
  }
  return props.modelValue === value
}

// 过滤后的选项
const filteredOptions = computed(() => {
  if (!props.options) return []
  if (!props.searchable || !searchQuery.value) return props.options

  const query = searchQuery.value.toLowerCase()
  return props.options.filter((option) => {
    const label = getOptionLabel(option).toLowerCase()
    return label.includes(query)
  })
})

// 切换下拉框显示
function toggleDropdown() {
  if (props.disabled) return
  if (show.value) {
    closeDropdown()
  } else {
    openDropdown()
  }
}

// 打开下拉框
async function openDropdown() {
  show.value = true
  emit('open')

  await nextTick()
  updateDropdownPosition()

  // 如果可搜索，自动聚焦搜索框
  if (props.searchable && searchInputRef.value) {
    searchInputRef.value.focus()
  }
}

// 关闭下拉框
function closeDropdown() {
  show.value = false
  searchQuery.value = ''
  emit('close')
}

// 更新下拉框位置
function updateDropdownPosition() {
  if (!triggerRef.value || !dropdownRef.value) return

  const triggerRect = triggerRef.value.getBoundingClientRect()
  const dropdownRect = dropdownRef.value.getBoundingClientRect()
  const viewportHeight = window.innerHeight
  const viewportWidth = window.innerWidth

  const SPACING = 4 // 下拉框与触发器的间距

  // 计算垂直位置
  let top = triggerRect.bottom + SPACING
  const spaceBelow = viewportHeight - triggerRect.bottom - SPACING
  const spaceAbove = triggerRect.top - SPACING

  // 如果下方空间不足，且上方空间更大，则向上展开
  if (spaceBelow < dropdownRect.height && spaceAbove > spaceBelow) {
    top = triggerRect.top - dropdownRect.height - SPACING
  }

  // 计算水平位置
  let left = triggerRect.left
  if (props.alignRight) {
    left = triggerRect.right - dropdownRect.width
  }

  // 确保不超出视口
  if (left + dropdownRect.width > viewportWidth) {
    left = viewportWidth - dropdownRect.width - 8
  }
  if (left < 8) {
    left = 8
  }

  dropdownStyle.value = {
    top: `${top}px`,
    left: `${left}px`,
    minWidth: `${triggerRect.width}px`,
  }
}

// 处理选项选择
function handleSelect(option: any) {
  if (isDisabled(option)) return

  const value = getOptionValue(option)

  if (props.multiple) {
    const currentValue = Array.isArray(props.modelValue) ? props.modelValue : []
    const newValue = currentValue.includes(value)
      ? currentValue.filter((v) => v !== value)
      : [...currentValue, value]

    emit('update:modelValue', newValue)
    emit('change', newValue)
  } else {
    emit('update:modelValue', value)
    emit('change', value)

    if (props.closeOnSelect) {
      closeDropdown()
    }
  }
}

// 处理 slot 内容的点击（用于 CuteDropdownItem）
function handleSlotClick(event: MouseEvent) {
  // 如果点击的是带有 .prevent 修饰符的元素，不关闭
  const target = event.target as HTMLElement
  
  // 检查是否点击了 CuteDropdownItem 按钮
  const dropdownItem = target.closest('.cute-dropdown-item')
  if (!dropdownItem) return
  
  // 检查事件是否被阻止默认行为（@click.prevent）
  if (event.defaultPrevented) return
  
  // 如果 closeOnSelect 为 true，关闭下拉菜单
  if (props.closeOnSelect) {
    closeDropdown()
  }
}

// 点击外部关闭
function handleClickOutside(event: MouseEvent) {
  if (!show.value) return

  const target = event.target as Node
  if (
    triggerRef.value?.contains(target) ||
    dropdownRef.value?.contains(target)
  ) {
    return
  }

  closeDropdown()
}

// 监听窗口大小变化
function handleResize() {
  if (show.value) {
    updateDropdownPosition()
  }
}

// 键盘事件处理
function handleKeydown(event: KeyboardEvent) {
  if (!show.value) return

  if (event.key === 'Escape') {
    closeDropdown()
    event.preventDefault()
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  window.addEventListener('resize', handleResize)
  document.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', handleClickOutside)
  window.removeEventListener('resize', handleResize)
  document.removeEventListener('keydown', handleKeydown)
})

// 监听显示状态变化，更新位置
watch(show, async (newShow) => {
  if (newShow) {
    await nextTick()
    updateDropdownPosition()
  }
})
</script>

<style scoped>
/* ==================== 下拉框容器 ==================== */
.cute-dropdown {
  position: relative;
  display: inline-block;
}

/* ==================== 默认触发器 ==================== */
.dropdown-trigger {
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.8rem;
  min-width: 12rem;
  height: 3.6rem;
  padding: 0 1.2rem;
  font-size: 1.4rem;
  font-weight: 500;
  color: var(--color-text-primary);
  background-color: var(--color-background-input);
  border: 1px solid var(--color-border-input);
  border-radius: 0.6rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.dropdown-trigger:hover {
  background-color: var(--color-background-input-hover);
  border-color: var(--color-border-input-hover);
}

.dropdown-trigger:focus {
  outline: none;
  border-color: var(--color-border-input-focus);
  box-shadow: var(--shadow-focus);
}

/* ==================== 下拉内容 ==================== */
.dropdown-content {
  position: fixed;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  background-color: var(--color-background-content);
  border: 1px solid var(--color-border-default);
  border-radius: 0.8rem;
  box-shadow: var(--shadow-lg);
  overflow: hidden;
}

/* ==================== 下拉头部 ==================== */
.dropdown-header {
  padding: 1.2rem 1.4rem;
  border-bottom: 1px solid var(--color-border-light);
}

.dropdown-title {
  font-size: 1.3rem;
  font-weight: 600;
  color: var(--color-text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* ==================== 搜索框 ==================== */
.dropdown-search {
  padding: 0.8rem 1rem;
  border-bottom: 1px solid var(--color-border-light);
}

.search-input {
  width: 100%;
  height: 3.2rem;
  padding: 0 1rem;
  font-size: 1.3rem;
  color: var(--color-text-primary);
  background-color: var(--color-background-input);
  border: 1px solid var(--color-border-input);
  border-radius: 0.4rem;
  outline: none;
  transition: all 0.2s ease;
}

.search-input::placeholder {
  color: var(--color-text-placeholder);
}

.search-input:focus {
  border-color: var(--color-border-input-focus);
  box-shadow: var(--shadow-focus);
}

/* ==================== 下拉主体 ==================== */
.dropdown-body {
  overflow-y: auto;
  padding: 0.4rem;
}

/* 滚动条样式 */
.dropdown-body::-webkit-scrollbar {
  width: 6px;
}

.dropdown-body::-webkit-scrollbar-track {
  background-color: transparent;
}

.dropdown-body::-webkit-scrollbar-thumb {
  background-color: var(--color-border-default);
  border-radius: 3px;
}

.dropdown-body::-webkit-scrollbar-thumb:hover {
  background-color: var(--color-border-strong);
}

/* ==================== 空状态 ==================== */
.dropdown-empty {
  padding: 2rem 1.4rem;
  font-size: 1.3rem;
  color: var(--color-text-tertiary);
  text-align: center;
}

/* ==================== 下拉项 ==================== */
.dropdown-item {
  display: flex;
  align-items: center;
  gap: 1rem;
  width: 100%;
  min-height: 3.6rem;
  padding: 0.8rem 1.2rem;
  font-size: 1.4rem;
  color: var(--color-text-primary);
  background-color: transparent;
  border: none;
  border-radius: 0.4rem;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s ease;
}

.dropdown-item:hover:not(.disabled) {
  background-color: var(--color-background-hover);
}

.dropdown-item.active {
  background-color: var(--color-background-selected);
  color: var(--color-text-accent);
  font-weight: 500;
}

.dropdown-item.disabled {
  opacity: var(--opacity-disabled);
  cursor: not-allowed;
}

.item-icon {
  flex-shrink: 0;
  color: var(--color-text-secondary);
}

.dropdown-item.active .item-icon {
  color: var(--color-text-accent);
}

.item-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-check {
  flex-shrink: 0;
  color: var(--color-text-accent);
}

/* ==================== 下拉底部 ==================== */
.dropdown-footer {
  padding: 0.8rem 1rem;
  border-top: 1px solid var(--color-border-light);
}
</style>
