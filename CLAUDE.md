# CLAUDE.md

## What this is

A strength-training app for building long-term training plans and tracking workouts. Active development, not a finished product. The Rust workspace holds the core logic and HTTP server. `web/` is a Vite + React + TypeScript SPA, served by the Axum server in production; `structure-ffi` holds the Flutter bindings a mobile app will consume.

## Commands

Web frontend (`web/`, an npm project):

```bash
npm run app        # backend + Vite together on :5173, against a seeded persistent dev.db
npm run app:reset  # the same, after wiping dev.db back to fresh seed data
npm run shot -- /mesocycles/1   # screenshot the running app into web/.shots/
npm run dev        # Vite alone; proxies /api to a backend you started yourself on :3000
npm run verify     # the required pre-merge gate: typecheck + lint + unit + component + contract + production build + e2e
```

Frontend verification workflow (screenshots, seeded dev data, the Playwright MCP fallback): see `web/CLAUDE.md`.

### Test harnesses mirror, never reconstruct

A test or dev harness mounts the real thing inside the app's own wrappers, never a hand-built approximation of them — a copy that drifts reports on code that no longer exists, and it misleads in both directions: a green harness hiding a broken app, or a broken harness blamed on the app. Where there is no shared wrapper to reuse, extract one; the harness needing it is a finding about the code, not a licence to duplicate. Drift is the harness case of the principle below: prefer a structure that cannot diverge over a check that notices divergence afterwards.

The converse also holds: when production is correct and only the harness is awkward, fix the harness. Don't widen a component's API so a dev page can render.

Within the Rust workspace, tests live next to the code in `#[cfg(test)] mod tests` blocks. Persistence tests use an in-memory SQLite database via `connection::init_db(":memory:")` — no fixtures or external DB needed. The server's HTTP-level tests live in `structure-server/tests/`, driving `router(Store::open(":memory:"))` through `tower::ServiceExt::oneshot` (API routes are under `/api`); shared request helpers are in `tests/common/mod.rs`.

## Git

- Branch per change off `main` with a descriptive kebab-case name (e.g. `split-exercises-module`); land it through a GitHub PR rather than committing to `main` directly. The pre-commit hook refuses a commit on `main` outright.
- Write a short commit subject line phrased as a command — e.g. "Add set validation", "Split exercises module" (not "Added…" or "Splitting…").
- **The PR is the unit that is verified whole**, so it is the unit to bisect: `git bisect --first-parent` considers only the merge commits on `main`, every one of which is a state CI passed. PRs merge as merge commits and keep their individual commits, which stay a chronological record rather than a curated history — bisecting without `--first-parent` walks into them and can land mid-change.
- Keep PRs small and focused — ideally under 500 lines of diff. Split larger work into a sequence of PRs.
- The checks are a hook, not a habit: `git config core.hooksPath .githooks` installs `.githooks/pre-commit`, which runs everything `npm run verify` runs except the production build and the e2e suite picking its legs from what the commit touches. `git commit --no-verify` skips it for a WIP commit; CI still has the final say. `--all-targets` on clippy is what reaches the test code.
- Scoping the legs by path is only safe because two of them span the Rust/TypeScript boundary and run from either side: the typecheck, which regenerates the web client's types from the Rust ones first, and the contract suite, which covers the behaviour the types don't carry. Narrow either leg and a Rust-only commit can start breaking the web silently.
- `.github/workflows/ci.yml` runs the whole gate — both crates and the full `npm run verify`, e2e included — on every PR and on pushes to `main`, unconditionally, so a green tick means the same thing on every commit. `main` requires `rust` and `web` as status checks and requires branches to be up to date before merging — without that pair of settings the workflow is a report rather than a gate, and the state CI passed is not the state that lands.
- Warnings block. Clippy runs with `-D warnings` and ESLint with `--max-warnings 0`, in the hook and in CI alike.

## Architecture

Three crates, layered domain → persistence, with `structure-ffi` and `structure-server` as thin consumers:

- **`structure-core`** — the heart of the app. Pure domain model plus SQLite persistence. No FFI or framework dependencies.
  - `domain/planning.rs` — the plan domain types are **content-only**: each carries its own fields and invariants but no parent reference; how the entities relate lives in the persistence FKs, not the domain types.
  - `domain/tracking.rs` — the performed-workout layer: `LoggedSession` → `LoggedExercise` → `LoggedSet`, with nullable links back to the plan.
  - `persistence/` — one module per entity. The plan is a **template model** enforced here by the FKs: structure is defined once per mesocycle — a `workout` belongs to the mesocycle, not a microcycle — and prescription varies per week — a `set_group` is keyed by `(planned_exercise, microcycle)`, a grid cell. Reuse rather than rebuild: `aggregates.rs` assembles the full `Mesocycle` grid and the full logged session, and `positions.rs` is the shared multi-column-scoped `reorder` helper.
- **`structure-ffi`** — `flutter_rust_bridge` bindings (pinned `=2.11.1`) over `structure-core`.
- **`structure-server`** — an HTTP server over `structure-core`: `/health` at the root, the API under `/api`, one route module per entity. It also serves the built `web/` SPA with a history-fallback. The web client's TypeScript types are generated from the Rust types the routes name, so the frontend cannot hold a different idea of the contract. Its conventions live in `structure-server/CLAUDE.md`.

### Conventions to follow when extending

These cover `structure-core`; the `structure-ffi` and `structure-server` conventions live in each crate's own `CLAUDE.md`.

- **Domain types are encapsulated.** Fields are private with getter methods; constructors are `pub(crate) fn new(...)`. Invariants are enforced in the constructor — e.g. `Set::new` rejects a `Set` whose `Load` doesn't match the exercise's `ExerciseType` (see `load_matches_exercise_type`), returning `SetValidationError::LoadMismatch`. Add validation here, not in callers.
- **Persistence modules share a shape.** Each has a `pub(super) fn create_*_table(conn)` (called from `connection::init_db`), public CRUD functions taking `&Connection`, and `#[cfg(test)] mod tests`. Functions that return joined/computed columns (e.g. `microcycle_count`) return a dedicated `*Row` struct rather than a domain type.
- **Enums are stored as TEXT with `CHECK` constraints**, not integers (see the table DDL). Rust↔string conversion is manual. To write, implement `as_str(&self) -> &'static str` on the enum (zero-allocation); add a `Display` that *delegates to `as_str`* only once the enum actually needs formatting (`{}` interpolation, logs, a thiserror `#[error]` field) — don't add it up front. Reading is *fallible*: a hand-written `*_from_str`/decoder returns a `Result` and surfaces an unexpected persisted value as a typed corruption error (e.g. `SetGroupError::Corrupt`) propagated with `?`, never `panic!`. `structure-core` is a library behind FFI and an HTTP server, where a panic on one bad row aborts the app or crashes a request; a corrupt row must degrade to a typed error instead (reference impl: `persistence/set_groups.rs`). Reserve `expect` for genuinely impossible cases (§rust-best-practices 4.2), e.g. a computed `MAX(position)+1` overflowing `u32`. Older code predates the fallible-read half: several persistence modules (`set_columns`, `logged_*`, `microcycles`, …) still `panic!` on a bad read, and some keep a local `*_to_str`/`*_from_str` free fn instead of `as_str` on the enum — migrate them to the fallible-read + on-the-enum `as_str` pattern when you touch them.
- **Errors use `thiserror`, one enum per persistence module** (`MesocycleError`, `SetGroupError`, …), each wrapping `rusqlite::Error` via `#[from]` and adding domain variants like `NotFound { id }`. Keep each error type in the module that produces it — don't recentralize them into a shared `error.rs`.
- **A type in a handler's `Json<…>` is a wire type.** Derive `TS`, and mark it `#[ts(export)]`. Only those need `export`; a type reached as another's field follows it, needing the derive alone. `npm run typecheck` rewrites `web/src/api/generated/` before it runs, so the web client has no other description of the contract, and no way to hold a stale one.

## Make illegal states unrepresentable, not merely detected

When something must not happen, prefer a structure in which it *cannot be expressed* over a check that notices it afterwards. A check fires late, and only for the cases someone thought to write; a structure removes the possibility, so nobody has to remember it. Reach for the check when the structure genuinely isn't available — and say which one you settled for, so the next reader knows.

What that has meant here:

- An invariant enforced in a constructor rather than validated by callers — `Set::new` rejects a `Load` that doesn't match its `ExerciseType`.
- A value derived where it is used rather than passed in, so no caller can disagree with another — `gridColumnCount`, after a `columnCount` prop that every caller computed identically and one computed wrongly.
- One shared component rather than a copy kept in step by convention — `GridFrame`, which owns the header row *because* `table-layout: fixed` makes it size every column beneath it, and hands that same week list down to the row groups, so none can lay out against a different one.
- A dependency substituted at a seam rather than a sentinel value hoping to neutralise it — `ApiProvider`, after an id that only made two of eight mutations harmless.
- A guard on an import rather than inside the component it guards, so a dev-only module never compiles into a production build at all.

## Code style

- Comments are short, precise, and only where they are needed. The one test is whether a comment helps a future reader understand something that isn't plain on its own — a function, a type, a field, a constant, a setting alike: where the name and signature already say it, none is needed; where it is complicated, a comment beats leaving the reader to work it out.
- A comment describes the code as it stands, never the change that produced it. No "this used to…", "no longer…", "now that X is generated" — the reader months from now never saw the previous state, and a commit message is where a change gets explained. When a change makes a comment wrong, rewrite it to say what the code does now rather than appending a contrast.
- Consult the `rust-best-practices` skill when writing or reviewing Rust.

### Function ordering

Declare things close to where they're used. A helper called from a single place sits right next to that function. A helper shared by a small cluster of functions goes just above the cluster; a general, module-wide utility (like the `*_from_str` converters) is grouped with its siblings in one predictable spot — the bottom of the module, above `#[cfg(test)] mod tests` — rather than scattered.
