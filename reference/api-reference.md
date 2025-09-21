# Cutie 后端 API 参考

本文档提供了 Cutie 桌面应用后端所有可用的 Tauri 命令（API）的目录。前端可以通过 `invoke` 函数调用这些命令与后端进行交互。

---

## 1. Projects (项目)

管理大型目标或长期探索。

- **`create_project(payload: CreateProjectPayload) -> Result<Project, String>`**
  - **功能**: 创建一个新项目。
  - **参数**:
    - `title: String`
    - `description: Option<String>`
    - `icon: Option<String>`
    - `color: Option<String>`
    - `status: Option<String>` (默认为 'active')
    - `metadata: Option<JsonValue>`

- **`get_project(id: Uuid) -> Result<Project, String>`**
  - **功能**: 获取指定ID的项目详情。

- **`list_projects() -> Result<Vec<Project>, String>`**
  - **功能**: 获取所有未删除的项目列表，按创建时间降序排列。

- **`update_project(id: Uuid, payload: UpdateProjectPayload) -> Result<Project, String>`**
  - **功能**: 更新指定ID的项目信息。
  - **参数**: 所有 `CreateProjectPayload` 中的字段均为可选。

- **`delete_project(id: Uuid) -> Result<(), String>`**
  - **功能**: 软删除一个项目。

---

## 2. Tasks (任务)

管理具体的待办事项。

- **`create_task(payload: CreateTaskPayload) -> Result<Task, String>`**
  - **功能**: 创建一个新任务。
  - **参数**:
    - `project_id: Option<Uuid>`
    - `title: String`
    - `status: Option<String>` (默认为 'todo')
    - `due_date: Option<DateTime<Utc>>`
    - `sort_key: String`
    - `metadata: Option<JsonValue>`

- **`get_task(id: Uuid) -> Result<Task, String>`**
  - **功能**: 获取指定ID的任务详情。

- **`list_tasks() -> Result<Vec<Task>, String>`**
  - **功能**: 获取所有未删除的任务列表，按 `sort_key` 升序排列。

- **`update_task(id: Uuid, payload: UpdateTaskPayload) -> Result<Task, String>`**
  - **功能**: 更新指定ID的任务信息。
  - **参数**: 所有 `CreateTaskPayload` 中的字段以及 `completed_at` 均为可选。

- **`delete_task(id: Uuid) -> Result<(), String>`**
  - **功能**: 软删除一个任务。

---

## 3. Checkpoints (检查点)

管理任务下的引导性步骤。

- **`create_checkpoint(payload: CreateCheckpointPayload) -> Result<Checkpoint, String>`**
  - **功能**: 为任务创建一个新检查点。

* **`list_inbox_tasks() -> Result<Vec<Task>, String>`**
  - **功能**: 获取所有收件箱中的任务（未关联任何项目和活动），按 `sort_key` 升序排列。

- **参数**:
  - `task_id: Uuid`
  - `title: String`
  - `sort_key: String`

- **`get_checkpoint(id: Uuid) -> Result<Checkpoint, String>`**
  - **功能**: 获取指定ID的检查点详情。

- **`list_checkpoints_for_task(task_id: Uuid) -> Result<Vec<Checkpoint>, String>`**
  - **功能**: 获取指定任务下的所有检查点列表，按 `sort_key` 升序排列。

- **`update_checkpoint(id: Uuid, payload: UpdateCheckpointPayload) -> Result<Checkpoint, String>`**
  - **功能**: 更新指定ID的检查点信息。
  - **参数**: `title`, `is_completed`, `sort_key` 均为可选。

- **`delete_checkpoint(id: Uuid) -> Result<(), String>`**
  - **功能**: 软删除一个检查点。

---

## 4. Activities (活动)

管理日历上的时间块。

- **`create_activity(payload: CreateActivityPayload) -> Result<Activity, String>`**
  - **功能**: 创建一个新活动。
  - **参数**:
    - `title: Option<String>`
    - `start_time: DateTime<Utc>`
    - `end_time: DateTime<Utc>`
    - `timezone: Option<String>`
    - `is_all_day: Option<bool>` (默认为 `false`)
    - `color: Option<String>`
    - `metadata: Option<JsonValue>`

- **`get_activity(id: Uuid) -> Result<Activity, String>`**
  - **功能**: 获取指定ID的活动详情。

- **`list_activities() -> Result<Vec<Activity>, String>`**
  - **功能**: 获取所有未删除的活动列表，按开始时间升序排列。

- **`update_activity(id: Uuid, payload: UpdateActivityPayload) -> Result<Activity, String>`**
  - **功能**: 更新指定ID的活动信息。
  - **参数**: 所有 `CreateActivityPayload` 中的字段均为可选。

- **`delete_activity(id: Uuid) -> Result<(), String>`**
  - **功能**: 软删除一个活动。

---

## 5. Tags (标签)

管理跨领域的分类标签。

- **`create_tag(payload: CreateTagPayload) -> Result<Tag, String>`**
  - **功能**: 创建一个新标签。
  - **参数**:
    - `title: String`
    - `color: Option<String>`
    - `sort_key: Option<String>`

- **`get_tag(id: Uuid) -> Result<Tag, String>`**
  - **功能**: 获取指定ID的标签详情。

- **`list_tags() -> Result<Vec<Tag>, String>`**
  - **功能**: 获取所有未删除的标签列表，按 `sort_key` 升序排列。

- **`update_tag(id: Uuid, payload: UpdateTagPayload) -> Result<Tag, String>`**
  - **功能**: 更新指定ID的标签信息。
  - **参数**: 所有 `CreateTagPayload` 中的字段均为可选。

- **`delete_tag(id: Uuid) -> Result<(), String>`**
  - **功能**: 软删除一个标签。

---

## 6. Linking (关系链接)

管理实体之间的关联。

- **`link_task_to_activity(task_id: Uuid, activity_id: Uuid) -> Result<(), String>`**
  - **功能**: 将一个任务链接到一个活动上。

- **`unlink_task_from_activity(task_id: Uuid, activity_id: Uuid) -> Result<(), String>`**
  - **功能**: 解除任务和活动之间的链接。

- **`add_tag_to_project(project_id: Uuid, tag_id: Uuid) -> Result<(), String>`**
  - **功能**: 为项目添加一个标签。

- **`remove_tag_from_project(project_id: Uuid, tag_id: Uuid) -> Result<(), String>`**
  - **功能**: 从项目中移除一个标签。

- **`add_tag_to_task(task_id: Uuid, tag_id: Uuid) -> Result<(), String>`**
  - **功能**: 为任务添加一个标签。

- **`remove_tag_from_task(task_id: Uuid, tag_id: Uuid) -> Result<(), String>`**
  - **功能**: 从任务中移除一个标签。

---

## 开发者指南：如何新增一个 API 命令

当需要为后端添加新的 API 功能时，请遵循以下标准流程，以确保代码的一致性和可维护性：

1.  **创建或修改命令文件**:
    - 在 `src-tauri/src/commands/` 目录下，找到与你的功能最相关的命令文件（例如，与任务相关的功能应放入 `task_commands.rs`）。
    - 如果是一个全新的领域，可以创建一个新的 `*_commands.rs` 文件。

2.  **实现命令函数**:
    - 在对应的命令文件中，使用 `#[tauri::command]` 宏来定义你的新函数。
    - 函数需要是 `async` 的，并接收 `pool: State<'_, DbPool>` 作为数据库连接池。
    - 函数的返回值应为 `Result<T, String>`，其中 `T` 是成功时返回给前端的数据类型。

3.  **注册模块 (如果需要)**:
    - 如果你创建了一个全新的命令文件（例如 `new_feature_commands.rs`），你必须在 `src-tauri/src/commands/mod.rs` 文件中添加一行 `pub mod new_feature_commands;` 来注册这个新模块。

4.  **注册命令处理器**:
    - 打开 `src-tauri/src/lib.rs` 文件。
    - 在 `tauri::Builder::default()` 的 `.invoke_handler()` 宏中，加入你的新命令路径，例如 `commands::your_module::your_new_command,`。

5.  **更新本文档**:
    - 最后，请务必在本 API 参考手册 (`reference/api-reference.md`) 中添加关于你新 API 的说明，包括其功能、参数和返回值。
