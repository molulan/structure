import { useFullMesocycle } from "./useFullMesocycle";
import { MesocycleGrid } from "./MesocycleGrid";
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
        <p className={styles.status}>Loading…</p>
      ) : mesocycle.isError ? (
        <p className={styles.status}>Could not load this mesocycle.</p>
      ) : (
        <>
          <div className={styles.title}>
            <h1 className={styles.name}>{mesocycle.data.name}</h1>
            <span className={styles.mode}>{mesocycle.data.mode}</span>
          </div>

          <MesocycleGrid
            microcycles={mesocycle.data.microcycles}
            workouts={mesocycle.data.workouts}
          />
        </>
      )}
    </div>
  );
}
