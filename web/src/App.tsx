import { MesocycleList } from "./features/mesocycles/MesocycleList";
import styles from "./App.module.css";

export function App() {
  return (
    <>
      <header className={styles.header}>
        <span className={styles.wordmark}>STRUCTURE</span>
        <span className={styles.tagline}>mesocycle builder</span>
      </header>
      <main>
        <MesocycleList />
      </main>
    </>
  );
}
