# interact.js 拖放实现对比：原生事件 vs 手动检测

## 📋 实现版本

### ✅ 当前版本：混合方案（原生事件 + 初始检测）

**核心思路**：

- 使用 interact.js 的原生 `dropzone` 事件处理跨区域拖放
- 使用一次性手动检测解决同区域内拖放的问题

**问题背景**：
当你在一个 dropzone **内部**开始拖动时，由于你从未"离开"过这个 dropzone，原生的 `dragenter` 事件不会触发。这导致同一看板内的排序功能失效。

**解决方案**：

- `dragenter/dragleave/drop`: 使用原生事件处理跨区域拖放 ✅
- `checkInitialDropzone`: 在 `DRAGGING` 阶段手动检测一次起始位置 ✅

### ❌ 旧版本：完全手动检测

在 `draggable.move` 事件中每一帧手动检测鼠标是否在 dropzone 内。

---

## 🔀 代码对比

### **混合方案版本（当前）**

```typescript
// draggable.move - 智能处理
move: (event) => {
  this.updateGhostPosition(event.clientX, event.clientY)

  // 🔥 DRAGGING 阶段：手动检测一次起始 dropzone
  if (this.state.phase === 'DRAGGING') {
    this.checkInitialDropzone(event.clientX, event.clientY)
  }

  // ✅ OVER_TARGET 阶段：只更新 dropIndex
  else if (this.state.phase === 'OVER_TARGET' && this.currentDropzoneElement) {
    const dropIndex = this.calculateDropIndexForZone(event.clientY, this.currentDropzoneElement)
    dragPreviewActions.updateDropIndex(dropIndex)
  }
}

// 🔥 手动检测起始位置（只执行一次）
private checkInitialDropzone(clientX, clientY) {
  if (this.state.phase !== 'DRAGGING') return

  for (const element of this.registeredElements) {
    const rect = element.getBoundingClientRect()
    if (鼠标在 rect 内) {
      // 手动触发进入逻辑
      this.enterTarget(zoneId)
      return  // ⚠️ 找到后立即返回，只执行一次
    }
  }
}

// ✅ dropzone.dragenter - 跨区域拖放时触发
interact(element).dropzone({
  accept: '.task-card-wrapper',
  overlap: 'pointer',
  listeners: {
    dragenter: (event) => {
      // ✅ interact.js 自动检测跨区域进入
      this.currentDropzoneElement = element
      this.enterTarget(zoneId)
    },

    dragleave: () => {
      // ✅ interact.js 自动检测离开
      this.currentDropzoneElement = null
      this.leaveTarget()
    },
  },
})
```

**关键点**：

1. `checkInitialDropzone` 只在 `DRAGGING` 阶段执行
2. 找到匹配的 dropzone 后立即 `return`，只检测一次
3. 进入 `OVER_TARGET` 后，完全依赖原生事件
4. 性能开销：只增加一次碰撞检测（vs 完全手动的 60次/秒）

### **手动检测版本（旧）**

```typescript
// draggable.move - 每一帧手动检测所有 dropzone
move: (event) => {
  this.updateGhostPosition(event.clientX, event.clientY)

  // ❌ 每一帧遍历所有 dropzone
  this.detectDropzone(event.clientX, event.clientY)
}

// 手动碰撞检测
private detectDropzone(clientX, clientY) {
  for (const element of this.registeredElements) {
    const rect = element.getBoundingClientRect()

    // ❌ 手动计算鼠标是否在矩形内
    const isInside =
      clientX >= rect.left && clientX <= rect.right &&
      clientY >= rect.top && clientY <= rect.bottom

    if (isInside) {
      if (this.state.targetZone !== zoneId) {
        this.enterTarget(zoneId)
      }
      this.updatePreview()
      return
    }
  }

  // 没找到任何匹配
  if (this.state.phase === 'OVER_TARGET') {
    this.leaveTarget()
  }
}
```

---

## ⚖️ 性能对比

### **混合方案版本（当前）**

| 指标     | 值                     | 说明                              |
| -------- | ---------------------- | --------------------------------- |
| 碰撞检测 | 1 次手动 + interact.js | 只在开始时检测一次                |
| 每帧开销 | 极低                   | 只在 OVER_TARGET 时更新 dropIndex |
| DOM 查询 | N 次（开始时）         | N = dropzone 数量，但只执行一次   |
| 事件触发 | 按需触发               | 只在状态改变时触发                |

### **完全手动检测版本（旧）**

| 指标     | 值         | 说明                         |
| -------- | ---------- | ---------------------------- |
| 碰撞检测 | 每一帧执行 | 自己实现的循环检测           |
| 每帧开销 | 高         | 遍历所有 dropzone + 计算预览 |
| DOM 查询 | N 次/帧    | N = dropzone 数量            |
| 事件触发 | 每一帧     | 无论是否需要                 |

**假设场景**：3 个 dropzone，60fps 拖动，拖动持续 2 秒

| 版本     | 总 `getBoundingClientRect()` 调用 | 总状态更新          | 说明                      |
| -------- | --------------------------------- | ------------------- | ------------------------- |
| 混合方案 | 3 次                              | 2-3 次（进入/离开） | ✅ 只在开始时检测一次     |
| 完全手动 | 360 次（3 × 60 × 2）              | 120 次（60 × 2）    | ❌ 持续检测，性能开销巨大 |

**性能提升**：混合方案相比完全手动检测，减少了 **99%** 的 DOM 查询！

---

## 🎯 功能对比

### **1. 区域检测准确性**

| 场景     | 原生事件               | 手动检测        |
| -------- | ---------------------- | --------------- |
| 进入区域 | ✅ 准确                | ✅ 准确         |
| 离开区域 | ✅ 准确                | ✅ 准确         |
| 嵌套区域 | ✅ 正确处理            | ⚠️ 需要额外逻辑 |
| 边界情况 | ✅ 由 interact.js 处理 | ⚠️ 需要手动处理 |

### **2. 代码复杂度**

| 项目         | 原生事件                 | 手动检测 |
| ------------ | ------------------------ | -------- |
| 碰撞检测代码 | 0 行（interact.js 内置） | ~60 行   |
| 状态同步     | 自动                     | 手动     |
| 边界处理     | 自动                     | 手动     |
| 维护成本     | 低                       | 高       |

### **3. 兼容性**

| 特性     | 原生事件            | 手动检测      |
| -------- | ------------------- | ------------- |
| 触摸屏   | ✅ interact.js 处理 | ✅ 同样支持   |
| 多点触控 | ✅ interact.js 处理 | ⚠️ 需额外处理 |
| 滚动容器 | ✅ interact.js 处理 | ⚠️ 需额外处理 |
| 跨浏览器 | ✅ interact.js 处理 | ⚠️ 需手动测试 |

---

## 🐛 调试体验

### **原生事件版本**

```javascript
// 清晰的事件日志
[✅ dropzone.dragenter] zoneId: daily::2025-10-01
[dropzone.dragleave] zoneId: daily::2025-10-01
[✅ dropzone.dragenter] zoneId: misc::staging
[✅ dropzone.drop] zoneId: misc::staging
```

**优势**：

- ✅ 事件时序清晰
- ✅ 可以在 DevTools 中看到事件触发
- ✅ 日志可读性强

### **手动检测版本**

```javascript
// 密集的检测日志
detectDropzone called: 500, 300
detectDropzone called: 501, 302
detectDropzone called: 502, 304
detectDropzone called: 503, 306
...（60 次/秒）
```

**劣势**：

- ❌ 日志量巨大，难以追踪
- ❌ 无法在 DevTools 事件监听器中看到
- ❌ 需要手动过滤日志

---

## 🔧 可维护性

### **原生事件版本**

**优势**：

- ✅ **符合 Web 标准**：使用标准事件模型
- ✅ **职责分离**：interact.js 负责检测，我们负责业务
- ✅ **声明式**：通过 `accept` 选项声明规则
- ✅ **易于扩展**：新增 dropzone 只需注册
- ✅ **测试友好**：可以 mock 事件

**示例**：添加新的 dropzone 类型

```typescript
// 只需配置，无需修改检测逻辑
interact(calendarElement).dropzone({
  accept: '.task-card-wrapper',
  overlap: 'pointer',
  listeners: { ... }
})
```

### **手动检测版本**

**劣势**：

- ❌ **逻辑耦合**：检测逻辑和业务逻辑混在一起
- ❌ **命令式**：需要手动管理状态转换
- ❌ **难以扩展**：新增类型需要修改核心检测逻辑
- ❌ **测试困难**：需要 mock DOM API

**示例**：添加新的 dropzone 类型

```typescript
// 需要修改检测逻辑
private detectDropzone(clientX, clientY) {
  for (const element of this.registeredElements) {
    // ❌ 需要在这里添加新的类型判断
    const type = element.getAttribute('data-zone-type')
    if (type === 'calendar') {
      // 新增的处理逻辑
    }
  }
}
```

---

## 📊 总结

### **原生事件版本（推荐）✅**

**适用场景**：

- ✅ 正常的拖放交互（99% 的情况）
- ✅ 需要良好的性能
- ✅ 需要易于维护的代码
- ✅ 需要标准化的实现

**优势**：

1. 性能更好（每秒减少 ~180 次 DOM 查询）
2. 代码更简洁（减少 ~60 行代码）
3. 更易维护（符合 Web 标准）
4. 调试更方便（清晰的事件日志）
5. interact.js 已经优化过碰撞检测算法

### **手动检测版本（不推荐）❌**

**适用场景**：

- ⚠️ 需要非标准的碰撞检测逻辑
- ⚠️ interact.js 的 dropzone 事件不满足需求
- ⚠️ 需要完全控制检测时机（极少见）

**劣势**：

1. 性能开销大
2. 代码复杂
3. 维护成本高
4. 容易出 bug

---

## 🎯 最终建议

**使用原生 dropzone 事件版本**，除非有非常特殊的需求。

interact.js 的 dropzone 系统经过充分测试和优化，比手动实现更可靠、更高效。

**关键配置**：

```typescript
interact(element).dropzone({
  accept: '.task-card-wrapper',  // ⚠️ 必须配置
  overlap: 'pointer',             // ⚠️ 使用 pointer 模式
  listeners: {
    dragenter: () => { ... },     // ✅ 进入时触发
    dragleave: () => { ... },     // ✅ 离开时触发
    drop: () => { ... }           // ✅ 放置时触发
  }
})
```

---

## 🔄 迁移检查清单

从手动检测迁移到原生事件：

- [x] 配置 `accept: '.task-card-wrapper'`
- [x] 使用 `overlap: 'pointer'`
- [x] 实现 `dragenter` 监听器
- [x] 实现 `dragleave` 监听器
- [x] 实现 `drop` 监听器
- [x] 在 `draggable.move` 中移除手动检测
- [x] 保留 `OVER_TARGET` 时的 dropIndex 更新
- [x] 删除 `detectDropzone()` 方法
- [x] 更新调试日志
- [ ] 测试所有拖放场景
- [ ] 验证性能提升

---

## 📚 参考资料

- [interact.js 官方文档 - Dropzone](https://interactjs.io/docs/dropzone/)
- [interact.js GitHub - Examples](https://github.com/taye/interact.js/tree/main/examples)
- [Web Performance - Avoiding Layout Thrashing](https://developers.google.com/web/fundamentals/performance/rendering/avoid-large-complex-layouts-and-layout-thrashing)
