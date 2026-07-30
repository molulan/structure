import type { FullMicrocycle, FullPlannedExercise, FullWorkout, Phase } from "../../api/types";
import { describeSetGroup, phaseLabel, PHASE_ORDER } from "./format";
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
  // A mesocycle can hold workouts before it has any weeks — structure hangs off
  // the mesocycle, not the microcycle — so a week-less grid still renders its
  // rows, with the missing weeks called out in place of the columns.
  const columnCount = microcycles.length > 0 ? microcycles.length + 1 : 2;
  const phasesInUse = PHASE_ORDER.filter((phase) => microcycles.some((w) => w.phase === phase));

  return (
    <>
      {/* Focusable so the weeks past the viewport edge can be reached without a
          pointer; a plain overflow container takes no keyboard focus. */}
      <div className={styles.scroll} role="region" aria-label="Plan grid" tabIndex={0}>
        <table className={styles.grid}>
          <thead>
            <tr>
              <td className={styles.corner} />
              {microcycles.length === 0 ? (
                <th scope="col" className={styles.noWeeks}>
                  No weeks yet.
                </th>
              ) : (
                microcycles.map((week) => (
                  <th key={week.id} scope="col" className={styles.weekHead}>
                    <span className={styles.weekLabel}>Week {week.position + 1}</span>{" "}
                    {week.phase && <PhaseBadge phase={week.phase} />}
                  </th>
                ))
              )}
            </tr>
          </thead>

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
              <tbody key={workout.id}>
                <tr>
                  <th scope="rowgroup" colSpan={columnCount} className={styles.workoutHead}>
                    <h2 className={styles.workoutName}>{workout.name}</h2>{" "}
                    {workout.planned_exercises.length > 0 && (
                      <span className={styles.workoutMeta}>
                        {exerciseCount(workout.planned_exercises.length)}
                      </span>
                    )}
                  </th>
                </tr>

                {workout.planned_exercises.length === 0 ? (
                  <tr>
                    <td className={styles.bandEmpty} colSpan={columnCount}>
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

      {/* The badges alone are only decodable by hovering their title, which
          never happens on touch — so the abbreviations in play are spelled out. */}
      {phasesInUse.length > 0 && (
        <section className={styles.legend} aria-label="Phase key">
          {phasesInUse.map((phase) => (
            <span key={phase} className={styles.legendItem}>
              <span className={styles.phase} data-phase={phase} aria-hidden="true">
                {phaseLabel(phase)}
              </span>{" "}
              {phase}
            </span>
          ))}
        </section>
      )}
    </>
  );
}

// The abbreviation is for the eye only; screen readers get the full phase name
// rather than an unpronounceable "INTENS".
function PhaseBadge({ phase }: { phase: Phase }) {
  return (
    <span className={styles.phase} data-phase={phase} title={phase}>
      <span aria-hidden="true">{phaseLabel(phase)}</span>
      <span className={styles.srOnly}>{phase}</span>
    </span>
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
              // The dash is decoration; the cell's meaning is spelled out for
              // screen readers, which an aria-label on a plain span would not
              // reliably reach.
              <>
                <span className={styles.unprescribed} aria-hidden="true">
                  —
                </span>
                <span className={styles.srOnly}>Not prescribed</span>
              </>
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
