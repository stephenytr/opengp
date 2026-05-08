# rat-salsa Cleanup: Delete Dead App Code

## TL;DR

> **Quick Summary**: Delete the ~5,300 lines of dead `App` struct/impl/extension code left behind after the rat-salsa migration, leaving only the live `AppState` + `GlobalState` + `run_tui` stack in `main.rs`. App behavior is identical — all behavior already lives in the new stack.
>
> **Key verified findings:**
> - `AppCommand` + `command_tx` are **ALIVE** — used by `state/billing.rs` for billing data loading
> - `command_rx` is **DEAD** — `take_command_rx()` is never called, channel is never consumed
> - `PendingBillingSaveData` field in `PatientWorkspace` is **DEAD** — `set_pending_billing()` is never called, field is always `None`
> - `RetryOperation`, `AppContextMenuAction` are **ALIVE** — defined in dead `ui/app.rs`, used by live `global.rs` and `main.rs`
> - `current_context: KeyContext` in `AppState` is **MIGRATION DEBT** — used by `main.rs:710, 575, 1186`
> - `ui/keybinds.rs` — keep `KeyContext`, `Action`, `KeybindRegistry` types; delete dead lookup table implementations
>
> **Deliverables**:
> - Move `RetryOperation`, `AppContextMenuAction` to live module (`global.rs` or new `types.rs`)
> - Move `PendingBillingSaveData` to workspace module (near where it's consumed)
> - Delete all `impl App` extension files (event_handler, state, keybinds, renderer)
> - Delete dead `AppCommand` channel infrastructure (`command_rx`, `take_command_rx`)
> - Delete dead `pending_billing` field from `PatientWorkspace`
> - Remove `pub use ui::app::App` from public exports
> - Audit `ui/keybinds.rs` — keep types, delete dead lookup tables
>
> **Estimated Effort**: Medium
> **Parallel Execution**: NO — sequential dependency: migrate types first, then delete
> **Critical Path**: Type migration → verify build → delete dead files → verify → cleanup → verify

---

## Context

### Original Request
User reports "something in recent changes has made things really difficult for you figure out what it is" — the rat-salsa migration left massive duplicate code that makes navigation and understanding impossible.

### Verified Findings (explore agent audit)

**What is live (in `src/main.rs`):**
- `AppState` struct (`ui/app/app_state.rs`) — all UI state, mutated by `event_fn` in `main.rs`
- `GlobalState` struct (`ui/app/global.rs`) — services + `SalsaAppContext` + `DialogStack`
- `event_fn(state, ctx, event) -> Control<AppEvent>` in `main.rs:604-1518` — all event handling
- `render(state, ctx, area, buf)` in `main.rs:600-1779` — all rendering
- `dialog_render(ctx, dialog, area, buf)` in `main.rs:519-565` — dialog rendering
- `init(state, ctx)` in `main.rs:286-293` — bootstrap
- 17 `spawn_async` helpers in `main.rs:295-523` — async task spawning
- `build_focus(state)` in `main.rs:684-694` — rat-focus focus building
- `handle_mouse_event` in `main.rs:1526-1858` — mouse handling

**What is dead (`struct App` ecosystem — ~5,300 lines):**

| File | Lines | Why Dead |
|---|---|---|
| `ui/app.rs` (`struct App`, `impl App`) | 1023 | `main.rs` never mentions `App`; only own tests call `handle_key_event` |
| `ui/app/event_handler.rs` (`handle_key_event` 457-line method) | 506 | Never called — `main.rs` has `event_fn` |
| `ui/app/event_handler/global.rs` (`handle_global_mouse_event` 326-line) | 410 | Never called — `main.rs` has `handle_mouse_event` |
| `ui/app/event_handler/appointment.rs` | 168 | `impl App` extension, never called |
| `ui/app/event_handler/workspace_tests.rs` | 285 | Tests against dead `App` |
| `ui/app/state.rs` + `ui/app/state/*.rs` (5 non-empty files) | 273 | `impl App` extension methods, never called |
| `ui/app/state/api_polling.rs` | 0 | Empty file — delete |
| `ui/app/keybinds/appointment.rs` | 113 | `impl App` keybind dispatch, never called |
| `ui/app/keybinds/billing.rs` | 56 | `impl App` keybind dispatch, never called |
| `ui/app/keybinds/clinical.rs` | 647 | `impl App` keybind dispatch, never called |
| `ui/app/keybinds/patient.rs` | 55 | `impl App` keybind dispatch, never called |
| `ui/app/keybinds/mod.rs` | 5 | Just re-exports, dead |
| `ui/app/renderer.rs` | 583 | `impl App { render }`, never called — `main.rs` has `render` |
| **`ui/keybinds.rs`** | 1208 | **PARTIALLY LIVE** — `KeyContext`, `Action`, `KeybindRegistry` types are live; lookup table implementations are dead |

**Key types status:**

| Type | Status | Evidence |
|---|---|---|
| `AppCommand` | **ALIVE** | Used by `state/billing.rs` to send `LoadBillingData` and `LoadPatientWorkspaceData` via `command_tx` |
| `command_rx` | **DEAD** | `take_command_rx()` never called — channel created but never consumed |
| `PendingBillingSaveData` | **DEAD** | Field in `PatientWorkspace` is always `None` — `set_pending_billing()` never called anywhere |
| `RetryOperation` | **ALIVE** | Defined in dead `ui/app.rs`, used by `global.rs:28` (DialogContent) and `main.rs:1002-1015` |
| `AppContextMenuAction` | **ALIVE** | Defined in dead `ui/app.rs`, used by `global.rs:7,25` and `main.rs:1041-1052` |
| `PendingClinicalSaveData` | **DEAD** | Defined in dead `ui/app.rs`, only used by dead `event_handler.rs` match arms |
| `PendingPatientData` | **ALIVE** | Defined in dead `ui/app.rs`, used by `main.rs:321,326,329,783,790` — `spawn_patient_save` |
| `PendingRescheduleData` | **DEAD** | Only used by dead `event_handler/appointment.rs` |
| `AppointmentStatusTransition` | **DEAD** | Only used by dead code |
| `current_context: KeyContext` | **MIGRATION DEBT** | Lives in `AppState`, used by `main.rs:710, 575, 1186` — NOT replaced by rat-focus yet |

**`PendingBillingSaveData` disconnect:**
- Type defined in dead `ui/app.rs`
- Field `pending_billing: Option<PendingBillingSaveData>` exists in `PatientWorkspace` (live)
- But `set_pending_billing()` is never called — field always `None`
- **Action**: delete the field from `PatientWorkspace` too

---

## Work Objectives

### Core Objective
Remove all code related to the old `struct App` without changing any application behavior.

### Concrete Deliverables

**Type migration (before deletion):**
- Move `RetryOperation` → `crates/opengp-ui/src/ui/app/global.rs` (already imported there from `ui/app.rs`, define permanently)
- Move `AppContextMenuAction` → same location as `RetryOperation`
- Move `PendingPatientData` → `crates/opengp-ui/src/ui/app/event.rs` (used by `main.rs` `spawn_patient_save`)

**Field deletion:**
- Remove `pending_billing: Option<PendingBillingSaveData>` from `PatientWorkspace` (dead field, always None)
- Remove `command_tx`/`command_rx`/`take_command_rx()` from `AppState` (channel dead, `AppCommand` still alive but channel-based approach abandoned)
- Remove `current_context: KeyContext` from `AppState` (migration debt — used by main.rs but should be replaced in future)

**Deletion (16 files, ~5,300 lines):**
- `crates/opengp-ui/src/ui/app.rs` — delete `struct App`, `impl App`, `PendingBillingSaveData`, `PendingClinicalSaveData`, `PendingRescheduleData`, `AppointmentStatusTransition`, keep `ClinicalWorkspaceLoadResult`, `ApiTaskError`
- `crates/opengp-ui/src/ui/app/event_handler.rs` — delete
- `crates/opengp-ui/src/ui/app/event_handler/global.rs` — delete
- `crates/opengp-ui/src/ui/app/event_handler/appointment.rs` — delete
- `crates/opengp-ui/src/ui/app/event_handler/workspace_tests.rs` — delete
- `crates/opengp-ui/src/ui/app/state.rs` — delete
- `crates/opengp-ui/src/ui/app/state/appointment.rs` — delete
- `crates/opengp-ui/src/ui/app/state/billing.rs` — delete
- `crates/opengp-ui/src/ui/app/state/clinical.rs` — delete
- `crates/opengp-ui/src/ui/app/state/patient.rs` — delete
- `crates/opengp-ui/src/ui/app/state/api_polling.rs` — delete (empty)
- `crates/opengp-ui/src/ui/app/keybinds/mod.rs` — delete
- `crates/opengp-ui/src/ui/app/keybinds/appointment.rs` — delete
- `crates/opengp-ui/src/ui/app/keybinds/billing.rs` — delete
- `crates/opengp-ui/src/ui/app/keybinds/clinical.rs` — delete
- `crates/opengp-ui/src/ui/app/keybinds/patient.rs` — delete
- `crates/opengp-ui/src/ui/app/renderer.rs` — delete

**Audit/Reduce:**
- `crates/opengp-ui/src/ui/keybinds.rs` — keep `KeyContext`, `Action`, `KeybindRegistry` types + trait; delete action lookup tables and context-specific binding implementations

**Export cleanup:**
- `crates/opengp-ui/src/lib.rs:25` — remove `pub use ui::app::App`
- `crates/opengp-ui/src/ui/mod.rs:22` — remove `pub use app::App`

### Definition of Done
- [ ] `cargo build --release` succeeds with zero warnings from deleted code
- [ ] `cargo test` passes (all crates)
- [ ] No `impl App` anywhere in the codebase
- [ ] No `handle_key_event` or `handle_global_mouse_event` anywhere
- [ ] No references to `struct App`
- [ ] All remaining types have clear home in live code

### Must Have
- App behavior unchanged — all workflows function identically
- All existing tests pass
- `cargo build --release` clean

### Must NOT Have (Guardrails)
- Do NOT touch `src/main.rs` — the live code
- Do NOT touch `crates/opengp-ui/src/ui/app/global.rs` structure — only add migrated types
- Do NOT touch `crates/opengp-ui/src/ui/app/app_state.rs` structure — only remove migration-debt fields
- Do NOT touch `crates/opengp-ui/src/ui/app/event.rs` — live AppEvent
- Do NOT touch `crates/opengp-ui/src/ui/app/error.rs` — live AppError
- Do NOT touch any component files — only deletion of extension/impl files
- Do NOT touch `crates/opengp-ui/src/ui/keybinds.rs` beyond the audit scope (keep live types, delete dead tables)
- Do NOT rename anything — pure deletion only

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: YES
- **Automated tests**: Tests-after (verify all pass after each phase gate)
- **Framework**: `cargo test`, `cargo build --release`
- **QA**: Agent-executed build + test at each phase gate

### QA Policy
Every phase gate is: `cargo build --release && cargo test -p opengp-ui && cargo test` (workspace). Evidence captured as command output.

---

## Execution Strategy

### Sequential Phases

```
Phase 1 — Type migration (before any deletion):
├── Task 1:  Move RetryOperation + AppContextMenuAction to global.rs
├── Task 2:  Move PendingPatientData to app/event.rs
├── Task 3:  Remove pending_billing field from PatientWorkspace (dead field)
└── Task 4:  Verify: cargo build --release

Phase 2 — Delete dead App extension files:
├── Task 5:  Delete ui/app/event_handler/ (4 files) + ui/app/event_handler.rs
├── Task 6:  Delete ui/app/state/ (5 files) + ui/app/state.rs + ui/app/state/api_polling.rs (empty)
├── Task 7:  Delete ui/app/keybinds/ (5 files)
├── Task 8:  Delete ui/app/renderer.rs
└── Task 9:  Verify: cargo build --release

Phase 3 — Delete struct App and types:
├── Task 10: Delete ui/app.rs body — keep ClinicalWorkspaceLoadResult, ApiTaskError; delete everything else
└── Task 11: Verify: cargo build --release

Phase 4 — Export + migration debt cleanup:
├── Task 12: Remove command_tx, command_rx from AppState (channel is dead)
├── Task 13: Remove current_context from AppState (migration debt)
├── Task 14: Remove pub use ui::app::App from lib.rs and ui/mod.rs
└── Task 15: Verify: cargo build --release + cargo test

Phase 5 — Audit keybinds.rs:
├── Task 16: Audit ui/keybinds.rs — keep KeyContext/Action/KeybindRegistry types; delete dead lookup tables
└── Task 17: Verify: cargo build --release + cargo test

Phase FINAL — Integration gate:
└── Task F1: Full build + test gate + no impl App anywhere
```

### Dependency Matrix (updated)
- **1,2**: None — can start immediately
- **3**: None — can start after initial verification (depends on 1, 2 completing, but parallelizable if independent)
- **4**: 3
- **5**: 4
- **6**: 5
- **7**: 3, 4, 5, 6
- **8**: 7
- **9**: 8
- **10**: 9
- **11**: 10
- **12**: 9, 10, 11
- **13**: 12
- **14**: 12 (independent of 13)
- **15**: 13, 14
- **F1, F2**: 15

---

## TODOs

- [x] 1. Migrate `RetryOperation` and `AppContextMenuAction` to `global.rs`

  **What to do**:
  - Copy `RetryOperation` enum (4 variants: Login, RefreshPatients, RefreshAppointments, RefreshConsultations) from `crates/opengp-ui/src/ui/app.rs:207-212` to `crates/opengp-ui/src/ui/app/global.rs`
  - Copy `AppContextMenuAction` enum (8 variants: PatientEdit, PatientDelete, PatientViewHistory, AppointmentEdit, AppointmentCancel, AppointmentReschedule, ClinicalEdit, ClinicalDelete, BillingEdit, BillingViewInvoice) from `crates/opengp-ui/src/ui/app.rs:225-240` to `global.rs`
  - Add `use` statements in `global.rs` for any types these enums reference (e.g., `uuid::Uuid`)
  - Update `pub use` in `crates/opengp-ui/src/ui/app.rs` to re-export the types from `global.rs` (as `pub use global::RetryOperation; pub use global::AppContextMenuAction;`) — this keeps existing imports in `src/main.rs` working during transition

  **Must NOT do**:
  - Do NOT change any enum variant names — keep all variants identical
  - Do NOT change `global.rs` DialogContent enum

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES — independent of Tasks 2, 3
  - **Parallel Group**: Phase 1 (with Tasks 2, 3)
  - **Blocks**: Task 4 (verification gate)
  - **Blocked By**: None

  **References**:
  - `crates/opengp-ui/src/ui/app.rs:207-212` — `RetryOperation` enum definition
  - `crates/opengp-ui/src/ui/app.rs:225-240` — `AppContextMenuAction` enum definition
  - `crates/opengp-ui/src/ui/app/global.rs:1-27` — current imports + DialogContent enum
  - `src/main.rs:1002-1015` — RetryOperation usage
  - `src/main.rs:1041-1052` — AppContextMenuAction usage

  **Acceptance Criteria**:
  - [ ] `pub enum RetryOperation` present in `global.rs`
  - [ ] `pub enum AppContextMenuAction` present in `global.rs`
  - [ ] `cargo build --release` exits 0 — all existing imports still resolve

  **QA Scenarios**:
  ```
  Scenario: Migrated types resolve for all consumers
    Tool: Bash
    Preconditions: Types moved to global.rs, re-exported from app.rs
    Steps:
      1. cargo build --release 2>&1
      2. Assert: exit code 0, no "cannot find type" errors
    Evidence: .sisyphus/evidence/task-1-build.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): migrate RetryOperation and AppContextMenuAction to global.rs`
  - Files: `crates/opengp-ui/src/ui/app/global.rs`, `crates/opengp-ui/src/ui/app.rs`

---

- [x] 2. Delete `pending_billing` field from `PatientWorkspace`

  **What to do**:
  - Remove `pub pending_billing: Option<PendingBillingSaveData>` field from `PatientWorkspace` in `crates/opengp-ui/src/ui/components/workspace/workspace.rs:61`
  - Remove the initialization `pending_billing: None` in `PatientWorkspace::new()` at line 95
  - Remove the `use crate::ui::app::PendingBillingSaveData;` import from workspace.rs (line 2)
  - Check if `PatientWorkspace::clone()` references `pending_billing` — if auto-derived `#[derive(Clone)]` doesn't need it, no change needed

  **Why this is safe**: `set_pending_billing()` is never called anywhere. The field is always `None`. No code reads it. Verified by explore agent audit.

  **Must NOT do**:
  - Do NOT remove the `PendingBillingSaveData` enum definition from `app.rs` yet — handled in Task 10

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES — independent of Tasks 1, 3
  - **Parallel Group**: Phase 1 (with Tasks 1, 3)
  - **Blocks**: Task 4 (verification gate)
  - **Blocked By**: None

  **References**:
  - `crates/opengp-ui/src/ui/components/workspace/workspace.rs:2` — import to remove
  - `crates/opengp-ui/src/ui/components/workspace/workspace.rs:61` — field declaration
  - `crates/opengp-ui/src/ui/components/workspace/workspace.rs:95` — initialization

  **Acceptance Criteria**:
  - [ ] `pending_billing` not present in `PatientWorkspace`
  - [ ] No import of `PendingBillingSaveData` in `workspace.rs`
  - [ ] `cargo build --release` exits 0

  **QA Scenarios**:
  ```
  Scenario: Workspace compiles without pending_billing
    Tool: Bash
    Steps:
      1. cargo check -p opengp-ui 2>&1
      2. Assert: exit code 0
      3. grep -n "pending_billing" crates/opengp-ui/src/ui/components/workspace/workspace.rs
      4. Assert: no matches
    Evidence: .sisyphus/evidence/task-2-pending-billing.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): remove dead pending_billing field from PatientWorkspace`
  - Files: `crates/opengp-ui/src/ui/components/workspace/workspace.rs`

---

- [x] 3. Delete `ui/app/event_handler/` (4 files) + `ui/app/event_handler.rs`

  **What to do**:
  - Delete files:
    - `crates/opengp-ui/src/ui/app/event_handler/global.rs`
    - `crates/opengp-ui/src/ui/app/event_handler/appointment.rs`
    - `crates/opengp-ui/src/ui/app/event_handler/workspace_tests.rs`
    - `crates/opengp-ui/src/ui/app/event_handler.rs`
  - After deletion, the empty `event_handler/` directory should be removed (git won't track empty dirs)

  **Must NOT do**:
  - Do NOT touch `crates/opengp-ui/src/ui/app/event.rs` — live AppEvent
  - Do NOT touch any other file

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential after Task 4 verify
  - **Parallel Group**: Phase 2 (with Tasks 4, 5, 6)
  - **Blocks**: Task 7 (next verify gate)
  - **Blocked By**: Task 4

  **References**:
  - `crates/opengp-ui/src/ui/app/event_handler.rs` — the main handler (506 lines, dead)
  - `crates/opengp-ui/src/ui/app/event_handler/` — subdirectory (410+168+285 lines, dead)

  **Acceptance Criteria**:
  - [ ] All 4 event_handler files deleted
  - [ ] `cargo build --release` exits 0
  - [ ] `cargo test -p opengp-ui` exits 0

  **QA Scenarios**:
  ```
  Scenario: event_handler files gone
    Tool: Bash
    Steps:
      1. ls crates/opengp-ui/src/ui/app/event_handler/ 2>&1
      2. Assert: directory empty or doesn't exist
      3. ls crates/opengp-ui/src/ui/app/event_handler.rs 2>&1
      4. Assert: file doesn't exist
    Evidence: .sisyphus/evidence/task-3-event-handler-gone.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): delete dead event_handler impl files`
  - Files: all 4 deleted files

---

- [x] 4. Delete `ui/app/state/` (5 files) + `ui/app/state.rs` + `ui/app/state/api_polling.rs` (empty)

  **What to do**:
  - Delete files:
    - `crates/opengp-ui/src/ui/app/state/appointment.rs`
    - `crates/opengp-ui/src/ui/app/state/billing.rs`
    - `crates/opengp-ui/src/ui/app/state/clinical.rs`
    - `crates/opengp-ui/src/ui/app/state/patient.rs`
    - `crates/opengp-ui/src/ui/app/state/api_polling.rs` (empty file)
    - `crates/opengp-ui/src/ui/app/state.rs`
  - Remove `mod state;` from `crates/opengp-ui/src/ui/app.rs`

  **WARNING**: `state/billing.rs` contains the `set_pending_billing()` method that was already verified as never called. Deleting this file is safe.

  **Must NOT do**:
  - Do NOT touch any live files

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential after Task 3
  - **Parallel Group**: Phase 2 (with Tasks 3, 5, 6)
  - **Blocks**: Task 7 (verify gate)
  - **Blocked By**: Task 3

  **References**:
  - `crates/opengp-ui/src/ui/app/state/` — all files are `impl App` extension methods, all dead

  **Acceptance Criteria**:
  - [ ] All 6 state files deleted
  - [ ] `mod state;` removed from `app.rs`
  - [ ] `cargo build --release` exits 0

  **QA Scenarios**:
  ```
  Scenario: state files gone
    Tool: Bash
    Steps:
      1. ls crates/opengp-ui/src/ui/app/state/ 2>&1
      2. Assert: no .rs files exist (directory empty or missing)
    Evidence: .sisyphus/evidence/task-4-state-gone.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): delete dead state impl files`
  - Files: all 6 deleted files + `app.rs` module declaration

- [x] 5. Delete `ui/app/keybinds/` (5 files) + remove `mod keybinds;` from `app.rs`

  **What to do**:
  - Delete files:
    - `crates/opengp-ui/src/ui/app/keybinds/mod.rs`
    - `crates/opengp-ui/src/ui/app/keybinds/appointment.rs`
    - `crates/opengp-ui/src/ui/app/keybinds/billing.rs`
    - `crates/opengp-ui/src/ui/app/keybinds/clinical.rs`
    - `crates/opengp-ui/src/ui/app/keybinds/patient.rs`
  - Remove `mod keybinds;` from `crates/opengp-ui/src/ui/app.rs`
  - **CRITICAL**: `ui/app.rs` currently has `mod keybinds;` per the module tree. After removing, confirm no `pub use keybinds::*` re-exports remain.

  **Must NOT do**:
  - Do NOT touch `crates/opengp-ui/src/ui/keybinds.rs` — that's the separate file handled in Task 16

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential after Task 4
  - **Parallel Group**: Phase 2 (with Tasks 3, 4, 6)
  - **Blocks**: Task 7
  - **Blocked By**: Task 4

  **References**:
  - `crates/opengp-ui/src/ui/app/keybinds/` — all 5 files are dead `impl App` keybind dispatch

  **Acceptance Criteria**:
  - [ ] All 5 keybinds files deleted
  - [ ] `mod keybinds;` removed from `app.rs`
  - [ ] `cargo build --release` exits 0

  **QA Scenarios**:
  ```
  Scenario: keybinds subdirectory gone
    Tool: Bash
    Steps:
      1. ls crates/opengp-ui/src/ui/app/keybinds/ 2>&1
      2. Assert: no .rs files exist
    Evidence: .sisyphus/evidence/task-5-keybinds-gone.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): delete dead keybinds impl files`
  - Files: 5 deleted files + `app.rs`

---

- [x] 6. Delete `ui/app/renderer.rs`

  **What to do**:
  - Delete file `crates/opengp-ui/src/ui/app/renderer.rs`
  - Remove `mod renderer;` from `crates/opengp-ui/src/ui/app.rs`
  - This file contains `impl App { render }` — 583 lines, completely dead. `main.rs` has its own `render()` function.

  **Must NOT do**:
  - Do NOT touch `src/main.rs` render function

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential after Task 5
  - **Parallel Group**: Phase 2 (with Tasks 3, 4, 5)
  - **Blocks**: Task 7
  - **Blocked By**: Task 5

  **References**:
  - `crates/opengp-ui/src/ui/app/renderer.rs` — 583 lines, dead

  **Acceptance Criteria**:
  - [ ] `renderer.rs` deleted
  - [ ] `mod renderer;` removed from `app.rs`
  - [ ] `cargo build --release` exits 0

  **QA Scenarios**:
  ```
  Scenario: renderer.rs gone
    Tool: Bash
    Steps:
      1. ls crates/opengp-ui/src/ui/app/renderer.rs 2>&1
      2. Assert: file doesn't exist
    Evidence: .sisyphus/evidence/task-6-renderer-gone.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): delete dead renderer impl`
  - Files: deleted file + `app.rs`

---

- [x] 7. Verify Phase 2 build gate

  **What to do**:
  - Run `cargo build --release` — must compile cleanly
  - Run `cargo test -p opengp-ui` — all tests must pass
  - Run `cargo test` — full workspace tests must pass
  - Confirm: `grep -r "^impl App" crates/opengp-ui/src --include="*.rs" | wc -l` — expected: matches only in `ui/app.rs` (the App struct being deleted in Task 8)
  - If any compilation errors, fix them (missing imports, removed modules referenced, etc.)

  **Must NOT do**:
  - Do NOT proceed to Phase 3 if build fails

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential gate
  - **Parallel Group**: Gate between Phase 2 and Phase 3
  - **Blocks**: Tasks 8, 9
  - **Blocked By**: Tasks 3, 4, 5, 6

  **Acceptance Criteria**:
  - [ ] `cargo build --release` exits 0
  - [ ] `cargo test -p opengp-ui` exits 0
  - [ ] `cargo test` exits 0

  **QA Scenarios**:
  ```
  Scenario: Full build + test pass
    Tool: Bash
    Steps:
      1. cargo build --release 2>&1 | tail -5
      2. Assert: "Finished" with no errors
      3. cargo test 2>&1 | tail -20
      4. Assert: "test result: ok" with 0 failures
    Evidence: .sisyphus/evidence/task-7-phase2-gate.txt
  ```

  **Commit**: NO — verification only

---

- [x] 8. Delete `struct App` and all dead types from `ui/app.rs`

  **What to do**:
  - In `crates/opengp-ui/src/ui/app.rs`, delete:
    - `struct App` (lines 82-255) — the entire struct with all fields
    - `impl App` blocks (lines 265-560) — the main `impl` containing `handle_key_event`, `render`, and related methods
    - `impl Default for App` (lines 566-575)
    - Test functions that call `app.handle_key_event()` (lines 620-1016)
    - `PendingClinicalSaveData` enum (lines 148-187)
    - `PendingBillingSaveData` enum (lines 189-199)
    - `PendingRescheduleData` struct (lines 215-223)
    - `AppointmentStatusTransition` enum (lines 202-205)
    - `ActiveContextMenu` enum (lines 244-255)
    - `pub use command::AppCommand;` re-export (line 34) — only if AppCommand is fully dead
    - `mod event_handler;` module declaration — removed in Task 3
    - `mod state;` module declaration — removed in Task 4
    - `mod keybinds;` module declaration — removed in Task 5
    - `mod renderer;` module declaration — removed in Task 6
  - **KEEP**: `ClinicalWorkspaceLoadResult` struct (lines 44-58) — used by `event.rs`
  - **KEEP**: `ApiTaskError` enum (lines 60-80) and its `impl` block
  - **KEEP**: `pub use` re-exports for live modules: `AppState`, `AppError`, `AppEvent`, `DialogContent`, `GlobalState`
  - After deletion, `ui/app.rs` should be ~120 lines (imports + ClinicalWorkspaceLoadResult + ApiTaskError + mod/puse declarations)

  **Must NOT do**:
  - Do NOT delete `ClinicalWorkspaceLoadResult` or `ApiTaskError`
  - Do NOT delete `pub use app_state::AppState;` and similar re-exports
  - Do NOT delete `AppContextMenuAction` or `RetryOperation` enums — they were moved to global.rs in Task 1

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential after Task 7
  - **Parallel Group**: Phase 3
  - **Blocks**: Task 9
  - **Blocked By**: Task 7

  **References**:
  - `crates/opengp-ui/src/ui/app.rs:82-575` — App struct + main impl (what to delete)
  - `crates/opengp-ui/src/ui/app.rs:44-80` — ClinicalWorkspaceLoadResult + ApiTaskError (what to keep)
  - `crates/opengp-ui/src/ui/app.rs:1-40` — imports + pub use (what to update)
  - `crates/opengp-ui/src/ui/app/event.rs:9` — imports ClinicalWorkspaceLoadResult

  **Acceptance Criteria**:
  - [ ] No `struct App` or `impl App` in `ui/app.rs`
  - [ ] No `PendingBillingSaveData`, `PendingClinicalSaveData`, `PendingRescheduleData`, `AppointmentStatusTransition`, `ActiveContextMenu` in `ui/app.rs`
  - [ ] `ClinicalWorkspaceLoadResult` and `ApiTaskError` still present
  - [ ] `pub use app_state::AppState;` etc. still present
  - [ ] `cargo build --release` exits 0

  **QA Scenarios**:
  ```
  Scenario: struct App gone from app.rs
    Tool: Bash
    Steps:
      1. grep -n "struct App\|impl App" crates/opengp-ui/src/ui/app.rs
      2. Assert: no matches
      3. cargo build --release 2>&1
      4. Assert: exit code 0
    Evidence: .sisyphus/evidence/task-8-app-gone.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): delete struct App and dead types from app.rs`
  - Files: `crates/opengp-ui/src/ui/app.rs`

---

- [x] 9. Remove `command_tx`, `command_rx` from AppState

  **What to do**:
  - In `crates/opengp-ui/src/ui/app/app_state.rs`, remove:
    - `pub command_tx: tokio::sync::mpsc::UnboundedSender<crate::ui::app::AppCommand>` (line 50)
    - `pub command_rx: Option<tokio::sync::mpsc::UnboundedReceiver<crate::ui::app::AppCommand>>` (line 51)
  - In `src/main.rs`:
    - Remove `let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel::<AppCommand>();` (line 102)
    - Remove `command_tx,` and `command_rx: Some(command_rx),` from `AppState` construction (lines 124-125)
    - Remove `use opengp_ui::ui::app::AppCommand` from imports (line 25) — if this breaks, check if AppCommand was also used elsewhere in main.rs
  - Verify: `grep -rn "command_tx\|command_rx" src/main.rs` should return no matches

  **Why this is safe**: `command_rx` is never consumed — `take_command_rx()` is never called. The channel approach was abandoned in favor of `spawn_async`.

  **Must NOT do**:
  - Do NOT remove `command_tx` if AppCommand is used elsewhere outside the channel (verify first)

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential after Task 8
  - **Parallel Group**: Phase 4 (with Tasks 10, 11)
  - **Blocks**: Task 12
  - **Blocked By**: Task 8

  **References**:
  - `crates/opengp-ui/src/ui/app/app_state.rs:50-51` — fields to remove
  - `src/main.rs:102,124-125` — channel creation + AppState construction
  - `src/main.rs:25` — import to update

  **Acceptance Criteria**:
  - [ ] No `command_tx` or `command_rx` in `app_state.rs`
  - [ ] No `command_tx` or `command_rx` in `src/main.rs`
  - [ ] `cargo build --release` exits 0

  **QA Scenarios**:
  ```
  Scenario: command channels removed
    Tool: Bash
    Steps:
      1. grep -rn "command_tx\|command_rx" crates/opengp-ui/src/ src/ --include="*.rs" 2>/dev/null
      2. Assert: no matches (or only in comment/doc)
      3. cargo build --release 2>&1
      4. Assert: exit code 0
    Evidence: .sisyphus/evidence/task-9-command-channel.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): remove dead command channel from AppState`
  - Files: `crates/opengp-ui/src/ui/app/app_state.rs`, `src/main.rs`

---

- [x] 10. Replace `current_context` with rat-focus context tracking, then remove from AppState

  **What to do**:
  - **Step A — Understand how it's used now:**
    - `main.rs:710` — sets `state.current_context` based on authentication and tab/focus state
    - `main.rs:575` — passes `state.current_context` to `help_overlay.set_context()`
    - `main.rs:1186` — calls `keybinds.lookup(*key, state.current_context)` for per-context key dispatch
  - **Step B — Build context from rat-focus state:**
    - In `main.rs`, add a function `current_key_context(state: &AppState) -> KeyContext` that derives the current context from the dialog stack and widget focus state (replicating the logic currently at line 710)
    - Example:
      ```rust
      fn current_key_context(state: &AppState) -> KeyContext {
          if !state.authenticated { return KeyContext::Login; }
          // Check dialog stack for active forms/modals
          // Check focused widget via Focus
          // Map to appropriate KeyContext
          state.tab_bar.selected().key_context()
      }
      ```
  - **Step C — Replace mutable field with derived value:**
    - Remove `pub current_context: KeyContext` from `AppState`
    - Remove initialization `current_context: KeyContext::Global` from `src/main.rs:110`
    - Replace `state.current_context` reads at lines 575 and 1186 with calls to `current_key_context(state)`
    - Remove the assignment at line 710 (no longer needed — derived on read)
  - **Step D — Verify:**
    - `cargo build --release` exits 0
    - `cargo test -p opengp-ui` exits 0
    - `grep -rn "current_context" crates/opengp-ui/src/ src/ --include="*.rs" | grep -v "//\|#" | wc -l` → 0 (or only in comment)

  **Must NOT do**:
  - Do NOT delete `KeyContext` from the repo — it's still used by keybind lookup
  - Do NOT break keybind functionality — `keybinds.lookup()` must still receive correct context

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: [`rust-best-practices`]

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential after Task 9
  - **Parallel Group**: Phase 4 (with Tasks 9, 11)
  - **Blocks**: Task 12
  - **Blocked By**: Task 9

  **References**:
  - `crates/opengp-ui/src/ui/app/app_state.rs:22` — field to remove
  - `src/main.rs:110` — initialization
  - `src/main.rs:575,710,1186` — all usages
  - `src/main.rs:684-694` — `build_focus(state)` for rat-focus context
  - `crates/opengp-ui/src/ui/keybinds.rs` — `KeyContext` enum variants

  **Acceptance Criteria**:
  - [ ] `current_context` removed from `AppState`
  - [ ] No mutable `state.current_context` assignments in `main.rs`
  - [ ] `current_key_context()` function derives context from dialog stack + focus state
  - [ ] `keybinds.lookup()` still works with derived context
  - [ ] `cargo build --release` exits 0
  - [ ] `cargo test` exits 0

  **QA Scenarios**:
  ```
  Scenario: No mutable current_context field
    Tool: Bash
    Steps:
      1. grep -rn "current_context" crates/opengp-ui/src/ src/ --include="*.rs" 2>/dev/null
      2. Assert: no matches (or only in function name current_key_context)
      3. cargo build --release 2>&1
      4. Assert: exit code 0
    Evidence: .sisyphus/evidence/task-10-current-context.txt
  ```

  **Commit**: YES
  - Message: `refactor(opengp-ui): replace mutable current_context with rat-focus derived context`
  - Files: `crates/opengp-ui/src/ui/app/app_state.rs`, `src/main.rs`

---

- [x] 11. Remove `pub use ui::app::App` from public exports

  **What to do**:
  - In `crates/opengp-ui/src/lib.rs`: remove line 25 (`pub use ui::app::App;`)
  - In `crates/opengp-ui/src/ui/mod.rs`: remove line 22 (`pub use app::App;`)
  - Verify no external consumers break: `grep -rn "opengp_ui::App\b" . --include="*.rs"` should return no matches outside opengp-ui itself

  **Must NOT do**:
  - Do NOT remove any other `pub use` items — only the `App` re-export

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential after Task 10
  - **Parallel Group**: Phase 4 (with Tasks 9, 10)
  - **Blocks**: Task 12
  - **Blocked By**: Task 10

  **References**:
  - `crates/opengp-ui/src/lib.rs:25` — `pub use ui::app::App;`
  - `crates/opengp-ui/src/ui/mod.rs:22` — `pub use app::App;`

  **Acceptance Criteria**:
  - [ ] No `pub use ... App;` in lib.rs or mod.rs
  - [ ] `cargo build --release` exits 0

  **QA Scenarios**:
  ```
  Scenario: App export removed
    Tool: Bash
    Steps:
      1. grep -n "pub use.*App\b" crates/opengp-ui/src/lib.rs crates/opengp-ui/src/ui/mod.rs
      2. Assert: no matches
      3. cargo build --release 2>&1
      4. Assert: exit code 0
    Evidence: .sisyphus/evidence/task-11-export-gone.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): remove dead App re-export from public API`
  - Files: `crates/opengp-ui/src/lib.rs`, `crates/opengp-ui/src/ui/mod.rs`

---

- [x] 12. Verify Phase 4 build gate

  **What to do**:
  - `cargo build --release` — must compile cleanly
  - `cargo test -p opengp-ui` — all tests pass
  - `cargo test` — full workspace
  - Confirm: `grep -r "^impl App" crates/opengp-ui/src --include="*.rs" | wc -l` → 0

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential gate
  - **Blocks**: Tasks 13, 14
  - **Blocked By**: Tasks 9, 10, 11

  **Acceptance Criteria**:
  - [ ] `cargo build --release` exits 0
  - [ ] `cargo test` exits 0
  - [ ] Zero `impl App` anywhere

  **QA Scenarios**:
  ```
  Scenario: Full build + test pass, no impl App
    Tool: Bash
    Steps:
      1. cargo build --release 2>&1 | tail -3
      2. Assert: exit code 0
      3. cargo test 2>&1 | tail -3
      4. Assert: pass
      5. grep -r "^impl App" crates/opengp-ui/src --include="*.rs" | wc -l
      6. Assert: 0
    Evidence: .sisyphus/evidence/task-12-phase4-gate.txt
  ```

  **Commit**: NO

---

- [x] 13. Delete `ui/app/command.rs` (if AppCommand is dead)

  **What to do**:
  - After removing `command_tx`/`command_rx` from AppState (Task 9), AppCommand is now unreferenced
  - Verify: `grep -rn "AppCommand" crates/opengp-ui/src/ src/ --include="*.rs" | grep -v "command.rs"` should return no matches
  - If confirmed dead: delete `crates/opengp-ui/src/ui/app/command.rs`
  - Remove `mod command;` from `crates/opengp-ui/src/ui/app.rs`
  - Remove `pub use command::AppCommand;` from `crates/opengp-ui/src/ui/app.rs` (line 34)
  - **If AppCommand still has live references**: skip deletion (leave for separate PR)

  **Must NOT do**:
  - Do NOT delete if any file outside `command.rs` references `AppCommand`

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential after Task 12
  - **Parallel Group**: Phase 5 (with Task 14)
  - **Blocks**: Task 15
  - **Blocked By**: Task 12

  **References**:
  - `crates/opengp-ui/src/ui/app/command.rs` — AppCommand enum

  **Acceptance Criteria**:
  - [ ] `command.rs` deleted OR confirmed to have live references (skip)
  - [ ] `cargo build --release` exits 0

  **QA Scenarios**:
  ```
  Scenario: AppCommand references zero
    Tool: Bash
    Steps:
      1. grep -rn "AppCommand" crates/opengp-ui/src/ src/ --include="*.rs" | grep -v "command.rs"
      2. Assert: no matches
    Evidence: .sisyphus/evidence/task-13-appcommand.txt
  ```

  **Commit**: YES (or NO if skipped)
  - Message: `chore(opengp-ui): delete dead AppCommand enum`
  - Files: `crates/opengp-ui/src/ui/app/command.rs`, `crates/opengp-ui/src/ui/app.rs`

---

- [x] 14. Audit `ui/keybinds.rs` — delete dead lookup tables

  **What to do**:
  - In `crates/opengp-ui/src/ui/keybinds.rs` (1208 lines):
  - **KEEP**: `KeyContext` enum (all variants)
  - **KEEP**: `Action` enum (all variants)
  - **KEEP**: `KeybindRegistry` struct (public fields `bindings`, `global_bindings`)
  - **KEEP**: `KeybindRegistry::global()` — accessed via `LazyLock`
  - **KEEP**: `KeybindRegistry::lookup()` method — used by `main.rs:1186`
  - **DELETE**: All context-specific action binding tables (the large blocks that populate `bindings` HashMap per KeyContext variant)
  - **DELETE**: The `LazyLock` initialization that builds the full multimap — replace with empty maps or a minimal registry
  - **AFTER**: Verify `cargo build --release` compiles — `KeybindRegistry.lookup()` must still exist
  - **AFTER**: Verify all consumers still import types they need: `appointment/calendar.rs`, `appointment/state.rs`, `help.rs`, `tabs.rs`, `global.rs`, `app_state.rs`

  **Must NOT do**:
  - Do NOT delete `KeyContext`, `Action`, or `KeybindRegistry` types
  - Do NOT delete `KeybindRegistry::lookup()` method
  - Do NOT rename anything

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES — independent of Task 13
  - **Parallel Group**: Phase 5 (with Task 13)
  - **Blocks**: Task 15
  - **Blocked By**: Task 12

  **References**:
  - `crates/opengp-ui/src/ui/keybinds.rs` — full file (1208 lines)
  - `src/main.rs:1186` — `.lookup(*key, state.current_context)` — must survive
  - `crates/opengp-ui/src/ui/components/appointment/calendar.rs:10` — imports Action, KeyContext, KeybindRegistry

  **Acceptance Criteria**:
  - [ ] `KeyContext`, `Action`, `KeybindRegistry` types still present
  - [ ] `KeybindRegistry::global()` and `KeybindRegistry::lookup()` still present
  - [ ] Dead lookup tables removed
  - [ ] `keybinds.rs` lines reduced significantly (from 1208 to ~200-300)
  - [ ] `cargo build --release` exits 0

  **QA Scenarios**:
  ```
  Scenario: keybinds.rs live types survive audit
    Tool: Bash
    Steps:
      1. grep "pub enum KeyContext\|pub enum Action\|pub struct KeybindRegistry" crates/opengp-ui/src/ui/keybinds.rs
      2. Assert: all three match
      3. grep "fn lookup" crates/opengp-ui/src/ui/keybinds.rs
      4. Assert: match found
      5. cargo build --release 2>&1
      6. Assert: exit code 0
    Evidence: .sisyphus/evidence/task-14-keybinds-audit.txt
  ```

  **Commit**: YES
  - Message: `chore(opengp-ui): remove dead keybind lookup tables from keybinds.rs`
  - Files: `crates/opengp-ui/src/ui/keybinds.rs`

---

- [x] 15. Verify Phase 5 build gate

  **What to do**:
  - `cargo build --release`
  - `cargo test -p opengp-ui`
  - `cargo test`
  - Final check: `grep -rn "handle_key_event\|handle_global_mouse_event" crates/opengp-ui/src/ src/ --include="*.rs" 2>/dev/null | grep -v "test\|#\[cfg(test)\]"` → 0

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — sequential gate before final
  - **Blocks**: Final task F1
  - **Blocked By**: Tasks 13, 14

  **Acceptance Criteria**:
  - [ ] Build passes
  - [ ] Tests pass
  - [ ] No dead method references

  **QA Scenarios**:
  ```
  Scenario: Final pre-F1 gate
    Tool: Bash
    Steps:
      1. cargo build --release 2>&1 | tail -3
      2. Assert: exit 0
      3. cargo test 2>&1 | tail -3
      4. Assert: pass
    Evidence: .sisyphus/evidence/task-15-phase5-gate.txt
  ```

  **Commit**: NO

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 2 agents run in PARALLEL. ALL must APPROVE. Rejection → fix → re-run.

- [x] F1. **Build + test + dead code audit** — `deep`

  Run ALL the following verifications. All must pass.

  ```bash
  # 1. Build
  cargo build --release 2>&1
  # Expected: exit 0, no errors, no warnings

  # 2. UI crate tests
  cargo test -p opengp-ui 2>&1
  # Expected: exit 0, no failures

  # 3. Full workspace tests
  cargo test 2>&1
  # Expected: exit 0, no failures

  # 4. No impl App anywhere
  grep -r "^impl App" crates/opengp-ui/src --include="*.rs" | wc -l
  # Expected: 0

  # 5. No struct App anywhere
  grep -r "^pub struct App\b" crates/opengp-ui/src --include="*.rs" | wc -l
  # Expected: 0

  # 6. No handle_key_event or handle_global_mouse_event in non-test code
  grep -rn "fn handle_key_event\|fn handle_global_mouse_event" crates/opengp-ui/src/ src/ --include="*.rs" 2>/dev/null | grep -v "test\|#\[cfg(test)\]" | wc -l
  # Expected: 0

  # 7. No dead files exist
  test -f crates/opengp-ui/src/ui/app/event_handler.rs && echo "FAIL: event_handler.rs exists" || true
  test -f crates/opengp-ui/src/ui/app/renderer.rs && echo "FAIL: renderer.rs exists" || true
  test -d crates/opengp-ui/src/ui/app/state && echo "FAIL: state/ exists" || true
  test -d crates/opengp-ui/src/ui/app/keybinds && echo "FAIL: keybinds/ exists" || true
  test -d crates/opengp-ui/src/ui/app/event_handler && echo "FAIL: event_handler/ exists" || true
  # Expected: no FAIL messages

  # 8. Live types in correct homes
  grep "pub enum RetryOperation" crates/opengp-ui/src/ui/app/global.rs && echo "OK: RetryOperation in global.rs" || echo "WARN: RetryOperation not found"
  grep "pub enum AppContextMenuAction" crates/opengp-ui/src/ui/app/global.rs && echo "OK: AppContextMenuAction in global.rs" || echo "WARN: AppContextMenuAction not found"

  # 9. src/main.rs unchanged (diff shows only removals of command_tx/rx/current_context)
  # Check that event_fn, render, init all still present:
  grep -c "fn event_fn\|fn render\|fn init\|fn handle_mouse_event\|fn build_focus" src/main.rs
  # Expected: 5+ function definitions remain
  ```

  **Output**: Pass/fail for each check. All must pass.

  **Completion signal**: `echo "ALL CHECKS PASSED"`

- [x] F2. **Scope fidelity check** — `deep`

  Read each file on the "deleted" list. Verify it no longer exists on disk. Read `crates/opengp-ui/src/ui/app.rs` — verify no `struct App`, no `impl App`, dead types gone, live types present. Read `crates/opengp-ui/src/ui/app/app_state.rs` — verify `command_tx`/`command_rx` gone. Read `crates/opengp-ui/src/ui/components/workspace/workspace.rs` — verify `pending_billing` gone. Read `crates/opengp-ui/src/lib.rs` and `ui/mod.rs` — verify `pub use ... App` gone.

  **Output**: `Scope [CLEAN / N issues] | Must-Have [N/N] | Must-NOT-Have [N/N] | VERDICT: APPROVE/REJECT`

---



---

## Commit Strategy

- **Task 1**: `chore(opengp-ui): migrate RetryOperation and AppContextMenuAction to global.rs` — global.rs, app.rs
- **Task 2**: `chore(opengp-ui): remove dead pending_billing field from PatientWorkspace` — workspace.rs
- **Task 3**: `chore(opengp-ui): delete dead event_handler impl files` — 4 files
- **Task 4**: `chore(opengp-ui): delete dead state impl files` — 6 files + app.rs
- **Task 5**: `chore(opengp-ui): delete dead keybinds impl files` — 5 files + app.rs
- **Task 6**: `chore(opengp-ui): delete dead renderer impl` — renderer.rs + app.rs
- **Task 7**: NO (verification gate)
- **Task 8**: `chore(opengp-ui): delete struct App and dead types from app.rs` — app.rs
- **Task 9**: `chore(opengp-ui): remove dead command channel from AppState` — app_state.rs, main.rs
- **Task 10**: `chore(opengp-ui): remove current_context migration debt from AppState` — app_state.rs, main.rs
- **Task 11**: `chore(opengp-ui): remove dead App re-export from public API` — lib.rs, mod.rs
- **Task 12**: NO (verification gate)
- **Task 13**: `chore(opengp-ui): delete dead AppCommand enum` — command.rs, app.rs (or skip if live)
- **Task 14**: `chore(opengp-ui): remove dead keybind lookup tables from keybinds.rs` — keybinds.rs
- **Task 15**: NO (verification gate)
- **F1, F2**: NO (verification only)

---

## Success Criteria

### Verification Commands
```bash
cargo build --release  # Expected: 0
cargo test -p opengp-ui # Expected: 0
cargo test              # Expected: 0
grep -r "^impl App" crates/opengp-ui/src --include="*.rs" | wc -l  # Expected: 0
grep -r "^pub struct App\b" crates/opengp-ui/src --include="*.rs" | wc -l  # Expected: 0
```

### Final Checklist
- [ ] `src/main.rs` unchanged — live stack untouched
- [ ] All `impl App` extension files deleted
- [ ] `struct App` deleted
- [ ] `pub use ui::app::App` removed from public exports
- [ ] `RetryOperation` / `AppContextMenuAction` live in `global.rs`
- [ ] `PendingPatientData` live in `event.rs`
- [ ] `PendingBillingSaveData` and `pending_billing` field removed
- [ ] `AppState` has no `command_tx`/`command_rx`/`current_context` migration debt
- [ ] `ui/keybinds.rs` reduced to live symbols only
- [ ] All tests pass