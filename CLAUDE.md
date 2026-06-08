# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is
## Product Context
- `structure` is a strength-training app for:
  - building long-term training plans (`Mesocycle`s)
  - tracking workouts through those plans
- Domain hierarchy: mesocycle -> microcycles -> workouts -> exercises -> sets.
- Workout tracking is primarily mobile.
- Plan creation should work on mobile and desktop; desktop matters for overview on larger screens.
- Training history, PR tracking, and history-based planning/auto-adjustment are intended features, not current guarantees.

## Commands

```sh
# Run the app (Linux only)
./scripts/run_linux.sh          # cargo build backend + flutter run -d linux

# Backend
cd backend && cargo test        # run Rust tests
cd backend && cargo build       # build only

# Frontend
cd frontend && flutter test     # run Flutter tests
cd frontend && flutter analyze  # lint

# Regenerate FRB bridge (after changing backend/src/api.rs or types it exposes)
flutter_rust_bridge_codegen generate
```

## Architecture

Flutter app (`frontend/`) + Rust library (`backend/`) connected via [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge).

**Rust layers** (innermost → outermost):
- `domain/planning.rs` — pure domain types (`Mesocycle`, `Workout`, `Set`, `Load`, `Effort`, etc.)
- `persistence/` — SQLite via `rusqlite`; works only with domain types
- `dto/planning.rs` — DTOs for FRB transport (all fields `pub(crate)`, all types `Copy` or derive it as needed)
- `api/` — orchestrates persistence, converts domain → DTO at the boundary, returned to Flutter

**Flutter layers:**
- `lib/src/bridge/` — **generated**, do not edit; Dart bindings for `crate::api`
- `lib/providers/` — Riverpod providers
- `lib/screens/` — UI screens and widgets

**Bridge config** (`flutter_rust_bridge.yaml`): `rust_input: crate::api`, `dart_output: frontend/lib/src/bridge/`. Only `pub` items in `backend/src/api.rs` (and its sub-modules) are exposed to Dart.

## Key rules

- **Never hand-edit generated files**: `backend/src/frb_generated.rs` and `frontend/lib/src/bridge/*`. Change Rust source, then regenerate.
- Always route feature requests, bug reports, and project tasks 
through the project-orchestrator agent by default.
- **Domain constructors are `pub(crate)`** (`Mesocycle::new`, `Workout::new`, etc.) — only callable from within the crate (i.e., persistence layer). DTOs use `pub` fields for FRB visibility; that is not a back-door to construction.
- **DDL functions are `pub(super)`** — visible to `sqlite.rs` for bootstrap only.
- **Table creation order** in `init_db`: mesocycles → microcycles → workouts → exercises → planned_exercises (parents before children).
- **`LD_LIBRARY_PATH`** in `run_linux.sh` points at `target/debug`; generated Dart loader in `frb_generated.dart` points at `../backend/target/release/`. Keep build mode and library path aligned when running manually.
- **SQLite FK enforcement**: `PRAGMA foreign_keys = ON` is set in `open_connection`. Do not rely on FK constraint errors as app signals — add explicit existence checks that produce meaningful `DomainError` variants (see `create_workout` pattern).
- **`Exercise` vs `PlannedExercise`**: `Exercise` is the browsable catalogue item; `PlannedExercise` is the workout-scoped instance with position and sets.