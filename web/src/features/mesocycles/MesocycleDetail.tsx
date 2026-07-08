import { useFullMesocycle } from "./useFullMesocycle";
import { describeSetGroup } from "./format";
import styles from "./MesocycleDetail.module.css";

interface Props {
  id: number;
  onBack: () => void;
}

export function MesocycleDetail({ id, onBack }: Props) {
  const mesocycle = useFullMesocycle(id);

  return (
    <div className={styles.wrap}>
      <button className={styles.back} onClick={onBack}>
        ← All mesocycles
      </button>

      {mesocycle.isPending ? (
        <p className={styles.empty}>Loading…</p>
      ) : mesocycle.isError ? (
        <p className={styles.empty}>Could not load this mesocycle.</p>
      ) : (
        <>
          <div className={styles.title}>
            <span className={styles.name}>{mesocycle.data.name}</span>
            <span className={styles.mode}>{mesocycle.data.mode}</span>
          </div>

          {mesocycle.data.microcycles.length === 0 ? (
            <p className={styles.empty}>No weeks yet.</p>
          ) : (
            mesocycle.data.microcycles.map((week) => (
              <section key={week.id} className={styles.week}>
                <div className={styles.weekHead}>
                  <span className={styles.weekNum}>Week {week.position + 1}</span>
                  {week.phase && <span className={styles.phase}>{week.phase}</span>}
                </div>

                {week.workouts.length === 0 ? (
                  <p className={styles.empty}>No workouts.</p>
                ) : (
                  week.workouts.map((workout) => (
                    <div key={workout.id} className={styles.workout}>
                      <h3 className={styles.workoutName}>{workout.name}</h3>
                      {workout.planned_exercises.map((planned) => (
                        <div key={planned.id} className={styles.exercise}>
                          <div className={styles.exerciseHead}>
                            <span className={styles.exerciseName}>{planned.exercise.name}</span>
                            <span className={styles.muscle}>
                              {planned.exercise.primary_muscle_group}
                            </span>
                          </div>
                          <ul className={styles.sets}>
                            {planned.set_groups.map((group) => (
                              <li key={group.id} className={styles.setLine}>
                                {describeSetGroup(group)}
                              </li>
                            ))}
                          </ul>
                        </div>
                      ))}
                    </div>
                  ))
                )}
              </section>
            ))
          )}
        </>
      )}
    </div>
  );
}
