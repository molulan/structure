import { useEffect, useState, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ApiError } from "../../api/client";
import type { FullPlannedExercise, FullWorkout, LibraryExercise } from "../../api/types";
import { benchPress, pushWorkout, squat, weeks } from "../../test/planFixtures";
import { AddExercise } from "../mesocycles/AddExercise";
import { MesocycleGrid } from "../mesocycles/MesocycleGrid";
import { PlanErrors, usePlanErrorSink } from "../mesocycles/PlanErrors";
import { WorkoutBand } from "../mesocycles/WorkoutBand";
import { libraryExercisesKey } from "../mesocycles/usePlanMutations";
// The band is a `<tbody>`: reproducing its real container means borrowing the
// grid's own table classes rather than approximating them here.
import gridStyles from "../mesocycles/MesocycleGrid.module.css";
import styles from "./Workbench.module.css";

/**
 * Every plan screen state that seed data cannot produce, on one page.
 *
 * `npm run app` covers the *domain's* state space — `scripts/seedDev.ts` puts a
 * four-week block on screen with every set-group encoding the grid can render.
 * What no seed reaches is the *app's* state space: a request that failed, a
 * library with nothing left to offer, a plan with exercises but no weeks. Those
 * live here, rendered from fixtures so they are always one URL away:
 *
 *     npm run shot -- /dev/workbench
 *
 * Two rules keep it useful:
 *
 * - **Nothing here touches the network.** Reads are seeded into each stage's own
 *   query cache and never refetched, so the page renders identically with no
 *   backend running — and `npm run shot`, which fails on any 4xx, stays honest.
 * - **Only states the seed cannot reach.** Anything visible at `/mesocycles/1`
 *   belongs there instead; a second copy would just be a second thing to update.
 */
export function Workbench() {
  return (
    <div className={styles.page}>
      <header className={styles.pageHead}>
        <h1 className={styles.pageTitle}>Workbench</h1>
        <p className={styles.pageNote}>
          Plan states that seed data can&apos;t produce, rendered from fixtures. The controls are
          live but point at no mesocycle, so pressing one fails by design.
        </p>
      </header>

      <Group name="PlanErrors">
        <Stage title="Request failed" note="What a rejected edit leaves on the grid.">
          <PlanErrors>
            <Report error={conflict} />
          </PlanErrors>
        </Stage>

        <Stage title="Transport failed" note="Offline: `fetch` rejects before there is a status.">
          <PlanErrors>
            <Report error={offline} />
          </PlanErrors>
        </Stage>
      </Group>

      <Group name="MesocycleGrid">
        <Stage title="Fresh mesocycle" note="Nothing prescribed yet — both notices at once.">
          <MesocycleGrid mesocycleId={NOWHERE} microcycles={[]} workouts={[]} />
        </Stage>

        <Stage
          title="Exercises before weeks"
          note="A plan can hold exercises with no weeks to prescribe them in."
          library={[squat]}
        >
          <MesocycleGrid mesocycleId={NOWHERE} microcycles={[]} workouts={[pushWorkout]} />
        </Stage>

        <Stage title="Weeks, no workouts" note="Columns with nothing under them.">
          <MesocycleGrid mesocycleId={NOWHERE} microcycles={weeks} workouts={[]} />
        </Stage>
      </Group>

      <Group name="WorkoutBand">
        <Stage title="Empty workout" note="Added, not yet filled." library={[benchPress, squat]}>
          <Band workout={emptyWorkout} />
        </Stage>

        <Stage
          title="Names at length"
          note="Where the row header and band heading give out."
          library={[benchPress]}
        >
          <Band workout={longNamedWorkout} />
        </Stage>
      </Group>

      <Group name="AddExercise">
        <Stage
          title="Library populated"
          note="The baseline the two below are read against."
          library={[benchPress, squat]}
        >
          <Picker />
        </Stage>

        <Stage
          title="Library exhausted"
          note="Everything in the library is already placed."
          library={[benchPress]}
        >
          <Picker />
        </Stage>

        <Stage
          title="Library empty"
          note="A first-run account, before anything exists to pick."
          library={[]}
        >
          <Picker />
        </Stage>
      </Group>
    </div>
  );
}

// No mesocycle has this id. The edit controls below are the real ones and fire
// real requests when clicked, so they have to land on nothing rather than on
// whatever `dev.db` currently holds.
const NOWHERE = 0x7fffffff;

// Hoisted so the effect that reports them doesn't see a new object each render.
const conflict = new ApiError(409, 'A workout named "Push" already exists in this mesocycle.');
const offline = new TypeError("Failed to fetch");

const emptyWorkout: FullWorkout = {
  id: 200,
  name: "Legs",
  position: 1,
  planned_exercises: [],
};

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

const longNamedWorkout: FullWorkout = {
  id: 201,
  name: "Lower Body — Hamstring and Glute Emphasis, Deload Variant",
  position: 2,
  planned_exercises: [longNamedPlanned],
};

function Group({ name, children }: { name: string; children: ReactNode }) {
  return (
    <section className={styles.group}>
      <h2 className={styles.groupName}>{name}</h2>
      <div className={styles.stages}>{children}</div>
    </section>
  );
}

function Stage({
  title,
  note,
  library,
  children,
}: {
  title: string;
  note: string;
  /** Seeds `useLibraryExercises` for this stage alone. */
  library?: LibraryExercise[];
  children: ReactNode;
}) {
  // One client per stage, because the states differ in what the library holds.
  // Seeded rather than fetched, and pinned fresh so nothing ever goes to the
  // network — the page has to render the same with the backend stopped.
  const [client] = useState(() => {
    const seeded = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          staleTime: Infinity,
          gcTime: Infinity,
          refetchOnMount: false,
          refetchOnWindowFocus: false,
          refetchOnReconnect: false,
        },
      },
    });
    seeded.setQueryData(libraryExercisesKey, library ?? []);
    return seeded;
  });

  return (
    <article className={styles.stage}>
      <div className={styles.stageHead}>
        <h3 className={styles.stageTitle}>{title}</h3>
        <p className={styles.stageNote}>{note}</p>
      </div>
      <div className={styles.frame}>
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
      </div>
    </article>
  );
}

/** Puts a failure on the banner without one: the sink is what a failed edit calls. */
function Report({ error }: { error: unknown }) {
  const { report } = usePlanErrorSink();
  useEffect(() => report(error), [report, error]);
  return null;
}

/** A band is a `<tbody>`, so it only lays out inside the grid's own table. */
function Band({ workout }: { workout: FullWorkout }) {
  return (
    <PlanErrors>
      <div className={gridStyles.scroll}>
        <table className={gridStyles.grid}>
          <WorkoutBand
            mesocycleId={NOWHERE}
            workout={workout}
            microcycles={weeks}
            columnCount={1 + weeks.length}
          />
        </table>
      </div>
    </PlanErrors>
  );
}

/**
 * `pushWorkout` already holds Bench Press, so what the picker offers is decided
 * by the stage's seeded library: both exercises leaves one to add, Bench Press
 * alone leaves none, and an empty library leaves nothing to have offered.
 */
function Picker() {
  return (
    <PlanErrors>
      <AddExercise mesocycleId={NOWHERE} workout={pushWorkout} />
    </PlanErrors>
  );
}
