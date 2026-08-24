import type { FullMicrocycle, FullWorkout, LibraryExercise } from "../api/wire";

// A one-workout, two-week plan: week 1 prescribed, week 2 left empty. Shared by
// the grid's component tests, so each one only spells out what it varies, and by
// the dev workbench. Neither owns it: a copy in either place drifts from the
// other, and the page whose purpose is judging how these components read would
// end up reporting on data no test covers.

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
