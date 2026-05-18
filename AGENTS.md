# AGENTS.md

## Product Context
- `structure` is a strength-training app for:
  - building long-term training plans (`Mesocycle`s)
  - tracking workouts through those plans
- Domain hierarchy: mesocycle -> microcycles -> workouts -> exercises -> sets.
- Workout tracking is primarily mobile.
- Plan creation should work on mobile and desktop; desktop matters for overview on larger screens.
- Training history, PR tracking, and history-based planning/auto-adjustment are intended features, not current guarantees.

## Repo Shape
- Rust + Flutter repo.
- Root `Cargo.toml` workspace members: `backend/`.
- `backend/` is the Rust domain/bridge crate used by Flutter.
- `frontend/` is the Flutter app.

## Entrypoints
- Flutter app entry: `frontend/lib/main.dart` (`RustLib.init()` runs before `runApp`).
- Main screen: `frontend/lib/screens/training_programs_screen.dart`.
- Rust bridge API: `backend/src/api.rs` (`pub mod mesocycles; pub mod microcycles;`).
- Core domain types: `backend/src/domain/planning.rs`.

## Generated Code
- `flutter_rust_bridge.yaml` is the bridge source of truth:
  - `rust_root: backend/`
  - `rust_input: crate::api`
  - `dart_output: frontend/lib/src/bridge/`
- Do not hand-edit generated files:
  - `backend/src/frb_generated.rs`
  - `frontend/lib/src/bridge/*`
- Change Rust source, then regenerate bridge outputs.

## Commands
- `flutter analyze` (from `frontend/`)
- `flutter test` (from `frontend/`)
- `./scripts/run_linux.sh`

## Gotchas
- `./scripts/run_linux.sh` uses `target/debug` via `LD_LIBRARY_PATH`.
- Generated Dart FRB loader points at `../backend/target/release/` in `frontend/lib/src/bridge/frb_generated.dart`; keep build mode and library path aligned.
- `frontend/test/widget_test.dart` is stale Flutter boilerplate, not a trustworthy app test.
- Prioritize `backend/` and `frontend/`, not `mesocycle_builder/`.
- SQLite foreign keys are declared with `REFERENCES` but NOT enforced unless `PRAGMA foreign_keys = ON` is set — will address when shared connection is introduced.
- Table creation order matters in `init_db`: parent tables before child tables (mesocycles before microcycles).
- `create_microcycle_table` is inconsistently named (should be `create_microcycles_table`) — not yet renamed.

## Architecture & Design Decisions
- **Domain/DTO/persistence/API layering**: domain types for business logic, DTOs for FRB transport, persistence works with domain types only, API orchestrates and converts domain → DTO at boundary.
- **FRB boundary**: keep generated code isolated in `frontend/lib/src/bridge/*`; keep `lib.rs` thin.
- **`pub(crate)` on domain constructors**: `Mesocycle::new`, `Microcycle::new`, and `Workout::new` are `pub(crate)` — only callable from within the crate (i.e., the persistence layer).
- **Per-call `init_db()`**: acceptable for now. Shared `AppState` with one connection at startup is a future concern.
- **DB path**: hardcoded `"structure.db"` in `api/mesocycles.rs` and `api/microcycles.rs`. Will be replaced when Flutter passes path via `path_provider`.

## Outstanding (before next slice)
- **`list_mesocycles` missing `ORDER BY`**: `SELECT id, name FROM mesocycles` has no `ORDER BY id` — add for deterministic ordering.
- **`create_microcycle_table` rename**: should be `create_microcycles_table` (plural, consistent with `create_mesocycles_table`). Also update the call in `sqlite.rs`.
- **`list_microcycles_returns_empty_list_for_mesocycle_with_no_microcycles` test**: missing from `persistence/microcycles.rs`.

## Still To Do (future slices)
- Persist `Workout` (naming TBD — see Outstanding above)
- Persist `Exercise` (with `exercise_type`)
- Persist `Set` (batch delete+reinsert approach, no individual id; needs `Weight::new` to reconstruct weighted/assisted sets from DB rows)
- Add `NotFound` and other meaningful variants to `MesocycleError` / `MicrocycleError`
- Replace magic `"structure.db"` string with a DB path constant
- Replace per-call `init_db()` with shared `AppState` (one connection at startup)
- Have Flutter pass DB path via `path_provider`
- Enable `PRAGMA foreign_keys = ON` when shared connection is introduced
- Regenerate FRB bridge after any DTO changes
- Replace `Result<(), String>` in `Exercise::add_set` with a typed `ExerciseError` using `thiserror`

## Relevant Files

### Rust crate (`backend/`)
- `backend/Cargo.toml` — deps: `rusqlite = { features = ["bundled"] }`, `thiserror`, `flutter_rust_bridge`, `serde`; `[lints.rust]` for FRB cfg warning
- `backend/src/lib.rs`
- `backend/src/domain/planning.rs` — all domain types: `Mesocycle`, `Microcycle`, `Workout`, `Exercise`, `Set`, `Weight`, `WeightUnit`, `ExerciseType`
- `backend/src/dto/planning.rs` — all DTOs with `From<&DomainType>` or `From<CopyType>` impls
- `backend/src/api.rs` — `pub mod mesocycles; pub mod microcycles;`
- `backend/src/api/mesocycles.rs` — `MesocycleError`, `list_mesocycles()`, `create_mesocycle(name)`
- `backend/src/api/microcycles.rs` — `MicrocycleError`, `list_microcycles(mesocycle_id)`, `create_microcycle(mesocycle_id)`
- `backend/src/persistence.rs` — `pub mod mesocycles; pub mod microcycles; pub mod sqlite;`
- `backend/src/persistence/sqlite.rs` — `open_connection(db_path)`, `init_db(db_path)`
- `backend/src/persistence/mesocycles.rs` — `create_mesocycles_table`, `create_mesocycle`, `list_mesocycles`; 4 tests
- `backend/src/persistence/microcycles.rs` — `create_microcycle_table`, `create_microcycle`, `list_microcycles`; 6 tests

### Flutter app (`frontend/`)
- `frontend/lib/src/bridge/` — generated FRB files, do not edit
- `frontend/lib/providers/training_program_list_provider.dart`
- `frontend/lib/screens/training_programs_screen.dart`

### Repo root
- `flutter_rust_bridge.yaml`
- `scripts/run_linux.sh`
