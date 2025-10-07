# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cutie is a task management desktop application built with:
- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri (sidecar architecture with Axum HTTP server)
- **Database**: SQLite (with SQLx migrations)
- **State Management**: Pinia stores
- **Real-time**: SSE (Server-Sent Events) for state synchronization

## Development Commands

**⚠️ IMPORTANT: DO NOT START DEV SERVERS**
- **NEVER run `pnpm dev`, `pnpm tauri dev`, or `cargo run`** - the user has HMR dev servers already running
- Only run build, test, and lint commands
- Package manager: **pnpm** (not npm or yarn)

### Frontend (Vue + Vite)
```bash
pnpm build           # Build frontend (runs vue-tsc -b && vite build)
pnpm preview         # Preview production build

# ❌ DO NOT RUN: pnpm dev (dev server already running)
```

### Backend (Tauri + Rust)
```bash
# Run tests
cargo test

# Check compilation
cargo check

# Database migrations (located in src-tauri/migrations/)
# Migrations run automatically on app startup

# ❌ DO NOT RUN: cargo run (dev server already running)
# ❌ DO NOT RUN: pnpm tauri dev (dev server already running)
```

### Linting
```bash
# Frontend
pnpm exec eslint src/        # JavaScript/TypeScript linting
pnpm exec prettier --check . # Code formatting check
pnpm exec stylelint "**/*.vue" # Vue styles linting

# Backend
cargo clippy           # Rust linting
cargo fmt --check      # Rust formatting check
```

## Architecture

### Backend: Feature-Sliced Architecture

The Rust backend uses a vertical slice architecture organized by business features:

```
src-tauri/src/
├── main.rs              # Entry point (Tauri + sidecar launcher)
├── lib.rs               # Library exports
├── config/              # Configuration (app, database, server)
├── startup/             # App initialization (AppState, database, sidecar)
├── entities/            # Domain models and DTOs
│   ├── task/            # Task entity (model, DTOs, enums)
│   ├── schedule/        # Schedule entity
│   ├── time_block/      # Time block entity
│   ├── area/            # Area entity
│   └── view_preference/ # View preferences
├── features/            # Feature modules (vertical slices)
│   ├── tasks/           # Task management
│   │   ├── endpoints/   # HTTP handlers (create, update, delete, etc.)
│   │   └── shared/      # Repositories, assemblers, business logic
│   ├── views/           # View-specific queries (staging, planned, daily, etc.)
│   ├── time_blocks/     # Time block management
│   ├── areas/           # Area (project/context) management
│   └── shared/          # Cross-feature infrastructure
│       ├── repositories/    # Shared repositories
│       └── transaction.rs   # Transaction helper
└── shared/              # Cross-cutting concerns
    ├── core/            # Error handling, utilities
    ├── database/        # Connection, pagination
    ├── events/          # SSE, domain events, event dispatcher
    ├── http/            # Responses, error handlers, middleware, extractors
    └── ports/           # Abstractions (Clock, IdGenerator)
```

**Key Architecture Principles:**
- **Single File Components (SFC)**: Each endpoint is a standalone file in `endpoints/` (no mod.rs needed)
- **Write Serialization**: All write operations use `AppState::acquire_write_permit()` to serialize SQLite writes at the application level
- **Event-Driven**: State changes emit domain events via SSE for real-time frontend updates
- **Transaction Helper**: Use `TransactionHelper` in `features/shared/transaction.rs` for consistent transaction handling

### Frontend: Modular Vue Architecture

```
src/
├── main.ts              # App entry point
├── App.vue              # Root component
├── router/              # Vue Router configuration
├── stores/              # Pinia state management
│   ├── task/            # Task store (modularized)
│   │   ├── index.ts           # Main store composition
│   │   ├── core.ts            # State & getters
│   │   ├── crud-operations.ts # Create/Update/Delete actions
│   │   ├── view-operations.ts # View-specific fetches
│   │   └── event-handlers.ts  # SSE event subscriptions
│   ├── area.ts          # Area/project management
│   ├── timeblock.ts     # Time block management
│   └── view.ts          # View preferences
├── views/               # Page-level components
│   ├── HomeView.vue     # Main kanban view (all incomplete tasks)
│   ├── StagingView.vue  # Staging kanban (unscheduled tasks)
│   ├── CalendarView.vue # Calendar with time blocks
│   └── ...
├── components/          # Reusable components
│   ├── parts/           # UI building blocks
│   │   └── kanban/      # Kanban-specific components
│   ├── templates/       # Layout templates
│   └── functional/      # Logic-heavy components
├── composables/         # Vue composables
│   └── drag/            # Drag-and-drop system (see below)
└── types/               # TypeScript type definitions
    ├── dtos.ts          # Backend DTOs (auto-generated from Rust)
    ├── api.ts           # API types
    └── drag.ts          # Drag-drop types
```

**Store Architecture:**
- State: Stores normalized data (single source of truth)
- Getters: Compute derived data (filtering, grouping)
- Actions: API calls, state mutations, event subscriptions

### Drag-and-Drop System

The app uses a custom cross-view drag system (`src/composables/drag/`) for moving tasks between different views (kanban columns, calendar, etc.). See `src/composables/drag/README.md` for detailed documentation.

**Key modules:**
- `useCrossViewDrag`: Main composable for cross-view drag operations
- `useDragTransfer`: Data transfer utilities
- `useAutoScroll`: Auto-scrolling during drag
- Strategy system: Defines behavior for different drag scenarios (e.g., status→date, date→status)

**Usage pattern:**
```typescript
const crossViewDrag = useCrossViewDrag()

// On drag start
crossViewDrag.startNormalDrag(task, viewMetadata)

// On drag over
const canDrop = crossViewDrag.canDrop(sourceView, targetView)

// On drop
const result = await crossViewDrag.handleDrop(targetView, event)
```

## Backend Development Patterns

### Single File Component (SFC) Architecture

Each endpoint follows the **SFC pattern** - a self-contained file with all layers:

```rust
// --- CABC V2.1 Documentation ---
/*
CABC for `create_task`

## 1. Endpoint Signature
POST /api/tasks

## 2. High-Level Behavior

### 2.1. User Story / Scenario
> As a user, I want to quickly create a new task on any kanban board (such as Staging
> or Daily view) so that I can immediately capture my thoughts without complex steps.

### 2.2. Core Business Logic
To enable quick creation, the backend creates a new `Task` entity in the database.
To make it immediately visible and sortable, the system simultaneously creates an
`Ordering` record for this new task in its context (determined by the `context` field
in the request, defaulting to Staging if not provided).

## 3. Input/Output Specification

### 3.1. Request
**Request Body:** `application/json`
```json
{
  "title": "string (required, 1-255 chars)",
  "area_id": "string (optional, UUID)",
  "context": "object (optional)"
}
```

### 3.2. Responses
**201 Created:** Returns `TaskCardDto` with all fields populated
**422 Unprocessable Entity:** Validation failed
**404 Not Found:** Referenced entity (Area/Project) does not exist

## 4. Validation Rules
- `title`:
  - MUST exist
  - MUST be non-empty string (after trim)
  - Length MUST be <= 255 characters
- `area_id` (if provided):
  - MUST be a valid UUID
  - MUST reference an existing area in database

## 5. Business Logic Walkthrough
1. Validate request via `validation` module
2. Start database transaction
3. Generate `task_id` via `app_state.id_generator().new_uuid()`
4. Get current time via `app_state.clock().now_utc()`
5. Construct `Task` domain entity
6. Persist `Task` via `database::insert_task()`
7. Determine sort context from request (default: `misc::staging`)
8. Calculate new sort order via `database::calculate_new_sort_order()`
9. Persist ordering via `database::insert_ordering()`
10. Commit transaction
11. Query full task data (with Area, schedules, etc.)
12. Assemble `TaskCardDto` via `TaskAssembler`
13. Emit SSE event (in separate transaction)
14. Return 201 Created with `TaskCardDto`

## 6. Edge Cases
- **`area_id` does not exist:** MUST return 404 Not Found and rollback transaction
- **Empty title (after trim):** MUST return 422 with validation error
- **Concurrent creation:** Sort order calculation handles race conditions correctly

## 7. Expected Side Effects

### 7.1. Database Operations (within transaction):
- **SELECT:** 1x on `areas` table (if `area_id` provided)
- **SELECT:** 1x on `orderings` table (to get max sort_order)
- **INSERT:** 1x into `tasks` table
- **INSERT:** 1x into `orderings` table

### 7.2. SSE Events (separate transaction):
- **INSERT:** 1x into `event_outbox` table
- **BROADCAST:** `task.created` event to all connected clients

### 7.3. Logging:
- Success: INFO level "Task created successfully" with task_id
- Failure: WARN/ERROR level with detailed error information

*(No other known side effects)*

## 8. Contract (Pre/Post-conditions & Invariants)

### 8.1. Pre-conditions
- Request body MUST be valid JSON
- `title` field MUST exist and be non-empty after trim
- If `area_id` provided, it MUST reference an existing area
- Database connection MUST be available

### 8.2. Post-conditions
- A new `Task` record exists in database with `created_at` = current time
- A new `Ordering` record exists linking the task to its context
- SSE event `task.created` has been queued for broadcast
- Response contains complete `TaskCardDto` with all fields populated
- Transaction has been committed successfully

### 8.3. Invariants
- Task `id` and `created_at` are immutable once created
- Every task MUST have at least one ordering record
- `sort_order` values are unique within a context
- SSE event payload MUST match HTTP response payload exactly
*/

// --- Dependencies ---
use axum::extract::State;
use crate::startup::AppState;
use crate::features::shared::TransactionHelper;

// --- Request/Response DTOs ---
#[derive(Deserialize)]
pub struct CreateTaskRequest { ... }

#[derive(Serialize)]
pub struct CreateTaskResponse { ... }

// --- HTTP Handler ---
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateTaskRequest>,
) -> Response {
    let _permit = app_state.acquire_write_permit().await;

    match logic::execute(&app_state, request).await {
        Ok(result) => created_response(result).into_response(),
        Err(err) => err.into_response(),
    }
}

// --- Validation Layer (optional) ---
mod validation {
    pub fn validate_request(req: &CreateTaskRequest) -> AppResult<ValidatedData> {
        // Complex validation logic
    }
}

// --- Business Logic Layer ---
mod logic {
    use super::*;

    pub async fn execute(app_state: &AppState, request: CreateTaskRequest) -> AppResult<Response> {
        // 1. Validation (optional)
        let validated = validation::validate_request(&request)?;

        // 2. Start transaction
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        // 3. Orchestrate database operations
        let task_id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        database::insert_task(&mut tx, task_id, &request, now).await?;

        // 4. Commit transaction
        TransactionHelper::commit(tx).await?;

        // 5. Assemble response (AFTER commit, to query full data)
        let task_card = assemble_full_task_card(app_state.db_pool(), task_id).await?;

        // 6. Emit SSE event (in separate transaction)
        let mut outbox_tx = TransactionHelper::begin(app_state.db_pool()).await?;
        let event = DomainEvent::new("task.created", "task", task_id, json!({ "task": task_card }));
        outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
        TransactionHelper::commit(outbox_tx).await?;

        Ok(Response { task: task_card })
    }
}

// --- Database Access Layer ---
mod database {
    use super::*;

    pub async fn insert_task(
        tx: &mut Transaction<'_, Sqlite>,
        task_id: Uuid,
        request: &CreateTaskRequest,
        created_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(r#"
            INSERT INTO tasks (id, title, created_at)
            VALUES (?, ?, ?)
        "#)
        .bind(task_id)
        .bind(&request.title)
        .bind(created_at)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::DatabaseError(e.into()))?;

        Ok(())
    }
}
```

### Shared Resources (Check Before Writing!)

**⚠️ ALWAYS check shared resources before implementing - avoid duplication!**

#### Cross-Feature Shared (`features/shared/`)
- **`TransactionHelper`**: `begin()`, `commit()` - Use for all transactions
- **`AreaRepository`**: `get_summary()`, `get_summaries_batch()` - Area lookups

#### Tasks Module Shared (`features/tasks/shared/`)
- **`TaskRepository`**: CRUD operations for tasks
  - `find_by_id_in_tx()`, `insert_in_tx()`, `update_in_tx()`, `soft_delete_in_tx()`
  - `set_completed_in_tx()`, `set_reopened_in_tx()`
- **`TaskScheduleRepository`**: Schedule management
  - `has_any_schedule()`, `create_in_tx()`, `delete_all_in_tx()`
- **`TaskTimeBlockLinkRepository`**: Task-TimeBlock relationships
  - `link_in_tx()`, `find_linked_time_blocks_in_tx()`
- **`TaskAssembler`**: DTO assembly
  - `task_to_card_basic()`, `task_to_card_full()`, `task_to_detail_basic()`
- **`LinkedTaskAssembler`**: Batch operations for linked tasks
- **`TimeBlockAssembler`**: Assemble TimeBlock views (⚠️ cross-feature dependency)

#### TimeBlocks Module Shared (`features/time_blocks/shared/`)
- **`TimeBlockRepository`**: CRUD operations for time blocks
  - `find_by_id_in_tx()`, `insert_in_tx()`, `update_in_tx()`, `soft_delete_in_tx()`
  - `truncate_to_in_tx()`, `find_in_range()`
- **`TimeBlockConflictChecker`**: `check_in_tx()` - Prevent overlaps

#### Core Utilities (`shared/core/utils/`)
- **`sort_order_utils.rs`**: LexoRank sorting
  - `generate_initial_sort_order()`, `get_rank_after()`, `get_rank_before()`, `get_mid_lexo_rank()`
- **`time_utils.rs`**: Time handling utilities

**⚠️ NEVER modify shared resources in feature development - use them or write endpoint-specific code!**

### Dependency Injection via AppState

**Use correct method names:**

```rust
// ✅ Correct
let task_id = app_state.id_generator().new_uuid();
let now = app_state.clock().now_utc();
let pool = app_state.db_pool();

// ❌ Wrong - these methods don't exist
let id = app_state.id_generator().generate();  // Compile error
let time = app_state.clock().now();            // Compile error
```

### Critical: SSE & HTTP Data Consistency

**⚠️ SSE events and HTTP responses MUST return identical data!**

**❌ Wrong - SSE gets incomplete data:**
```rust
// Business transaction
let mut tx = TransactionHelper::begin(pool).await?;
database::update_task(&mut tx, task_id).await?;

let mut task_card = TaskAssembler::task_to_card_basic(&task);
// task_card.schedules = None (default value!)

// ❌ SSE written BEFORE filling complete data
let event = DomainEvent::new("task.updated", "task", task_id, json!({ "task": task_card }));
outbox_repo.append_in_tx(&mut tx, &event).await?;
TransactionHelper::commit(tx).await?;

// Fill complete data AFTER SSE
task_card.schedules = get_schedules(pool, task_id).await?;

Ok(Response { task: task_card })  // HTTP gets complete data, SSE got incomplete!
```

**✅ Correct - Fill data BEFORE SSE:**
```rust
// 1. Business transaction
let mut tx = TransactionHelper::begin(pool).await?;
database::update_task(&mut tx, task_id).await?;
let mut task_card = TaskAssembler::task_to_card_basic(&task);
TransactionHelper::commit(tx).await?;

// 2. ⚠️ Fill ALL data BEFORE SSE
task_card.schedules = get_schedules(pool, task_id).await?;
task_card.area = get_area_summary(pool, area_id).await?;

// 3. Write SSE (separate transaction, complete data)
let mut outbox_tx = TransactionHelper::begin(pool).await?;
let event = DomainEvent::new("task.updated", "task", task_id, json!({ "task": task_card }));
outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
TransactionHelper::commit(outbox_tx).await?;

// 4. HTTP response (same complete data as SSE)
Ok(Response { task: task_card })
```

### Database Schema - READ BEFORE WRITING SQL!

**⚠️ NEVER guess table/column names - check schema first!**

**Schema location:** `src-tauri/migrations/20241001000000_initial_schema.sql`

**Common tables:**
- `tasks`, `areas`, `task_schedules`, `time_blocks`, `templates`, `orderings`, `projects`, `view_preferences`
- **All tables use plural names!**

**❌ Wrong:**
```rust
SELECT * FROM task_schedule WHERE ...  // ❌ No such table
```

**✅ Correct:**
```rust
SELECT * FROM task_schedules WHERE ... // ✅ Plural
```

### Error Handling

**Use `?` operator with auto-conversion:**

```rust
// ✅ Correct - automatic error conversion
let sort_order = get_rank_after(&max)?;  // SortOrderError → AppError
let task = sqlx::query_as(...).await?;   // sqlx::Error → AppError

// ❌ Wrong - manual construction of non-existent variants
AppError::LexoRankError(...)  // Compile error
```

### Database Access

- Use SQLx for type-safe SQL queries
- Repositories are in `features/<feature>/shared/repositories/`
- All queries use `&mut Transaction` for consistency
- Use `?` placeholders for parameterized queries

### Event System

Domain events (`shared/events/models.rs`) are emitted via SSE for real-time updates:
```rust
app_state.sse_state().broadcast(DomainEvent::TaskUpdated {
    task_id: task.id.clone()
});
```

Frontend subscribes to events in store `event-handlers.ts`.

## Frontend Development Patterns

### Creating a New Store

Follow the modularized pattern from `stores/task/`:
1. `core.ts`: Define state and getters
2. `crud-operations.ts`: Create/Update/Delete actions
3. `view-operations.ts`: Fetch/query actions
4. `event-handlers.ts`: SSE subscriptions
5. `index.ts`: Compose all modules

### API Calls

Use the centralized API client in `src/api/` (if exists) or fetch directly:
```typescript
const response = await fetch(`http://localhost:${port}/api/tasks`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(payload),
})
```

### SSE Subscriptions

Initialize in store's `initEventSubscriptions()`:
```typescript
const eventSource = new EventSource(`http://localhost:${port}/api/events/stream`)
eventSource.addEventListener('TaskUpdated', (event) => {
  const data = JSON.parse(event.data)
  this.addOrUpdateTask(data.task)
})
```

## Key Concepts

### Task States & Transitions

#### Task Status
- **Completed**: `completed_at` field has value
- **Incomplete**: `completed_at` field is NULL
- **Archived**: `archived_at` field has value

#### Valid Schedule (有效排期)
**Definition**: A task has a schedule record with `scheduled_date` >= today
- Past schedules don't count as "valid schedule" (only for historical records)
- This determines if a task appears in staging vs. date-based views

#### Schedule Outcome States
Each schedule has an `outcome` field tracking the task's progress on that day:

```
PLANNED              # Task scheduled but not worked on yet
    ↓ [click presence ★]
PRESENCE_LOGGED      # Task actively worked on
    ↓ [click complete ✓]
COMPLETED_ON_DAY     # Task completed on this day
    ↓ [click complete again]
PRESENCE_LOGGED      # Reopen (back to in-progress)
```

**Valid Transitions**:
- PLANNED → PRESENCE_LOGGED (click ★ presence button)
- PLANNED → COMPLETED_ON_DAY (click ✓ complete button)
- PRESENCE_LOGGED → PLANNED (click ★ again to cancel presence)
- PRESENCE_LOGGED → COMPLETED_ON_DAY (click ✓ complete button)
- COMPLETED_ON_DAY → PRESENCE_LOGGED (click ✓ again to reopen)

**Note**: CARRIED_OVER is a reserved field, not currently used.

### Kanban Filtering Rules

#### Staging (暂存区)
**Filter conditions**:
- Task is incomplete (`completed_at` IS NULL)
- Task is not archived (`archived_at` IS NULL)
- Task has NO valid schedule (no schedule with `scheduled_date` >= today)

**Mental model**: Inbox for unscheduled tasks

#### Daily View (当日看板)
**Filter conditions**:
- Task is not archived (`archived_at` IS NULL)
- Task has a schedule for today (`scheduled_date` = today)

**Mental model**: Today's work dashboard
**Key design**: Completed tasks still show in daily view (completion is today's outcome)

#### Past Date View (过去看板)
**Filter conditions**:
- Task is not archived (`archived_at` IS NULL)
- Task has a schedule for that past date (`scheduled_date` = specific past date)

**Mental model**: Historical work log

#### Future Date View (未来看板)
**Filter conditions**:
- Task is not archived (`archived_at` IS NULL)
- Task has a schedule for that future date (`scheduled_date` = specific future date)

**Mental model**: Future planning

### Core Business Events

#### Task Completion Event
**Trigger**: Click complete button on incomplete task

**Steps**:
1. Set `tasks.completed_at` = current timestamp
2. Complete all subtasks (`subtasks.is_completed` = true)
3. Set today's schedule: `outcome = 'COMPLETED_ON_DAY'`
4. Truncate ongoing time_block (set end_time to now)
5. Delete all future schedules and time_blocks
6. Move task to bottom of incomplete tasks section

#### Task Reopen Event
**Trigger**: Click complete button on completed task

**Steps**:
1. Set `tasks.completed_at` = NULL
2. Set today's schedule: `outcome = 'PRESENCE_LOGGED'` (back from COMPLETED_ON_DAY)

**Note**: Only modifies today's schedule, doesn't affect time_blocks

#### Create Schedule Event
**Trigger**: Drag task from staging to date view

**Steps**:
1. Create schedule with `scheduled_date` = target date
2. If dragged to past date: auto-complete task (`completed_at` = that date, `outcome` = 'COMPLETED_ON_DAY')

#### Return to Staging Event
**Trigger**: Drag task from date view to staging

**Steps**:
1. Delete all schedules and time_blocks for today and future dates
2. Preserve past schedules and time_blocks (including outcome values)
3. If task is completed: auto-reopen (`completed_at` = NULL)

### View Metadata System

Used by drag-and-drop to identify source/target views:
```typescript
const viewMetadata: ViewMetadata = {
  type: 'date',           // 'status' | 'date' | 'project' | 'calendar'
  id: 'daily-2025-10-03',
  config: { date: '2025-10-03' },
  label: '2025年10月3日',
}
```

### SQLite Write Serialization

The app uses application-level write serialization to avoid SQLite lock contention:
- All write operations acquire `AppState::acquire_write_permit()` before starting a transaction
- The permit is automatically released when dropped (RAII pattern)
- Ensures only one write transaction executes at a time

## Data Schema & Coupling

**⚠️ Critical: Modifying data structures has cascading effects across the entire stack!**

### Data Structure Layers

```
Database Schema (SQLite migrations)
    ↓
Backend Entity (Rust entities/*/model.rs)
    ↓
Backend DTO (Rust entities/*/response_dtos.rs)
    ↓
Assembler (features/*/shared/assembler.rs)
    ↓
Frontend DTO (TypeScript src/types/dtos.ts)
    ↓
Pinia Store (src/stores/*.ts)
    ↓
Vue Components (src/components/*.vue, src/views/*.vue)
```

### Modification Checklist: Adding a Field

**Example: Adding `priority` field to Task**

#### Backend Changes
- [ ] **Schema**: `src-tauri/migrations/*.sql` - Add column
- [ ] **Entity**: `entities/task/model.rs`
  - [ ] Task struct - add field
  - [ ] TaskRow struct - add field
  - [ ] TryFrom implementation - map field
- [ ] **DTOs**: `entities/task/response_dtos.rs`
  - [ ] TaskCardDto - add field
  - [ ] TaskDetailDto - add field (if needed)
- [ ] **Request DTOs**: `entities/task/request_dtos.rs`
  - [ ] CreateTaskRequest - add field
  - [ ] UpdateTaskRequest - add field
- [ ] **Assembler**: `features/tasks/shared/assembler.rs`
  - [ ] Update `task_to_card_basic()` to include field
- [ ] **Repositories**: All SQL queries
  - [ ] `create_task.rs` - INSERT statement
  - [ ] `update_task.rs` - UPDATE statement
  - [ ] `get_task.rs` - SELECT statement
  - [ ] All view endpoints - SELECT statements
- [ ] **⚠️ Cross-Feature Check**: Search for all places that assemble this DTO
  ```bash
  grep -rn "TaskCardDto {" src-tauri/src/features
  grep -rn "SELECT.*FROM tasks" src-tauri/src/features
  ```

#### Frontend Changes
- [ ] **DTOs**: `src/types/dtos.ts` - Add field to TaskCard interface
- [ ] **Store**: `src/stores/task/*.ts`
  - [ ] Payload types (CreateTaskPayload, UpdateTaskPayload)
- [ ] **Components**: All components that display/edit tasks
  - [ ] Task card display
  - [ ] Task editor modal
  - [ ] Any task list views

### Cross-Feature Dependencies

**⚠️ Some entities are used by multiple features - update ALL locations!**

**Example: TimeBlock entity**
- Primary: `features/time_blocks/`
- **Also used by**: `features/tasks/shared/assemblers/time_block_assembler.rs`
- **Also used by**: `features/tasks/shared/repositories/task_time_block_link_repository.rs`

**When modifying TimeBlock, update:**
1. `features/time_blocks/` (all endpoints and shared code)
2. `features/tasks/shared/assemblers/time_block_assembler.rs` (SQL + DTO assembly)
3. `features/tasks/shared/repositories/task_time_block_link_repository.rs` (SQL queries)

**Find all cross-feature usages:**
```bash
grep -rn "TimeBlockViewDto {" src-tauri/src/features
grep -rn "SELECT.*FROM time_blocks" src-tauri/src/features/tasks
```

### Common Mistakes

1. **❌ Schema changed but entity not updated** → SQL column count mismatch
2. **❌ DTO changed but assembler not updated** → Compile error "missing field"
3. **❌ Backend DTO changed but frontend not updated** → TypeScript errors, undefined values
4. **❌ Cross-feature assemblers forgotten** → Runtime errors in unrelated features

## View Context Key System

Views use a context key system to persist user preferences (sorting, filters, etc.):

### Context Key Format

```
{type}::{identifier}
```

### Context Key Types

| Type | Format | Example |
|------|--------|---------|
| Misc Views | `misc::{id}` | `misc::staging`, `misc::all`, `misc::planned` |
| Daily Kanban | `daily::{YYYY-MM-DD}` | `daily::2025-10-01` |
| Area Filter | `area::{uuid}` | `area::a1b2c3d4-...` |
| Project View | `project::{uuid}` | `project::proj-uuid-...` |

### Database Schema

```sql
CREATE TABLE view_preferences (
    context_key TEXT PRIMARY KEY NOT NULL,  -- e.g., 'misc::staging', 'daily::2025-10-01'
    sorted_task_ids TEXT NOT NULL,          -- JSON array: '["uuid1", "uuid2"]'
    updated_at TEXT NOT NULL                -- UTC timestamp
);
```

### API Endpoints

- **GET** `/view-preferences/:context_key` - Get view preferences
- **PUT** `/view-preferences` - Save view preferences

### Frontend Usage

```typescript
// Generate context key
function getContextKey(context: ViewContext): string {
  switch (context.type) {
    case 'misc':
      return `misc::${context.id}`
    case 'daily':
      return `daily::${context.date}`  // YYYY-MM-DD format
    case 'area':
      return `area::${context.areaId}`
    case 'project':
      return `project::${context.projectId}`
  }
}

// Example keys
'misc::staging'
'daily::2025-10-01'
'area::a1b2c3d4-1234-5678-90ab-cdef12345678'
'project::proj-uuid-1234'
```

## Complete Development Workflow

### Workflow 1: Adding a New Endpoint to Existing Feature

**Example: Add archive task endpoint**

#### Step 1: Check Shared Resources
```bash
# Check if similar functionality exists
grep -rn "archive" src-tauri/src/features/tasks/shared/repositories
```

#### Step 2: Create Endpoint File
**File:** `src-tauri/src/features/tasks/endpoints/archive_task.rs`

```rust
// Follow SFC pattern (see "Backend Development Patterns" above)
```

#### Step 3: Register Route
**File:** `src-tauri/src/features/tasks/mod.rs`

```rust
pub mod endpoints {
    pub mod archive_task;
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/:id/archive", post(endpoints::archive_task::handle))
}
```

#### Step 4: Frontend Integration

**Update Store:** `src/stores/task/crud-operations.ts`
```typescript
async archiveTask(taskId: string) {
  const response = await fetch(`${baseUrl}/tasks/${taskId}/archive`, {
    method: 'POST',
  })
  // Handle response
}
```

**Subscribe to SSE:** `src/stores/task/event-handlers.ts`
```typescript
eventSource.addEventListener('task.archived', (event) => {
  const { task_id } = JSON.parse(event.data)
  this.removeTask(task_id)
})
```

#### Step 5: Test
- [ ] Test API endpoint with curl/Postman
- [ ] Test SSE event emission
- [ ] Test frontend UI
- [ ] Check data consistency

---

### Workflow 2: Adding a New Field to Existing Entity

**Example: Add `priority` field to Task**

#### Step 1: Update Schema
**File:** `src-tauri/migrations/20241001000000_initial_schema.sql`

```sql
ALTER TABLE tasks ADD COLUMN priority TEXT DEFAULT 'medium';
```

**Delete old database:**
```bash
rm src-tauri/*.db*
# Restart app to run migrations
```

#### Step 2: Update Backend Entity
**File:** `src-tauri/src/entities/task/model.rs`

```rust
pub struct Task {
    // ... existing fields
    pub priority: String,  // ← Add
}

pub struct TaskRow {
    // ... existing fields
    pub priority: String,  // ← Add
}

impl TryFrom<TaskRow> for Task {
    fn try_from(row: TaskRow) -> Result<Self, Self::Error> {
        Ok(Task {
            // ... existing fields
            priority: row.priority,  // ← Add
        })
    }
}
```

#### Step 3: Update DTOs
**File:** `src-tauri/src/entities/task/response_dtos.rs`

```rust
pub struct TaskCardDto {
    // ... existing fields
    pub priority: String,  // ← Add
}
```

**File:** `src-tauri/src/entities/task/request_dtos.rs`

```rust
pub struct CreateTaskRequest {
    // ... existing fields
    pub priority: Option<String>,  // ← Add
}

pub struct UpdateTaskRequest {
    // ... existing fields
    pub priority: Option<String>,  // ← Add
}
```

#### Step 4: Update Assembler
**File:** `src-tauri/src/features/tasks/shared/assembler.rs`

```rust
pub fn task_to_card_basic(task: &Task) -> TaskCardDto {
    TaskCardDto {
        // ... existing fields
        priority: task.priority.clone(),  // ← Add
    }
}
```

#### Step 5: Update ALL SQL Queries
**Find all queries:**
```bash
grep -rn "SELECT.*FROM tasks" src-tauri/src/features
grep -rn "INSERT INTO tasks" src-tauri/src/features
```

**Update each query:**
```rust
// SELECT
SELECT id, title, ..., priority FROM tasks

// INSERT
INSERT INTO tasks (id, title, ..., priority) VALUES (?, ?, ..., ?)
sqlx::query(...).bind(&task.priority)

// UPDATE (if editable)
UPDATE tasks SET priority = ? WHERE id = ?
```

#### Step 6: Check Cross-Feature Dependencies
```bash
grep -rn "TaskCardDto {" src-tauri/src/features
```

Update any cross-feature assemblers found.

#### Step 7: Update Frontend
**File:** `src/types/dtos.ts`

```typescript
export interface TaskCard {
  // ... existing fields
  priority: string
}
```

**File:** `src/stores/task/crud-operations.ts`

```typescript
export interface CreateTaskPayload {
  // ... existing fields
  priority?: string
}
```

**Update UI components** to display/edit priority

#### Step 8: Test End-to-End
- [ ] Create task with priority → Check database
- [ ] Update priority → Check SSE event
- [ ] View task → Check frontend display
- [ ] Check all views (staging, daily, etc.)

---

### Workflow 3: Creating a New Feature

**Example: Add Tags feature**

#### Step 1: Database Schema
**File:** `src-tauri/migrations/20241001000000_initial_schema.sql`

```sql
CREATE TABLE tags (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    color TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE task_tags (
    task_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (task_id, tag_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);
```

#### Step 2: Create Entity Files
```
src-tauri/src/entities/tag/
├── model.rs          # Tag, TagRow, TryFrom
├── request_dtos.rs   # CreateTagRequest, UpdateTagRequest
├── response_dtos.rs  # TagDto
└── mod.rs            # Re-exports
```

#### Step 3: Create Feature Module
```
src-tauri/src/features/tags/
├── endpoints/
│   ├── create_tag.rs
│   ├── get_tag.rs
│   ├── update_tag.rs
│   ├── delete_tag.rs
│   └── list_tags.rs
├── shared/
│   ├── repositories/
│   │   └── tag_repository.rs
│   └── assembler.rs
├── mod.rs            # Route registration
└── API_SPEC.md       # API documentation
```

#### Step 4: Implement Endpoints
Each endpoint follows SFC pattern (see "Backend Development Patterns")

#### Step 5: Register Routes
**File:** `src-tauri/src/features/tags/mod.rs`

```rust
use axum::{routing::{get, post, patch, delete}, Router};
use crate::startup::AppState;

pub mod endpoints;
pub mod shared;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(endpoints::create_tag::handle).get(endpoints::list_tags::handle))
        .route("/:id", get(endpoints::get_tag::handle)
                       .patch(endpoints::update_tag::handle)
                       .delete(endpoints::delete_tag::handle))
}
```

**File:** `src-tauri/src/features/mod.rs`

```rust
pub mod tags;

pub fn create_feature_routes() -> Router<AppState> {
    Router::new()
        .nest("/tags", tags::create_routes())
        // ... other features
}
```

#### Step 6: Frontend Implementation

**Create Store:** `src/stores/tag.ts`
```typescript
export const useTagStore = defineStore('tag', () => {
  // State
  const tags = ref<Map<string, Tag>>(new Map())

  // Getters
  const allTags = computed(() => Array.from(tags.value.values()))

  // Actions
  async function createTag(payload: CreateTagPayload) { ... }
  async function fetchTags() { ... }

  // SSE subscriptions
  function initEventSubscriptions() { ... }

  return { tags, allTags, createTag, fetchTags, initEventSubscriptions }
})
```

**Create Components:**
- `src/components/parts/TagBadge.vue`
- `src/components/parts/TagSelector.vue`
- `src/views/TagManagementView.vue`

#### Step 7: Integration Testing
- [ ] API endpoints work correctly
- [ ] SSE events emit and received
- [ ] Frontend updates in real-time
- [ ] Data persists in database

---

### Pre-Development Checklist

**Before writing any code:**

- [ ] Read `src-tauri/migrations/*.sql` to understand schema
- [ ] Check `references/SFC_SPEC.md` for shared resources
- [ ] Search for similar functionality in existing features
- [ ] Plan cross-feature dependencies (if any)

**During development:**

- [ ] Use TransactionHelper for all transactions
- [ ] Use AppState methods correctly (`.new_uuid()`, `.now_utc()`)
- [ ] Fill complete data BEFORE emitting SSE
- [ ] Use shared resources instead of duplicating code

**Before committing:**

- [ ] Run `cargo check` - no compilation errors
- [ ] Run `cargo clippy` - no linting warnings
- [ ] Test API with curl/Postman
- [ ] Test SSE events in browser console
- [ ] Test frontend UI manually
- [ ] Check data in database directly

---

### Quick Reference Commands

```bash
# Find all usages of a DTO
grep -rn "TaskCardDto {" src-tauri/src/features

# Find all SQL queries for a table
grep -rn "SELECT.*FROM tasks" src-tauri/src

# Find all SSE event emissions
grep -rn "DomainEvent::new" src-tauri/src

# Check if shared resource exists
ls src-tauri/src/features/tasks/shared/repositories
ls src-tauri/src/shared/core/utils

# Delete database and restart (to run migrations)
rm src-tauri/*.db*
# Then restart dev server
```

## Important Notes

- **⚠️ Dev Servers**: NEVER start dev servers (`pnpm dev`, `pnpm tauri dev`, `cargo run`) - user has HMR dev servers running
- **Package Manager**: Use **pnpm** exclusively (not npm or yarn)
- **Port Discovery**: Sidecar server uses dynamic port selection; frontend listens for `sidecar-port-discovered` event
- **Migrations**: SQLite migrations are in `src-tauri/migrations/` and run automatically on startup
- **Type Generation**: Rust structs with `#[derive(TS)]` generate TypeScript types (see `ts-rs` crate)
- **Development Folder**: `develop/` contains experimental features (vue-draxis) - DO NOT use in production
- **Schema First**: ALWAYS read schema before writing SQL - never guess table/column names
- **Shared Resources**: Check shared repositories/assemblers before implementing - avoid duplication
- **Data Consistency**: SSE and HTTP must return identical data - fill complete data BEFORE emitting events
- **Cross-Feature Dependencies**: When modifying shared entities (Task, TimeBlock, Area), check ALL features that use them
- **CABC V2.1 Documentation**: EVERY endpoint and public method MUST include a complete CABC documentation comment with all 8 sections:
  1. Endpoint/Function Signature
  2. High-Level Behavior (User Story + Core Logic)
  3. Input/Output Specification (Request + Responses)
  4. Validation Rules
  5. Business Logic Walkthrough
  6. Edge Cases
  7. Expected Side Effects
  8. Contract (Pre-conditions, Post-conditions, Invariants)

## CABC V2.1 Specification Summary

**CABC (Cutie API Behavior Contract)** is a contract-based documentation standard for all API endpoints and public methods.

### Required Sections

1. **Endpoint/Function Signature**: Precise function signature for machine verification
2. **High-Level Behavior**: Business-level description (1-2 sentences) + user story
3. **Input/Output Specification**: Request/response schemas with all status codes
4. **Validation Rules**: Exhaustive list of all input constraints (type, format, range, null checks)
5. **Business Logic Walkthrough**: Step-by-step implementation flow
6. **Edge Cases**: Behavior for non-happy-path scenarios (idempotency, invalid input, state conflicts)
7. **Expected Side Effects**: ALL observable effects beyond return value:
   - Database operations (SELECT/INSERT/UPDATE/DELETE counts)
   - Transaction boundaries
   - SSE events
   - External API calls
   - Cache invalidations
   - Logging
8. **Contract (Pre/Post-conditions & Invariants)**:
   - **Pre-conditions**: What MUST be true before calling (caller's responsibility)
   - **Post-conditions**: What MUST be true after successful execution (method's guarantee)
   - **Invariants**: System rules that MUST NEVER be violated

### Key Principles

- **Completeness**: A CABC is incomplete if it fails to define behavior for any foreseeable input/state combination
- **Verifiability**: Post-conditions translate directly to test assertions
- **Contract-First**: Pre-conditions define caller responsibilities; post-conditions define method guarantees
- **Explicitness**: No implicit behavior - document everything observable

### Example Usage

```rust
// See SFC Architecture section for complete example
/*
CABC for `create_task`

## 1. Endpoint Signature
POST /api/tasks

## 2. High-Level Behavior
### 2.1. User Story
> As a user, I want to quickly create a new task...

### 2.2. Core Business Logic
Creates a Task entity and Ordering record...

[... continue with all 8 sections ...]
*/
```
