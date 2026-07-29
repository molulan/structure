import type { FullMicrocycle, FullPlannedExercise, FullWorkout } from "../../api/types";
import { describeSetGroup, phaseLabel } from "./format";
import styles from "./MesocycleGrid.module.css";

interface Props {
  microcycles: FullMicrocycle[];
  workouts: FullWorkout[];
}

/**
 * The plan grid: weeks across, exercise slots down, each cell the set groups
 * prescribed for that (planned exercise, week). Read-only.
 */
export function MesocycleGrid({ microcycles, workouts }: Props) {
  if (microcycles.length === 0) {
    return <p className={styles.empty}>No weeks yet.</p>;
  }

  return (
    <div className={styles.scroll}>
      <table className={styles.grid}>
        <thead>
          <tr>
            <td className={styles.corner} />
            {microcycles.map((week) => (
              <th key={week.id} scope="col" className={styles.weekHead}>
                <span className={styles.weekLabel}>Week {week.position + 1}</span>{" "}
                {week.phase && (
                  <span className={styles.phase} data-phase={week.phase} title={week.phase}>
                    {phaseLabel(week.phase)}
                  </span>
                )}
              </th>
            ))}
          </tr>
        </thead>

        {workouts.length === 0 ? (
          <tbody>
            <tr>
              <td className={styles.bandEmpty} colSpan={microcycles.length + 1}>
                No workouts yet.
              </td>
            </tr>
          </tbody>
        ) : (
          workouts.map((workout) => (
            <tbody key={workout.id} className={styles.band}>
              <tr>
                <th
                  scope="colgroup"
                  colSpan={microcycles.length + 1}
                  className={styles.workoutHead}
                >
                  <span className={styles.workoutName}>{workout.name}</span>{" "}
                  {workout.planned_exercises.length > 0 && (
                    <span className={styles.workoutMeta}>
                      {exerciseCount(workout.planned_exercises.length)}
                    </span>
                  )}
                </th>
              </tr>

              {workout.planned_exercises.length === 0 ? (
                <tr>
                  <td className={styles.bandEmpty} colSpan={microcycles.length + 1}>
                    No exercises yet.
                  </td>
                </tr>
              ) : (
                workout.planned_exercises.map((planned) => (
                  <ExerciseRow key={planned.id} planned={planned} microcycles={microcycles} />
                ))
              )}
            </tbody>
          ))
        )}
      </table>
    </div>
  );
}

function ExerciseRow({
  planned,
  microcycles,
}: {
  planned: FullPlannedExercise;
  microcycles: FullMicrocycle[];
}) {
  // Index the cells by week rather than trusting `prescriptions` to be parallel
  // to `microcycles`: a week with nothing prescribed still has to line up under
  // its own column.
  const cells = new Map(planned.prescriptions.map((p) => [p.microcycle_id, p.set_groups]));

  return (
    <tr>
      <th scope="row" className={styles.rowHead}>
        <span className={styles.exerciseName}>{planned.exercise.name}</span>
        <span className={styles.muscle}>{planned.exercise.primary_muscle_group}</span>
      </th>
      {microcycles.map((week) => {
        const setGroups = cells.get(week.id) ?? [];
        return (
          <td key={week.id} className={styles.cell}>
            {setGroups.length === 0 ? (
              <span className={styles.unprescribed} aria-label="Not prescribed">
                —
              </span>
            ) : (
              setGroups.map((group) => (
                <div key={group.id} className={styles.setLine}>
                  {describeSetGroup(group)}
                </div>
              ))
            )}
          </td>
        );
      })}
    </tr>
  );
}

function exerciseCount(count: number): string {
  return `${count} ${count === 1 ? "exercise" : "exercises"}`;
}
