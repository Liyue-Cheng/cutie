# Develop 文件夹

这个文件夹包含了项目开发过程中的实验性功能和未成熟的功能库。

## 📁 文件夹结构

### vue-draxis/
自研的Vue拖放库，目前处于开发阶段，功能不完整。

**包含文件：**
- `drag/` - 拖放功能核心实现
  - `directives/` - Vue指令实现
    - `c-draggable.ts` - 可拖拽指令
    - `c-droppable.ts` - 可放置指令
  - `drag-coordinator.ts` - 拖放协调器
  - `index.ts` - 主入口文件
  - `types.ts` - 类型定义
  - `useDragCreator.ts` - 程序化拖拽创建
  - `useDraggable.ts` - 可拖拽组合式函数
  - `useDroppable.ts` - 可放置组合式函数
- `DragRenderer.vue` - 拖放渲染器组件
- `DragTestView.vue` - 拖放功能测试页面

**状态：** 🚧 开发中 - 功能不完整，不建议在生产环境使用

**替代方案：** 项目目前使用 `vuedraggable@next` 作为拖放功能的实现

## ⚠️ 注意事项

1. **不要在生产环境使用** - 这些功能库处于开发阶段，可能存在bug
2. **随时可能被移除** - 如果功能被正式实现或废弃，相关文件会被清理
3. **仅供开发参考** - 可以作为学习参考或功能实现的灵感来源

## 🔄 如何重新启用

如果需要重新启用这些功能，需要：

1. 将相关文件移回 `src/` 目录
2. 在 `src/main.ts` 中重新导入
3. 在 `src/App.vue` 中重新注册组件
4. 在 `src/router/index.ts` 中重新添加路由

## 📝 历史记录

- **2025-01-27** - 将Vue-Draxis拖放库移动到develop文件夹，因为功能不完整且已被vuedraggable替代
