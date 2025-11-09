# 月视图数据主动拉取

## 问题描述

日历月视图切换时，没有主动从后端拉取该月份的数据，导致：

1. 循环任务没有正确生成
2. 数据可能不完整
3. 依赖已有的缓存数据，可能过期

## 解决方案

在 `handleDatesSet` 回调中，当视图类型为月视图时，主动拉取可见日期范围的数据。

### 实现逻辑

```typescript
const handleDatesSet = async (dateInfo: DatesSetArg) => {
  // ... 其他逻辑 ...

  // 🔥 月视图：拉取可见日期范围的数据（包括循环任务生成）
  if (props.viewType === 'month') {
    const startDate = dateInfo.start // FullCalendar 提供的可见范围起始日期
    const endDate = dateInfo.end // FullCalendar 提供的可见范围结束日期

    // 转换为 YYYY-MM-DD 格式
    const startDateStr = formatDate(startDate)
    const endDateStr = formatDate(endDate)

    // 拉取该月份的时间块数据（后端会自动生成循环任务）
    await timeBlockStore.fetchTimeBlocksForRange(startDateStr, endDateStr)
  }
}
```

### 关键点

1. **`dateInfo.start` 和 `dateInfo.end`**：
   - FullCalendar 提供的可见日期范围
   - 月视图通常包含上月末尾和下月开头的几天
   - 例如：2024年11月的月视图可能显示 `2024-10-27` 到 `2024-12-01`

2. **后端自动生成循环任务**：
   - 后端 API `/time-blocks?start_date=XXX&end_date=XXX` 会自动生成该范围内的循环任务实例
   - 前端不需要额外处理循环任务的展开逻辑

3. **触发时机**：
   - 初次加载月视图
   - 切换到上一个月/下一个月
   - 从其他视图切换到月视图

## 修改文件

- `src/components/parts/CuteCalendar.vue`
  - 第 143-203 行：修改 `handleDatesSet` 为 async 函数
  - 第 159-191 行：添加月视图数据拉取逻辑

## API 调用

### 请求

```
GET /time-blocks?start_date=2024-11-01&end_date=2024-11-30
```

### 响应

```json
[
  {
    "id": "uuid-1",
    "title": "会议",
    "start_time": "2024-11-05T09:00:00Z",
    "end_time": "2024-11-05T10:00:00Z",
    "is_all_day": false,
    "linked_tasks": [...]
  },
  {
    "id": "uuid-2",
    "title": "每日站会（循环任务实例）",
    "start_time": "2024-11-01T10:00:00Z",
    "end_time": "2024-11-01T10:30:00Z",
    "is_all_day": false,
    "linked_tasks": [...]
  },
  // ... 更多时间块
]
```

## 数据流

```
用户操作
  ↓
切换到月视图 / 切换月份
  ↓
FullCalendar 触发 datesSet 事件
  ↓
handleDatesSet(dateInfo)
  ↓
检测到 viewType === 'month'
  ↓
调用 timeBlockStore.fetchTimeBlocksForRange(start, end)
  ↓
后端 API: GET /time-blocks?start_date=XXX&end_date=XXX
  ↓
后端生成循环任务实例并返回
  ↓
前端 Store 更新数据
  ↓
calendarEvents computed 自动重新计算
  ↓
FullCalendar 显示更新后的事件
```

## 优势

### ✅ 数据完整性

- 每次切换月份都会拉取最新数据
- 循环任务由后端生成，保证正确性

### ✅ 性能优化

- 只拉取可见范围的数据，不会加载整年的数据
- 利用 FullCalendar 提供的精确日期范围

### ✅ 简化前端逻辑

- 前端不需要处理循环任务的展开
- 后端统一管理循环任务生成规则

## 测试要点

### 1. 初次加载月视图

- [ ] 打开应用，切换到月视图
- [ ] 应该看到网络请求：`GET /time-blocks?start_date=...&end_date=...`
- [ ] 日历应该显示该月的所有任务和时间块

### 2. 切换月份

- [ ] 点击"上一个月"按钮
- [ ] 应该看到新的网络请求，拉取上个月的数据
- [ ] 日历应该显示上个月的任务

### 3. 循环任务

- [ ] 创建一个每日循环任务
- [ ] 切换到月视图
- [ ] 应该看到该月每一天都有这个循环任务的实例

### 4. 视图切换

- [ ] 从日视图切换到月视图
- [ ] 应该触发数据拉取
- [ ] 从月视图切换到日视图
- [ ] 不应该触发额外的拉取（日视图有自己的数据加载逻辑）

### 5. 筛选功能

- [ ] 在月视图下，取消勾选"循环任务"
- [ ] 循环任务应该隐藏（但数据已经拉取）
- [ ] 重新勾选，循环任务应该重新显示

## 注意事项

1. **避免重复拉取**：
   - 如果快速切换月份，可能会触发多次请求
   - 可以考虑添加防抖或取消之前的请求

2. **加载状态**：
   - 目前没有显示加载指示器
   - 可以考虑在拉取数据时显示 loading 状态

3. **错误处理**：
   - 已经添加了 try-catch 和日志记录
   - 如果拉取失败，用户可能看不到完整数据

## 相关 API

### TimeBlockStore

```typescript
/**
 * 拉取指定日期范围的时间块
 * @param startDate YYYY-MM-DD
 * @param endDate YYYY-MM-DD
 */
async function fetchTimeBlocksForRange(startDate: string, endDate: string): Promise<TimeBlockView[]>
```

### FullCalendar DatesSetArg

```typescript
interface DatesSetArg {
  start: Date // 可见范围的起始日期
  end: Date // 可见范围的结束日期
  view: ViewApi // 当前视图信息
}
```

## 提交信息

```
feat: 月视图切换时主动拉取日期范围数据

- 在 handleDatesSet 回调中检测月视图
- 调用 fetchTimeBlocksForRange 拉取可见日期范围的数据
- 确保循环任务由后端正确生成
- 避免依赖不完整的缓存数据

Benefits:
- 循环任务正确显示
- 数据始终是最新的
- 简化前端循环任务处理逻辑
```
