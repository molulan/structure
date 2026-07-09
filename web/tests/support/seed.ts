// Shared seed for the "full mesocycle" setup that both the contract test and the
// e2e spec must exercise identically. Each caller passes its own `post` (contract
// uses fetch, e2e uses Playwright's request), so the mesocycle → … → set-group
// chain lives in one place: an endpoint-shape change touches this file, not two.
// Granular tests that need a different shape seed their own setup directly.

export type Post = (path: string, body?: unknown) => Promise<{ id: number }>;

export interface SeededTree {
  mesocycleId: number;
  microcycleId: number;
  workoutId: number;
  plannedExerciseId: number;
}

/**
 * Builds one microcycle → "Push" workout → "Bench Press" exercise → planned
 * exercise with three set groups (a range, an AMRAP, and a myorep match),
 * exercising every set-group encoding. Returns the ids for follow-up calls
 * (e.g. setting a phase). Does not set a phase — that's caller-specific.
 */
export async function buildFullMesocycle(post: Post, name: string): Promise<SeededTree> {
  const mesocycle = await post("/mesocycles", { name, mode: "Manual" });
  const microcycle = await post(`/mesocycles/${mesocycle.id}/microcycles`);
  const workout = await post(`/microcycles/${microcycle.id}/workouts`, { name: "Push" });
  const exercise = await post("/library-exercises", {
    name: "Bench Press",
    exercise_type: "Weighted",
    primary_muscle_group: "Chest",
    secondary_muscle_groups: ["Triceps", "Shoulders"],
  });
  const planned = await post(`/workouts/${workout.id}/planned-exercises`, {
    library_exercise_id: exercise.id,
  });
  await post(`/planned-exercises/${planned.id}/set-groups`, {
    number_of_sets: 3,
    set_group_type: {
      Prescribed: { set_type: "Regular", reps: { Range: { min: 8, max: 12 } }, intensity: { Rir: 2 } },
    },
  });
  await post(`/planned-exercises/${planned.id}/set-groups`, {
    number_of_sets: 1,
    set_group_type: {
      Prescribed: { set_type: "Regular", reps: { AtLeast: 12 }, intensity: { Rir: 0 } },
    },
  });
  await post(`/planned-exercises/${planned.id}/set-groups`, {
    number_of_sets: 2,
    set_group_type: "MyorepMatch",
  });
  return {
    mesocycleId: mesocycle.id,
    microcycleId: microcycle.id,
    workoutId: workout.id,
    plannedExerciseId: planned.id,
  };
}
