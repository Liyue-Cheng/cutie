# 路由审计报告

## 发现的不一致问题

### 1. tasks/complete_task.rs ❌
- **端点文档**: `POST /api/tasks/{id}/completion`
- **路由注册**: `PATCH /:id/complete`
- **前端调用**: `POST /tasks/${id}/completion`
- **问题**: 路径和方法都不匹配！

### 2. tasks/reopen_task.rs ❌
- **端点文档**: `DELETE /api/tasks/{id}/completion`
- **路由注册**: `PATCH /:id/reopen`
- **前端调用**: `DELETE /tasks/${id}/completion`
- **问题**: 路径和方法都不匹配！

### 3. tasks/archive_task.rs ❌
- **端点文档**: `POST /api/tasks/{id}/archive`
- **路由注册**: `PATCH /:id/archive`
- **问题**: 方法不匹配 (POST vs PATCH)

### 4. tasks/unarchive_task.rs ❌
- **端点文档**: `POST /api/tasks/{id}/unarchive`
- **路由注册**: `PATCH /:id/unarchive`
- **问题**: 方法不匹配 (POST vs PATCH)

### 5. tasks/return_to_staging.rs ❌
- **端点文档**: `POST /api/tasks/{id}/return-to-staging`
- **路由注册**: `PATCH /:id/return-to-staging`
- **问题**: 方法不匹配 (POST vs PATCH)

### 6. tasks/restore_task.rs ❌
- **端点文档**: `PATCH /api/tasks/{id}/restore`
- **路由注册**: `PATCH /:id/restore`
- **状态**: ✅ 一致

### 7. tasks/add_schedule.rs ⚠️
- **端点文档**: `POST /api/tasks/{id}/schedules`
- **路由注册**: `POST /:id/schedules/:date`
- **问题**: 路由多了 `:date` 参数（可能合理）

### 8. tasks/permanently_delete_task.rs ❌
- **端点文档**: `DELETE /api/tasks/{id}/permanently`
- **路由注册**: `DELETE /:id/permanently-delete`
- **问题**: 路径不匹配 (/permanently vs /permanently-delete)

### 9. time_blocks/link_task.rs ❌
- **端点文档**: `POST /api/time-blocks/{block_id}/link-task`
- **路由注册**: `PATCH /:id/link-task`
- **问题**: 方法不匹配 (POST vs PATCH)

### 10. trash/empty_trash.rs ❌
- **端点文档**: `POST /api/trash/empty`
- **路由注册**: `DELETE /empty`
- **问题**: 方法不匹配 (POST vs DELETE)

### 11. area/update_area.rs ⚠️
- **端点文档**: `PUT /api/areas/{id}`
- **路由注册**: `PATCH /:id`
- **说明**: 已故意从 PUT 改为 PATCH（部分更新）

### 12. recurrences/batch_update_*.rs ⚠️
- **batch_update_instances 端点文档**: `PATCH /api/recurrences/:id/instances/batch`
- **batch_update_instances 路由注册**: `PATCH /batch-update-instances`
- **问题**: 路径完全不匹配！缺少 :id 参数

- **batch_update_template_and_instances 端点文档**: `PATCH /api/recurrences/:id/template-and-instances`
- **batch_update_template_and_instances 路由注册**: `PATCH /batch-update-template-and-instances`
- **问题**: 路径完全不匹配！缺少 :id 参数

## 总结

- **严重问题**: 12个
- **需要修复的路由**: tasks (8个), time_blocks (1个), trash (1个), recurrences (2个)

