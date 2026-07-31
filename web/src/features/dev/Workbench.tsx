import { useEffect, useState, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ApiError } from "../../api/client";
import type { FullWorkout, LibraryExercise } from "../../api/types";
import { ApiProvider } from "../../lib/ApiProvider";
import { AddExercise } from "../mesocycles/AddExercise";
import { GridFrame } from "../mesocycles/GridFrame";
import { MesocycleGrid } from "../mesocycles/MesocycleGrid";
import { PlanErrors, usePlanErrorSink } from "../mesocycles/PlanErrors";
import { WorkoutBand } from "../mesocycles/WorkoutBand";
import { libraryExercisesKey } from "../mesocycles/usePlanMutations";
import {
  benchPress,
  emptyWorkout,
  longNamedWorkout,
  pushWorkout,
  squat,
  weeks,
} from "./fixtures";
import { offlineApi } from "./offlineApi";
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
 * - **Nothing here reaches the network.** Each stage gets `offlineApi`, whose
 *   every method rejects, so the page renders and behaves identically with no
 *   backend running and no control can edit real data — whatever id it carries.
 *   Reads are seeded into the stage's own query cache and pinned fresh, so a
 *   stage shows its state rather than a failed load.
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
          the real ones, wired to a client with no backend behind it: pressing one reports a
          failure without sending anything.
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

// The grid needs a mesocycle id and no request is ever sent, so the value only
// has to be recognisable while reading the stages.
const NOWHERE = 0x7fffffff;

// Hoisted so the effect that reports them doesn't see a new object each render.
const conflict = new ApiError(409, 'A workout named "Push" already exists in this mesocycle.');
const offline = new TypeError("Failed to fetch");

/*
 * The group and stage labels are placards about the specimen, not content
 * inside it, so they stay out of the heading hierarchy: a `WorkoutBand` heads
 * itself with an `<h2>`, and chrome ranked above it would invert the outline on
 * the one page meant for judging how these components read. `aria-label` keeps
 * each one navigable as a landmark instead.
 */
function Group({ name, children }: { name: string; children: ReactNode }) {
  return (
    <section className={styles.group} aria-label={name}>
      <p className={styles.groupName}>{name}</p>
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
  // `staleTime: Infinity` is the whole of it: seeded data that never goes stale
  // is never refetched on mount, on focus or on reconnect, so each stage shows
  // the library it was given rather than a load that failed against `offlineApi`.
  const [client] = useState(() => {
    const seeded = new QueryClient({
      defaultOptions: { queries: { staleTime: Infinity } },
    });
    seeded.setQueryData(libraryExercisesKey, library ?? []);
    return seeded;
  });

  return (
    <article className={styles.stage} aria-label={title}>
      <div className={styles.stageHead}>
        <p className={styles.stageTitle}>{title}</p>
        <p className={styles.stageNote}>{note}</p>
      </div>
      <div className={styles.frame}>
        <QueryClientProvider client={client}>
          <ApiProvider client={offlineApi}>{children}</ApiProvider>
        </QueryClientProvider>
      </div>
    </article>
  );
}

/**
 * Puts a failure on the banner without one: the sink is what a failed edit
 * calls. Dismiss is part of what these stages exist to show, so raising it
 * again has to be possible — otherwise the first click empties the stage until
 * the page is reloaded.
 */
function Report({ error }: { error: unknown }) {
  const { report } = usePlanErrorSink();
  const [raised, setRaised] = useState(0);

  useEffect(() => report(error), [report, error, raised]);

  return (
    <button className={styles.raise} type="button" onClick={() => setRaised((n) => n + 1)}>
      Raise again
    </button>
  );
}

/**
 * A band is a `<tbody>`, so it only lays out inside the grid's own table — the
 * real `GridFrame`, not a copy of it, or this page would report on a container
 * the app doesn't have.
 */
function Band({ workout }: { workout: FullWorkout }) {
  return (
    <PlanErrors>
      <GridFrame>
        <WorkoutBand mesocycleId={NOWHERE} workout={workout} microcycles={weeks} />
      </GridFrame>
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
