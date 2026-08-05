import type { FullPlannedExercise, FullWorkout, LibraryExercise } from "../../api/types";
import { weeks } from "../../fixtures/plan";

/*
 * Adversarial input rather than a plan state, so it is the workbench's alone:
 * seeding a name this long would spoil every other screenshot of the demo block.
 * Everything the page shares with the component tests comes from
 * `src/fixtures/plan` instead.
 */

const longNamedExercise: LibraryExercise = {
  id: 3,
  name: "Single-Leg Romanian Deadlift (Dumbbell, Deficit)",
  exercise_type: "Weighted",
  primary_muscle_group: "Hamstrings",
  secondary_muscle_groups: ["Glutes", "Back"],
};

const longNamedPlanned: FullPlannedExercise = {
  id: 2000,
  position: 0,
  exercise: longNamedExercise,
  prescriptions: weeks.map((week) => ({ microcycle_id: week.id, set_groups: [] })),
};

export const longNamedWorkout: FullWorkout = {
  id: 201,
  name: "Lower Body — Hamstring and Glute Emphasis, Deload Variant",
  position: 2,
  planned_exercises: [longNamedPlanned],
};
