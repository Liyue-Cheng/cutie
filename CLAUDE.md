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

**Cutie**: Task management desktop app with **CPU-inspired frontend architecture**

**Tech Stack:**
- **Frontend**: Vue 3 + TypeScript + Vite + CPU Pipeline System
- **Backend**: Rust + Tauri (Axum HTTP server)
- **Database**: SQLite (migrations in `src-tauri/migrations/`)
- **State Management**: Pinia stores with RTL hardware principles
- **Real-time**: SSE for state synchronization
- **Execution Model**: 5-stage CPU pipeline (IF-SCH-EX-RES-WB) with out-of-order execution

**Key Innovation**: Frontend as CPU with parallel instruction execution (3x speedup), resource conflict detection, instruction scheduling, and complete observability.

---

## Development Commands

**⚠️ CRITICAL: DO NOT START DEV SERVERS**
- **NEVER run `pnpm dev`, `pnpm tauri dev`, or `cargo run`** - Already running
- Package manager: **pnpm** only

```bash
# Frontend
pnpm build
pnpm exec eslint src/
pnpm exec prettier --check .

# Backend
cargo test
cargo check
cargo clippy
```

---

## Architecture

### Backend: Feature-Sliced Architecture
```
src-tauri/src/
├── entities/                # Domain models & DTOs
├── features/                # Feature modules (vertical slices)
│   ├── tasks/, views/, time_blocks/, areas/
│   └── shared/              # Cross-feature infrastructure
└── shared/                  # Cross-cutting concerns
```

**Key Principles:**
- **SFC**: Each endpoint = standalone file
- **Write Serialization**: Use `AppState::acquire_write_permit()`
- **Event-Driven**: Emit SSE for real-time updates
- **TransactionHelper**: For all DB transactions

### Frontend: CPU Pipeline (5-stage)
```
Components → pipeline.dispatch('type', payload)
    ↓
IF (Instruction Fetch) → SCH (Scheduler) → EX (Execute) → RES (Response) → WB (Write Back)
```

**Benefits**: Sequential: 3s → Parallel: 1s (3x speedup)

**Directory Structure:**
```
src/
├── cpu/                     # Pipeline system
├── infra/                   # Hardware-inspired infrastructure
├── stores/                  # RTL hardware design (registers/wires)
├── components/              # Atomic design layers
├── composables/             # Business logic
└── views/                   # Page components
```

---

## CABC Documentation Standard

**CABC (Cutie API Behavior Contract)** - Required for ALL endpoints.

**8 Required Sections:**
1. **Endpoint Signature** - Precise signature
2. **High-Level Behavior** - User story + core logic
3. **Input/Output Specification** - Request/response schemas
4. **Validation Rules** - Input constraints
5. **Business Logic Walkthrough** - Step-by-step flow
6. **Edge Cases** - Non-happy-path scenarios
7. **Expected Side Effects** - Database ops, SSE events, logging
8. **Contract** - Pre/post-conditions, invariants

---

## Backend Development

### SFC Pattern (Single File Component)
```rust
// --- CABC Documentation ---
// --- Dependencies ---
// --- Request/Response DTOs ---
// --- HTTP Handler ---
pub async fn handle(State(app_state): State<AppState>, Json(request): Json<Request>) -> Response {
    let _permit = app_state.acquire_write_permit().await;
    // ... business logic
}
// --- Business Logic ---
// --- Database Access ---
```

### Shared Resources Reference
**⚠️ Check before implementing to avoid duplication!**

**Cross-Feature** (`features/shared/`):
- `TransactionHelper`: `begin()`, `commit()`
- `AreaRepository`

**Tasks** (`features/tasks/shared/`):
- `TaskRepository`, `TaskScheduleRepository`, `TaskTimeBlockLinkRepository`
- `TaskAssembler`, `LinkedTaskAssembler`, `TimeBlockAssembler`

**TimeBlocks** (`features/time_blocks/shared/`):
- `TimeBlockRepository`, `TimeBlockConflictChecker`

### AppState Methods
```rust
// ✅ Correct
let id = app_state.id_generator().new_uuid();
let now = app_state.clock().now_utc();
let pool = app_state.db_pool();
```

### Critical: SSE & HTTP Data Consistency
**⚠️ SSE events and HTTP responses MUST return identical data!**

Pattern: Transaction → Commit → Fill complete data → Emit SSE → Return response

### Database Schema
- **Schema location**: `src-tauri/migrations/20241001000000_initial_schema.sql`
- **Tables**: `tasks`, `areas`, `task_schedules`, `time_blocks`, `orderings`, `projects`, `view_preferences`

---

## Frontend Development

### CPU Pipeline Usage
```typescript
import { pipeline } from '@/cpu'

pipeline.start()
pipeline.dispatch('task.complete', { id: taskId })
pipeline.dispatch('task.update', { id: taskId, title: 'Updated' })
```

### Store Pattern V4.0: RTL Hardware Design
```typescript
// Store = Register File
export const useTaskStore = defineStore('task', () => {
  // STATE (registers)
  const tasks = ref(new Map<string, TaskCard>())

  // GETTERS (wires + mux)
  const allTasks = computed(() => Array.from(tasks.value.values()))
  const getTaskById_Mux = (id: string) => tasks.value.get(id)

  // MUTATIONS (write ports)
  const addOrUpdateTask_mut = (task: TaskCard) => tasks.value.set(task.id, task)

  // DMA transfers
  const fetchAllTasks_DMA = async () => { /* bulk loading */ }

  // EVENT HANDLING (SSE interrupts)
  const initEventSubscriptions = () => { /* SSE handlers */ }
})
```

**Store Structure:**
```
stores/{feature}/
├── index.ts         # Composition root
├── core.ts          # State, getters
├── mutations.ts     # Register operations
├── loaders.ts       # DMA bulk loading
└── event-handlers.ts # SSE interrupts
```

### Component Architecture
```
src/components/
├── parts/          # Atoms (CuteButton, CuteIcon)
├── templates/      # Molecules (CuteCard, TwoRowLayout)
├── functional/     # Organisms (ContextMenuHost)
└── alias/          # Semantic aliases
```

### Debugging
**CPU Debug View**: `/cpu-debug` route
**Browser Console**:
```javascript
cpuPipeline.help()
cpuPipeline.dispatch('debug.fetch_baidu', {})
appLogger.setLevel('DEBUG')
appLogger.filterByTag(['API', 'CPU'])
```

---

## Key Concepts

**Status:**
- Completed: `completed_at` has value
- Incomplete: `completed_at` IS NULL
- Archived: `archived_at` has value

**Valid Schedule:** Schedule with `scheduled_date` >= today

**View Filtering:**
- **Staging**: incomplete + not archived + NO valid schedule
- **Daily**: not archived + has schedule for today
- **Past/Future Date**: not archived + has schedule for that date

**Core Business Events:**
- **Task Completion**: Set `completed_at`, complete subtasks, set outcome, truncate time_block, delete future schedules
- **Task Reopen**: Set `completed_at` = NULL, set outcome = PRESENCE_LOGGED
- **Create Schedule**: Create schedule, auto-complete if past date
- **Return to Staging**: Delete schedules >= today, preserve past data

**View Context Format**: `{type}::{identifier}` (e.g., `daily::2025-10-01`, `area::{uuid}`)

---

## Data Schema & Coupling

**⚠️ Modifying data structures has cascading effects!**

**Data Flow:**
```
Database Schema → Backend Entity → Backend DTO → Assembler → Frontend DTO → Store → Components
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

---

## Development Workflows

### Adding New Endpoint
1. Check shared resources
2. Create SFC endpoint file with CABC docs
3. Register route in mod.rs
4. Update frontend store & SSE handlers
5. Test API, SSE, UI, data consistency

### Adding Field to Entity
1. Update schema (migration, delete old DB)
2. Update entity (model.rs)
3. Update DTOs (response & request)
4. Update assembler
5. Update ALL SQL queries
6. Check cross-features
7. Update frontend (DTOs, store, components)
8. Test end-to-end

### Creating New Feature
1. Schema (create tables)
2. Entities (model.rs, DTOs)
3. Feature module (endpoints/, shared/, mod.rs)
4. Implement endpoints (SFC + CABC)
5. Register routes
6. Frontend (store, components)
7. Test (API, SSE, UI, persistence)

---

## Quick Reference

### Pre-Development Checklist
- [ ] Read schema: `src-tauri/migrations/*.sql`
- [ ] Check shared resources
- [ ] Use TransactionHelper for transactions
- [ ] Use correct AppState methods (`.new_uuid()`, `.now_utc()`)
- [ ] Fill complete data BEFORE SSE
- [ ] Never modify shared resources in features

### Testing Checklist
- [ ] `cargo check` - no errors
- [ ] `cargo clippy` - no warnings
- [ ] Test API, SSE, UI manually
- [ ] Check database directly

### Useful Commands
```bash
# Find DTO usages
grep -rn "TaskCardDto {" src-tauri/src/features

# Find SQL queries
grep -rn "SELECT.*FROM tasks" src-tauri/src

# Find SSE emissions
grep -rn "DomainEvent::new" src-tauri/src

# Reset database
rm src-tauri/*.db*
```

### Important Notes
- **Port Discovery**: Frontend listens for `sidecar-port-discovered` event
- **Type Generation**: Rust `#[derive(TS)]` generates TypeScript types
- **Development Folder**: `develop/` is experimental - DO NOT use in production
