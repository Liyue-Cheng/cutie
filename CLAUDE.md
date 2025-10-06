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

### Creating a New Endpoint

1. Create endpoint file in `features/<feature>/endpoints/<action>.rs`:
```rust
use axum::extract::State;
use axum::Json;
use crate::startup::AppState;

pub async fn handle(
    State(app_state): State<AppState>,
    Json(payload): Json<YourRequest>,
) -> Result<Json<YourResponse>, AppError> {
    // 1. Acquire write permit (if writing)
    let _permit = app_state.acquire_write_permit().await;

    // 2. Start transaction
    let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

    // 3. Business logic
    let result = repository::do_something(&mut tx, payload).await?;

    // 4. Commit transaction
    TransactionHelper::commit(tx).await?;

    // 5. Emit domain event (if state changed)
    app_state.sse_state().broadcast(DomainEvent::TaskUpdated { task_id });

    Ok(Json(result))
}
```

2. Declare endpoint in feature's `mod.rs`:
```rust
pub mod endpoints {
    pub mod your_endpoint;
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/your-path", post(endpoints::your_endpoint::handle))
}
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

## Important Notes

- **⚠️ Dev Servers**: NEVER start dev servers (`pnpm dev`, `pnpm tauri dev`, `cargo run`) - user has HMR dev servers running
- **Package Manager**: Use **pnpm** exclusively (not npm or yarn)
- **Port Discovery**: Sidecar server uses dynamic port selection; frontend listens for `sidecar-port-discovered` event
- **Migrations**: SQLite migrations are in `src-tauri/migrations/` and run automatically on startup
- **Type Generation**: Rust structs with `#[derive(TS)]` generate TypeScript types (see `ts-rs` crate)
- **Development Folder**: `develop/` contains experimental features (vue-draxis) - DO NOT use in production
