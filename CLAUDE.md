# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A strength-training app for building long-term training plans and tracking workouts. Active development, not a finished product. The Rust workspace holds the core logic; the frontend is not in the tree yet (`mobile/` and `web/` are empty placeholders).

## Commands

Rust workspace (edition 2024, resolver 3):

```bash
cargo build                       # build all crates
cargo test --workspace            # run all tests
cargo test -p structure-core      # test a single crate
cargo test mesocycle              # run tests matching a substring
cargo test created_mesocycle_appears_in_list_with_correct_id_name_and_mode  # single test
cargo fmt
cargo clippy --workspace
```

Tests live next to the code in `#[cfg(test)] mod tests` blocks. Persistence tests use an in-memory SQLite database via `sqlite::init_db(":memory:")` — no fixtures or external DB needed.

## Architecture

Three crates, layered domain → persistence → FFI:

- **`structure-core`** — the heart of the app. Pure domain model plus SQLite persistence. No FFI or framework dependencies.
  - `domain/planning.rs` — the domain types: `Mesocycle` → `Microcycle` → `Workout` → `PlannedExercise` → `Set`, plus `Exercise`, `Load`, `Effort` (`Rir`/`Rpe`), `Weight`. This is the training-plan hierarchy.
  - `persistence/` — one module per entity (`mesocycles`, `microcycles`, `workouts`, `exercises`, `sets`). `sqlite.rs` owns connection setup (`open_connection`, `init_db`) and creates every table.
- **`structure-ffi`** — `flutter_rust_bridge` bindings (pinned `=2.11.1`) over `structure-core`. Compiled as `cdylib`/`staticlib`/`rlib` for the future Flutter app. `api/` mirrors the persistence modules; `dto/planning.rs` holds the wire types.
- **`structure-server`** — backend server, currently a stub (`main` just prints).

### Conventions to follow when extending

- **Domain types are encapsulated.** Fields are private with getter methods; constructors are `pub(crate) fn new(...)`. Invariants are enforced in the constructor — e.g. `PlannedExercise::new` and `add_set` reject a `Set` whose `Load` doesn't match the `Exercise`'s `ExerciseType` (see `load_matches_exercise_type`), returning `PlannedExerciseValidationError`. Add validation here, not in callers.
- **Persistence modules share a shape.** Each has a `pub(super) fn create_*_table(conn)` (called from `sqlite::init_db`), public CRUD functions taking `&Connection`, and `#[cfg(test)] mod tests`. Functions that return joined/computed columns (e.g. `microcycle_count`) return a dedicated `*Row` struct rather than a domain type.
- **Enums are stored as TEXT with `CHECK` constraints**, not integers (see the table DDL). Rust↔string conversion is manual: `Display`/`to_string()` to write, a hand-written `*_from_str` to read (which `panic!`s on unexpected values, treating it as DB corruption).
- **Errors use `thiserror`, one enum per persistence module** (`MesocycleError`, `SetError`, …), each wrapping `rusqlite::Error` via `#[from]` and adding domain variants like `NotFound { id }`. Keep each error type in the module that produces it (a prior refactor split a shared `error.rs` apart deliberately).
- **The FFI layer is a thin DTO-mapping shell.** For each domain type `X` there's an `XDTO` in `dto/planning.rs` annotated `#[frb]`, with `From<&X> for XDTO` and (where input is needed) `From<XDTO> for X`. `api/` functions are `#[frb(sync)]`, open the DB (`sqlite::init_db("structure.db")`), call into `structure-core`, and map rows/domain types to DTOs. Put no business logic here.
