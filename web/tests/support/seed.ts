// Shared seed for the "full mesocycle" setup that both the contract test and the
// e2e spec must exercise identically. Each caller passes its own `post` (contract
// uses fetch, e2e uses Playwright's request), so the mesocycle → … → set-group
// chain lives in one place: an endpoint-shape change touches this file, not two.
// Granular tests that need a different shape seed their own setup directly.

export type Post = (path: string, body?: unknown) => Promise<{ id: number }>;

export interface SeededTree {
  mesocycleId: number;
  /** The three weeks, in position order. */
  microcycleIds: number[];
  workoutId: number;
  plannedExerciseId: number;
}

/**
 * Builds a three-week grid: one "Push" workout holding one "Bench Press" slot,
 * prescribed differently per week — week 1 gets three set groups (a range, an
 * AMRAP and a myorep match, exercising every encoding), week 2 a single group,
 * and week 3 nothing at all, so the empty cell is covered too. Returns the ids
 * for follow-up calls (e.g. setting a phase); no phase is set — that's
 * caller-specific.
 */
export async function buildFullMesocycle(post: Post, name: string): Promise<SeededTree> {
  const mesocycle = await post("/mesocycles", { name, mode: "Manual" });
  const weeks = [
    await post(`/mesocycles/${mesocycle.id}/microcycles`),
    await post(`/mesocycles/${mesocycle.id}/microcycles`),
    await post(`/mesocycles/${mesocycle.id}/microcycles`),
  ];
  const workout = await post(`/mesocycles/${mesocycle.id}/workouts`, { name: "Push" });
  const exercise = await post("/library-exercises", {
    name: "Bench Press",
    exercise_type: "Weighted",
    primary_muscle_group: "Chest",
    secondary_muscle_groups: ["Triceps", "Shoulders"],
  });
  const planned = await post(`/workouts/${workout.id}/planned-exercises`, {
    library_exercise_id: exercise.id,
  });

  const cell = (microcycleId: number) =>
    `/planned-exercises/${planned.id}/microcycles/${microcycleId}/set-groups`;

  await post(cell(weeks[0].id), {
    number_of_sets: 3,
    set_group_type: {
      Prescribed: { set_type: "Regular", reps: { Range: { min: 8, max: 12 } }, intensity: { Rir: 2 } },
    },
  });
  await post(cell(weeks[0].id), {
    number_of_sets: 1,
    set_group_type: {
      Prescribed: { set_type: "Regular", reps: { AtLeast: 12 }, intensity: { Rir: 0 } },
    },
  });
  await post(cell(weeks[0].id), {
    number_of_sets: 2,
    set_group_type: "MyorepMatch",
  });
  await post(cell(weeks[1].id), {
    number_of_sets: 4,
    set_group_type: {
      Prescribed: { set_type: "Regular", reps: { Exact: 8 }, intensity: { Rir: 1 } },
    },
  });

  return {
    mesocycleId: mesocycle.id,
    microcycleIds: weeks.map((week) => week.id),
    workoutId: workout.id,
    plannedExerciseId: planned.id,
  };
}
