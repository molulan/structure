---
name: project-app-structure
description: Current Flutter app navigation and screen structure for the structure app
metadata:
  type: project
---

Navigation is imperative (`Navigator.push` / `MaterialPageRoute`). No named routes, no `go_router`. `main.dart` uses `MaterialApp` with `home: TrainingProgramsScreen()`.

Screen inventory (as of 2026-05-27):
- `training_programs_screen.dart` — two-column layout (300px list left, detail right). Entry point.
- `training_program_builder_screen.dart` — being built; pushed via `Navigator.push` after mesocycle creation. Receives `MesocycleDTO` via constructor.

State management: Riverpod (`flutter_riverpod`). Providers live in `lib/providers/`. Pattern is `FutureProvider` backed by bridge calls.

DTOs come from `lib/src/bridge/dto/planning.dart` (generated — never edit). Key types: `MesocycleDTO`, `MicrocycleDTO`, `WorkoutDTO`.

**Why no named routes:** App is simple, no deep linking needed. Named routes add rigidity with no benefit. Decision to revisit if `go_router` becomes justified.
