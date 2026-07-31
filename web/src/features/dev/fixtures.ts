import type {
  FullMicrocycle,
  FullPlannedExercise,
  FullWorkout,
  LibraryExercise,
} from "../../api/types";

/*
 * Fixtures shaped for looking at, kept apart from `src/test/planFixtures` for
 * the reason `scripts/seedDev.ts` is kept apart from `tests/support/seed.ts`:
 * those exist to be asserted against and change whenever an assertion needs
 * them to, which is no reason for this page to change what it shows. Sharing
 * them would also point app code at test code.
 */

export const benchPress: LibraryExercise = {
  id: 1,
  name: "Bench Press",
  exercise_type: "Weighted",
  primary_muscle_group: "Chest",
  secondary_muscle_groups: ["Triceps", "Shoulders"],
};

/** Not placed in `pushWorkout` — the one the picker can still offer. */
export const squat: LibraryExercise = {
  id: 2,
  name: "Squat",
  exercise_type: "Weighted",
  primary_muscle_group: "Quads",
  secondary_muscle_groups: ["Glutes"],
};

/** Long enough to find where a row header and a band heading give out. */
const longNamedExercise: LibraryExercise = {
  id: 3,
  name: "Single-Leg Romanian Deadlift (Dumbbell, Deficit)",
  exercise_type: "Weighted",
  primary_muscle_group: "Hamstrings",
  secondary_muscle_groups: ["Glutes", "Back"],
};

export const weeks: FullMicrocycle[] = [
  { id: 10, position: 0, phase: "Accumulation" },
  { id: 11, position: 1, phase: null },
];

export const pushWorkout: FullWorkout = {
  id: 100,
  name: "Push",
  position: 0,
  planned_exercises: [
    {
      id: 1000,
      position: 0,
      exercise: benchPress,
      prescriptions: [
        {
          microcycle_id: 10,
          set_groups: [
            {
              id: 1,
              position: 0,
              number_of_sets: 3,
              set_group_type: {
                Prescribed: {
                  set_type: "Regular",
                  reps: { Range: { min: 8, max: 12 } },
                  intensity: { Rir: 2 },
                },
              },
            },
            { id: 2, position: 1, number_of_sets: 2, set_group_type: "MyorepMatch" },
          ],
        },
        { microcycle_id: 11, set_groups: [] },
      ],
    },
  ],
};

export const emptyWorkout: FullWorkout = {
  id: 200,
  name: "Legs",
  position: 1,
  planned_exercises: [],
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
