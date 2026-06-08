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
- Rust bridge API: `backend/src/api.rs` (`pub mod mesocycles; pub mod microcycles; pub mod workouts;`).
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
- SQLite foreign keys: `rusqlite` with the `bundled` feature compiles SQLite with `SQLITE_DEFAULT_FOREIGN_KEYS=1` — FK enforcement is **on by default in tests** (`:memory:` DB). On a real device using system SQLite, the default is off — `PRAGMA foreign_keys = ON` is already set in `open_connection` in `persistence/sqlite.rs` and is active for all connections including production.
- Do not rely on FK constraint errors as application error signals — always add an explicit existence check in the persistence layer for parent references (see `create_workout` for the pattern). This makes behaviour consistent regardless of SQLite build and produces meaningful errors instead of opaque constraint failures.
- Table creation order matters in `init_db`: parent tables before child tables (mesocycles before microcycles).

## Architecture & Design Decisions

- **Domain/DTO/persistence/API layering**: domain types for business logic, DTOs for FRB transport, persistence works with domain types only, API orchestrates and converts domain → DTO at boundary.
- **FRB boundary**: keep generated code isolated in `frontend/lib/src/bridge/*`; keep `lib.rs` thin.
- **`pub(crate)` on domain constructors**: `Mesocycle::new`, `Microcycle::new`, `Workout::new`, `Exercise::new`, `PlannedExercise::new` are `pub(crate)` — only callable from within the crate (i.e., the persistence layer).
- **Per-call `init_db()`**: acceptable for now. Shared `AppState` with one connection at startup is a future concern.
- **DB path**: hardcoded `"structure.db"` in all API modules. Will be replaced when Flutter passes path via `path_provider`.
- **Workout identity**: Workout has both `name: String` (user-supplied, required, non-empty) and `position: u32` (0-based ordering within the microcycle). Name is the identity; position controls sort order. Display label is the name, not a derived "Day N".
- **Error layering**: `errors.rs` holds shared error types used by both persistence and API layers. Persistence `create_*` and `list_*` functions that perform existence checks return `Result<T, DomainError>`. Persistence `get_*` functions and DDL functions return `rusqlite::Result` — they have no domain opinions. API functions return `Result<T, DomainError>` to Flutter; `?` on `rusqlite::Result` in the API works via `#[from] rusqlite::Error` on each error type.
- **`_exists` helpers**: private (`fn`, not `pub`) helpers in each persistence module that return `rusqlite::Result<bool>`. Used by `create_*` and `list_*` for parent existence checks. Do NOT reuse `get_*` functions for existence checks — `_exists` helpers use `COUNT(*)` and keep error types narrow. Exception: `create_planned_exercise` uses `get_exercise` instead of `exercise_exists` because it needs the full `Exercise` value to construct `PlannedExercise`.
- **DDL function visibility**: `pub(super)` on all DDL functions — visible to `sqlite.rs` for bootstrap, not part of the crate's public API.
- **`PRAGMA foreign_keys = ON`**: set in `open_connection` in `persistence/sqlite.rs` — active for all connections including production.
- **`Exercise` is the browsable catalogue** (`id`, `name`, `exercise_type`) — the thing the user picks from a library. `PlannedExercise` is the workout-scoped instance (`id`, `exercise: Exercise`, `position`, `sets: Vec<Set>`). These are different domain concepts.
- **`ExerciseType` has `Display` impl** in `domain/planning.rs` — used by persistence layer via `.to_string()` for DB storage. Avoids a separate helper function.
- **`exercise_type_from_str`**: private helper in `persistence/exercises.rs` for reading `ExerciseType` back from DB TEXT. Uses `panic!` on unknown values — data came from our code, corruption = panic is correct.
- **`Set` is an enum, `Load` is a separate enum**: set type (how reps are performed) and load (what resistance) are independent dimensions; `add_set` validates `(ExerciseType, Load)` pair.
- **`Set` variants**: `Regular { load, reps, effort }`, `Myorep { load, reps }`, `MyorepMatch { load, reps }`, `Drop { load, reps, effort }`. `Myorep` sets structurally have no `effort` field.
- **`Load` variants**: `Bodyweight`, `WeightedBodyweight { added_weight: Option<Weight> }`, `AssistedBodyweight { assistance: Option<Weight> }`, `Weighted { weight: Option<Weight> }`.
- **`Effort` enum**: `Rir(Rir)` or `Rpe(Rpe)`. Both are `Copy`.
- **`Rpe(u8)`**: validated range 1–11; 11 = failure/past failure (shown as "10+" in UI). Integer, not float.
- **`Rir(i8)`**: validated range -1 to 10; -1 = past failure/beyond failure.
- **`ExerciseType` variants**: `Bodyweight`, `WeightedBodyweight`, `AssistedBodyweight`, `Weighted`.
- **`From<T>` by value for `Copy` types in DTOs**: not `From<&T>`; enables clean `.map(TypeDTO::from)` on `Option<T>`.
- **`SetDTO` derives `Copy`**: requires entire chain (`LoadDTO`, `WeightDTO`, `EffortDTO`, `RpeDTO`, `RirDTO`) to also be `Copy`.
- **DTO fields are `pub`** (not `pub(crate)`) — required for FRB codegen to generate Dart accessors. Domain construction is gated by `pub(crate)` constructors on domain types, not by DTO field visibility.
- **`RpeDTO(pub u8)` and `RirDTO(pub i8)`**: tuple struct fields must be `pub` for FRB visibility.
- **`ExerciseTypeDTO`** has both `From<ExerciseType>` (domain → DTO) and `From<ExerciseTypeDTO> for ExerciseType` (DTO → domain) — both live in `dto/planning.rs` to keep all conversions out of the domain layer.
- **`u32::try_from().expect()`**: used for `COUNT(*)` → `u32` and DB-stored `position` → `u32` conversions. Pragmatic choice: these values are guaranteed non-negative and within u32 range for this domain.
- **Two-step query pattern in `list_planned_exercises`**: fully collect raw `(i64, i64, i64)` tuples from `query_map` into a `Vec` first, then call `get_exercise` per row in a second loop. Required because rusqlite does not allow reusing `conn` while a `query_map` iterator is active (statement still holds a borrow). `list_exercises` does not need this because it makes no secondary queries.
- **`exercises` and `planned_exercises` in one persistence file**: `Exercise` (catalogue) and `PlannedExercise` (workout-scoped) share `persistence/exercises.rs` because `list_planned_exercises` must join both and the two concepts are tightly coupled. Split if a standalone catalogue browsing screen is added later.
- **Tracking types (`PerformedSet`, `PerformedLoad`, `WorkoutExecution`, etc.)**: explicitly out of scope until planning slice is complete and stable.
- **`FromIterator` for `Result<Vec<T>, E>`** does not invoke `From` — `.map_err` before `.collect()` always required at `query_map` boundary.
- **`.optional()` does not invoke `From`** — requires explicit `.map_err` when function returns a domain error type.
- **`?` is the only mechanism that invokes `From` automatically**.

## Still To Do (future slices)

- Regenerate FRB bridge (next immediate step)
- Implement `save_sets` in `persistence/exercises.rs` and wire up sets persistence
- Add `get_mesocycle`, `get_microcycle`, `get_workout` to API layer
- Add typed error variants for empty/invalid name in `create_mesocycle` and `create_workout` (currently surfaces as opaque `Database` error from SQLite `CHECK` constraint)
- Replace `Result<(), String>` in `PlannedExercise::add_set` with a typed error using `thiserror`
- Add by-value `From<T>` impls to non-Copy DTOs and switch `.iter().map()` to `.into_iter().map()` in API layer (low priority)
- Replace magic `"structure.db"` string with a DB path constant
- Replace per-call `init_db()` with shared `AppState` (one connection at startup)
- Have Flutter pass DB path via `path_provider`
- Design tracking types (`PerformedSet`, `PerformedLoad`, `WorkoutExecution`, performed workout) — deferred until planning slice is complete
- Reconsider error type granularity (`ExerciseError` vs `PlannedExerciseError`) if cross-entity operations become awkward

## Relevant Files

### Rust crate (`backend/`)
- `backend/Cargo.toml` — deps: `rusqlite = { features = ["bundled"] }`, `thiserror`, `flutter_rust_bridge`, `serde`; `[lints.rust]` for FRB cfg warning
- `backend/src/lib.rs` — declares `pub mod errors`
- `backend/src/errors.rs` — `MesocycleError`, `MicrocycleError` (+ `AssociatedMesocycleNotFound`, `NotFound`), `WorkoutError` (+ `AssociatedMicrocycleNotFound`, `NotFound`), `ExerciseError` (+ `DuplicateName`, `NotFound`), `PlannedExerciseError` (+ `AssociatedWorkoutNotFound`, `AssociatedExerciseNotFound`, `NotFound`); all with `#[from] rusqlite::Error`
- `backend/src/domain/planning.rs` — all domain types: `Mesocycle`, `Microcycle`, `Workout`, `Exercise` (`id`, `name`, `exercise_type`; `pub(crate) new`; `Display` impl on `ExerciseType`), `PlannedExercise` (`id`, `exercise: Exercise`, `position`, `sets: Vec<Set>`; `pub(crate) new`; `add_set` validates load/type pair), `Set` (enum: `Regular`, `Myorep`, `MyorepMatch`, `Drop`), `Load` (enum), `Effort` (enum), `Rpe(u8)` (1–11), `Rir(i8)` (-1–10), `Weight`, `WeightUnit`, `ExerciseType`
- `backend/src/dto/planning.rs` — all DTOs: `MesocycleDTO`, `MicrocycleDTO`, `WorkoutDTO`, `ExerciseDTO`, `ExerciseTypeDTO`, `PlannedExerciseDTO`, `SetDTO`, `LoadDTO`, `WeightDTO`, `WeightUnitDTO`, `EffortDTO`, `RpeDTO(pub u8)`, `RirDTO(pub i8)`; all with `#[frb]`, `pub` fields, correct derives; `From<ExerciseTypeDTO> for ExerciseType` in this file (not domain layer)
- `backend/src/api.rs` — `pub mod mesocycles; pub mod microcycles; pub mod workouts; pub mod exercises;`
- `backend/src/api/mesocycles.rs` — `list_mesocycles()`, `create_mesocycle(name: String)` — errors from `crate::errors`
- `backend/src/api/microcycles.rs` — `list_microcycles(mesocycle_id)`, `create_microcycle(mesocycle_id)`
- `backend/src/api/workouts.rs` — `list_workouts(microcycle_id)`, `create_workout(microcycle_id, name: String)`
- `backend/src/api/exercises.rs` — `create_exercise(name: String, exercise_type: ExerciseTypeDTO)`, `get_exercise(id)`, `list_exercises()`, `create_planned_exercise(workout_id, exercise_id)`, `get_planned_exercise(id)`, `list_planned_exercises(workout_id)`; all `#[frb(sync)]`
- `backend/src/persistence.rs` — `pub mod mesocycles; pub mod microcycles; pub mod workouts; pub mod exercises; pub mod sqlite;`
- `backend/src/persistence/sqlite.rs` — `open_connection(db_path)` (sets `PRAGMA foreign_keys = ON`), `init_db(db_path)` (calls all `create_*_table` functions in order: mesocycles → microcycles → workouts → exercises → planned_exercises)
- `backend/src/persistence/mesocycles.rs` — `create_mesocycles_table` (pub(super)), `create_mesocycle`, `get_mesocycle`, `list_mesocycles`; 6 tests
- `backend/src/persistence/microcycles.rs` — `create_microcycles_table` (pub(super)), `create_microcycle`, `get_microcycle`, `list_microcycles`; 9 tests
- `backend/src/persistence/workouts.rs` — `create_workouts_table` (pub(super)), `create_workout`, `get_workout`, `list_workouts`; 11 tests
- `backend/src/persistence/exercises.rs` — `create_exercises_table`, `create_planned_exercises_table` (both pub(super)); `workout_exists`, `exercise_name_exists`, `exercise_type_from_str` (private helpers); `create_exercise`, `get_exercise`, `list_exercises`, `create_planned_exercise`, `get_planned_exercise`, `list_planned_exercises`; ~20 tests

### Flutter app (`frontend/`)
- `frontend/lib/src/bridge/` — generated FRB files, do not edit
- `frontend/lib/providers/training_program_list_provider.dart`
- `frontend/lib/screens/training_programs_screen.dart`

### Repo root
- `flutter_rust_bridge.yaml`
- `scripts/run_linux.sh`
