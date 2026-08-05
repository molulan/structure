import { useEffect, useState, type ReactNode } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ApiError } from "../../api/client";
import type { FullWorkout, LibraryExercise } from "../../api/types";
import { ApiProvider } from "../../lib/ApiProvider";
import { AddExercise } from "../mesocycles/AddExercise";
import { GridFrame } from "../mesocycles/GridFrame";
import { PlanErrors, usePlanErrorSink } from "../mesocycles/PlanErrors";
import { WorkoutBand } from "../mesocycles/WorkoutBand";
import { libraryExercisesKey } from "../mesocycles/usePlanMutations";
import { benchPress, pushWorkout, squat, weeks } from "../../fixtures/plan";
import { longNamedWorkout } from "./fixtures";
import { offlineApi } from "./offlineApi";
import styles from "./Workbench.module.css";

/**
 * The plan screen states that seed data has no way to hold, on one page.
 *
 * Most degenerate states *are* seedable, and belong in `scripts/seedDev.ts`
 * instead — seeded they stay clickable, so "+ Week" on an empty plan can be
 * watched actually working, where everything here is frozen by construction.
 * Three kinds don't fit in a database and live here:
 *
 * 1. **States no data can produce** — a rejected edit, a `fetch` that never got
 *    a status. A row cannot express "the request came back 409".
 * 2. **States that contradict the seed's other content** — an empty library
 *    can't coexist with a plan, since planned exercises reference library rows.
 * 3. **Adversarial input that isn't a state at all** — a name at length is a
 *    stress test, and seeding one would degrade every other screenshot.
 *
 *     npm run shot -- /dev/workbench
 *
 * **Nothing here reaches the network.** Each stage gets `offlineApi`, whose
 * every method rejects, so the page renders and behaves identically with no
 * backend running and no control can edit real data — whatever id it carries.
 * Reads are seeded into the stage's own query cache and pinned fresh, so a stage
 * shows its state rather than a failed load.
 */
export function Workbench() {
  return (
    <div className={styles.page}>
      <header className={styles.pageHead}>
        <h1 className={styles.pageTitle}>Workbench</h1>
        <p className={styles.pageNote}>
          Plan states a database has no way to hold, rendered from fixtures — everything else
          lives in the seed, where it stays clickable. The controls here are the real ones, wired
          to a client with no backend behind it: pressing one reports a failure without sending
          anything.
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

      <Group name="WorkoutBand">
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
 * real `GridFrame`, not a copy of it. The header row it brings is what decides
 * the column widths under `table-layout: fixed`, so a name is judged here at
 * the 190px the label column is everywhere else.
 */
function Band({ workout }: { workout: FullWorkout }) {
  return (
    <PlanErrors>
      <GridFrame mesocycleId={NOWHERE} microcycles={weeks}>
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
