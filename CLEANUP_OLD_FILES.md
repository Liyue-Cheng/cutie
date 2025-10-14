# 需要删除的旧文件清单

## ✅ 已移动到 infra，需要删除的旧文件

### services/ 目录

以下文件已移动到 `src/infra/`，可以安全删除：

```bash
# 已移动到 infra/commandBus/
services/commandBus/
  - index.ts               # → infra/commandBus/index.ts
  - CommandBus.ts          # → infra/commandBus/CommandBus.ts
  - types.ts               # → infra/commandBus/types.ts
  - handlers/              # → infra/commandBus/handlers/
  - README.md              # → infra/commandBus/README.md

# 已移动到 infra/correlation/
services/correlationId.ts  # → infra/correlation/correlationId.ts

# 已移动到 infra/errors/
services/errorHandler.ts   # → infra/errors/errorHandler.ts

# 已移动到 infra/events/
services/events.ts         # → infra/events/events.ts

# 已移动到 infra/logging/
services/logger.ts         # → infra/logging/logger.ts
services/loggerSettings.ts # → infra/logging/loggerSettings.ts
services/loggerConfig.ts   # → infra/logging/loggerConfig.ts（如果存在）
```

### stores/shared/ 目录

以下文件已移动到 `src/infra/http/`，可以安全删除：

```bash
# 已移动到 infra/http/
stores/shared/api-client.ts      # → infra/http/api-client.ts
stores/shared/error-handler.ts   # → infra/http/error-handler.ts
```

---

## ✅ 已删除的废弃文件（已完成）

以下文件已在重构中删除，功能已被新架构取代：

```bash
# CRUD 操作（已被 Command Bus 取代）
stores/task/crud-operations.ts   # ✅ 已删除

# 视图操作（已被 loaders.ts 取代）
stores/task/view-operations.ts   # ✅ 已删除

# 旧的 Correlation Tracker（已被 transactionProcessor 取代）
stores/shared/correlation-tracker.ts  # ✅ 已删除

# 旧的 Composable（已被 commandBus 取代）
composables/useTaskOperations.ts     # ✅ 已删除
```

---

## 📋 删除命令（PowerShell）

### 删除 services/ 中的旧文件

```powershell
cd C:\Users\liyue\Desktop\projects\dashboard\cutie\src

# 删除 commandBus 目录
Remove-Item -Recurse -Force services\commandBus\

# 删除单个文件
Remove-Item services\correlationId.ts
Remove-Item services\errorHandler.ts
Remove-Item services\events.ts
Remove-Item services\logger.ts
Remove-Item services\loggerSettings.ts

# 如果 loggerConfig.ts 存在
Remove-Item services\loggerConfig.ts -ErrorAction SilentlyContinue
```

### 删除 stores/shared/ 中的旧文件

```powershell
cd C:\Users\liyue\Desktop\projects\dashboard\cutie\src\stores\shared

# 删除已移动的文件
Remove-Item api-client.ts
Remove-Item error-handler.ts
```

---

## ⚠️ 保留的文件

### services/ 目录保留

```bash
services/ai.ts            # ✅ 保留（业务逻辑，不是基础设施）
services/viewAdapter.ts   # ✅ 保留（业务逻辑）
```

### stores/shared/ 目录保留

```bash
stores/shared/index.ts        # ✅ 保留（需要更新导入路径）
stores/shared/state-utils.ts # ✅ 保留（工具函数）
```

---

## 🔄 需要更新导入路径的文件

删除旧文件后，需要更新以下文件中的导入路径：

### 需要更新的导入

```typescript
// 修改前
import { apiPost } from '@/stores/shared'
import { logger } from '@/infra/logging/logger'
import { commandBus } from '@/commandBus'

// 修改后
import { apiPost } from '@/infra/http'
import { logger } from '@/infra/logging'
import { commandBus } from '@/infra/commandBus'
```

### 预计需要更新的文件数量

- Stores: ~10 个文件
- Components: ~20 个文件
- Composables: ~10 个文件
- Main.ts: 1 个文件

**总计：约 40 个文件需要更新导入路径**

---

## 💡 建议的删除顺序

1. **先测试**：确保应用可以正常运行
2. **再删除 services/ 中的旧文件**
3. **最后删除 stores/shared/ 中的旧文件**
4. **更新 `stores/shared/index.ts` 导入路径**

---

**注意：删除前请确保已提交 git，以便回滚！**
