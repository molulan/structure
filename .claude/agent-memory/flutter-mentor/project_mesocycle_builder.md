---
name: mesocycle-builder
description: Authoritative design for the mesocycle builder screen (manual mode = desktop-only spreadsheet grid)
metadata:
  type: project
---

The mesocycle builder is a separate screen `lib/screens/training_program_builder_screen.dart`, pushed via Navigator.push/MaterialPageRoute after mesocycle creation, receiving MesocycleDTO by constructor. No named routes.

There are two creation modes (MesocycleModeDTO: manual, algorithmic). MANUAL mode is being built FIRST and is desktop-only. The ALGORITHMIC mode has NOT been designed yet — its details will be discussed in a later session; do not assume anything about it.

Manual mode layout = four-quadrant spreadsheet grid: microcycles as vertical rows ("Week 1"...), workouts as horizontal columns ("Workout A"...). Pinned left = week labels, pinned top = workout headers, pinned right = per-muscle-group volume summary (DEFERRED — no muscle-group data yet). Middle cell grid scrolls 2D. Variable row height (rows grow to fit content, NOT fixed-height scrolling cells). Fixed workout count per microcycle, position-based alignment, no explicit column entity.

**Why:** Decided with software-architect in a prior session; manual layout is the hard case and is expected to be the foundation algorithmic mode builds on later.

**How to apply:** Data flow goes through ONE aggregate Dart-composed provider (composes listMicrocycles/listWorkouts/listPlannedExercises N+1 for now) so swapping to a future Rust aggregate API is a one-file change. Bridge calls are synchronous and throw. Step 1 = grid shell with synced ScrollControllers; cell interactions (add/remove/rename/reorder, add exercise/set, copy row) are LATER steps.
