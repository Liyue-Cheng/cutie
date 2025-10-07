# CLAUDE.md

This file provides guidance to Claude Code when working with this codebase.

## Table of Contents

- [Project Overview](#project-overview)
- [Development Commands](#development-commands)
- [Architecture](#architecture)
- [CABC Documentation Standard](#cabc-documentation-standard)
- [Backend Development](#backend-development)
- [Frontend Development](#frontend-development)
- [Key Concepts](#key-concepts)
- [Data Schema & Coupling](#data-schema--coupling)
- [Development Workflows](#development-workflows)
- [Quick Reference](#quick-reference)

---

## Project Overview

Cutie is a task management desktop application built with:
- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri (sidecar architecture with Axum HTTP server)
- **Database**: SQLite (with SQLx migrations in `src-tauri/migrations/`)
- **State Management**: Pinia stores
- **Real-time**: SSE (Server-Sent Events) for state synchronization

---

## Development Commands

**⚠️ CRITICAL: DO NOT START DEV SERVERS**
- **NEVER run `pnpm dev`, `pnpm tauri dev`, or `cargo run`** - HMR dev servers are already running
- Only run build, test, and lint commands
- Package manager: **pnpm** (not npm or yarn)

### Frontend Commands
```bash
pnpm build           # Build frontend (vue-tsc -b && vite build)
pnpm preview         # Preview production build
pnpm exec eslint src/
pnpm exec prettier --check .
pnpm exec stylelint "**/*.vue"
```

### Backend Commands
```bash
cargo test           # Run tests
cargo check          # Check compilation
cargo clippy         # Linting
cargo fmt --check    # Format checking
```

---

## Architecture

### Backend: Feature-Sliced Architecture

Vertical slice architecture organized by business features:

```
src-tauri/src/
├── main.rs, lib.rs          # Entry point & library exports
├── config/                  # Configuration
├── startup/                 # AppState, database, sidecar initialization
├── entities/                # Domain models & DTOs
│   ├── task/                # Task entity (model, DTOs, enums)
│   ├── schedule/            # Schedule entity
│   ├── time_block/          # Time block entity
│   ├── area/                # Area entity
│   └── view_preference/     # View preferences
├── features/                # Feature modules (vertical slices)
│   ├── tasks/               # Task management
│   │   ├── endpoints/       # HTTP handlers (SFC pattern)
│   │   └── shared/          # Repositories, assemblers
│   ├── views/               # View queries (staging, daily, etc.)
│   ├── time_blocks/         # Time block management
│   ├── areas/               # Area management
│   └── shared/              # Cross-feature infrastructure
│       ├── repositories/    # Shared repositories
│       └── transaction.rs   # TransactionHelper
└── shared/                  # Cross-cutting concerns
    ├── core/                # Error handling, utilities
    ├── database/            # Connection, pagination
    ├── events/              # SSE, domain events
    ├── http/                # Responses, middleware, extractors
    └── ports/               # Abstractions (Clock, IdGenerator)
```

**Key Principles:**
- **Single File Components (SFC)**: Each endpoint is a standalone file (no mod.rs)
- **Write Serialization**: Use `AppState::acquire_write_permit()` for all write operations
- **Event-Driven**: Emit domain events via SSE for real-time updates
- **Transaction Helper**: Use `TransactionHelper` for all database transactions

### Frontend: Modular Vue Architecture

```
src/
├── main.ts, App.vue         # Entry point & root
├── router/                  # Vue Router
├── stores/                  # Pinia stores (modularized)
│   └── task/
│       ├── index.ts         # Store composition
│       ├── core.ts          # State & getters
│       ├── crud-operations.ts
│       ├── view-operations.ts
│       └── event-handlers.ts  # SSE subscriptions
├── views/                   # Page components
├── components/
│   ├── parts/               # UI building blocks
│   ├── templates/           # Layout templates
│   └── functional/          # Logic components
├── composables/
│   └── drag/                # Drag-and-drop system
└── types/                   # TypeScript definitions
```

**Store Pattern:**
- State: Normalized data (single source of truth)
- Getters: Derived data (filtering, grouping)
- Actions: API calls, mutations, SSE subscriptions

### Drag-and-Drop System

Custom cross-view drag system in `src/composables/drag/` (see `README.md` in that folder)

**Key composables:**
- `useCrossViewDrag` - Main drag orchestration
- `useDragTransfer` - Data transfer utilities
- `useAutoScroll` - Auto-scroll during drag

---

## CABC Documentation Standard

**CABC (Cutie API Behavior Contract)** - Required for ALL endpoints and public methods.

### 8 Required Sections

1. **Endpoint/Function Signature** - Precise signature for verification
2. **High-Level Behavior** - User story + core business logic
3. **Input/Output Specification** - Request/response schemas + status codes
4. **Validation Rules** - All input constraints (type, format, range, nulls)
5. **Business Logic Walkthrough** - Step-by-step implementation flow
6. **Edge Cases** - Non-happy-path scenarios
7. **Expected Side Effects** - ALL observable effects:
   - Database ops (SELECT/INSERT/UPDATE/DELETE counts)
   - Transaction boundaries
   - SSE events
   - External calls
   - Logging
8. **Contract** - Pre-conditions, post-conditions, invariants

### Template

```rust
/*
CABC for `endpoint_name`

## 1. Endpoint Signature
POST /api/resource

## 2. High-Level Behavior
### 2.1. User Story
> As a user, I want to...

### 2.2. Core Business Logic
[Brief explanation]

## 3. Input/Output Specification
[Request/Response details]

## 4. Validation Rules
- field: constraints

## 5. Business Logic Walkthrough
1. Step 1
2. Step 2
...

## 6. Edge Cases
- Case: Behavior

## 7. Expected Side Effects
### Database Operations:
- SELECT: Nx on table
- INSERT: Nx into table

### SSE Events:
- event.name

## 8. Contract
### Pre-conditions:
- Condition

### Post-conditions:
- Guarantee

### Invariants:
- Rule that MUST NEVER be violated
*/
```

---

## Backend Development

### Single File Component (SFC) Pattern

Each endpoint is a self-contained file with all layers:

```rust
// --- CABC V2.1 Documentation (see template above) ---

// --- Dependencies ---
use axum::extract::State;
use crate::startup::AppState;
use crate::features::shared::TransactionHelper;

// --- Request/Response DTOs ---
#[derive(Deserialize)]
pub struct CreateRequest { ... }

// --- HTTP Handler ---
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateRequest>,
) -> Response {
    let _permit = app_state.acquire_write_permit().await;

    match logic::execute(&app_state, request).await {
        Ok(result) => ok_response(result).into_response(),
        Err(err) => err.into_response(),
    }
}

// --- Validation (optional) ---
mod validation { ... }

// --- Business Logic ---
mod logic {
    pub async fn execute(app_state: &AppState, request: Request) -> AppResult<Response> {
        // 1. Validation
        // 2. Start transaction
        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;
        // 3. Business operations
        // 4. Commit
        TransactionHelper::commit(tx).await?;
        // 5. Assemble full response (query after commit)
        // 6. Emit SSE (separate transaction, with complete data)
        // 7. Return response
    }
}

// --- Database Access ---
mod database {
    pub async fn insert(tx: &mut Transaction<'_, Sqlite>, ...) -> AppResult<()> { ... }
}
```

### Shared Resources Reference

**⚠️ ALWAYS check these before implementing - avoid duplication!**

#### Cross-Feature (`features/shared/`)
- `TransactionHelper`: `begin()`, `commit()`
- `AreaRepository`: `get_summary()`, `get_summaries_batch()`

#### Tasks (`features/tasks/shared/`)
- `TaskRepository`: `find_by_id_in_tx()`, `insert_in_tx()`, `update_in_tx()`, `soft_delete_in_tx()`, `set_completed_in_tx()`, `set_reopened_in_tx()`
- `TaskScheduleRepository`: `has_any_schedule()`, `create_in_tx()`, `delete_all_in_tx()`
- `TaskTimeBlockLinkRepository`: `link_in_tx()`, `find_linked_time_blocks_in_tx()`
- `TaskAssembler`: `task_to_card_basic()`, `task_to_card_full()`, `task_to_detail_basic()`
- `LinkedTaskAssembler`: Batch operations
- `TimeBlockAssembler`: Cross-feature dependency

#### TimeBlocks (`features/time_blocks/shared/`)
- `TimeBlockRepository`: `find_by_id_in_tx()`, `insert_in_tx()`, `update_in_tx()`, `soft_delete_in_tx()`, `truncate_to_in_tx()`, `find_in_range()`
- `TimeBlockConflictChecker`: `check_in_tx()`

#### Core Utilities (`shared/core/utils/`)
- `sort_order_utils.rs`: LexoRank sorting - `generate_initial_sort_order()`, `get_rank_after()`, `get_rank_before()`, `get_mid_lexo_rank()`
- `time_utils.rs`: Time handling utilities

### AppState Methods

```rust
// ✅ Correct
let id = app_state.id_generator().new_uuid();
let now = app_state.clock().now_utc();
let pool = app_state.db_pool();

// ❌ Wrong
let id = app_state.id_generator().generate();  // Doesn't exist
let now = app_state.clock().now();             // Doesn't exist
```

### Critical: SSE & HTTP Data Consistency

**⚠️ SSE events and HTTP responses MUST return identical data!**

```rust
// ✅ Correct Pattern
let mut tx = TransactionHelper::begin(pool).await?;
database::update(&mut tx, ...).await?;
TransactionHelper::commit(tx).await?;

// ⚠️ Fill ALL data BEFORE SSE
let mut dto = assemble_basic(&entity);
dto.schedules = fetch_schedules(pool, id).await?;
dto.area = fetch_area(pool, area_id).await?;

// Emit SSE (separate transaction, complete data)
let mut outbox_tx = TransactionHelper::begin(pool).await?;
let event = DomainEvent::new("resource.updated", "resource", id, json!({ "data": dto }));
outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
TransactionHelper::commit(outbox_tx).await?;

// Return (same data as SSE)
Ok(Response { data: dto })
```

### Database Schema

**⚠️ NEVER guess table/column names - always read schema first!**

**Schema location:** `src-tauri/migrations/20241001000000_initial_schema.sql`

**Tables:** `tasks`, `areas`, `task_schedules`, `time_blocks`, `orderings`, `projects`, `view_preferences` (all plural)

### Error Handling

```rust
// ✅ Correct - use ? operator with auto-conversion
let rank = get_rank_after(&max)?;        // SortOrderError → AppError
let task = sqlx::query_as(...).await?;   // sqlx::Error → AppError
```

---

## Frontend Development

### Store Pattern

Follow modularized pattern from `stores/task/`:
1. `core.ts` - State & getters
2. `crud-operations.ts` - Create/Update/Delete
3. `view-operations.ts` - Fetch/query
4. `event-handlers.ts` - SSE subscriptions
5. `index.ts` - Compose modules

### API Calls

```typescript
const response = await fetch(`http://localhost:${port}/api/tasks`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(payload),
})
```

### SSE Subscriptions

```typescript
const eventSource = new EventSource(`http://localhost:${port}/api/events/stream`)
eventSource.addEventListener('TaskUpdated', (event) => {
  const data = JSON.parse(event.data)
  this.addOrUpdateTask(data.task)
})
```

---

## Key Concepts

### Task States

**Status:**
- Completed: `completed_at` has value
- Incomplete: `completed_at` IS NULL
- Archived: `archived_at` has value

**Valid Schedule:** Task has schedule with `scheduled_date` >= today (past schedules don't count)

**Schedule Outcomes:**
```
PLANNED → PRESENCE_LOGGED → COMPLETED_ON_DAY
   ↓           ↓                    ↓
[click ★]  [click ✓]          [click ✓ again to reopen]
```

### View Filtering Rules

| View | Conditions |
|------|------------|
| **Staging** | incomplete + not archived + NO valid schedule |
| **Daily** | not archived + has schedule for today |
| **Past Date** | not archived + has schedule for that past date |
| **Future Date** | not archived + has schedule for that future date |

### Core Business Events

**Task Completion:**
1. Set `completed_at` = now
2. Complete all subtasks
3. Set today's schedule outcome = `COMPLETED_ON_DAY`
4. Truncate ongoing time_block
5. Delete all future schedules & time_blocks

**Task Reopen:**
1. Set `completed_at` = NULL
2. Set today's schedule outcome = `PRESENCE_LOGGED`

**Create Schedule (drag from staging):**
1. Create schedule with target date
2. If past date: auto-complete task

**Return to Staging (drag from date view):**
1. Delete schedules & time_blocks >= today
2. Preserve past data
3. Auto-reopen if completed

### View Context Key System

Format: `{type}::{identifier}`

| Type | Example |
|------|---------|
| Misc | `misc::staging`, `misc::all` |
| Daily | `daily::2025-10-01` |
| Area | `area::{uuid}` |
| Project | `project::{uuid}` |

### SQLite Write Serialization

- All write ops acquire `AppState::acquire_write_permit()` before transaction
- RAII pattern (auto-released on drop)
- Ensures only one write transaction at a time

---

## Data Schema & Coupling

**⚠️ Modifying data structures has cascading effects!**

### Data Flow

```
Database Schema → Backend Entity → Backend DTO → Assembler
    ↓
Frontend DTO → Pinia Store → Vue Components
```

### Adding a Field Checklist

**Backend:**
- [ ] Schema: Add column in migration
- [ ] Entity: Update model.rs (struct + TryFrom)
- [ ] DTOs: Update response_dtos.rs & request_dtos.rs
- [ ] Assembler: Update assembly logic
- [ ] Repositories: Update ALL SQL queries
- [ ] Cross-feature check: `grep -rn "DtoName {" src-tauri/src/features`

**Frontend:**
- [ ] DTOs: Update src/types/dtos.ts
- [ ] Store: Update payload types
- [ ] Components: Update UI

### Cross-Feature Dependencies

**Example:** TimeBlock is used by:
1. `features/time_blocks/` (primary)
2. `features/tasks/shared/assemblers/time_block_assembler.rs`
3. `features/tasks/shared/repositories/task_time_block_link_repository.rs`

**Find usages:**
```bash
grep -rn "TimeBlockViewDto {" src-tauri/src/features
grep -rn "SELECT.*FROM time_blocks" src-tauri/src/features/tasks
```

---

## Development Workflows

### Adding a New Endpoint

1. **Check shared resources** - Avoid duplication
2. **Create endpoint file** - Follow SFC pattern with CABC docs
3. **Register route** - Update feature's mod.rs
4. **Frontend integration** - Update store & SSE handlers
5. **Test** - API, SSE, UI, data consistency

### Adding a Field to Entity

1. **Update schema** - Add column in migration, delete old DB
2. **Update entity** - model.rs (struct, TryFrom)
3. **Update DTOs** - response & request
4. **Update assembler** - Include new field
5. **Update ALL SQL** - SELECT, INSERT, UPDATE queries
6. **Check cross-features** - Search for DTO usages
7. **Update frontend** - DTOs, store, components
8. **Test end-to-end** - Create, update, view across all views

### Creating a New Feature

1. **Schema** - Create tables
2. **Entities** - model.rs, request/response DTOs
3. **Feature module** - endpoints/, shared/, mod.rs
4. **Implement endpoints** - Follow SFC + CABC
5. **Register routes** - features/mod.rs
6. **Frontend** - Store, components
7. **Test** - API, SSE, UI, persistence

---

## Quick Reference

### Pre-Development Checklist

- [ ] Read schema: `src-tauri/migrations/*.sql`
- [ ] Check shared resources (see "Shared Resources Reference")
- [ ] Search for similar functionality
- [ ] Use TransactionHelper for all transactions
- [ ] Use correct AppState methods (`.new_uuid()`, `.now_utc()`)
- [ ] Fill complete data BEFORE SSE
- [ ] Never modify shared resources in features

### Testing Checklist

- [ ] `cargo check` - no errors
- [ ] `cargo clippy` - no warnings
- [ ] Test API (curl/Postman)
- [ ] Test SSE in browser console
- [ ] Test UI manually
- [ ] Check database directly

### Useful Commands

```bash
# Find DTO usages
grep -rn "TaskCardDto {" src-tauri/src/features

# Find SQL queries
grep -rn "SELECT.*FROM tasks" src-tauri/src

# Find SSE emissions
grep -rn "DomainEvent::new" src-tauri/src

# Reset database (delete and restart app to run migrations)
rm src-tauri/*.db*
```

### Important Notes

- **Port Discovery**: Sidecar uses dynamic port; frontend listens for `sidecar-port-discovered` event
- **Type Generation**: Rust structs with `#[derive(TS)]` generate TypeScript types
- **Development Folder**: `develop/` is experimental - DO NOT use in production
