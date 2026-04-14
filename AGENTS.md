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
- Root `Cargo.toml` workspace members: `application_core/`, `mesocycle_builder/`.
- `application_core/` is the Rust domain/bridge crate used by Flutter.
- `frontend/` is the Flutter app.
- `mesocycle_builder/` is legacy test code and expected to be removed; do not build new work around it.

## Entrypoints
- Flutter app entry: `frontend/lib/main.dart` (`RustLib.init()` runs before `runApp`).
- Main screen: `frontend/lib/screens/training_programs_screen.dart`.
- Rust bridge API: `application_core/src/api.rs`.
- Core domain types: `application_core/src/lib.rs`.

## Generated Code
- `flutter_rust_bridge.yaml` is the bridge source of truth:
  - `rust_root: application_core/`
  - `rust_input: crate::api`
  - `dart_output: frontend/lib/src/bridge/`
- Do not hand-edit generated files:
  - `application_core/src/frb_generated.rs`
  - `frontend/lib/src/bridge/*`
- Change Rust source, then regenerate bridge outputs.

## Commands
- `flutter analyze` (from `frontend/`)
- `flutter test` (from `frontend/`)
- `./run_linux.sh`

## Gotchas
- `./run_linux.sh` uses `target/debug` via `LD_LIBRARY_PATH`.
- Generated Dart FRB loader points at `../application_core/target/release/` in `frontend/lib/src/bridge/frb_generated.dart`; keep build mode and library path aligned.
- `frontend/test/widget_test.dart` is stale Flutter boilerplate, not a trustworthy app test.
- Prioritize `application_core/` and `frontend/`, not `mesocycle_builder/`.
