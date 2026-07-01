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

Within the Rust workspace, tests live next to the code in `#[cfg(test)] mod tests` blocks. Persistence tests use an in-memory SQLite database via `connection::init_db(":memory:")` — no fixtures or external DB needed. The server's HTTP-level tests live in `structure-server/tests/`, driving `router(Store::open(":memory:"))` through `tower::ServiceExt::oneshot`; shared request helpers are in `tests/common/mod.rs`.

## Git

- Branch per change off `main` with a descriptive kebab-case name (e.g. `split-exercises-module`); land it through a GitHub PR rather than committing to `main` directly.
- Keep PRs small and focused — ideally under 500 lines of diff. Split larger work into a sequence of PRs.
- Write a short commit subject line phrased as a command — e.g. "Add set validation", "Split exercises module" (not "Added…" or "Splitting…").
- Run `cargo fmt` and `cargo clippy --workspace` before committing.

## Architecture

Three crates, layered domain → persistence, with `structure-ffi` and `structure-server` as thin consumers:

- **`structure-core`** — the heart of the app. Pure domain model plus SQLite persistence. No FFI or framework dependencies.
  - `domain/planning.rs` — the training-plan hierarchy `Mesocycle` → `Microcycle` → `Workout` → `PlannedExercise` → `Set`, plus `LibraryExercise` (the reusable exercise a `PlannedExercise` references), `Load`, `Effort`, `Weight`.
  - `persistence/` — one module per entity (`mesocycles`, `microcycles`, `workouts`, `library_exercises`, `planned_exercises`, `sets`). `connection.rs` opens connections and builds the schema (`init_db`); `store.rs` wraps one in `Store`, a cloneable `Arc<Mutex<Connection>>` handle (`open`, `with_conn`); `aggregates.rs` reads the full `Mesocycle` tree; `positions.rs` is the shared `reorder` helper.
- **`structure-ffi`** — `flutter_rust_bridge` bindings (pinned `=2.11.1`) over `structure-core`. Compiled as `cdylib`/`staticlib`/`rlib` for consumption by the Flutter app. `api/` holds `#[frb(sync)]` wrappers per entity; `dto/planning.rs` holds the wire types.
- **`structure-server`** — an Axum 0.8 HTTP server over `structure-core`; `lib.rs` exposes `router(store)`, with one route module per entity. See the server conventions below.

### Conventions to follow when extending

- **Domain types are encapsulated.** Fields are private with getter methods; constructors are `pub(crate) fn new(...)`. Invariants are enforced in the constructor — e.g. `Set::new` rejects a `Set` whose `Load` doesn't match the exercise's `ExerciseType` (see `load_matches_exercise_type`), returning `SetValidationError::LoadMismatch`. Add validation here, not in callers.
- **Persistence modules share a shape.** Each has a `pub(super) fn create_*_table(conn)` (called from `connection::init_db`), public CRUD functions taking `&Connection`, and `#[cfg(test)] mod tests`. Functions that return joined/computed columns (e.g. `microcycle_count`) return a dedicated `*Row` struct rather than a domain type.
- **Enums are stored as TEXT with `CHECK` constraints**, not integers (see the table DDL). Rust↔string conversion is manual. To write, implement `as_str(&self) -> &'static str` on the enum (zero-allocation); add a `Display` that *delegates to `as_str`* only once the enum actually needs formatting (`{}` interpolation, logs, a thiserror `#[error]` field) — don't add it up front. Reading is *fallible*: a hand-written `*_from_str`/decoder returns a `Result` and surfaces an unexpected persisted value as a typed corruption error (e.g. `SetGroupError::Corrupt`) propagated with `?`, never `panic!`. `structure-core` is a library behind FFI and an HTTP server, where a panic on one bad row aborts the app or crashes a request; a corrupt row must degrade to a typed error instead (reference impl: `persistence/set_groups.rs`). Reserve `expect` for genuinely impossible cases (§rust-best-practices 4.2), e.g. a computed `MAX(position)+1` overflowing `u32`. Older code predates the fallible-read half: several persistence modules (`set_columns`, `sets`, `logged_*`, `microcycles`, …) still `panic!` on a bad read, and some keep a local `*_to_str`/`*_from_str` free fn instead of `as_str` on the enum — migrate them to the fallible-read + on-the-enum `as_str` pattern when you touch them.
- **Errors use `thiserror`, one enum per persistence module** (`MesocycleError`, `SetError`, …), each wrapping `rusqlite::Error` via `#[from]` and adding domain variants like `NotFound { id }`. Keep each error type in the module that produces it — don't recentralize them into a shared `error.rs`.
- **The FFI layer is a thin DTO-mapping shell.** For each domain type `X` there's an `XDTO` in `dto/planning.rs` annotated `#[frb]`, with `From<&X> for XDTO` and (where input is needed) `From<XDTO> for X`. `api/` functions are `#[frb(sync)]`, open the DB (`connection::init_db("structure.db")`), call into `structure-core`, and map rows/domain types to DTOs. Put no business logic here.
- **The server is a thin HTTP layer; logic stays in `structure-core`.** Handlers take `State<Store>`, run queries via `store.with_conn(|conn| …)`, and return `Result<Json<…>, ApiError>`. Request bodies are `Deserialize` structs in `dto.rs`; `error.rs` maps each persistence error to a status code via `From<…Error> for ApiError`. Add an endpoint by extending an entity's `routes()`, not by adding logic in the handler.

## Code style

- Comments are rare and explain *why*, not *what* — reserve them for non-obvious rationale (a constraint, a subtle invariant). Let names carry the meaning.
- Consult the `rust-best-practices` skill when writing or reviewing Rust.

### Function ordering

Declare things close to where they're used. A helper called from a single place sits right next to that function. A helper shared by a small cluster of functions goes just above the cluster; a general, module-wide utility (like the `*_from_str` converters) is grouped with its siblings in one predictable spot — the bottom of the module, above `#[cfg(test)] mod tests` — rather than scattered.
