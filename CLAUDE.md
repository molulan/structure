# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A strength-training app for building long-term training plans and tracking workouts. Active development, not a finished product. The Rust workspace holds the core logic and HTTP server. `web/` is a Vite + React + TypeScript SPA, served by the Axum server in production; `structure-ffi` holds the Flutter bindings a mobile app will consume.

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

Web frontend (`web/`, an npm project):

```bash
npm run app        # backend + Vite together on :5173, against a seeded persistent dev.db
npm run app:reset  # the same, after wiping dev.db back to fresh seed data
npm run shot -- /mesocycles/1   # screenshot the running app into web/.shots/
npm run dev        # Vite alone; proxies /api to a backend you started yourself on :3000
npm run verify     # the required pre-merge gate: typecheck + lint + unit + component + contract + e2e
```

### Verifying frontend changes

Every change under `web/` ends with a look at the running app, not just green tests. Leave `npm run app` up — it serves a seeded four-week plan, so no screen is empty — and take a `npm run shot -- <route>` after each edit: Vite hot-reloads, so the screenshot is current within a second or two, and the script fails on console errors and 4xx/5xx responses — and on a capture or comparison it could not complete — which a screenshot alone would hide. It keeps each route's last frame in `.shots/prev/` and reports how much of the page moved — `changed 240 pixels (<0.1%) vs a frame 40s old`, with the changed regions in `.shots/diff/`. Read the count before the image: an edit that never reached the screen renders exactly like one that worked, and `identical to the previous frame` is what tells them apart. An edit that changes the page's height reports the size alongside the count for the area both frames share. Any framing but the default — `--viewport` or `--viewport-only` — writes under its own name, so a mobile check doesn't become the baseline the next desktop capture is judged against. Iterate against that; save the full `npm run verify` for before the commit rather than between edits. The dev data lives in `web/scripts/seedDev.ts` — extend it when a screen needs something the block doesn't cover yet, and keep it distinct from `tests/support/seed.ts`, which is shaped for assertions rather than for looking at.

For the times a still frame isn't enough — clicking into a cell, hovering, filling a form, reading the console or network log — `.mcp.json` configures a headless Playwright MCP server against the same running stack. Reach for `npm run shot` first: it answers "does this look right" in one command, where the MCP server costs tool-schema overhead in every session. On `browser_take_screenshot`, omit `filename` so output lands in the gitignored `web/.shots/mcp/`; a relative filename resolves against the repo root instead and leaves junk in the working tree.

### Test harnesses mirror, never reconstruct

A test or dev harness mounts the real thing inside the app's own wrappers, never a hand-built approximation of them — a copy that drifts reports on code that no longer exists, and it misleads in both directions: a green harness hiding a broken app, or a broken harness blamed on the app. Where there is no shared wrapper to reuse, extract one; the harness needing it is a finding about the code, not a licence to duplicate. Drift is the harness case of the principle below: prefer a structure that cannot diverge over a check that notices divergence afterwards.

The converse also holds: when production is correct and only the harness is awkward, fix the harness. Don't widen a component's API so a dev page can render.

Within the Rust workspace, tests live next to the code in `#[cfg(test)] mod tests` blocks. Persistence tests use an in-memory SQLite database via `connection::init_db(":memory:")` — no fixtures or external DB needed. The server's HTTP-level tests live in `structure-server/tests/`, driving `router(Store::open(":memory:"))` through `tower::ServiceExt::oneshot` (API routes are under `/api`); shared request helpers are in `tests/common/mod.rs`.

## Git

- Branch per change off `main` with a descriptive kebab-case name (e.g. `split-exercises-module`); land it through a GitHub PR rather than committing to `main` directly.
- Keep PRs small and focused — ideally under 500 lines of diff. Split larger work into a sequence of PRs.
- Write a short commit subject line phrased as a command — e.g. "Add set validation", "Split exercises module" (not "Added…" or "Splitting…").
- Run `cargo fmt` and `cargo clippy --workspace` before committing.

## Architecture

Three crates, layered domain → persistence, with `structure-ffi` and `structure-server` as thin consumers:

- **`structure-core`** — the heart of the app. Pure domain model plus SQLite persistence. No FFI or framework dependencies.
  - `domain/planning.rs` — the plan domain types: `Mesocycle`, `Microcycle` (`position`, `phase`), `Workout` (`name`, `position`), `PlannedExercise` (references a `LibraryExercise`), and `SetGroup` (`number_of_sets` plus a `SetGroupType` — `Prescribed { set_type, reps: RepTarget, intensity: Intensity }` or `MyorepMatch`), alongside `LibraryExercise`, `Phase`, `Weight`, `MuscleGroup`, `ExerciseType`. These are **content-only** — each carries its own fields and invariants but no parent reference; how the entities relate lives in the persistence FKs, not the domain types.
  - `domain/tracking.rs` — the performed-workout layer: `LoggedSession` → `LoggedExercise` → `LoggedSet`, with nullable links back to the plan.
  - `persistence/` — one module per entity (`mesocycles`, `microcycles`, `workouts`, `library_exercises`, `planned_exercises`, `set_groups`, plus the tracking modules `logged_sessions`, `logged_exercises`, `logged_sets`). The plan is a **template model** enforced here by the FKs: structure is defined once per mesocycle — a `workout` belongs to the mesocycle, not a microcycle — and prescription varies per week — a `set_group` is keyed by `(planned_exercise, microcycle)`, a grid cell. `connection.rs` opens connections and builds the schema (`init_db`); `store.rs` wraps one in `Store`, a cloneable `Arc<Mutex<Connection>>` handle (`open`, `with_conn`); `aggregates.rs` assembles the full `Mesocycle` grid (workouts × microcycles, each planned exercise carrying a `Prescription` per week) and the full logged session; `positions.rs` is the shared multi-column-scoped `reorder` helper.
- **`structure-ffi`** — `flutter_rust_bridge` bindings (pinned `=2.11.1`) over `structure-core`. Compiled as `cdylib`/`staticlib`/`rlib` for consumption by a Flutter app. `api/` holds `#[frb(sync)]` wrappers per entity; `dto/planning.rs` holds the wire types.
- **`structure-server`** — an Axum 0.8 HTTP server over `structure-core`. `lib.rs` exposes `router(store)` — `/health` at the root and the API under `/api` (one route module per entity, nested) — and `app(store, web_dir)`, which wraps `router` to also serve the built `web/` SPA with an SPA history-fallback when a web dir is present. See the server conventions below.

### Conventions to follow when extending

- **Domain types are encapsulated.** Fields are private with getter methods; constructors are `pub(crate) fn new(...)`. Invariants are enforced in the constructor — e.g. `Set::new` rejects a `Set` whose `Load` doesn't match the exercise's `ExerciseType` (see `load_matches_exercise_type`), returning `SetValidationError::LoadMismatch`. Add validation here, not in callers.
- **Persistence modules share a shape.** Each has a `pub(super) fn create_*_table(conn)` (called from `connection::init_db`), public CRUD functions taking `&Connection`, and `#[cfg(test)] mod tests`. Functions that return joined/computed columns (e.g. `microcycle_count`) return a dedicated `*Row` struct rather than a domain type.
- **Enums are stored as TEXT with `CHECK` constraints**, not integers (see the table DDL). Rust↔string conversion is manual. To write, implement `as_str(&self) -> &'static str` on the enum (zero-allocation); add a `Display` that *delegates to `as_str`* only once the enum actually needs formatting (`{}` interpolation, logs, a thiserror `#[error]` field) — don't add it up front. Reading is *fallible*: a hand-written `*_from_str`/decoder returns a `Result` and surfaces an unexpected persisted value as a typed corruption error (e.g. `SetGroupError::Corrupt`) propagated with `?`, never `panic!`. `structure-core` is a library behind FFI and an HTTP server, where a panic on one bad row aborts the app or crashes a request; a corrupt row must degrade to a typed error instead (reference impl: `persistence/set_groups.rs`). Reserve `expect` for genuinely impossible cases (§rust-best-practices 4.2), e.g. a computed `MAX(position)+1` overflowing `u32`. Older code predates the fallible-read half: several persistence modules (`set_columns`, `logged_*`, `microcycles`, …) still `panic!` on a bad read, and some keep a local `*_to_str`/`*_from_str` free fn instead of `as_str` on the enum — migrate them to the fallible-read + on-the-enum `as_str` pattern when you touch them.
- **Errors use `thiserror`, one enum per persistence module** (`MesocycleError`, `SetGroupError`, …), each wrapping `rusqlite::Error` via `#[from]` and adding domain variants like `NotFound { id }`. Keep each error type in the module that produces it — don't recentralize them into a shared `error.rs`.
- **The FFI layer is a thin DTO-mapping shell.** For each domain type `X` there's an `XDTO` in `dto/planning.rs` annotated `#[frb]`, with `From<&X> for XDTO` and (where input is needed) `From<XDTO> for X`. `api/` functions are `#[frb(sync)]`, open the DB (`connection::init_db("structure.db")`), call into `structure-core`, and map rows/domain types to DTOs. Put no business logic here.
- **The server is a thin HTTP layer; logic stays in `structure-core`.** Handlers take `State<Store>`, run queries via `store.with_conn(|conn| …)`, and return `Result<Json<…>, ApiError>`. Request bodies are `Deserialize` structs in `dto.rs`; `error.rs` maps each persistence error to a status code via `From<…Error> for ApiError`. Add an endpoint by extending an entity's `routes()`, not by adding logic in the handler.

## Make illegal states unrepresentable, not merely detected

When something must not happen, prefer a structure in which it *cannot be expressed* over a check that notices it afterwards. A check fires late, and only for the cases someone thought to write; a structure removes the possibility, so nobody has to remember it. Reach for the check when the structure genuinely isn't available — and say which one you settled for, so the next reader knows.

What that has meant here:

- An invariant enforced in a constructor rather than validated by callers — `Set::new` rejects a `Load` that doesn't match its `ExerciseType`.
- A value derived where it is used rather than passed in, so no caller can disagree with another — `gridColumnCount`, after a `columnCount` prop that every caller computed identically and one computed wrongly.
- One shared component rather than a copy kept in step by convention — `GridFrame`, which owns the header row *because* `table-layout: fixed` makes it size every column beneath it, and hands that same week list down to the row groups, so none can lay out against a different one.
- A dependency substituted at a seam rather than a sentinel value hoping to neutralise it — `ApiProvider`, after an id that only made two of eight mutations harmless.
- A guard on an import rather than inside the component it guards, so a dev-only module never compiles into a production build at all.

## Code style

- Comments are rare and explain *why*, not *what* — reserve them for non-obvious rationale (a constraint, a subtle invariant). Let names carry the meaning.
- Consult the `rust-best-practices` skill when writing or reviewing Rust.

### Function ordering

Declare things close to where they're used. A helper called from a single place sits right next to that function. A helper shared by a small cluster of functions goes just above the cluster; a general, module-wide utility (like the `*_from_str` converters) is grouped with its siblings in one predictable spot — the bottom of the module, above `#[cfg(test)] mod tests` — rather than scattered.
