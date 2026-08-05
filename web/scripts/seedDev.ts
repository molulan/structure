import { createApiClient, type ApiClient } from "../src/api/client";
import type {
  CreateLibraryExercise,
  Intensity,
  RepTarget,
  SetGroupType,
} from "../src/api/types";

// The dev database's contents: one plausible four-week block plus three plans
// caught mid-build, all created over HTTP so they go through the same validation
// as the app itself. This is deliberately *not* tests/support/seed.ts — that seed
// is shaped for assertions (a bare grid with one empty cell), while this one
// exists to be looked at and clicked through, so it favours a realistic training
// block over minimal coverage.

export const REFERENCE_PLAN_NAME = "Upper/Lower Hypertrophy";

/**
 * Plans whose interest is their *shape* rather than their contents: the states
 * a plan passes through while it is being built. They can't live inside the
 * reference block — it can't be both empty and four weeks deep.
 *
 * The names say both what each one is and that it is not yours. They are rebuilt
 * on every start, so a "+ Week" pressed on the empty one is gone by the next —
 * which is the point, since a fixture that has been used is no longer the state
 * it is named for. Build on `REFERENCE_PLAN_NAME` or a plan of your own.
 */
export const FIXTURE_PLAN_NAMES = {
  empty: "Fixture (resets) — no weeks or workouts",
  exercisesNoWeeks: "Fixture (resets) — exercises but no weeks",
  weeksNoExercises: "Fixture (resets) — weeks but no exercises",
} as const;

interface SetGroupSpec {
  number_of_sets: number;
  set_group_type: SetGroupType;
}

interface ExercisePlan {
  exercise: CreateLibraryExercise;
  /** One cell per week, in week order. An empty array leaves that week unprescribed. */
  weeks: SetGroupSpec[][];
}

interface WorkoutPlan {
  name: string;
  exercises: ExercisePlan[];
}

/**
 * A four-week progression for one exercise: a set added each week at one less
 * rep in reserve, then a deload back to the starting volume at RIR 4. Enough
 * variation across the grid that a change to cell rendering is visible.
 */
function ramp(startingSets: number, reps: RepTarget): SetGroupSpec[][] {
  const prescribed = (number_of_sets: number, intensity: Intensity): SetGroupSpec[] => [
    { number_of_sets, set_group_type: { Prescribed: { set_type: "Regular", reps, intensity } } },
  ];
  return [
    prescribed(startingSets, { Rir: 3 }),
    prescribed(startingSets + 1, { Rir: 2 }),
    prescribed(startingSets + 2, { Rir: 1 }),
    prescribed(startingSets, { Rir: 4 }),
  ];
}

const REGULAR = (reps: RepTarget, intensity: Intensity, number_of_sets: number): SetGroupSpec => ({
  number_of_sets,
  set_group_type: { Prescribed: { set_type: "Regular", reps, intensity } },
});

const MYOREP_MATCH = (number_of_sets: number): SetGroupSpec => ({
  number_of_sets,
  set_group_type: "MyorepMatch",
});

const PLAN: WorkoutPlan[] = [
  {
    name: "Upper A",
    exercises: [
      {
        exercise: {
          name: "Barbell Bench Press",
          exercise_type: "Weighted",
          primary_muscle_group: "Chest",
          secondary_muscle_groups: ["Triceps", "Shoulders"],
        },
        weeks: ramp(3, { Range: { min: 6, max: 8 } }),
      },
      {
        exercise: {
          name: "Barbell Row",
          exercise_type: "Weighted",
          primary_muscle_group: "Back",
          secondary_muscle_groups: ["Biceps", "Traps"],
        },
        weeks: ramp(3, { Range: { min: 8, max: 10 } }),
      },
      {
        exercise: {
          name: "Seated Overhead Press",
          exercise_type: "Weighted",
          primary_muscle_group: "Shoulders",
          secondary_muscle_groups: ["Triceps"],
        },
        weeks: ramp(2, { Range: { min: 8, max: 12 } }),
      },
      {
        exercise: {
          name: "Cable Lateral Raise",
          exercise_type: "Weighted",
          primary_muscle_group: "Shoulders",
          secondary_muscle_groups: [],
        },
        // A myorep-match finisher, dropped entirely in the deload week — so the
        // grid shows both the unit-variant encoding and an empty cell.
        weeks: [[MYOREP_MATCH(2)], [MYOREP_MATCH(3)], [MYOREP_MATCH(3)], []],
      },
    ],
  },
  {
    name: "Lower A",
    exercises: [
      {
        exercise: {
          name: "Back Squat",
          exercise_type: "Weighted",
          primary_muscle_group: "Quads",
          secondary_muscle_groups: ["Glutes", "Hamstrings"],
        },
        weeks: ramp(3, { Range: { min: 5, max: 8 } }),
      },
      {
        exercise: {
          name: "Romanian Deadlift",
          exercise_type: "Weighted",
          primary_muscle_group: "Hamstrings",
          secondary_muscle_groups: ["Glutes", "Back"],
        },
        weeks: ramp(3, { Range: { min: 8, max: 10 } }),
      },
      {
        exercise: {
          name: "Standing Calf Raise",
          exercise_type: "Weighted",
          primary_muscle_group: "Calves",
          secondary_muscle_groups: [],
        },
        weeks: [
          [REGULAR({ Range: { min: 10, max: 15 } }, { Rir: 2 }, 3), MYOREP_MATCH(1)],
          [REGULAR({ Range: { min: 10, max: 15 } }, { Rir: 1 }, 3), MYOREP_MATCH(2)],
          [REGULAR({ Range: { min: 10, max: 15 } }, { Rir: 1 }, 4), MYOREP_MATCH(2)],
          [REGULAR({ Range: { min: 10, max: 15 } }, { Rir: 4 }, 2)],
        ],
      },
    ],
  },
  {
    name: "Upper B",
    exercises: [
      {
        exercise: {
          name: "Weighted Pull-Up",
          exercise_type: "WeightedBodyweight",
          primary_muscle_group: "Back",
          secondary_muscle_groups: ["Biceps"],
        },
        weeks: ramp(3, { Range: { min: 5, max: 8 } }),
      },
      {
        exercise: {
          name: "Incline Dumbbell Press",
          exercise_type: "Weighted",
          primary_muscle_group: "Chest",
          secondary_muscle_groups: ["Shoulders", "Triceps"],
        },
        weeks: ramp(3, { Range: { min: 8, max: 12 } }),
      },
      {
        exercise: {
          name: "Cable Triceps Pushdown",
          exercise_type: "Weighted",
          primary_muscle_group: "Triceps",
          secondary_muscle_groups: [],
        },
        weeks: ramp(2, { Range: { min: 10, max: 15 } }),
      },
      {
        exercise: {
          name: "Incline Dumbbell Curl",
          exercise_type: "Weighted",
          primary_muscle_group: "Biceps",
          secondary_muscle_groups: [],
        },
        // An AMRAP set closes every week out — the `AtLeast` rep target.
        weeks: [
          [REGULAR({ Range: { min: 10, max: 12 } }, { Rir: 2 }, 2), REGULAR({ AtLeast: 12 }, { Rir: 0 }, 1)],
          [REGULAR({ Range: { min: 10, max: 12 } }, { Rir: 1 }, 3), REGULAR({ AtLeast: 12 }, { Rir: 0 }, 1)],
          [REGULAR({ Range: { min: 10, max: 12 } }, { Rir: 1 }, 3), REGULAR({ AtLeast: 12 }, { Rir: 0 }, 1)],
          [REGULAR({ Range: { min: 10, max: 12 } }, { Rir: 4 }, 2)],
        ],
      },
    ],
  },
  {
    name: "Lower B",
    exercises: [
      {
        exercise: {
          name: "Leg Press",
          exercise_type: "Weighted",
          primary_muscle_group: "Quads",
          secondary_muscle_groups: ["Glutes"],
        },
        weeks: ramp(3, { Range: { min: 10, max: 12 } }),
      },
      {
        exercise: {
          name: "Seated Leg Curl",
          exercise_type: "Weighted",
          primary_muscle_group: "Hamstrings",
          secondary_muscle_groups: [],
        },
        weeks: ramp(3, { Range: { min: 10, max: 15 } }),
      },
      {
        exercise: {
          name: "Barbell Hip Thrust",
          exercise_type: "Weighted",
          primary_muscle_group: "Glutes",
          secondary_muscle_groups: ["Hamstrings"],
        },
        // Prescribed by RPE rather than RIR, so that encoding appears too.
        weeks: [
          [REGULAR({ Range: { min: 8, max: 10 } }, { Rpe: 7 }, 3)],
          [REGULAR({ Range: { min: 8, max: 10 } }, { Rpe: 8 }, 3)],
          [REGULAR({ Range: { min: 8, max: 10 } }, { Rpe: 9 }, 4)],
          [REGULAR({ Range: { min: 8, max: 10 } }, { Rpe: 6 }, 2)],
        ],
      },
    ],
  },
];

/**
 * Builds the dev block in the server at `baseUrl` (an `/api` root) and returns
 * the new mesocycle's id. Library exercises are reused by name when they already
 * exist, since their names are globally unique — so a half-finished seed can be
 * re-run without tripping over its own rows.
 */
export async function seedDevPlan(baseUrl: string): Promise<number> {
  const api = createApiClient(baseUrl);

  const mesocycle = await api.createMesocycle({ name: REFERENCE_PLAN_NAME, mode: "Manual" });

  const weeks = [];
  for (const phase of ["Accumulation", "Accumulation", "Intensification", "Deload"] as const) {
    const week = await api.addMicrocycle(mesocycle.id);
    await api.setMicrocyclePhase(week.id, phase);
    weeks.push(week);
  }

  const existing = new Map((await api.listLibraryExercises()).map((e) => [e.name, e.id]));

  for (const workoutPlan of PLAN) {
    const workout = await api.addWorkout(mesocycle.id, workoutPlan.name);
    for (const { exercise, weeks: cells } of workoutPlan.exercises) {
      const planned = await api.addPlannedExercise(
        workout.id,
        await libraryIdFor(api, existing, exercise),
      );
      for (const [index, week] of weeks.entries()) {
        for (const spec of cells[index] ?? []) {
          await addSetGroup(baseUrl, planned.id, week.id, spec);
        }
      }
    }
  }

  await rebuildFixturePlans(baseUrl);

  return mesocycle.id;
}

/**
 * How to build each fixture plan, keyed by the name that identifies it.
 *
 * They are seeded rather than mocked on the workbench because seeded they stay
 * clickable: "+ Week" on an empty plan can be watched actually working. Each is
 * defined by what it *lacks*, which is why the seed contract test pins their
 * shape — nothing here would fail if a future server started handing out a first
 * week with every new mesocycle.
 */
const FIXTURE_PLANS: Record<
  string,
  (api: ApiClient, library: Map<string, number>) => Promise<void>
> = {
  [FIXTURE_PLAN_NAMES.empty]: async (api) => {
    await api.createMesocycle({ name: FIXTURE_PLAN_NAMES.empty, mode: "Manual" });
  },

  [FIXTURE_PLAN_NAMES.exercisesNoWeeks]: async (api, library) => {
    const plan = await api.createMesocycle({
      name: FIXTURE_PLAN_NAMES.exercisesNoWeeks,
      mode: "Manual",
    });
    // Borrowed from the reference block rather than named here. The draft only
    // has to hold *some* exercises, so naming them would be a coupling that
    // reads as detail: renaming one in `PLAN` would leave this plan empty, or
    // abort the seed with the reference block already written.
    for (const source of PLAN.slice(0, 2)) {
      const workout = await api.addWorkout(plan.id, source.name);
      for (const { exercise } of source.exercises.slice(0, 2)) {
        await api.addPlannedExercise(workout.id, await libraryIdFor(api, library, exercise));
      }
    }
  },

  [FIXTURE_PLAN_NAMES.weeksNoExercises]: async (api) => {
    const plan = await api.createMesocycle({
      name: FIXTURE_PLAN_NAMES.weeksNoExercises,
      mode: "Manual",
    });
    for (let week = 0; week < 3; week += 1) {
      await api.addMicrocycle(plan.id);
    }
    // Left empty on purpose: an added workout with nothing in it yet is a state
    // of its own, and it needs weeks around it to show the band spanning them.
    await api.addWorkout(plan.id, "Legs");
  },
};

/**
 * Discards the fixture plans and builds them again, touching nothing else.
 *
 * Unconditional because a fixture is only worth having while it still *is* the
 * state it is named for, and using one as intended destroys that: press "+ Week"
 * on the empty plan and it is no longer empty. Checking whether each still has
 * the right shape would need a predicate per fixture, kept in step with its
 * builder by hand — so instead there is no state in which a fixture is wrong,
 * because it is remade before anyone sees it.
 *
 * The block you build on is not touched: `npm run app` seeds that only into an
 * empty database, and `--reset` is still how you ask for a clean one.
 */
export async function rebuildFixturePlans(baseUrl: string): Promise<string[]> {
  const api = createApiClient(baseUrl);
  const stale = (await api.listMesocycles()).filter((plan) => plan.name in FIXTURE_PLANS);
  for (const plan of stale) {
    await deleteMesocycle(baseUrl, plan.id);
  }

  const library = new Map((await api.listLibraryExercises()).map((e) => [e.name, e.id]));
  for (const build of Object.values(FIXTURE_PLANS)) {
    await build(api, library);
  }

  return Object.keys(FIXTURE_PLANS);
}

// Deleting a mesocycle has no client method: the app cannot delete a plan, so
// production would gain unused surface. Kept local for the same reason as
// `addSetGroup` below.
async function deleteMesocycle(baseUrl: string, id: number): Promise<void> {
  const response = await fetch(`${baseUrl}/mesocycles/${id}`, { method: "DELETE" });
  if (!response.ok) {
    throw new Error(`DELETE /mesocycles/${id} → ${response.status}: ${await response.text()}`);
  }
}

// Library exercise names are globally unique, so a row that exists is the one
// we want and a missing one is ours to create — which is what lets a
// half-finished seed be re-run, and what lets the fixtures be rebuilt into a
// database whose reference block predates an edit to `PLAN`.
async function libraryIdFor(
  api: ApiClient,
  library: Map<string, number>,
  exercise: CreateLibraryExercise,
): Promise<number> {
  const known = library.get(exercise.name);
  if (known !== undefined) return known;

  const created = await api.createLibraryExercise(exercise);
  library.set(exercise.name, created.id);
  return created.id;
}

// Cell prescription has no client method yet — the grid can't edit cells, so the
// app has no call for one. Kept here rather than added to the shared client so
// production code gains no unused surface.
async function addSetGroup(
  baseUrl: string,
  plannedExerciseId: number,
  microcycleId: number,
  spec: SetGroupSpec,
): Promise<void> {
  const path = `/planned-exercises/${plannedExerciseId}/microcycles/${microcycleId}/set-groups`;
  const response = await fetch(`${baseUrl}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(spec),
  });
  if (!response.ok) {
    throw new Error(`POST ${path} → ${response.status}: ${await response.text()}`);
  }
}

/** How many exercise rows the seed builds — reported by the dev stack's banner. */
export const SEEDED_EXERCISE_COUNT = PLAN.reduce((n, w) => n + w.exercises.length, 0);

export const SEEDED_WORKOUT_COUNT = PLAN.length;
