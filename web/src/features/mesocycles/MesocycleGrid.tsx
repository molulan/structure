import { useState } from "react";
import type { FullMicrocycle, FullWorkout } from "../../api/wire";
import { GridFrame, gridColumnCount } from "./GridFrame";
import { PlanErrors } from "./PlanErrors";
import { WorkoutBand } from "./WorkoutBand";
import { useAddWeek, useAddWorkout } from "./usePlanMutations";
import styles from "./MesocycleGrid.module.css";

interface Props {
  mesocycleId: number;
  microcycles: FullMicrocycle[];
  workouts: FullWorkout[];
}

/**
 * The plan grid: weeks across, exercise slots down, each cell the set groups
 * prescribed for that (planned exercise, week). Structure is editable here —
 * weeks, workouts and their exercises; the cells themselves are still read-only.
 */
export function MesocycleGrid({ mesocycleId, microcycles, workouts }: Props) {
  const columnCount = gridColumnCount(microcycles);

  return (
    <PlanErrors>
      <AddWeek mesocycleId={mesocycleId} />

      <GridFrame mesocycleId={mesocycleId} microcycles={microcycles}>
        {workouts.length === 0 ? (
          <tbody>
            <tr>
              <td className={styles.bandEmpty} colSpan={columnCount}>
                No workouts yet.
              </td>
            </tr>
          </tbody>
        ) : (
          workouts.map((workout) => (
            <WorkoutBand key={workout.id} workout={workout} />
          ))
        )}
      </GridFrame>

      <AddWorkout mesocycleId={mesocycleId} />
    </PlanErrors>
  );
}

function AddWeek({ mesocycleId }: { mesocycleId: number }) {
  const addWeek = useAddWeek(mesocycleId);

  return (
    <div className={styles.toolbar}>
      <button
        className={styles.addButton}
        onClick={() => addWeek.mutate()}
        disabled={addWeek.isPending}
      >
        + Week
      </button>
    </div>
  );
}

function AddWorkout({ mesocycleId }: { mesocycleId: number }) {
  const addWorkout = useAddWorkout(mesocycleId);
  const [name, setName] = useState("");
  const trimmed = name.trim();

  function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    if (!trimmed) return;
    addWorkout.mutate(trimmed, { onSuccess: () => setName("") });
  }

  return (
    <form className={styles.addWorkout} onSubmit={onSubmit}>
      <input
        className={styles.input}
        placeholder="New workout name"
        aria-label="New workout name"
        value={name}
        onChange={(event) => setName(event.target.value)}
      />
      <button
        className={styles.addButton}
        type="submit"
        disabled={!trimmed || addWorkout.isPending}
      >
        + Workout
      </button>
    </form>
  );
}
