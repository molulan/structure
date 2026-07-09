import { useState } from "react";
import { MesocycleList } from "./features/mesocycles/MesocycleList";
import { MesocycleDetail } from "./features/mesocycles/MesocycleDetail";
import styles from "./App.module.css";

export function App() {
  const [selectedId, setSelectedId] = useState<number | null>(null);

  return (
    <>
      <header className={styles.header}>
        <span className={styles.wordmark}>STRUCTURE</span>
        <span className={styles.tagline}>mesocycle builder</span>
      </header>
      <main>
        {selectedId === null ? (
          <MesocycleList onSelect={setSelectedId} />
        ) : (
          <MesocycleDetail id={selectedId} onBack={() => setSelectedId(null)} />
        )}
      </main>
    </>
  );
}
