# 混合方案详解：解决同区域拖放问题

## 🎯 问题描述

### **原生 dropzone 事件的限制**

interact.js 的 `dropzone` 事件在以下情况下**不会触发**：

```
场景1：跨区域拖放 ✅
┌─────────────┐     拖动      ┌─────────────┐
│  看板 A     │ ──────────> │  看板 B     │
│  [任务1]    │             │             │
└─────────────┘             └─────────────┘

✅ dragenter 触发！因为从 A 区域进入 B 区域


场景2：同区域内拖放 ❌
┌─────────────┐
│  看板 A     │
│  [任务1] ◄──┐  在内部拖动
│  [任务2]    │
│  [任务3]    │
└─────────────┘

❌ dragenter 不触发！因为从未离开 A 区域
```

### **问题根源**

当你在一个 dropzone **内部**点击并开始拖动时：

1. 鼠标已经在 dropzone 内
2. `dragenter` 事件只在**进入**时触发
3. 由于你从未"离开"过，所以不会触发进入事件
4. 结果：同一看板内的排序功能失效

---

## 🔧 混合方案设计

### **核心思路**

```typescript
if (刚开始拖动 && 还没进入任何区域) {
  // 手动检测一次起始位置
  checkInitialDropzone()
} else if (已经在某个区域内) {
  // 完全依赖原生事件
  // - 移动到其他区域 → dragenter 自动触发
  // - 离开当前区域 → dragleave 自动触发
}
```

### **状态转换流程**

```
用户按下鼠标
    ↓
IDLE → PREPARING
    ↓
移动超过 5px
    ↓
PREPARING → DRAGGING
    ↓
┌─────────────────────────────────────────┐
│  🔥 关键分支点                          │
├─────────────────────────────────────────┤
│                                          │
│  在 DRAGGING 阶段，每一帧执行：         │
│                                          │
│  checkInitialDropzone(clientX, clientY) │
│      ↓                                   │
│  找到起始 dropzone？                    │
│      ↓                                   │
│  是：手动触发 enterTarget()             │
│      进入 OVER_TARGET 状态              │
│      之后完全依赖原生事件               │
│      ✅ 解决同区域拖放问题              │
│                                          │
│  否：继续等待原生 dragenter             │
│      ✅ 正常的跨区域拖放                │
│                                          │
└─────────────────────────────────────────┘
    ↓
DRAGGING → OVER_TARGET
    ↓
此后完全依赖原生事件：
  - 移动到其他区域 → dragenter 触发
  - 离开当前区域 → dragleave 触发
  - 释放鼠标 → drop 触发
```

---

## 📊 详细实现

### **1. draggable.move 事件处理**

```typescript
move: (event) => {
  // 1. 更新幽灵元素（始终执行）
  this.updateGhostPosition(event.clientX, event.clientY)

  // 2. 检查是否达到拖拽阈值
  if (this.state.phase === 'PREPARING') {
    const distance = getDistance(startPos, { x: event.clientX, y: event.clientY })
    if (distance >= 5) {
      this.startDragging() // PREPARING → DRAGGING
    }
  }

  // 3. 🔥 核心：智能分支处理
  if (this.state.phase === 'DRAGGING' && this.state.session) {
    // 场景A：刚进入 DRAGGING 阶段，手动检测起始位置
    this.checkInitialDropzone(event.clientX, event.clientY)
    // ⚠️ 这个方法会在找到 dropzone 后将状态改为 OVER_TARGET
    // ⚠️ 之后不再执行，因为 phase 已经不是 DRAGGING
  } else if (this.state.phase === 'OVER_TARGET' && this.currentDropzoneElement) {
    // 场景B：已经在某个区域内，实时更新 dropIndex
    const dropIndex = this.calculateDropIndexForZone(event.clientY, this.currentDropzoneElement)
    dragPreviewActions.updateDropIndex(dropIndex)
  }
}
```

### **2. checkInitialDropzone 方法**

```typescript
/**
 * 🔥 检查初始 dropzone
 * 用于解决"在起始 dropzone 内开始拖动时，原生 dragenter 不会触发"的问题
 *
 * 特点：
 * - 只在 DRAGGING 阶段执行
 * - 找到匹配后立即 return，只执行一次
 * - 执行后状态变为 OVER_TARGET，不再进入此分支
 */
private checkInitialDropzone(clientX: number, clientY: number) {
  // 双重保护：确保只在 DRAGGING 阶段执行
  if (this.state.phase !== 'DRAGGING') return
  if (!this.state.session) return

  // 遍历所有已注册的 dropzone
  for (const element of this.registeredElements) {
    const rect = element.getBoundingClientRect()

    // 检查鼠标是否在矩形内
    const isInside =
      clientX >= rect.left &&
      clientX <= rect.right &&
      clientY >= rect.top &&
      clientY <= rect.bottom

    if (isInside) {
      const zoneId = element.getAttribute('data-zone-id')
      const type = element.getAttribute('data-zone-type')

      if (zoneId) {
        logger.debug('[🔍 Manual check] Found initial dropzone:', zoneId)

        // 手动触发进入逻辑（模拟 dragenter）
        this.currentDropzoneElement = element
        const isPhysicalZone = type === 'kanban'

        if (isPhysicalZone) {
          // Kanban 区域：显示实体预览
          const dropIndex = this.calculateDropIndexForZone(clientY, element)
          dragPreviewActions.setKanbanPreview({
            ghostTask: this.state.session.object.data,
            sourceZoneId: this.state.session.source.viewId,
            targetZoneId: zoneId,
            mousePosition: { x: clientX, y: clientY },
            dropIndex,
          })
        } else {
          // 日历等非物理区域：触发回弹
          dragPreviewActions.triggerRebound()
        }

        // 进入目标区域状态（DRAGGING → OVER_TARGET）
        this.enterTarget(zoneId, dropIndex)

        // ⚠️ 关键：找到后立即返回
        // 下一帧 phase === 'OVER_TARGET'，不再进入此方法
        return
      }
    }
  }

  // 如果循环结束都没找到，说明鼠标不在任何 dropzone 内
  // 等待原生 dragenter 事件触发（跨区域拖放场景）
}
```

### **3. 原生 dropzone 事件**

```typescript
interact(element).dropzone({
  accept: '.task-card-wrapper',
  overlap: 'pointer',

  listeners: {
    dragenter: (event) => {
      // ✅ 跨区域拖放时自动触发
      logger.debug('[✅ dropzone.dragenter]', zoneId)

      this.currentDropzoneElement = element
      const dropIndex = this.calculateDropIndexForZone(clientY, element)

      dragPreviewActions.setKanbanPreview({
        ghostTask: this.state.session.object.data,
        sourceZoneId: this.state.session.source.viewId,
        targetZoneId: zoneId,
        mousePosition: { x: clientX, y: clientY },
        dropIndex,
      })

      this.enterTarget(zoneId, dropIndex)
    },

    dragleave: () => {
      // ✅ 离开当前区域时自动触发
      logger.debug('[dropzone.dragleave]', zoneId)

      this.currentDropzoneElement = null
      this.leaveTarget()

      // 延迟检查是否需要回弹
      setTimeout(() => {
        if (this.state.phase !== 'OVER_TARGET') {
          dragPreviewActions.triggerRebound()
        }
      }, 10)
    },

    drop: async () => {
      // ✅ 释放鼠标时触发
      logger.debug('[✅ dropzone.drop]', zoneId)
      await this.executeDrop()
    },
  },
})
```

---

## 🎭 完整场景演示

### **场景1：同区域内排序**

```
1. 用户在"今日任务"看板内点击"任务A"
   ↓
2. IDLE → PREPARING
   ↓
3. 移动 5px
   ↓
4. PREPARING → DRAGGING
   ↓
5. 🔥 checkInitialDropzone 执行
   for (dropzone of registeredDropzones) {
     if (鼠标在 dropzone 内) {
       找到了！是"今日任务"看板
       ↓
       手动触发 enterTarget("daily::today")
       ↓
       DRAGGING → OVER_TARGET ✅
       ↓
       return (不再检测)
     }
   }
   ↓
6. 之后每一帧：
   phase === 'OVER_TARGET' ✅
   ↓
   只更新 dropIndex（光标上下移动）
   ↓
   预览元素实时调整位置
   ↓
7. 释放鼠标
   ↓
   dropzone.drop 触发
   ↓
   执行排序操作 ✅
```

### **场景2：跨区域拖放**

```
1. 用户在"今日任务"看板内点击"任务A"
   ↓
2. IDLE → PREPARING → DRAGGING
   ↓
3. 🔥 checkInitialDropzone 执行
   找到"今日任务"看板
   ↓
   DRAGGING → OVER_TARGET
   ↓
4. 鼠标移动到"明日任务"看板
   ↓
5. ✅ 原生 dragleave 触发（离开今日任务）
   this.leaveTarget()
   ↓
6. ✅ 原生 dragenter 触发（进入明日任务）
   this.enterTarget("daily::tomorrow")
   ↓
7. 预览切换到新看板 ✅
   ↓
8. 释放鼠标
   ↓
   dropzone.drop 触发
   ↓
   执行跨看板移动操作 ✅
```

---

## 📈 性能分析

### **完全手动检测方案（旧）**

```
每一帧（60fps）：
  for (每个 dropzone) {
    getBoundingClientRect()  ← 强制重排
    计算是否在内部
    更新状态
    更新预览
  }

拖动 2 秒：
- 总检测次数：120 帧 × 3 dropzone = 360 次
- DOM 查询：360 次 getBoundingClientRect()
- 状态更新：~120 次
```

### **混合方案（当前）**

```
第 1-N 帧（DRAGGING 阶段）：
  checkInitialDropzone()
    for (每个 dropzone) {
      getBoundingClientRect()
      找到后立即 return  ← 只执行一次！
    }

第 N+1 帧之后（OVER_TARGET 阶段）：
  只更新 dropIndex（无 DOM 查询）
  依赖原生 dragenter/dragleave

拖动 2 秒：
- 总检测次数：1 次（找到后停止）
- DOM 查询：3 次（最多遍历所有 dropzone 一次）
- 状态更新：2-3 次（进入/离开）

性能提升：减少 99% 的 DOM 查询！
```

---

## ✅ 优势总结

### **1. 兼容性完美**

| 场景              | 是否工作 | 实现方式         |
| ----------------- | -------- | ---------------- |
| 同区域内排序      | ✅       | 手动检测一次     |
| 跨区域拖放        | ✅       | 原生事件         |
| 拖出所有区域      | ✅       | 原生 dragleave   |
| 多层嵌套 dropzone | ✅       | interact.js 处理 |

### **2. 性能优秀**

- ✅ 只在开始时检测一次（vs 完全手动的 60次/秒）
- ✅ 之后完全依赖原生事件（零开销）
- ✅ 减少 99% 的 DOM 查询

### **3. 代码简洁**

- ✅ `checkInitialDropzone` 只有 ~50 行
- ✅ 逻辑清晰：DRAGGING 时检测一次，OVER_TARGET 时依赖原生
- ✅ 易于调试：日志清晰区分手动检测和原生事件

### **4. 易于维护**

- ✅ 符合 Web 标准（尽可能使用原生事件）
- ✅ 职责分离（interact.js 负责大部分，我们只补充初始检测）
- ✅ 向后兼容（如果 interact.js 未来优化，我们的代码仍然有效）

---

## 🎯 最佳实践

### **什么时候使用混合方案**

✅ **推荐使用**：

- 需要支持同区域内拖放排序
- 需要支持跨区域拖放移动
- 需要良好的性能
- 需要标准化的实现

❌ **不需要**：

- 只有跨区域拖放（纯原生事件即可）
- 不需要同区域排序（纯原生事件即可）

### **调试技巧**

查看控制台日志，区分两种进入方式：

```javascript
// 手动检测触发的进入
[🔍 Manual check] Found initial dropzone: daily::today

// 原生事件触发的进入
[✅ dropzone.dragenter] zoneId: daily::tomorrow
```

---

## 🔄 未来改进方向

如果 interact.js 未来版本改进了 dropzone 检测逻辑，我们可以：

1. 监控 interact.js 更新日志
2. 如果支持了"起始 dropzone 检测"，移除 `checkInitialDropzone`
3. 完全回到纯原生事件方案

这就是混合方案的优势：**向后兼容，易于升级**。
