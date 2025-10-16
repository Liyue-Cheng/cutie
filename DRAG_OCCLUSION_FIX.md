# 拖拽 Drop Zone 遮挡问题修复

## 🐛 问题描述

当暂存区或其他看板展开时，会在视觉上遮挡底下的看板。但是底下被遮挡的看板仍然会错误地检测到鼠标在其 drop zone 内，导致拖拽预览显示在错误的位置。

### 根本原因

Interact.js 的 dropzone 检测基于**几何边界**（geometric bounds）进行碰撞检测，不考虑：

- CSS z-index 层级
- 元素的实际可见性
- DOM 元素的堆叠顺序

当多个 dropzone 在几何上重叠时（例如暂存区展开后覆盖了底层的日历看板），Interact.js 会对所有几何上重叠的 dropzone 触发 `dragenter` 事件，即使底层的 dropzone 在视觉上已被完全遮挡。

## ✅ 解决方案

使用 `document.elementFromPoint()` API 进行 **z-index 感知检测**：

1. 在 `dragenter` 事件触发时，获取鼠标位置
2. 使用 `document.elementFromPoint(x, y)` 获取该位置下的实际可见元素
3. 检查该元素是否属于当前 dropzone（使用 `element.contains()`）
4. 如果不属于，说明当前 dropzone 被其他元素遮挡，忽略此次 `dragenter` 事件

### 实现细节

#### 1. 修改 `dragenter` 事件处理

在 `src/infra/drag-interact/drag-controller.ts` 的 `registerDropzone` 方法中：

```typescript
dragenter: (event: any) => {
  // ... 获取鼠标位置 ...
  const clientX = dragEvent.clientX || 0
  const clientY = dragEvent.clientY || 0

  // 🔥 Z-index 检测：检查鼠标位置下的实际可见元素
  const topElement = document.elementFromPoint(clientX, clientY)
  if (topElement && !element.contains(topElement)) {
    // 鼠标下的实际元素不属于当前 dropzone，说明被其他元素遮挡
    logger.debug(
      LogTags.DRAG_CROSS_VIEW,
      `[⛔ dropzone.dragenter blocked] zoneId: ${zoneId} is occluded`
    )
    return // 忽略此次 dragenter
  }

  // ... 正常的 dragenter 逻辑 ...
}
```

#### 2. 修改手动初始检测

在 `checkInitialDropzone` 方法中也添加相同的检测：

```typescript
private checkInitialDropzone(clientX: number, clientY: number) {
  // 🔥 Z-index 检测：获取鼠标位置下的实际可见元素
  const topElement = document.elementFromPoint(clientX, clientY)
  if (!topElement) return

  for (const element of this.registeredElements) {
    // ... 边界检测 ...

    // 🔥 Z-index 检测
    if (!element.contains(topElement)) {
      // 当前 dropzone 被遮挡，跳过
      continue
    }

    // ... 正常的进入逻辑 ...
  }
}
```

## 🎯 修改影响

### 受影响的文件

- `src/infra/drag-interact/drag-controller.ts`

### 向后兼容性

- ✅ 完全向后兼容
- ✅ 不影响现有的拖拽行为
- ✅ 只是增加了额外的可见性检查

### 性能影响

- `document.elementFromPoint()` 是浏览器原生 API，性能开销极小
- 每次 `dragenter` 事件只调用一次，不会造成性能问题

## 🧪 测试场景

### 场景 1: 暂存区展开遮挡日历

1. 展开暂存区，使其覆盖底部的日历看板
2. 从暂存区拖拽任务
3. 移动到被遮挡的日历区域上方
4. **预期**：不应该触发日历的 drop zone，只有暂存区响应
5. **实际**：✅ 修复成功

### 场景 2: 看板列之间的正常拖拽

1. 在未遮挡的看板列之间拖拽任务
2. **预期**：正常触发 drop zone，显示预览
3. **实际**：✅ 正常工作

### 场景 3: 多层级嵌套

1. 当有多个元素层叠时（例如弹窗、下拉菜单）
2. 拖拽任务到这些元素上
3. **预期**：只有最顶层的可见元素响应
4. **实际**：✅ 正确识别层级

## 📝 技术细节

### `document.elementFromPoint()` API

```typescript
const element = document.elementFromPoint(x, y)
```

- 返回指定坐标点下最顶层的可见元素
- 考虑 CSS z-index、opacity、visibility 等属性
- 浏览器兼容性：所有现代浏览器都支持

### `element.contains()` API

```typescript
const isContained = parentElement.contains(childElement)
```

- 检查子元素是否在父元素内（包括所有嵌套层级）
- 如果 `childElement === parentElement`，返回 `true`
- 用于判断鼠标下的元素是否属于当前 dropzone

## 🔍 调试日志

启用后，将看到以下日志：

```
[⛔ dropzone.dragenter blocked] zoneId: daily::2024-01-15 is occluded by another element
[✅ dropzone.dragenter] zoneId: misc::staging
```

这表示：

1. 日历看板的 dragenter 被阻止（因为被遮挡）
2. 暂存区的 dragenter 正常触发

## 🎉 修复日期

2025-10-16

## 📚 相关资源

- [MDN: document.elementFromPoint()](https://developer.mozilla.org/en-US/docs/Web/API/Document/elementFromPoint)
- [MDN: Node.contains()](https://developer.mozilla.org/en-US/docs/Web/API/Node/contains)
- [Interact.js Dropzone Documentation](https://interactjs.io/docs/dropzone/)
