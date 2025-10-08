# Cutie Feature Development Guide (AI Version)

**Version**: 1.0
**Last Updated**: 2025-10-08

## Core Principles

1. Code must match CABC documentation exactly
2. Backend must return actual database state, never rely on defaults
3. SSE events and HTTP responses must return identical data
4. Always read database schema before writing SQL
5. Use shared resources, never duplicate existing functionality
6. Never modify shared resources in `features/shared` or `features/xxx/shared`

## Required Reading Order

1. `src-tauri/migrations/20241001000000_initial_schema.sql`
2. This document's shared resources section
3. `notes/业务逻辑.md`
4. `references/SFC_SPEC.md`
5. `references/DATA_SCHEMA_COUPLING.md`
6. `ai-doc/LESSONS_LEARNED.md`

## Backend Development Flow

### Step 1: Design

**1.1 Read Schema**

All table names are plural: `tasks`, `areas`, `time_blocks`, `orderings`

```rust
// ❌ Wrong
SELECT * FROM ordering WHERE ...

// ✅ Correct
SELECT * FROM orderings WHERE ...
```

**1.2 Check Shared Resources**

Search shared resources list before implementing:
- Repository exists? Use it
- Assembler exists? Use it
- Utility exists? Use it

**1.3 Reference Implementations**

- Simple CRUD: `features/areas/endpoints/create_area.rs`
- Complex logic: `features/tasks/endpoints/complete_task.rs`
- Cross-entity: `features/time_blocks/endpoints/create_from_task.rs`

### Step 2: Entity Layer

**2.1 Model** (`entities/xxx/model.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

#[derive(Debug, FromRow)]
pub struct EntityRow {
    pub id: String,  // Database stores as TEXT
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_deleted: bool,
}

impl TryFrom<EntityRow> for Entity {
    type Error = uuid::Error;
    fn try_from(row: EntityRow) -> Result<Self, Self::Error> {
        Ok(Entity {
            id: Uuid::parse_str(&row.id)?,
            name: row.name,
            created_at: row.created_at,
            updated_at: row.updated_at,
            is_deleted: row.is_deleted,
        })
    }
}
```

**2.2 Request DTOs** (`entities/xxx/request_dtos.rs`)

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateEntityRequest {
    pub name: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateEntityRequest {
    pub name: Option<String>,
    pub color: Option<String>,
}
```

**2.3 Response DTOs** (`entities/xxx/response_dtos.rs`)

```rust
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct EntityDto {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**2.4 Module Exports** (`entities/xxx/mod.rs`)

```rust
pub mod model;
pub mod request_dtos;
pub mod response_dtos;

pub use model::*;
pub use request_dtos::*;
pub use response_dtos::*;
```

Add to `entities/mod.rs`:
```rust
pub mod xxx;
```

### Step 3: Endpoint (SFC Pattern)

**File**: `features/xxx/endpoints/create_xxx.rs`

```rust
// ==================== CABC Documentation ====================
/*
CABC for `create_xxx`

## 1. Endpoint Signature
POST /api/xxx

## 2. High-Level Behavior
### 2.1 User Story
> As a user, I want to...

### 2.2 Core Business Logic
[Description]

## 3. Input/Output Specification
### 3.1 Request
{ "name": "string (required)" }

### 3.2 Response
**201 Created:**
{ "id": "uuid", "name": "string", ... }

## 4. Validation Rules
- name: required, non-empty, length <= 255

## 5. Business Logic Walkthrough
1. Validate input
2. Start transaction
3. Generate UUID and timestamp
4. Insert database
5. Commit transaction
6. Return result

## 6. Edge Cases
- name empty: 422
- name duplicate: 409 (if unique constraint)

## 7. Expected Side Effects
### Database Operations:
- INSERT: 1 record to xxx table
- Transaction: begin() → commit()

### SSE Events:
- xxx.created

## 8. Contract
### Pre-conditions:
- request.name not empty

### Post-conditions:
- New record exists in database
- Return complete EntityDto

### Invariants:
- id and created_at never change once created
*/

// ==================== Dependencies ====================
use axum::{extract::State, response::{IntoResponse, Response}, Json};
use serde::Deserialize;
use crate::{
    entities::xxx::{Entity, EntityDto, CreateEntityRequest},
    features::shared::TransactionHelper,
    shared::{core::error::{AppError, AppResult}, http::responses::created_response},
    startup::AppState,
};

// ==================== HTTP Handler ====================
pub async fn handle(
    State(app_state): State<AppState>,
    Json(request): Json<CreateEntityRequest>,
) -> Response {
    match logic::execute(&app_state, request).await {
        Ok(dto) => created_response(dto).into_response(),
        Err(err) => err.into_response(),
    }
}

// ==================== Validation ====================
mod validation {
    use super::*;
    pub fn validate_request(request: &CreateEntityRequest) -> AppResult<()> {
        let mut errors = Vec::new();
        if request.name.trim().is_empty() {
            errors.push("name cannot be empty");
        }
        if request.name.len() > 255 {
            errors.push("name too long");
        }
        if !errors.is_empty() {
            return Err(AppError::ValidationFailed(errors.join(", ")));
        }
        Ok(())
    }
}

// ==================== Business Logic ====================
mod logic {
    use super::*;
    pub async fn execute(
        app_state: &AppState,
        request: CreateEntityRequest,
    ) -> AppResult<EntityDto> {
        validation::validate_request(&request)?;

        let id = app_state.id_generator().new_uuid();
        let now = app_state.clock().now_utc();

        let mut tx = TransactionHelper::begin(app_state.db_pool()).await?;

        let entity = Entity {
            id, name: request.name, created_at: now,
            updated_at: now, is_deleted: false,
        };

        database::insert_in_tx(&mut tx, &entity).await?;
        TransactionHelper::commit(tx).await?;

        let dto = EntityDto {
            id: entity.id, name: entity.name,
            created_at: entity.created_at, updated_at: entity.updated_at,
        };

        Ok(dto)
    }
}

// ==================== Database Access ====================
mod database {
    use super::*;
    use sqlx::{Transaction, Sqlite};

    pub async fn insert_in_tx(
        tx: &mut Transaction<'_, Sqlite>,
        entity: &Entity,
    ) -> AppResult<()> {
        let query = r#"
            INSERT INTO xxx_table (id, name, created_at, updated_at, is_deleted)
            VALUES (?, ?, ?, ?, ?)
        "#;
        sqlx::query(query)
            .bind(entity.id.to_string())
            .bind(&entity.name)
            .bind(entity.created_at)
            .bind(entity.updated_at)
            .bind(entity.is_deleted)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.into()))?;
        Ok(())
    }
}
```

**Critical Checks**

```rust
// ✅ Correct AppState methods
let id = app_state.id_generator().new_uuid();
let now = app_state.clock().now_utc();
let pool = app_state.db_pool();

// ❌ Wrong - methods don't exist
let id = app_state.id_generator().generate();
let now = app_state.clock().now();
```

```rust
// ✅ Correct - use TransactionHelper
use crate::features::shared::TransactionHelper;
let mut tx = TransactionHelper::begin(pool).await?;
TransactionHelper::commit(tx).await?;
```

```rust
// ✅ Correct - use shared Repository
use crate::features::tasks::shared::repositories::TaskRepository;
let task = TaskRepository::find_by_id_in_tx(&mut tx, task_id).await?;

// ❌ Wrong - duplicate implementation
mod database {
    pub async fn find_task(...) { ... }  // Already exists!
}
```

### Step 4: SSE Events

**SSE Data Consistency Rule**

SSE events and HTTP responses MUST return identical data.

```rust
// ❌ Wrong - SSE before filling complete data
let mut tx = TransactionHelper::begin(pool).await?;
database::update_something(&mut tx, task_id).await?;
let mut task_card = TaskAssembler::task_to_card_basic(&task);
// task_card.schedules = None (default, not filled)

// Writing SSE with incomplete data
let event = DomainEvent::new("task.updated", "task", task_id, json!({
    "task": task_card,  // schedules = None ❌
}));
outbox_repo.append_in_tx(&mut tx, &event).await?;
TransactionHelper::commit(tx).await?;

// Fill complete data later
task_card.schedules = assemble_schedules(pool, task_id).await?;
Ok(Response { task: task_card })  // schedules = Some([...]) ✅

// ✅ Correct - fill complete data before SSE
let mut tx = TransactionHelper::begin(pool).await?;
database::update_something(&mut tx, task_id).await?;
let mut task_card = TaskAssembler::task_to_card_basic(&task);
TransactionHelper::commit(tx).await?;

// Fill ALL data BEFORE SSE
task_card.schedules = assemble_schedules(pool, task_id).await?;
task_card.area = get_area_summary(pool, area_id).await?;

// Write SSE with complete data
let mut outbox_tx = TransactionHelper::begin(pool).await?;
let event = DomainEvent::new("task.updated", "task", task_id, json!({
    "task": task_card,  // schedules = Some([...]) ✅
}));
outbox_repo.append_in_tx(&mut outbox_tx, &event).await?;
TransactionHelper::commit(outbox_tx).await?;

Ok(Response { task: task_card })
```

### Step 5: Register Routes

**Feature Routes** (`features/xxx/mod.rs`)

```rust
use axum::{routing::{get, post, patch, delete}, Router};
use crate::startup::AppState;

pub mod endpoints {
    pub mod create_xxx;
    pub mod list_xxx;
    pub mod update_xxx;
    pub mod delete_xxx;
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(endpoints::list_xxx::handle).post(endpoints::create_xxx::handle))
        .route("/:id", get(endpoints::get_xxx::handle).patch(endpoints::update_xxx::handle).delete(endpoints::delete_xxx::handle))
}
```

**Global Routes** (`features/mod.rs`)

```rust
pub mod xxx;

pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .nest("/xxx", xxx::create_routes())
}
```

### Step 6: API Documentation

Write CABC in endpoint file comments first, then use `doc-composer` tool to generate API docs.

## Frontend Development Flow

### Step 1: DTO Types

**File**: `src/types/dtos.ts`

```typescript
export interface Entity {
  id: string
  name: string
  created_at: string
  updated_at: string
}
```

### Step 2: Pinia Store

**Structure** (reference `stores/task/`):

```
stores/xxx/
├── index.ts           # Store composition
├── core.ts            # State & Getters
├── crud-operations.ts # Create/Update/Delete
├── view-operations.ts # Fetch/Query
└── event-handlers.ts  # SSE subscriptions
```

**2.1 Core** (`stores/xxx/core.ts`)

```typescript
import { ref, computed } from 'vue'

export const entities = ref(new Map<string, Entity>())

export const allEntities = computed(() =>
  Array.from(entities.value.values())
)

export const getEntityById = computed(() => (id: string) =>
  entities.value.get(id)
)

export function addOrUpdateEntity(entity: Entity) {
  const newMap = new Map(entities.value)
  newMap.set(entity.id, entity)
  entities.value = newMap  // Create new object for reactivity
}

export function removeEntity(id: string) {
  const newMap = new Map(entities.value)
  newMap.delete(id)
  entities.value = newMap
}

export function clearAll() {
  entities.value = new Map()
}
```

**2.2 CRUD** (`stores/xxx/crud-operations.ts`)

```typescript
import { apiBaseUrl } from '@/composables/useApiConfig'
import { addOrUpdateEntity, removeEntity } from './core'

export async function createEntity(payload: CreateEntityPayload): Promise<Entity> {
  const response = await fetch(`${apiBaseUrl.value}/xxx`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  if (!response.ok) throw new Error('Failed to create entity')
  const entity: Entity = await response.json()  // Parse directly, not .data
  addOrUpdateEntity(entity)
  return entity
}

export async function updateEntity(id: string, payload: UpdateEntityPayload): Promise<Entity> {
  const response = await fetch(`${apiBaseUrl.value}/xxx/${id}`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(payload),
  })
  if (!response.ok) throw new Error('Failed to update entity')
  const entity: Entity = await response.json()
  addOrUpdateEntity(entity)
  return entity
}

export async function deleteEntity(id: string): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/xxx/${id}`, { method: 'DELETE' })
  if (!response.ok) throw new Error('Failed to delete entity')
  removeEntity(id)
}
```

**2.3 View** (`stores/xxx/view-operations.ts`)

```typescript
import { apiBaseUrl } from '@/composables/useApiConfig'
import { addOrUpdateEntity, clearAll } from './core'

export async function fetchAllEntities(): Promise<void> {
  const response = await fetch(`${apiBaseUrl.value}/xxx`)
  if (!response.ok) throw new Error('Failed to fetch entities')
  const entities: Entity[] = await response.json()  // Parse array directly
  clearAll()
  entities.forEach(addOrUpdateEntity)
}
```

**2.4 Event Handlers** (`stores/xxx/event-handlers.ts`)

```typescript
import { getEventSubscriber } from '@/services/events'
import { addOrUpdateEntity, removeEntity } from './core'

export function initEventSubscriptions() {
  const subscriber = getEventSubscriber()
  if (!subscriber) return

  subscriber.on('xxx.created', handleEntityCreatedEvent)
  subscriber.on('xxx.updated', handleEntityUpdatedEvent)
  subscriber.on('xxx.deleted', handleEntityDeletedEvent)
}

function handleEntityCreatedEvent(event: any) {
  const entity = event.payload?.entity
  if (entity) addOrUpdateEntity(entity)
}

function handleEntityUpdatedEvent(event: any) {
  const entity = event.payload?.entity
  if (entity) addOrUpdateEntity(entity)
}

function handleEntityDeletedEvent(event: any) {
  const entityId = event.payload?.entity_id
  if (entityId) removeEntity(entityId)
}
```

**2.5 Composition** (`stores/xxx/index.ts`)

```typescript
import { defineStore } from 'pinia'
import * as core from './core'
import * as crud from './crud-operations'
import * as view from './view-operations'
import * as events from './event-handlers'

export const useEntityStore = defineStore('entity', () => {
  return {
    entities: core.entities,
    allEntities: core.allEntities,
    getEntityById: core.getEntityById,
    createEntity: crud.createEntity,
    updateEntity: crud.updateEntity,
    deleteEntity: crud.deleteEntity,
    fetchAllEntities: view.fetchAllEntities,
    initEventSubscriptions: events.initEventSubscriptions,
  }
})
```

**2.6 Initialize SSE** (`composables/useApiConfig.ts`)

```typescript
import { useEntityStore } from '@/stores/xxx'
const entityStore = useEntityStore()
entityStore.initEventSubscriptions()
```

### Step 3: Register SSE Listeners

**File**: `src/services/events.ts`

```typescript
this.eventSource.addEventListener('xxx.created', (e: MessageEvent) => {
  this.handleEvent('xxx.created', e.data)
})
this.eventSource.addEventListener('xxx.updated', (e: MessageEvent) => {
  this.handleEvent('xxx.updated', e.data)
})
this.eventSource.addEventListener('xxx.deleted', (e: MessageEvent) => {
  this.handleEvent('xxx.deleted', e.data)
})
```

### Step 4: UI Components

```vue
<script setup lang="ts">
import { onMounted } from 'vue'
import { useEntityStore } from '@/stores/xxx'

const entityStore = useEntityStore()

onMounted(async () => {
  await entityStore.fetchAllEntities()
})

async function handleCreate(name: string) {
  await entityStore.createEntity({ name })
}

async function handleUpdate(id: string, name: string) {
  await entityStore.updateEntity(id, { name })
}

async function handleDelete(id: string) {
  await entityStore.deleteEntity(id)
}
</script>

<template>
  <div>
    <ul>
      <li v-for="entity in entityStore.allEntities" :key="entity.id">
        {{ entity.name }}
        <button @click="handleDelete(entity.id)">Delete</button>
      </li>
    </ul>
  </div>
</template>
```

### Step 5: Add Routes

**File**: `src/router/index.ts`

```typescript
{
  path: '/xxx',
  name: 'xxx',
  component: () => import('../views/XxxView.vue'),
}
```

## Shared Resources

### Backend

**Cross-Feature** (`features/shared/`)

```rust
// AreaRepository (features/shared/repositories/area_repository.rs)
pub async fn get_summary(executor: impl sqlx::Executor<'_, Database = Sqlite>, area_id: Uuid) -> AppResult<Option<AreaSummary>>
pub async fn get_summaries_batch(executor: impl sqlx::Executor<'_, Database = Sqlite>, area_ids: &[Uuid]) -> AppResult<Vec<AreaSummary>>

// TransactionHelper (features/shared/transaction.rs)
pub async fn begin(pool: &SqlitePool) -> AppResult<Transaction<'_, Sqlite>>
pub async fn commit(tx: Transaction<'_, Sqlite>) -> AppResult<()>
```

**Tasks** (`features/tasks/shared/`)

```rust
// TaskRepository
pub async fn find_by_id_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid) -> AppResult<Option<Task>>
pub async fn find_by_id(pool: &SqlitePool, task_id: Uuid) -> AppResult<Option<Task>>
pub async fn insert_in_tx(tx: &mut Transaction<'_, Sqlite>, task: &Task) -> AppResult<()>
pub async fn update_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid, request: &UpdateTaskRequest) -> AppResult<Task>
pub async fn soft_delete_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid) -> AppResult<()>
pub async fn set_completed_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid, completed_at: DateTime<Utc>) -> AppResult<()>
pub async fn set_reopened_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid, updated_at: DateTime<Utc>) -> AppResult<()>

// TaskScheduleRepository
pub async fn has_any_schedule(executor: impl sqlx::Executor<'_, Database = Sqlite>, task_id: Uuid) -> AppResult<bool>
pub async fn has_schedule_for_day_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid, scheduled_day: NaiveDate) -> AppResult<bool>
pub async fn create_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid, scheduled_day: NaiveDate) -> AppResult<()>
pub async fn update_today_to_completed_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid, now: DateTime<Utc>) -> AppResult<()>
pub async fn delete_future_schedules_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid, now: DateTime<Utc>) -> AppResult<()>
pub async fn delete_all_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid) -> AppResult<()>
pub async fn get_all_for_task(pool: &SqlitePool, task_id: Uuid) -> AppResult<Vec<TaskSchedule>>

// TaskTimeBlockLinkRepository
pub async fn link_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid, block_id: Uuid) -> AppResult<()>
pub async fn delete_all_for_task_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid) -> AppResult<()>
pub async fn delete_all_for_block_in_tx(tx: &mut Transaction<'_, Sqlite>, block_id: Uuid) -> AppResult<()>
pub async fn find_linked_time_blocks_in_tx(tx: &mut Transaction<'_, Sqlite>, task_id: Uuid) -> AppResult<Vec<TimeBlock>>
pub async fn is_exclusive_link_in_tx(tx: &mut Transaction<'_, Sqlite>, block_id: Uuid, task_id: Uuid) -> AppResult<bool>
pub async fn count_remaining_tasks_in_block_in_tx(tx: &mut Transaction<'_, Sqlite>, block_id: Uuid) -> AppResult<i64>

// TaskAssembler
pub fn task_to_card_basic(task: &Task) -> TaskCardDto
pub fn task_to_card_full(task: &Task, schedule_status: ScheduleStatus, area: Option<AreaSummary>, schedule_info: Option<ScheduleInfo>) -> TaskCardDto
pub fn task_to_detail_basic(task: &Task) -> TaskDetailDto

// LinkedTaskAssembler
pub async fn get_summaries_batch(executor: impl sqlx::Executor<'_, Database = Sqlite>, task_ids: &[Uuid]) -> AppResult<Vec<LinkedTaskSummary>>
pub async fn get_for_time_block(executor: impl sqlx::Executor<'_, Database = Sqlite>, block_id: Uuid) -> AppResult<Vec<LinkedTaskSummary>>

// TimeBlockAssembler
pub async fn assemble_for_event_in_tx(tx: &mut Transaction<'_, Sqlite>, time_block_ids: &[Uuid]) -> AppResult<Vec<TimeBlockViewDto>>
pub async fn assemble_view(block: &TimeBlock, pool: &SqlitePool) -> AppResult<TimeBlockViewDto>
```

**TimeBlocks** (`features/time_blocks/shared/`)

```rust
// TimeBlockRepository
pub async fn find_by_id_in_tx(tx: &mut Transaction<'_, Sqlite>, block_id: Uuid) -> AppResult<Option<TimeBlock>>
pub async fn find_by_id(pool: &SqlitePool, block_id: Uuid) -> AppResult<Option<TimeBlock>>
pub async fn insert_in_tx(tx: &mut Transaction<'_, Sqlite>, block: &TimeBlock) -> AppResult<()>
pub async fn update_in_tx(tx: &mut Transaction<'_, Sqlite>, block_id: Uuid, request: &UpdateTimeBlockRequest, updated_at: DateTime<Utc>) -> AppResult<TimeBlock>
pub async fn soft_delete_in_tx(tx: &mut Transaction<'_, Sqlite>, block_id: Uuid) -> AppResult<()>
pub async fn truncate_to_in_tx(tx: &mut Transaction<'_, Sqlite>, block_id: Uuid, end_time: DateTime<Utc>) -> AppResult<()>
pub async fn find_in_range(pool: &SqlitePool, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> AppResult<Vec<TimeBlock>>
pub async fn exists_in_tx(tx: &mut Transaction<'_, Sqlite>, block_id: Uuid) -> AppResult<bool>

// TimeBlockConflictChecker
pub async fn check_in_tx(tx: &mut Transaction<'_, Sqlite>, start_time: DateTime<Utc>, end_time: DateTime<Utc>, exclude_id: Option<Uuid>) -> AppResult<()>
```

**Views** (`features/views/shared/`)

```rust
// ViewTaskCardAssembler
pub async fn assemble_full(task: &Task, pool: &SqlitePool) -> AppResult<TaskCardDto>
pub async fn assemble_batch(tasks: &[Task], pool: &SqlitePool) -> AppResult<Vec<TaskCardDto>>
pub async fn assemble_with_status(task: &Task, pool: &SqlitePool, status: ScheduleStatus) -> AppResult<TaskCardDto>
```

**Core Utils** (`shared/core/utils/`)

```rust
// sort_order_utils.rs
use crate::shared::core::utils::{
    generate_initial_sort_order,
    get_rank_after,
    get_rank_before,
    get_mid_lexo_rank,
};

// ✅ Correct
let sort_order = get_rank_after(&max)?;

// ❌ Wrong - manual implementation
let mut chars: Vec<char> = max.chars().collect();
*last_char = ((*last_char as u8) + 1) as char;
```

### Frontend

```typescript
// useApiConfig.ts
import { apiBaseUrl, waitForApiReady } from '@/composables/useApiConfig'

// ✅ Correct
const response = await fetch(`${apiBaseUrl.value}/tasks`)

// ❌ Wrong
const response = await fetch('http://127.0.0.1:3538/api/tasks')

// events.ts
import { getEventSubscriber } from '@/services/events'
const subscriber = getEventSubscriber()
if (subscriber) {
  subscriber.on('task.created', handleTaskCreatedEvent)
}
```

## Data Schema Modification Impact

### Complete Impact Chain

```
Database Schema (SQLite)
  ↓
Backend Entity (Rust entities)
  ↓
Backend Request DTO (Request DTOs)
  ↓
Backend Response DTO (Response DTOs)
  ↓
Assembler (entity to DTO conversion)
  ↓
Repository (database read/write)
  ↓
Endpoint (API endpoints)
  ↓
Frontend Types (TypeScript types)
  ↓
Pinia Store (state management)
  ↓
Vue Components (UI)
```

### Field Addition Checklist

**Backend**:
- [ ] Schema: `migrations/xxx.sql` add field
- [ ] Entity: `entities/xxx/model.rs` Entity struct
- [ ] EntityRow: `entities/xxx/model.rs` XxxRow struct
- [ ] TryFrom: `TryFrom<XxxRow>` implementation
- [ ] Request DTO: `entities/xxx/request_dtos.rs`
- [ ] Response DTO: `entities/xxx/response_dtos.rs`
- [ ] Assembler: update conversion logic
- [ ] Repository: update all SQL SELECT/INSERT/UPDATE
- [ ] Cross-feature check: `grep -rn "DtoName {" src-tauri/src/features`

**Frontend**:
- [ ] DTO: `src/types/dtos.ts`
- [ ] Store: update payload types
- [ ] UI: update display and edit logic

### Cross-Feature Dependencies

TimeBlock cross-feature usage:
- Primary: `features/time_blocks/`
- Cross-feature assembler: `features/tasks/shared/assemblers/time_block_assembler.rs`
- Cross-feature repository: `features/tasks/shared/repositories/task_time_block_link_repository.rs`

**Find cross-feature usages**:
```bash
grep -rn "TimeBlockViewDto {" src-tauri/src/features
grep -rn "SELECT.*FROM time_blocks" src-tauri/src/features
```

## Key Lessons

### 1. Never Hardcode API Port

```typescript
// ❌ Wrong
const response = await fetch(`http://127.0.0.1:3538/api/time-blocks/${id}/link-task`, { ... })

// ✅ Correct
import { apiBaseUrl } from '@/composables/useApiConfig'
const response = await fetch(`${apiBaseUrl.value}/time-blocks/${id}/link-task`, { ... })
```

### 2. Backend Enum Format Inconsistency

Backend has two enums:
- Input: `Outcome` (UPPERCASE: `PLANNED`, `PRESENCE_LOGGED`)
- Output: `DailyOutcome` (snake_case: `planned`, `presence_logged`)

```typescript
// ✅ Correct - receive: snake_case (from DTO)
const isPresenceLogged = computed(() => {
  return currentScheduleOutcome.value === 'presence_logged'
})

// ✅ Correct - send: UPPERCASE (to API)
const newOutcome = newCheckedValue ? 'PRESENCE_LOGGED' : 'PLANNED'
await taskStore.updateSchedule(taskId, date, { outcome: newOutcome })
```

### 3. Complete Data Flow Required

Data flow breakpoints:
```
Database (tasks.estimated_duration)
  ↓ ✅ Task entity has field
  ↓ ❌ TaskCardDto missing field ← Breakpoint 1
  ↓ ❌ Assembler not mapping ← Breakpoint 2
  ↓ ✅ Frontend DTO has field
  ↓ ✅ UI display (receives undefined, shows NaN)
  ↓ ❌ Update endpoint not handling ← Breakpoint 3
  ✗ Cannot write to database
```

### 4. SSE Event Chain - 7 Layer Issues

1. Business logic flaw: based on title judgment, not source_type
2. Store missing SSE: TimeBlockStore has no event subscription
3. Endpoint missing SSE: create_from_task not sending SSE
4. EventSource not registered: events.ts missing addEventListener
5. area_id not updated: link_task not inheriting task's area_id
6. SSE payload incomplete: only ID, no complete data
7. API doesn't exist: frontend calling non-existent `/api/time-blocks?ids=X`

**SSE Chain Checklist**:

Backend:
- [ ] Endpoint sends SSE event (EventOutbox)
- [ ] SSE payload contains complete data, not just ID
- [ ] Event type naming consistent

Middleware (events.ts):
- [ ] EventSource.addEventListener registered
- [ ] handleEvent parses and dispatches correctly

Frontend Store:
- [ ] Store implements initEventSubscriptions
- [ ] Store subscribes to all relevant events
- [ ] Event handler processes data correctly
- [ ] useApiConfig.ts calls initEventSubscriptions

Testing:
- [ ] Console shows SSE event logs
- [ ] Store handler called correctly
- [ ] UI updates in real-time, no manual refresh

### 5. Orphan Time Block Deletion Logic Flaw

```rust
// ❌ Wrong - based on title
if time_block.title == deleted_task.title { ... }

// ✅ Correct - based on source_type
pub struct SourceInfo {
    pub source_type: String,  // "native::from_task" | "native::manual" | "external::*"
    pub created_by_task_id: Option<Uuid>,
}

if source_info.source_type == "native::from_task" {
    return Ok(true);  // Orphan + auto-created = delete
}
Ok(false)  // Other sources preserved
```

## Development Checklists

### Backend

**Pre-Development**:
- [ ] Read schema: `migrations/xxx.sql`
- [ ] Check shared resources
- [ ] Select reference implementation

**Entity Layer**:
- [ ] Create Entity struct
- [ ] Create EntityRow struct
- [ ] Implement TryFrom<EntityRow>
- [ ] Create Request DTOs
- [ ] Create Response DTOs
- [ ] Export modules

**Endpoint (SFC)**:
- [ ] Write complete CABC documentation
- [ ] Implement HTTP Handler
- [ ] Implement Validation (if needed)
- [ ] Implement Business Logic
- [ ] Implement Database Access
- [ ] Use correct trait methods (`new_uuid()`, `now_utc()`)
- [ ] Use TransactionHelper
- [ ] Reuse shared resources, no duplication
- [ ] Query actual state, don't rely on defaults
- [ ] Fill complete data before SSE
- [ ] SSE and HTTP return same data

**Route Registration**:
- [ ] Register in feature's mod.rs
- [ ] Register in features/mod.rs

**Testing**:
- [ ] Run `cargo check`
- [ ] Run `cargo clippy`
- [ ] Test API
- [ ] Test SSE events
- [ ] Test complete data flow

### Frontend

**Type Layer**:
- [ ] Add interface to `src/types/dtos.ts`

**Store Layer**:
- [ ] Create core.ts (State & Getters)
- [ ] Create crud-operations.ts
- [ ] Create view-operations.ts
- [ ] Create event-handlers.ts
- [ ] Compose in index.ts
- [ ] Initialize SSE in useApiConfig.ts

**SSE Layer**:
- [ ] Register addEventListener in events.ts

**UI Layer**:
- [ ] Create management/list component
- [ ] Create edit/detail component
- [ ] Add routes
- [ ] Add navigation links

**Testing**:
- [ ] Check linter errors
- [ ] Test CRUD operations
- [ ] Test SSE real-time updates
- [ ] Test complete workflow

### Data Structure Modification

**Backend**:
- [ ] Schema: migrations/xxx.sql
- [ ] Entity: entities/xxx/model.rs (Entity + EntityRow + TryFrom)
- [ ] Request DTO: entities/xxx/request_dtos.rs
- [ ] Response DTO: entities/xxx/response_dtos.rs
- [ ] Assembler: features/xxx/shared/assembler.rs
- [ ] Repository: all SELECT/INSERT/UPDATE SQL
- [ ] Cross-feature assemblers: `grep -rn "XxxDto {" src-tauri/src/features`
- [ ] Cross-feature repositories: `grep -rn "SELECT.*FROM xxx" src-tauri/src/features`

**Frontend**:
- [ ] DTO: src/types/dtos.ts
- [ ] Store: src/stores/xxx.ts
- [ ] UI: display and edit logic

## Debugging

### Common Errors

**Error 1**: `no column found for name: xxx`
- Cause: Forgot to add field in SQL SELECT
- Fix: Update all SQL queries for this table

**Error 2**: `missing field 'xxx' in initializer`
- Cause: Assembler or DTO initialization missing field
- Fix: Update assembler and all DTO initializations

**Error 3**: `method not found in IdGenerator`
- Cause: Wrong method name
- Fix: Use `new_uuid()` not `generate()`

### Find Duplicates

```bash
# Find all DTO assemblies
grep -rn "TaskCardDto {" src-tauri/src/features

# Find all SQL queries
grep -rn "SELECT.*FROM tasks" src-tauri/src

# Find all SSE emissions
grep -rn "DomainEvent::new" src-tauri/src
```

### SSE Debugging Order

1. Backend sending? Check logs, event_outbox table
2. Network transfer? DevTools → Network → EventStream
3. EventSource receiving? Check addEventListener registration
4. Store subscribed? Check initEventSubscriptions called
5. Handler executing? Add console.log
6. Data processing? Verify payload structure

## Summary

Core development steps:
1. Read Schema
2. Check shared resources
3. Reference similar features
4. Follow SFC specification
5. Fill complete data
6. SSE consistency with HTTP
7. Complete end-to-end testing

Core principles:
- Schema first: read database, don't guess
- Reuse first: use shared resources, no duplication
- Data reality: query actual state, no defaults
- SSE consistency: fill data before sending events
- Documentation driven: code must match CABC
